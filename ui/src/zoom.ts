/**
 * The Viewer's zoom (BRW-5): how large the photo is shown and which part
 * is in the middle. Sizes are in device pixels, so 100% shows one of the
 * photo's pixels on one of the screen's (DSP-1).
 */

/** The photo's own upright size, and the stage's, both in pixels. */
export type Size = { width: number; height: number };

/** Screen pixels per photo pixel, and the photo point at the stage's center. */
export type View = { zoom: number; x: number; y: number };

/** A part of the photo, in its own pixels. */
export type Region = { x: number; y: number; width: number; height: number };

/** Zoom levels for the − and + buttons and keys. */
export const STEPS = [0.125, 0.25, 0.5, 1, 2, 4];
export const MAX_ZOOM = 4;

/** Fitted to the stage, never enlarged. */
export function fitZoom(photo: Size, stage: Size): number {
  return Math.min(1, stage.width / photo.width, stage.height / photo.height);
}

export function fitView(photo: Size, stage: Size): View {
  return { zoom: fitZoom(photo, stage), x: photo.width / 2, y: photo.height / 2 };
}

/** Within the zoom range, and with the photo kept on the stage: centered
 * where it's smaller than the stage, its edges no further in than the
 * stage's where it's larger. */
export function clampView(view: View, photo: Size, stage: Size): View {
  const zoom = Math.min(MAX_ZOOM, Math.max(fitZoom(photo, stage), view.zoom));
  const axis = (center: number, length: number, room: number) => {
    const half = room / 2 / zoom;
    return length <= 2 * half ? length / 2 : Math.min(length - half, Math.max(half, center));
  };
  return {
    zoom,
    x: axis(view.x, photo.width, stage.width),
    y: axis(view.y, photo.height, stage.height),
  };
}

/** Zooms to `zoom`, keeping the photo point under `point` (device pixels
 * from the stage's center) where it is, as wheel and pinch zoom do. */
export function zoomAt(
  view: View,
  zoom: number,
  point: { x: number; y: number },
  photo: Size,
  stage: Size,
): View {
  const target = { x: view.x + point.x / view.zoom, y: view.y + point.y / view.zoom };
  const next = clampView({ ...view, zoom }, photo, stage);
  return clampView(
    { zoom: next.zoom, x: target.x - point.x / next.zoom, y: target.y - point.y / next.zoom },
    photo,
    stage,
  );
}

/** The next level in `direction`: up from fit to the first step above it,
 * down to the step below, or to fit. */
export function step(view: View, direction: 1 | -1, photo: Size, stage: Size): number {
  const fit = fitZoom(photo, stage);
  if (direction > 0) return STEPS.find((level) => level > view.zoom * 1.01) ?? MAX_ZOOM;
  const lower = [...STEPS].reverse().find((level) => level < view.zoom / 1.01);
  return lower === undefined || lower < fit ? fit : lower;
}

export function isFit(view: View, photo: Size, stage: Size): boolean {
  return view.zoom <= fitZoom(photo, stage) * 1.001;
}

/** The part of the photo on the stage, with `margin` (a fraction of the
 * stage) around it so small pans don't reveal its edges, in whole pixels
 * and within the photo. */
export function visibleRegion(view: View, photo: Size, stage: Size, margin: number): Region {
  const halfWidth = (stage.width * (0.5 + margin)) / view.zoom;
  const halfHeight = (stage.height * (0.5 + margin)) / view.zoom;
  const left = Math.max(0, Math.floor(view.x - halfWidth));
  const top = Math.max(0, Math.floor(view.y - halfHeight));
  const right = Math.min(photo.width, Math.ceil(view.x + halfWidth));
  const bottom = Math.min(photo.height, Math.ceil(view.y + halfHeight));
  return { x: left, y: top, width: right - left, height: bottom - top };
}
