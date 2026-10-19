import { describe, expect, it } from "vitest";
import { fitSize } from "./fit";

describe("fitSize", () => {
  it("shows a preview that fits at its own size", () => {
    expect(fitSize(1600, 900, 1000, 600, 2)).toEqual({ width: 800, height: 450 });
  });

  it("shrinks a portrait photo to the area's height, keeping its shape", () => {
    // A 3:4 preview decoded to the area's long side would spill past the top.
    expect(fitSize(1440, 1920, 1440, 810, 1)).toEqual({ width: 607.5, height: 810 });
  });

  it("shrinks a wide photo to the area's width", () => {
    expect(fitSize(3000, 1000, 1000, 800, 1)).toEqual({ width: 1000, height: 1000 / 3 });
  });
});
