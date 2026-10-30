import { convertFileSrc } from "@tauri-apps/api/core";
import type { RenderSettings } from "../bindings";
import layout from "./generated/params.json";
import { cropPixels, type Placement, WHOLE } from "./geometry";

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
  /**
   * Uploads the photo, with the image its luminance guide is built from
   * (for highlights and shadows), and where it's placed: the photo's full
   * size, which part of it `image` is, and which part of the result it
   * draws. Both images are upright, and the guide always covers all of the
   * photo; drawing places them as the settings say. The canvas must already
   * have the size the settings drawn give it (`drawnSize`).
   */
  upload(image: LinearImage, guide: LinearImage, placement: Placement): void;
  /** Draws the photo with these settings on the canvas, for the display. */
  draw(settings: RenderSettings): void;
  /** Renders the photo at its own resolution, placed as the settings say (`drawnSize`), encoded as sRGB like export, and reads it back as 8-bit RGBA rows, top first. */
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
  placement: Placement = { full: { width: 1, height: 1 } },
): ArrayBuffer {
  const buffer = new ArrayBuffer(PARAMS_SIZE);
  const view = new DataView(buffer);
  const part = placement.part ?? WHOLE;
  const shown = placement.view ?? WHOLE;
  const crop = cropPixels(settings, placement.full);
  // Red-eye's circles, as many as the block holds (seura-render's
  // MAX_RED_EYE), each x, y and radius in a vec4.
  const circles = layout.fields.find((field) => field.name === "red_eye");
  const redEye = (settings.redEye ?? []).slice(0, circles?.count ?? 0);
  // The fields that aren't settings: which pass, and where the draw sits.
  const draw: Record<string, number | boolean> = {
    output_srgb: outputSrgb,
    draw_pass: pass,
    tile_x: part.x,
    tile_y: part.y,
    tile_width: part.width,
    tile_height: part.height,
    full_width: placement.full.width,
    full_height: placement.full.height,
    view_x: shown.x,
    view_y: shown.y,
    view_width: shown.width,
    view_height: shown.height,
    crop_left: crop.x,
    crop_top: crop.y,
    crop_pixels_width: crop.width,
    crop_pixels_height: crop.height,
    red_eye_count: redEye.length,
  };
  for (const field of layout.fields) {
    if (field.type === "vec4f") {
      redEye.forEach(({ x, y, radius }, i) => {
        const at = field.offset + i * (field.stride ?? 16);
        [x, y, radius, 0].forEach((v, k) => view.setFloat32(at + k * 4, v ?? 0, true));
      });
      continue;
    }
    // Every other field is a number or a flag.
    const value =
      field.name in draw
        ? draw[field.name]
        : (settings[settingName(field.name) as keyof RenderSettings] as number | boolean | null);
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
