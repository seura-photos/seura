/**
 * The size, in CSS pixels, at which a preview of `width` × `height` device
 * pixels fits inside an area, keeping its shape: at most its own size at
 * the screen's density `scale`, never larger than the area (DSP-1).
 */
export function fitSize(
  width: number,
  height: number,
  areaWidth: number,
  areaHeight: number,
  scale: number,
): { width: number; height: number } {
  const natural = { width: width / scale, height: height / scale };
  const shrink = Math.min(1, areaWidth / natural.width, areaHeight / natural.height);
  return { width: natural.width * shrink, height: natural.height * shrink };
}
