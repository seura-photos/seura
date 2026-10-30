import type { RenderSettings } from "../bindings";
import {
  type Backend,
  GUIDE_PASSES,
  type LinearImage,
  PARAMS_SIZE,
  type Region,
  WHOLE,
  guideKey,
  needsGuide,
  orientedSize,
  params,
} from "./backend";
import shader from "./generated/shader.wgsl?raw";

// WebGPU's flag values (from the WebGPU specification). TypeScript's DOM
// library declares the interfaces but not these constants.
const BUFFER = { MAP_READ: 0x1, COPY_DST: 0x8, UNIFORM: 0x40 };
const TEXTURE = { COPY_SRC: 0x1, COPY_DST: 0x2, TEXTURE_BINDING: 0x4, RENDER_ATTACHMENT: 0x10 };
const FRAGMENT = 0x2;
const MAP_READ = 0x1;

/**
 * The preview on WebGPU, running the same WGSL as export. Returns `null`
 * where the web view has no WebGPU (WKWebView before macOS Tahoe 26), so
 * the caller can fall back to WebGL2.
 */
export async function createWebGpu(canvas: HTMLCanvasElement | null): Promise<Backend | null> {
  const adapter = await navigator.gpu?.requestAdapter();
  if (!adapter) return null;
  const device = await adapter.requestDevice();
  const module = device.createShaderModule({ code: shader });
  const canvasFormat = navigator.gpu.getPreferredCanvasFormat();
  const context = (canvas?.getContext("webgpu") ?? null) as GPUCanvasContext | null;
  // The preview is encoded as Display P3; the web view maps it to the display.
  context?.configure({
    device,
    format: canvasFormat,
    colorSpace: "display-p3",
    alphaMode: "opaque",
  });

  // The shaders' bindings, as export's renderer lays them out: the stage
  // holds 32-bit floats, which can't be filtered everywhere, and is only
  // read pixel by pixel.
  const layout = device.createBindGroupLayout({
    entries: [
      { binding: 0, visibility: FRAGMENT, buffer: { type: "uniform" } },
      { binding: 1, visibility: FRAGMENT, texture: { sampleType: "float" } },
      { binding: 2, visibility: FRAGMENT, sampler: { type: "filtering" } },
      { binding: 3, visibility: FRAGMENT, texture: { sampleType: "float" } },
      { binding: 4, visibility: FRAGMENT, sampler: { type: "filtering" } },
      { binding: 5, visibility: FRAGMENT, texture: { sampleType: "unfilterable-float" } },
    ],
  });
  const pipelineLayout = device.createPipelineLayout({ bindGroupLayouts: [layout] });
  const pipelines = new Map<GPUTextureFormat, GPURenderPipeline>();
  const pipeline = (format: GPUTextureFormat) => {
    let found = pipelines.get(format);
    if (!found) {
      found = device.createRenderPipeline({
        layout: pipelineLayout,
        vertex: { module, entryPoint: "vs_main" },
        fragment: { module, entryPoint: "fs_main", targets: [{ format }] },
        primitive: { topology: "triangle-list" },
      });
      pipelines.set(format, found);
    }
    return found;
  };
  const uniforms = device.createBuffer({
    size: PARAMS_SIZE,
    usage: BUFFER.UNIFORM | BUFFER.COPY_DST,
  });
  // Linear for the display, which may scale; nearest for the check, which
  // must sample each pixel exactly as export does. The guide is always
  // sampled bilinearly, as export does.
  const linear = device.createSampler({ magFilter: "linear", minFilter: "linear" });
  const nearest = device.createSampler({ magFilter: "nearest", minFilter: "nearest" });
  // Bound where a draw reads nothing.
  const empty = (format: GPUTextureFormat) =>
    device.createTexture({ size: { width: 1, height: 1 }, format, usage: TEXTURE.TEXTURE_BINDING });
  const emptyImage = empty("rgba16float");
  const emptyStage = empty("rgba32float");
  let photo: GPUTexture | null = null;
  let size = { width: 0, height: 0 };
  let region: Region = WHOLE;
  let guideImage: GPUTexture | null = null;
  // The guide, and the white balance it was built for.
  let guide: { texture: GPUTexture; key: string } | null = null;

  const bindings = (
    shown: GPUTexture,
    sampler: GPUSampler,
    guided: GPUTexture,
    stage: GPUTexture,
  ) =>
    device.createBindGroup({
      layout,
      entries: [
        { binding: 0, resource: { buffer: uniforms } },
        { binding: 1, resource: shown.createView() },
        { binding: 2, resource: sampler },
        { binding: 3, resource: guided.createView() },
        { binding: 4, resource: linear },
        { binding: 5, resource: stage.createView() },
      ],
    });

  const pass = (
    encoder: GPUCommandEncoder,
    target: GPUTexture,
    format: GPUTextureFormat,
    group: GPUBindGroup,
  ) => {
    const drawing = encoder.beginRenderPass({
      colorAttachments: [
        {
          view: target.createView(),
          loadOp: "clear",
          storeOp: "store",
          clearValue: { r: 0, g: 0, b: 0, a: 1 },
        },
      ],
    });
    drawing.setPipeline(pipeline(format));
    drawing.setBindGroup(0, group);
    drawing.draw(3);
    drawing.end();
  };

  /** The guide for these settings, built if it isn't yet; `null` if none is needed. */
  const guideFor = (settings: RenderSettings): GPUTexture | null => {
    if (!needsGuide(settings) || !guideImage) return null;
    const key = guideKey(settings);
    if (guide?.key === key) return guide.texture;
    guide?.texture.destroy();
    const guideSize = { width: guideImage.width, height: guideImage.height };
    const drawn = (format: GPUTextureFormat) =>
      device.createTexture({
        size: guideSize,
        format,
        usage: TEXTURE.RENDER_ATTACHMENT | TEXTURE.TEXTURE_BINDING,
      });
    const textures = {
      image: guideImage,
      one: drawn("rgba32float"),
      two: drawn("rgba32float"),
      guide: drawn("rgba16float"),
    };
    // Each pass has its own parameters, so each is submitted on its own.
    for (const step of GUIDE_PASSES) {
      device.queue.writeBuffer(uniforms, 0, params(settings, false, step.pass));
      const encoder = device.createCommandEncoder();
      const group = bindings(emptyImage, linear, emptyImage, textures[step.from]);
      pass(encoder, textures[step.to], textures[step.to].format, group);
      device.queue.submit([encoder.finish()]);
    }
    textures.one.destroy();
    textures.two.destroy();
    guide = { texture: textures.guide, key };
    return guide.texture;
  };

  const render = (
    target: GPUTexture,
    format: GPUTextureFormat,
    sampler: GPUSampler,
    settings: RenderSettings,
    outputSrgb: boolean,
  ) => {
    if (!photo) return null;
    const guided = guideFor(settings) ?? emptyImage;
    device.queue.writeBuffer(uniforms, 0, params(settings, outputSrgb, 0, region));
    const encoder = device.createCommandEncoder();
    pass(encoder, target, format, bindings(photo, sampler, guided, emptyStage));
    return encoder;
  };

  const texture = (image: LinearImage) => {
    const imageSize = { width: image.width, height: image.height };
    const made = device.createTexture({
      size: imageSize,
      format: "rgba16float",
      usage: TEXTURE.TEXTURE_BINDING | TEXTURE.COPY_DST,
    });
    device.queue.writeTexture(
      { texture: made },
      image.data,
      { bytesPerRow: image.width * 8 },
      imageSize,
    );
    return made;
  };

  return {
    name: "webgpu",
    upload(image: LinearImage, guideSource: LinearImage, part: Region = WHOLE) {
      photo?.destroy();
      guideImage?.destroy();
      guide?.texture.destroy();
      guide = null;
      size = { width: image.width, height: image.height };
      region = part;
      photo = texture(image);
      guideImage = texture(guideSource);
    },
    draw(settings: RenderSettings) {
      if (!context) return;
      const encoder = render(context.getCurrentTexture(), canvasFormat, linear, settings, false);
      if (encoder) device.queue.submit([encoder.finish()]);
    },
    async read(settings: RenderSettings) {
      const drawn = orientedSize(size, settings);
      const target = device.createTexture({
        size: drawn,
        format: "rgba8unorm",
        usage: TEXTURE.RENDER_ATTACHMENT | TEXTURE.COPY_SRC,
      });
      const encoder = render(target, "rgba8unorm", nearest, settings, true);
      if (!encoder) throw new Error("no photo uploaded");
      // Rows in a copy to a buffer are padded to 256 bytes.
      const row = drawn.width * 4;
      const padded = Math.ceil(row / 256) * 256;
      const buffer = device.createBuffer({
        size: padded * drawn.height,
        usage: BUFFER.COPY_DST | BUFFER.MAP_READ,
      });
      encoder.copyTextureToBuffer({ texture: target }, { buffer, bytesPerRow: padded }, drawn);
      device.queue.submit([encoder.finish()]);
      await buffer.mapAsync(MAP_READ);
      const mapped = new Uint8Array(buffer.getMappedRange());
      const pixels = new Uint8Array(row * drawn.height);
      for (let y = 0; y < drawn.height; y++) {
        pixels.set(mapped.subarray(y * padded, y * padded + row), y * row);
      }
      buffer.unmap();
      buffer.destroy();
      target.destroy();
      return pixels;
    },
    destroy() {
      photo?.destroy();
      guideImage?.destroy();
      guide?.texture.destroy();
      emptyImage.destroy();
      emptyStage.destroy();
      uniforms.destroy();
      context?.unconfigure();
      device.destroy();
    },
  };
}
