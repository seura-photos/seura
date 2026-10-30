import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import {
  aspectOf,
  choiceOf,
  fits,
  fitScale,
  moveBox,
  ratioBox,
  reduced,
  resizeBox,
  shrinkToFit,
  turnEdges,
  wholeFrame,
} from "./crop";

type FitCase = {
  frame: [number, number];
  angle: number;
  crop: { x: number; y: number; width: number; height: number };
  scale: number;
};

// The edit format's vectors for keeping a straightened photo's corners out
// of view (formulas appendix, geometry.md). Tests run from ui/.
const vectors = JSON.parse(
  readFileSync("../third_party/edit-format/tests/formulas/geometry/crop-inside.json", "utf-8"),
) as { cases: FitCase[] };

const frame = { width: 6000, height: 4000 };

describe("crop mode's box", () => {
  it("shrinks to fit as the appendix's vectors do (EDT-3)", () => {
    for (const test of vectors.cases) {
      const [width, height] = test.frame;
      const box = {
        left: test.crop.x * width,
        top: test.crop.y * height,
        right: (test.crop.x + test.crop.width) * width,
        bottom: (test.crop.y + test.crop.height) * height,
      };
      expect(fitScale(box, { width, height }, test.angle)).toBeCloseTo(test.scale, 6);
    }
  });

  it("grows back as the angle returns, up to the box the user set (EDT-3)", () => {
    const set = wholeFrame(frame);
    const tilted = shrinkToFit(set, frame, 8);
    expect(tilted.right - tilted.left).toBeLessThan(6000);
    expect(fits(tilted, frame, 8)).toBe(true);
    expect(shrinkToFit(set, frame, 0)).toEqual(set);
  });

  it("slides along the turned photo's edge rather than stopping (wireframes §6)", () => {
    const box = { left: 2000, top: 1000, right: 4000, bottom: 3000 };
    // Dragged far down and to the right, unstraightened: it ends in the
    // corner, having slid along each edge.
    const moved = moveBox(box, 5000, 5000, frame, 0);
    expect(moved.right).toBeCloseTo(6000, 3);
    expect(moved.bottom).toBeCloseTo(4000, 3);
    // Straightened, it still slides, and stays on the photo.
    const tilted = shrinkToFit(box, frame, 10);
    const slid = moveBox(tilted, 3000, 0, frame, 10);
    expect(fits(slid, frame, 10)).toBe(true);
    expect(slid.left).toBeGreaterThan(tilted.left);
  });

  it("shrinks a box with a ratio from a corner, one direction at a time, as keys move it", () => {
    const box = { left: 1000, top: 1000, right: 4000, bottom: 3000 };
    for (const [dx, dy] of [
      [-100, 0],
      [0, -100],
    ] as const) {
      const smaller = resizeBox(box, "bottom-right", dx, dy, 1.5, frame, 0, 16);
      expect(smaller.right - smaller.left).toBeLessThan(3000);
      expect((smaller.right - smaller.left) / (smaller.bottom - smaller.top)).toBeCloseTo(1.5, 6);
    }
    const shrunk = resizeBox(box, "top-left", 100, 0, 1.5, frame, 0, 16);
    expect(shrunk.right - shrunk.left).toBeCloseTo(2900, 6);
  });

  it("slides to where a straightened photo's slanted edges stop it", () => {
    const angle = 10;
    const box = { left: 2750, top: 1750, right: 3250, bottom: 2250 };
    const moved = moveBox(box, 5000, 5000, frame, angle);
    expect(fits(moved, frame, angle)).toBe(true);
    // In the corner: no further across or down.
    const across = moveBox(moved, 2, 0, frame, angle);
    const down = moveBox(moved, 0, 2, frame, angle);
    expect(across.left - moved.left).toBeLessThan(0.01);
    expect(down.top - moved.top).toBeLessThan(0.01);
  });

  it("keeps a ratio from a corner, and its smallest size", () => {
    const box = { left: 1000, top: 1000, right: 4000, bottom: 3000 };
    const larger = resizeBox(box, "bottom-right", 600, 0, 1.5, frame, 0, 16);
    expect((larger.right - larger.left) / (larger.bottom - larger.top)).toBeCloseTo(1.5, 6);
    expect([larger.left, larger.top]).toEqual([1000, 1000]);
    const tiny = resizeBox(box, "bottom-right", -9000, -9000, null, frame, 0, 16);
    expect([tiny.right - tiny.left, tiny.bottom - tiny.top]).toEqual([16, 16]);
  });

  it("sets the largest box of a ratio, centered where the box was", () => {
    const square = ratioBox(wholeFrame(frame), 1, frame, 0);
    expect(square).toEqual({ left: 1000, top: 0, right: 5000, bottom: 4000 });
    const tilted = ratioBox(wholeFrame(frame), 1.5, frame, 5);
    expect(fits(tilted, frame, 5)).toBe(true);
    expect((tilted.right - tilted.left) / (tilted.bottom - tilted.top)).toBeCloseTo(1.5, 6);
  });

  it("stores Original as the photo's own ratio, and shows it as Original again", () => {
    expect(reduced(4032, 3024)).toBe("4:3");
    expect(aspectOf("original", false, { width: 6000, height: 3376 })).toBe("375:211");
    expect(aspectOf("3:2", true, frame)).toBe("2:3");
    expect(aspectOf("freeform", false, frame)).toBeNull();
    expect(choiceOf("3:2", frame)).toEqual({ choice: "original", portrait: false });
    expect(choiceOf("2:3", frame)).toEqual({ choice: "original", portrait: true });
    expect(choiceOf("9:16", frame)).toEqual({ choice: "16:9", portrait: true });
    expect(choiceOf(null, frame)).toEqual({ choice: "freeform", portrait: false });
  });

  it("turns with the photo, as the core turns a saved crop (EDT-4)", () => {
    // A quarter turn right: the core's (1 − y − h, x, h, w) in fractions.
    const box = { left: 750, top: 1000, right: 3750, bottom: 2500 };
    expect(turnEdges(box, frame, true)).toEqual({
      left: 1500,
      top: 750,
      right: 3000,
      bottom: 3750,
    });
    expect(turnEdges(turnEdges(box, frame, true), { width: 4000, height: 6000 }, false)).toEqual(
      box,
    );
  });
});
