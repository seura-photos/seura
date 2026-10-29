import type { RenderSettings } from "../bindings";
import { type Backend, type LinearImage, PARAMS_SIZE, params } from "./backend";
import shader from "./generated/shader.wgsl?raw";

// WebGPU's flag values (from the WebGPU specification). TypeScript's DOM
// library declares the interfaces but not these constants.
const BUFFER = { MAP_READ: 0x1, COPY_DST: 0x8, UNIFORM: 0x40 };
const TEXTURE = { COPY_SRC: 0x1, COPY_DST: 0x2, TEXTURE_BINDING: 0x4, RENDER_ATTACHMENT: 0x10 };
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

  const pipelines = new Map<GPUTextureFormat, GPURenderPipeline>();
  const pipeline = (format: GPUTextureFormat) => {
    let found = pipelines.get(format);
    if (!found) {
      found = device.createRenderPipeline({
        layout: "auto",
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
  // must sample each pixel exactly as export does.
  const linear = device.createSampler({ magFilter: "linear", minFilter: "linear" });
  const nearest = device.createSampler({ magFilter: "nearest", minFilter: "nearest" });
  let photo: GPUTexture | null = null;
  let size = { width: 0, height: 0 };

  const render = (
    target: GPUTexture,
    format: GPUTextureFormat,
    sampler: GPUSampler,
    settings: RenderSettings,
    outputSrgb: boolean,
  ) => {
    if (!photo) return null;
    device.queue.writeBuffer(uniforms, 0, params(settings, outputSrgb));
    const chosen = pipeline(format);
    const bindings = device.createBindGroup({
      layout: chosen.getBindGroupLayout(0),
      entries: [
        { binding: 0, resource: { buffer: uniforms } },
        { binding: 1, resource: photo.createView() },
        { binding: 2, resource: sampler },
      ],
    });
    const encoder = device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
        {
          view: target.createView(),
          loadOp: "clear",
          storeOp: "store",
          clearValue: { r: 0, g: 0, b: 0, a: 1 },
        },
      ],
    });
    pass.setPipeline(chosen);
    pass.setBindGroup(0, bindings);
    pass.draw(3);
    pass.end();
    return encoder;
  };

  return {
    name: "webgpu",
    upload(image: LinearImage) {
      photo?.destroy();
      size = { width: image.width, height: image.height };
      photo = device.createTexture({
        size,
        format: "rgba16float",
        usage: TEXTURE.TEXTURE_BINDING | TEXTURE.COPY_DST,
      });
      device.queue.writeTexture(
        { texture: photo },
        image.data,
        { bytesPerRow: image.width * 8 },
        size,
      );
    },
    draw(settings: RenderSettings) {
      if (!context) return;
      const encoder = render(context.getCurrentTexture(), canvasFormat, linear, settings, false);
      if (encoder) device.queue.submit([encoder.finish()]);
    },
    async read(settings: RenderSettings) {
      const target = device.createTexture({
        size,
        format: "rgba8unorm",
        usage: TEXTURE.RENDER_ATTACHMENT | TEXTURE.COPY_SRC,
      });
      const encoder = render(target, "rgba8unorm", nearest, settings, true);
      if (!encoder) throw new Error("no photo uploaded");
      // Rows in a copy to a buffer are padded to 256 bytes.
      const row = size.width * 4;
      const padded = Math.ceil(row / 256) * 256;
      const buffer = device.createBuffer({
        size: padded * size.height,
        usage: BUFFER.COPY_DST | BUFFER.MAP_READ,
      });
      encoder.copyTextureToBuffer({ texture: target }, { buffer, bytesPerRow: padded }, size);
      device.queue.submit([encoder.finish()]);
      await buffer.mapAsync(MAP_READ);
      const mapped = new Uint8Array(buffer.getMappedRange());
      const pixels = new Uint8Array(row * size.height);
      for (let y = 0; y < size.height; y++) {
        pixels.set(mapped.subarray(y * padded, y * padded + row), y * row);
      }
      buffer.unmap();
      buffer.destroy();
      target.destroy();
      return pixels;
    },
    destroy() {
      photo?.destroy();
      uniforms.destroy();
      context?.unconfigure();
      device.destroy();
    },
  };
}
