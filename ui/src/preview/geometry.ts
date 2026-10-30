import type { RenderSettings } from "../bindings";

/**
 * The geometry stage as one coordinate transform (edit format, formulas
 * appendix, `geometry.md`), on the UI's side: how big the result is, and
 * which part of the upright photo a part of it samples, for sizing canvases
 * and cutting zoomed-in parts. The shader places each pixel itself
 * (`main.wgsl`'s `upright_position`); the core does the same for export
 * (`crates/seura-render/src/geometry.rs`).
 */

export type Size = { width: number; height: number };

/** A rectangle, in pixels or as fractions, as the context says. */
export type Region = { x: number; y: number; width: number; height: number };

export const WHOLE: Region = { x: 0, y: 0, width: 1, height: 1 };

/** The settings that place pixels: `seura.orient`, `seura.straighten`, `seura.crop`. */
export type Geometry = Pick<
  RenderSettings,
  "rotate" | "flipH" | "flipV" | "straighten" | "cropX" | "cropY" | "cropWidth" | "cropHeight"
>;

/** Nothing placed differently. */
export const UPRIGHT: Geometry = {
  rotate: 0,
  flipH: false,
  flipV: false,
  straighten: 0,
  cropX: 0,
  cropY: 0,
  cropWidth: 1,
  cropHeight: 1,
};

/** The settings' geometry alone. */
export function geometryOf(settings: Geometry): Geometry {
  return {
    rotate: settings.rotate,
    flipH: settings.flipH,
    flipV: settings.flipV,
    straighten: settings.straighten,
    cropX: settings.cropX,
    cropY: settings.cropY,
    cropWidth: settings.cropWidth,
    cropHeight: settings.cropHeight,
  };
}

/** A key that changes whenever the geometry does. */
export function geometryKey(geometry: Geometry): string {
  return JSON.stringify(geometryOf(geometry));
}

/** round() in the appendix: halves round up. */
const round = (value: number) => Math.floor(value + 0.5);

/** The oriented photo's size, for an upright photo of `size`. */
export function orientedSize(size: Size, geometry: Geometry): Size {
  return geometry.rotate % 180 === 90
    ? { width: size.height, height: size.width }
    : { width: size.width, height: size.height };
}

/** The crop in whole pixels of the oriented photo, for an upright photo of
 * `full` size. Numbers from Rust can be null, which would mean NaN; edit
 * files hold only finite numbers, so there is none here. */
export function cropPixels(geometry: Geometry, full: Size): Region {
  const frame = orientedSize(full, geometry);
  return {
    x: round((geometry.cropX ?? 0) * frame.width),
    y: round((geometry.cropY ?? 0) * frame.height),
    width: Math.max(1, round((geometry.cropWidth ?? 1) * frame.width)),
    height: Math.max(1, round((geometry.cropHeight ?? 1) * frame.height)),
  };
}

/** The result's size for an upright photo of `full` size: its crop's. */
export function outputSize(geometry: Geometry, full: Size): Size {
  const crop = cropPixels(geometry, full);
  return { width: crop.width, height: crop.height };
}

/** Where position `x`, `y` of the result (pixels; a pixel's center is at
 * i + 0.5) is in the upright photo, in its pixels. */
export function uprightPosition(
  geometry: Geometry,
  full: Size,
  x: number,
  y: number,
): [number, number] {
  const frame = orientedSize(full, geometry);
  const crop = cropPixels(geometry, full);
  // The crop, then the turn back about the frame's center.
  const [dx, dy] = [crop.x + x - frame.width / 2, crop.y + y - frame.height / 2];
  const theta = ((geometry.straighten ?? 0) * Math.PI) / 180;
  const [sin, cos] = [Math.sin(theta), Math.cos(theta)];
  let qx = frame.width / 2 + dx * cos + dy * sin;
  let qy = frame.height / 2 - dx * sin + dy * cos;
  // seura.orient: its flips undone, then its turn.
  if (geometry.flipH) qx = frame.width - qx;
  if (geometry.flipV) qy = frame.height - qy;
  switch (geometry.rotate) {
    case 90:
      return [qy, frame.width - qx];
    case 180:
      return [frame.width - qx, frame.height - qy];
    case 270:
      return [frame.height - qy, qx];
    default:
      return [qx, qy];
  }
}

/**
 * The part of the upright photo that `view` of the result (pixels) samples:
 * the box around where its corners land, `margin` pixels wider on each side
 * for the filter, in whole pixels within the photo.
 */
export function sourceRegion(geometry: Geometry, full: Size, view: Region, margin: number): Region {
  const corners = [
    [view.x, view.y],
    [view.x + view.width, view.y],
    [view.x, view.y + view.height],
    [view.x + view.width, view.y + view.height],
  ].map(([x, y]) => uprightPosition(geometry, full, x ?? 0, y ?? 0));
  const xs = corners.map(([x]) => x);
  const ys = corners.map(([, y]) => y);
  const clamp = (value: number, low: number, high: number) => Math.min(Math.max(value, low), high);
  const left = clamp(Math.floor(Math.min(...xs) - margin), 0, full.width - 1);
  const top = clamp(Math.floor(Math.min(...ys) - margin), 0, full.height - 1);
  const right = Math.max(left + 1, clamp(Math.ceil(Math.max(...xs) + margin), 1, full.width));
  const bottom = Math.max(top + 1, clamp(Math.ceil(Math.max(...ys) + margin), 1, full.height));
  return { x: left, y: top, width: right - left, height: bottom - top };
}

/**
 * Where a draw sits: the upright photo's full size, which geometry is
 * measured in; the part of it the photo texture holds; and the part of the
 * result the draw covers, each as fractions. Without a part or a view,
 * the whole of each.
 */
export type Placement = { full: Size; part?: Region; view?: Region };

/** The size to draw an image of `image`'s size placed so, at its own
 * resolution: the canvas's size. */
export function drawnSize(image: Size, geometry: Geometry, placement: Placement): Size {
  const { full } = placement;
  const part = placement.part ?? WHOLE;
  const view = placement.view ?? WHOLE;
  const result = outputSize(geometry, full);
  const scale = image.width / Math.max(1, part.width * full.width);
  return {
    width: Math.max(1, Math.round(view.width * result.width * scale)),
    height: Math.max(1, Math.round(view.height * result.height * scale)),
  };
}
