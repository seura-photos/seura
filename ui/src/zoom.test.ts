import { describe, expect, it } from "vitest";
import { clampView, fitView, fitZoom, isFit, step, visibleRegion, zoomAt } from "./zoom";

const photo = { width: 4000, height: 3000 };
const stage = { width: 2000, height: 1000 };

describe("zoom (BRW-5)", () => {
  it("fits the photo to the stage, never enlarging it", () => {
    expect(fitZoom(photo, stage)).toBeCloseTo(1 / 3);
    expect(fitZoom({ width: 400, height: 300 }, stage)).toBe(1);
  });

  it("steps up from fit and back down to it", () => {
    const fit = fitView(photo, stage);
    expect(step(fit, 1, photo, stage)).toBe(0.5);
    expect(step({ ...fit, zoom: 1 }, 1, photo, stage)).toBe(2);
    expect(step({ ...fit, zoom: 1 }, -1, photo, stage)).toBe(0.5);
    expect(step({ ...fit, zoom: 0.5 }, -1, photo, stage)).toBeCloseTo(1 / 3);
    expect(step({ ...fit, zoom: 4 }, 1, photo, stage)).toBe(4);
  });

  it("keeps the photo on the stage", () => {
    // At 100%, the stage shows 2000 × 1000 of the photo: the center stays
    // at least 1000 and 500 from the edges.
    expect(clampView({ zoom: 1, x: 0, y: 5000 }, photo, stage)).toEqual({
      zoom: 1,
      x: 1000,
      y: 2500,
    });
    // Narrower than the stage, it stays centered across.
    expect(clampView({ zoom: 0.4, x: 0, y: 0 }, photo, stage).x).toBe(2000);
    // Never below fit or above the largest zoom.
    expect(clampView({ zoom: 0.01, x: 0, y: 0 }, photo, stage).zoom).toBeCloseTo(1 / 3);
    expect(clampView({ zoom: 99, x: 2000, y: 1500 }, photo, stage).zoom).toBe(4);
    expect(isFit(fitView(photo, stage), photo, stage)).toBe(true);
  });

  it("zooms about the pointer, keeping the point under it still", () => {
    const view = { zoom: 1, x: 2000, y: 1500 };
    // 200 device pixels right of center is photo x 2200; after zooming to 2
    // it must still be 200 pixels right of center.
    const zoomed = zoomAt(view, 2, { x: 200, y: 0 }, photo, stage);
    expect(zoomed.zoom).toBe(2);
    expect(zoomed.x + 200 / zoomed.zoom).toBeCloseTo(2200);
  });

  it("asks for the part on the stage with a margin, within the photo", () => {
    expect(visibleRegion({ zoom: 1, x: 2000, y: 1500 }, photo, stage, 0)).toEqual({
      x: 1000,
      y: 1000,
      width: 2000,
      height: 1000,
    });
    expect(visibleRegion({ zoom: 1, x: 1000, y: 500 }, photo, stage, 0.25)).toEqual({
      x: 0,
      y: 0,
      width: 2500,
      height: 1250,
    });
  });
});
