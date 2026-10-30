import { describe, expect, it, vi } from "vitest";
import type { RenderSettings } from "../bindings";
import { orientedSize, PARAMS_SIZE, params, settingName, uprightRegion } from "./backend";
import layout from "./generated/params.json";

vi.mock("@tauri-apps/api/core", () => ({ convertFileSrc: (path: string) => path }));

// Every setting, each with its own value, exact in 32 bits, so a field
// written in the wrong place shows. Typed in full, so a setting added in
// Rust must be added here.
const settings: RenderSettings = {
  rotate: 90,
  flipH: true,
  flipV: false,
  straighten: -2.5,
  cropX: 0.125,
  cropY: 0.25,
  cropWidth: 0.5,
  cropHeight: 0.625,
  exposureEv: 0.75,
  temperature: 10,
  tint: -5,
  highlights: -30,
  shadows: 40,
  contrast: 15,
  saturation: -100,
  blackAndWhite: 100,
  sepia: 50,
  vignetteAmount: -20,
  vignetteSize: 70,
};

describe("the shaders' uniform block", () => {
  it("has a setting for every field of Params", () => {
    const drawing = ["output_srgb", "draw_pass", "tile_x", "tile_y", "tile_width", "tile_height"];
    const names = layout.fields
      .map((field) => field.name)
      .filter((name) => !drawing.includes(name));
    expect(names.map(settingName).sort()).toEqual(Object.keys(settings).sort());
  });

  it("writes each setting where naga lays it out", () => {
    const region = { x: 0.25, y: 0.5, width: 0.125, height: 0.375 };
    const view = new DataView(params(settings, true, 3, region));
    expect(view.byteLength).toBe(PARAMS_SIZE);
    const drawing: Record<string, number> = {
      output_srgb: 1,
      draw_pass: 3,
      tile_x: 0.25,
      tile_y: 0.5,
      tile_width: 0.125,
      tile_height: 0.375,
    };
    for (const field of layout.fields) {
      const value =
        field.name in drawing
          ? drawing[field.name]
          : settings[settingName(field.name) as keyof RenderSettings];
      const read =
        field.type === "u32"
          ? view.getUint32(field.offset, true)
          : view.getFloat32(field.offset, true);
      expect([field.name, read]).toEqual([field.name, Number(value)]);
    }
  });

  it("fills a whole number of 16-byte rows, as WebGL2's block layout needs", () => {
    expect(PARAMS_SIZE % 16).toBe(0);
    expect(PARAMS_SIZE).toBeGreaterThanOrEqual(layout.size);
  });

  it("encodes for the display unless asked for export", () => {
    const view = new DataView(params(settings, false));
    const output = layout.fields.find((field) => field.name === "output_srgb");
    expect(view.getUint32(output?.offset ?? -1, true)).toBe(0);
  });
});

describe("uprightRegion", () => {
  // Where upright pixel x, y of a width × height photo lands, as the edit
  // format defines seura.orient: turned clockwise, then flipped as seen.
  function drawnPixel(
    orient: { rotate: number; flipH: boolean; flipV: boolean },
    x: number,
    y: number,
    width: number,
    height: number,
  ) {
    let [u, v]: [number, number] =
      orient.rotate === 90
        ? [height - 1 - y, x]
        : orient.rotate === 180
          ? [width - 1 - x, height - 1 - y]
          : orient.rotate === 270
            ? [y, width - 1 - x]
            : [x, y];
    const drawn = orientedSize({ width, height }, orient);
    if (orient.flipH) u = drawn.width - 1 - u;
    if (orient.flipV) v = drawn.height - 1 - v;
    return [u, v] as const;
  }

  it("finds each drawn pixel where it came from, for every turn and flip (EDT-4)", () => {
    const [width, height] = [5, 3];
    for (const rotate of [0, 90, 180, 270]) {
      for (const [flipH, flipV] of [
        [false, false],
        [true, false],
        [false, true],
        [true, true],
      ] as const) {
        const orient = { rotate, flipH, flipV };
        const drawn = orientedSize({ width, height }, orient);
        for (let y = 0; y < height; y++) {
          for (let x = 0; x < width; x++) {
            const [u, v] = drawnPixel(orient, x, y, width, height);
            const at = uprightRegion(
              {
                x: u / drawn.width,
                y: v / drawn.height,
                width: 1 / drawn.width,
                height: 1 / drawn.height,
              },
              orient,
            );
            expect([at.x * width, at.y * height, at.width * width, at.height * height]).toEqual([
              expect.closeTo(x, 9),
              expect.closeTo(y, 9),
              expect.closeTo(1, 9),
              expect.closeTo(1, 9),
            ]);
          }
        }
      }
    }
  });
});
