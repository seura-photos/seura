import { describe, expect, it } from "vitest";
import { type Geometry, UPRIGHT, uprightPosition } from "./preview/geometry";
import { BRUSH_DEFAULT, countCorrections, resultPosition, screenMapping, stepped } from "./redeye";

const full = { width: 4000, height: 3000 };

/** Every turn and flip, straightened and cropped. */
const geometries: Geometry[] = [0, 90, 180, 270].flatMap((rotate) =>
  [
    [false, false],
    [true, false],
    [false, true],
  ].map(([flipH, flipV]) => ({
    rotate,
    flipH: flipH ?? false,
    flipV: flipV ?? false,
    straighten: 7.5,
    cropX: 0.1,
    cropY: 0.15,
    cropWidth: 0.7,
    cropHeight: 0.6,
  })),
);

describe("red-eye's geometry", () => {
  it("places a point of the upright photo where the result samples it", () => {
    for (const geometry of [UPRIGHT, ...geometries]) {
      for (const [x, y] of [
        [1200, 900],
        [3100.5, 250.25],
        [10, 2990],
      ]) {
        const [rx, ry] = resultPosition(geometry, full, x ?? 0, y ?? 0);
        const [ux, uy] = uprightPosition(geometry, full, rx, ry);
        expect(ux).toBeCloseTo(x ?? 0, 6);
        expect(uy).toBeCloseTo(y ?? 0, 6);
      }
    }
  });

  it("maps circles to the stage and back, the radius scaled with the zoom", () => {
    const placed = { left: 40, top: 30, width: 1400, height: 900 };
    for (const geometry of geometries) {
      const mapping = screenMapping(geometry, full, placed);
      const circle = { x: 0.42, y: 0.37, radius: 0.01 };
      const shown = mapping.toScreen(circle);
      const back = mapping.fromScreen(shown.x, shown.y, shown.radius);
      expect(back.x).toBeCloseTo(circle.x, 9);
      expect(back.y).toBeCloseTo(circle.y, 9);
      expect(back.radius).toBeCloseTo(circle.radius, 9);
      expect(shown.inside).toBe(true);
    }
    // At 100% of a 4000-pixel-wide result, a pixel is a CSS pixel.
    const mapping = screenMapping(UPRIGHT, full, { left: 0, top: 0, width: 4000, height: 3000 });
    expect(mapping.toScreen({ x: 0.5, y: 0.5, radius: 0.01 })).toEqual({
      x: 2000,
      y: 1500,
      radius: 30,
      inside: true,
    });
  });

  it("counts the corrections the crop hides (wireframes, Red-eye mode)", () => {
    const cropped = { ...UPRIGHT, cropX: 0.5, cropWidth: 0.5 };
    const circles = [
      { x: 0.7, y: 0.5, radius: 0.01 },
      { x: 0.2, y: 0.5, radius: 0.01 },
      { x: 0.9, y: 0.2, radius: 0.01 },
    ];
    expect(countCorrections(cropped, full, circles)).toEqual({ total: 3, outside: 1 });
    expect(countCorrections(UPRIGHT, full, circles)).toEqual({ total: 3, outside: 0 });
  });

  it("steps a brush or circle by a quarter each way, within limits", () => {
    expect(stepped(BRUSH_DEFAULT, 1)).toBe(30);
    expect(stepped(BRUSH_DEFAULT, -1)).toBeCloseTo(19.2);
    expect(stepped(BRUSH_DEFAULT, -20)).toBe(6);
    expect(stepped(BRUSH_DEFAULT, 30)).toBe(240);
  });
});
