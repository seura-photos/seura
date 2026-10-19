import { convertFileSrc } from "@tauri-apps/api/core";

/**
 * Pixels in the working space (linear light, Display P3 primaries) as RGBA
 * half floats, fetched from the core. The core decodes and converts; the
 * UI never reads a photo itself (decision 0009).
 */
export type LinearImage = { width: number; height: number; data: Uint16Array };

/**
 * One way of running the shared shaders (decision 0004): WebGPU, or WebGL2
 * with naga's translation of the same WGSL where WebGPU is missing.
 */
export interface Backend {
  readonly name: "webgpu" | "webgl2";
  /** Uploads the photo. The canvas must already have its size. */
  upload(image: LinearImage): void;
  /** Draws the photo with these settings on the canvas, for the display. */
  draw(exposureEv: number): void;
  /** Renders the photo at its own size encoded as sRGB, like export, and reads it back as 8-bit RGBA rows, top first. */
  read(exposureEv: number): Promise<Uint8Array>;
  destroy(): void;
}

/** The shader's `Params`: 16 bytes, as in `crates/seura-render/shaders/main.wgsl`. */
export function params(exposureEv: number, outputSrgb: boolean): ArrayBuffer {
  const buffer = new ArrayBuffer(16);
  new Float32Array(buffer, 0, 1)[0] = exposureEv;
  new Uint32Array(buffer, 4, 1)[0] = outputSrgb ? 1 : 0;
  return buffer;
}

/**
 * Fetches working-space pixels from the `seura://` protocol: `preview` (the
 * photo open in the Editor) or `check-image`. The body is the width and
 * height as little-endian 32-bit integers, then the half floats.
 */
export async function fetchLinear(path: "preview" | "check-image"): Promise<LinearImage> {
  const response = await fetch(convertFileSrc(path, "seura"), { cache: "no-store" });
  if (!response.ok) throw new Error(`${path}: ${response.status}`);
  const buffer = await response.arrayBuffer();
  const size = new DataView(buffer, 0, 8);
  const width = size.getUint32(0, true);
  const height = size.getUint32(4, true);
  const data = new Uint16Array(buffer, 8, width * height * 4);
  return { width, height, data };
}
