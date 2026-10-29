import { convertFileSrc } from "@tauri-apps/api/core";
import type { RenderSettings } from "../bindings";
import layout from "./generated/params.json";

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
  draw(settings: RenderSettings): void;
  /** Renders the photo at its own size encoded as sRGB, like export, and reads it back as 8-bit RGBA rows, top first. */
  read(settings: RenderSettings): Promise<Uint8Array>;
  destroy(): void;
}

/** The size in bytes of the shaders' `Params`, the uniform block. */
export const PARAMS_SIZE = layout.size;

/** A field of `Params` as the settings name it: `exposure_ev` is `exposureEv`. */
export function settingName(field: string): string {
  return field.replace(/_([a-z])/g, (_, letter: string) => letter.toUpperCase());
}

/**
 * The shaders' `Params` for these settings (`crates/seura-render/shaders/
 * main.wgsl`), each field where naga lays it out: the layout is generated
 * from the WGSL with the shaders (`generated/params.json`), so it can't
 * drift from them.
 */
export function params(settings: RenderSettings, outputSrgb: boolean): ArrayBuffer {
  const buffer = new ArrayBuffer(layout.size);
  const view = new DataView(buffer);
  for (const field of layout.fields) {
    const value =
      field.name === "output_srgb"
        ? outputSrgb
        : settings[settingName(field.name) as keyof RenderSettings];
    // Numbers from Rust can be null, which would mean NaN; edit files hold
    // only finite numbers, so there is none here.
    const number = typeof value === "boolean" ? Number(value) : (value ?? 0);
    if (field.type === "u32") view.setUint32(field.offset, number, true);
    else view.setFloat32(field.offset, number, true);
  }
  return buffer;
}

/**
 * Fetches working-space pixels from the `seura://` protocol: `preview` (the
 * photo open in the Editor), `view-<id>` (a photo decoded for the Viewer),
 * `view-region` (the part cut for it while zoomed in), or `check-image`. The body is the width and height as little-endian
 * 32-bit integers, then the half floats.
 */
export async function fetchLinear(
  path: "preview" | "check-image" | "view-region" | `view-${number}`,
): Promise<LinearImage> {
  const response = await fetch(convertFileSrc(path, "seura"), { cache: "no-store" });
  if (!response.ok) throw new Error(`${path}: ${response.status}`);
  const buffer = await response.arrayBuffer();
  const size = new DataView(buffer, 0, 8);
  const width = size.getUint32(0, true);
  const height = size.getUint32(4, true);
  const data = new Uint16Array(buffer, 8, width * height * 4);
  return { width, height, data };
}
