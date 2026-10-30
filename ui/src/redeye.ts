import type { RedEyeCircle } from "./bindings";
import { t } from "./messages";
import {
  cropPixels,
  type Geometry,
  orientedSize,
  outputSize,
  type Size,
  uprightPosition,
} from "./preview/geometry";

/**
 * Red-eye mode's arithmetic (EDT-7; wireframes, "Red-eye mode"): where a
 * correction is on screen, and where a point on screen is in the photo.
 * Corrections are stored on the upright photo, before geometry (edit
 * format, "Coordinates"): x and y as fractions of it, the radius as a
 * fraction of its shorter side. The screen shows the result, turned,
 * straightened and cropped, placed and zoomed by `useZoom`.
 */

/** A correction: x and y as fractions of the upright photo, the radius as
 * a fraction of its shorter side. */
export type Circle = { x: number; y: number; radius: number };

/** The circles in a photo's settings. Numbers from Rust can be null, which
 * would mean NaN; edit files hold only finite numbers, so there is none. */
export function circlesOf(circles: RedEyeCircle[] | undefined): Circle[] {
  return (circles ?? []).map(({ x, y, radius }) => ({
    x: x ?? 0,
    y: y ?? 0,
    radius: radius ?? 0,
  }));
}

/** The most corrections a photo can have (the edit format's limit). */
export const MAX_CORRECTIONS = 64;

/** A click's circle, in CSS pixels across, until `[` and `]` change it. */
export const BRUSH_DEFAULT = 24;
const BRUSH_MIN = 6;
const BRUSH_MAX = 240;
/** A click's circle as the user last set it, kept while the app runs. */
export const brushMemory = { across: BRUSH_DEFAULT };
/** Each `[` or `]` scales a circle by this much. */
export const BRUSH_STEP = 1.25;

/** A brush or circle size `steps` presses of `[` (−1) or `]` (+1) on. */
export function stepped(size: number, steps: number, min = BRUSH_MIN, max = BRUSH_MAX): number {
  return Math.min(max, Math.max(min, size * BRUSH_STEP ** steps));
}

/** Where position `x`, `y` of the upright photo (pixels) is in the result
 * (pixels): `uprightPosition` the other way. */
export function resultPosition(
  geometry: Geometry,
  full: Size,
  x: number,
  y: number,
): [number, number] {
  const frame = orientedSize(full, geometry);
  const crop = cropPixels(geometry, full);
  // seura.orient: turned, then its flips.
  let qx: number;
  let qy: number;
  switch (geometry.rotate) {
    case 90:
      [qx, qy] = [frame.width - y, x];
      break;
    case 180:
      [qx, qy] = [frame.width - x, frame.height - y];
      break;
    case 270:
      [qx, qy] = [y, frame.height - x];
      break;
    default:
      [qx, qy] = [x, y];
  }
  if (geometry.flipH) qx = frame.width - qx;
  if (geometry.flipV) qy = frame.height - qy;
  // The turn about the frame's center, then the crop.
  const theta = ((geometry.straighten ?? 0) * Math.PI) / 180;
  const [sin, cos] = [Math.sin(theta), Math.cos(theta)];
  const [ex, ey] = [qx - frame.width / 2, qy - frame.height / 2];
  return [
    frame.width / 2 + ex * cos - ey * sin - crop.x,
    frame.height / 2 + ex * sin + ey * cos - crop.y,
  ];
}

/** Where the result is on screen: its box in CSS pixels, from `useZoom`. */
export type Placed = { left: number; top: number; width: number; height: number };

/** A correction as red-eye mode draws it: its center and radius in CSS
 * pixels of the stage, and whether it's inside the crop. */
export type OnScreen = { x: number; y: number; radius: number; inside: boolean };

/**
 * The mapping between the stage (CSS pixels) and the upright photo, for the
 * photo's `full` upright size, its geometry, and where its result is shown.
 */
export function screenMapping(geometry: Geometry, full: Size, placed: Placed) {
  const result = outputSize(geometry, full);
  // CSS pixels for each pixel of the result, and so of the upright photo:
  // geometry turns and moves the photo but never scales it.
  const scale = placed.width / Math.max(1, result.width);
  const shorter = Math.min(full.width, full.height);
  return {
    /** A circle's place on the stage. */
    toScreen(circle: Circle): OnScreen {
      const [rx, ry] = resultPosition(
        geometry,
        full,
        circle.x * full.width,
        circle.y * full.height,
      );
      return {
        x: placed.left + rx * scale,
        y: placed.top + ry * scale,
        radius: circle.radius * shorter * scale,
        inside: rx >= 0 && ry >= 0 && rx <= result.width && ry <= result.height,
      };
    },
    /** The circle at a point of the stage, with a radius in CSS pixels. */
    fromScreen(x: number, y: number, radius: number): Circle {
      const [ux, uy] = uprightPosition(
        geometry,
        full,
        (x - placed.left) / scale,
        (y - placed.top) / scale,
      );
      return {
        x: clamp01(ux / full.width),
        y: clamp01(uy / full.height),
        radius: Math.max(1e-4, Math.min(1, radius / scale / shorter)),
      };
    },
  };
}

const clamp01 = (value: number) => Math.min(1, Math.max(0, value));

/** How many corrections there are, and how many of them the crop hides. */
export function countCorrections(geometry: Geometry, full: Size, circles: Circle[]) {
  const result = outputSize(geometry, full);
  const outside = circles.filter((circle) => {
    const [x, y] = resultPosition(geometry, full, circle.x * full.width, circle.y * full.height);
    return x < 0 || y < 0 || x > result.width || y > result.height;
  }).length;
  return { total: circles.length, outside };
}

/** "3 corrections", "3 corrections, 1 outside the crop", or none yet. */
export function countText({ total, outside }: { total: number; outside: number }): string {
  const corrections = t("red-eye-count", { total });
  return outside > 0 ? t("red-eye-count-outside", { corrections, outside }) : corrections;
}
