import { describe, expect, it, vi } from "vitest";
import type { RenderSettings } from "../bindings";
import { PARAMS_SIZE, params, settingName } from "./backend";
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
    const names = layout.fields.map((field) => field.name).filter((name) => name !== "output_srgb");
    expect(names.map(settingName).sort()).toEqual(Object.keys(settings).sort());
  });

  it("writes each setting where naga lays it out", () => {
    const view = new DataView(params(settings, true));
    expect(view.byteLength).toBe(PARAMS_SIZE);
    for (const field of layout.fields) {
      const value =
        field.name === "output_srgb"
          ? 1
          : settings[settingName(field.name) as keyof RenderSettings];
      const read =
        field.type === "u32"
          ? view.getUint32(field.offset, true)
          : view.getFloat32(field.offset, true);
      expect([field.name, read]).toEqual([field.name, Number(value)]);
    }
  });

  it("encodes for the display unless asked for export", () => {
    const view = new DataView(params(settings, false));
    const output = layout.fields.find((field) => field.name === "output_srgb");
    expect(view.getUint32(output?.offset ?? -1, true)).toBe(0);
  });
});
