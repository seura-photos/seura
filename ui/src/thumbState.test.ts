import { describe, expect, it } from "vitest";
import { thumbLabel, type ThumbState } from "./thumbState";

const ready: ThumbState = { version: 1, cloudOnly: false, needsCodec: false, failed: false };

describe("thumbLabel", () => {
  it("shows the thumbnail when it's ready", () => {
    expect(thumbLabel(ready)).toBeNull();
    expect(thumbLabel(undefined)).toBeNull();
  });

  it("says when a photo is only in the cloud (LIB-9)", () => {
    expect(thumbLabel({ ...ready, cloudOnly: true })).toBe("In the cloud");
  });

  it("says when Windows lacks Microsoft's HEIC extensions (decision 0014)", () => {
    expect(thumbLabel({ ...ready, needsCodec: true })).toBe(
      "Needs Microsoft's HEIF and HEVC extensions",
    );
  });

  it("says when a photo can't be shown", () => {
    expect(thumbLabel({ ...ready, failed: true })).toBe("Can't show this photo");
  });
});
