import { convertFileSrc } from "@tauri-apps/api/core";
import type { RenderSettings } from "../bindings";
import layout from "./generated/params.json";

/**
 * Pixels in the working space (linear light, Display P3 primaries) as RGBA
 * half floats, fetched from the core. The core decodes and converts; the
 * UI never reads a photo itself (decision 0009).
 */
export type LinearImage = { width: number; height: number; data: Uint16Array };

/** A part of a photo, as fractions of it. */
export type Region = { x: number; y: number; width: number; height: number };

export const WHOLE: Region = { x: 0, y: 0, width: 1, height: 1 };

/**
 * One way of running the shared shaders (decision 0004): WebGPU, or WebGL2
 * with naga's translation of the same WGSL where WebGPU is missing.
 */
export interface Backend {
  readonly name: "webgpu" | "webgl2";
  /**
   * Uploads the photo, with the image its luminance guide is built from
   * (for highlights and shadows), and which part of the photo `image` is:
   * the guide always covers all of it. The canvas must already have its
   * size.
   */
  upload(image: LinearImage, guide: LinearImage, region?: Region): void;
  /** Draws the photo with these settings on the canvas, for the display. */
  draw(settings: RenderSettings): void;
  /** Renders the photo at its own size encoded as sRGB, like export, and reads it back as 8-bit RGBA rows, top first. */
  read(settings: RenderSettings): Promise<Uint8Array>;
  destroy(): void;
}

/**
 * The size in bytes of the buffer for the shaders' `Params`, the uniform
 * block: naga's size, rounded up to 16 bytes, as GLSL's block layout
 * (std140) rounds a struct. A smaller buffer, which WebGPU accepts, makes
 * WebGL2 refuse to draw.
 */
export const PARAMS_SIZE = Math.ceil(layout.size / 16) * 16;

/**
 * The luminance guide's passes (`crates/seura-render/shaders/guide.wgsl`),
 * the same as export's (`renderer.rs`): each reads the last one's result
 * and draws into the next, at the guide's size. `image` is the guide's
 * image; `one` and `two` hold 32-bit floats; `guide` is the result.
 */
export const GUIDE_PASSES = [
  { pass: 1, from: "image", to: "one" },
  { pass: 2, from: "one", to: "two" },
  { pass: 3, from: "two", to: "one" },
  { pass: 2, from: "one", to: "two" },
  { pass: 4, from: "two", to: "guide" },
] as const;

/** Whether these settings draw with the guide. */
export function needsGuide(settings: RenderSettings): boolean {
  return (settings.highlights ?? 0) !== 0 || (settings.shadows ?? 0) !== 0;
}

/**
 * What the guide depends on besides the photo: white balance. Exposure is
 * added to it as it's drawn, so a guide built for one key serves any
 * brightness.
 */
export function guideKey(settings: RenderSettings): string {
  return `${settings.temperature ?? 0} ${settings.tint ?? 0}`;
}

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
export function params(
  settings: RenderSettings,
  outputSrgb: boolean,
  pass = 0,
  region: Region = WHOLE,
): ArrayBuffer {
  const buffer = new ArrayBuffer(PARAMS_SIZE);
  const view = new DataView(buffer);
  // The fields that aren't settings: which pass, and which part of the photo.
  const draw: Record<string, number | boolean> = {
    output_srgb: outputSrgb,
    draw_pass: pass,
    tile_x: region.x,
    tile_y: region.y,
    tile_width: region.width,
    tile_height: region.height,
  };
  for (const field of layout.fields) {
    const value =
      field.name in draw
        ? draw[field.name]
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
 * photo open in the Editor; `preview-region`, the part cut while zoomed in), `view-<id>` (a photo decoded for the Viewer),
 * `view-region` (the part cut for it while zoomed in), or `check-image`;
 * or the image each's luminance guide is built from, `preview-guide`,
 * `view-guide-<id>` and `check-guide`. The body is the width and height as
 * little-endian 32-bit integers, then the half floats.
 */
export async function fetchLinear(
  path:
    | "preview"
    | "preview-guide"
    | "preview-region"
    | "check-image"
    | "check-guide"
    | "view-region"
    | `view-${number}`
    | `view-guide-${number}`,
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
