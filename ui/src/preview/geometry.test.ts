import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import {
  drawnSize,
  type Geometry,
  outputSize,
  sourceRegion,
  UPRIGHT,
  uprightPosition,
} from "./geometry";

type PositionCase = {
  upright: [number, number];
  orient: { rotate: number; flipH: boolean; flipV: boolean };
  angle: number;
  crop: { x: number; y: number; width: number; height: number };
  outputSize: [number, number];
  pixels: [number, number][];
  positions: [number, number][];
};

// The edit format's vectors for the geometry stage (formulas appendix,
// geometry.md), which the core and the shader are checked against too.
const positions = JSON.parse(
  // Tests run from ui/.
  readFileSync("../third_party/edit-format/tests/formulas/geometry/positions.json", "utf-8"),
) as { cases: PositionCase[] };

function geometryOf(test: PositionCase): Geometry {
  return {
    ...test.orient,
    straighten: test.angle,
    cropX: test.crop.x,
    cropY: test.crop.y,
    cropWidth: test.crop.width,
    cropHeight: test.crop.height,
  };
}

describe("geometry", () => {
  it("places each pixel as the appendix's vectors do", () => {
    for (const test of positions.cases) {
      const geometry = geometryOf(test);
      const full = { width: test.upright[0], height: test.upright[1] };
      const size = outputSize(geometry, full);
      expect([size.width, size.height]).toEqual(test.outputSize);
      test.pixels.forEach(([i, j], k) => {
        const found = uprightPosition(geometry, full, i + 0.5, j + 0.5);
        const [x, y] = test.positions[k] ?? [NaN, NaN];
        expect(found[0]).toBeCloseTo(x, 6);
        expect(found[1]).toBeCloseTo(y, 6);
      });
    }
  });

  it("cuts a part that holds every position it samples, with room for the filter", () => {
    const geometry: Geometry = {
      rotate: 90,
      flipH: true,
      flipV: false,
      straighten: -12.5,
      cropX: 0.2,
      cropY: 0.15,
      cropWidth: 0.6,
      cropHeight: 0.55,
    };
    const full = { width: 700, height: 500 };
    const view = { x: 40, y: 30, width: 100, height: 80 };
    const part = sourceRegion(geometry, full, view, 3);
    for (let y = view.y; y < view.y + view.height; y++) {
      for (let x = view.x; x < view.x + view.width; x++) {
        const [u, v] = uprightPosition(geometry, full, x + 0.5, y + 0.5);
        expect(u - 2).toBeGreaterThanOrEqual(part.x);
        expect(u + 2).toBeLessThanOrEqual(part.x + part.width);
        expect(v - 2).toBeGreaterThanOrEqual(part.y);
        expect(v + 2).toBeLessThanOrEqual(part.y + part.height);
      }
    }
    expect(part.x + part.width).toBeLessThanOrEqual(700);
  });

  it("sizes a canvas as the result, at the image's resolution", () => {
    const full = { width: 4000, height: 3000 };
    // A preview a tenth of the photo's size, cropped to its left half.
    const cropped = { ...UPRIGHT, cropWidth: 0.5 };
    expect(drawnSize({ width: 400, height: 300 }, cropped, { full })).toEqual({
      width: 200,
      height: 300,
    });
    // On its side.
    expect(drawnSize({ width: 400, height: 300 }, { ...UPRIGHT, rotate: 90 }, { full })).toEqual({
      width: 300,
      height: 400,
    });
    // A zoomed-in part: a quarter of the result, from an image at half the
    // photo's resolution.
    expect(
      drawnSize({ width: 1000, height: 750 }, UPRIGHT, {
        full,
        part: { x: 0, y: 0, width: 0.5, height: 0.5 },
        view: { x: 0, y: 0, width: 0.25, height: 0.25 },
      }),
    ).toEqual({ width: 500, height: 375 });
  });
});
