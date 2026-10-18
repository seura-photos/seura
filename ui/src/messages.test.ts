import { describe, expect, it } from "vitest";
import { t } from "./messages";

describe("t", () => {
  it("returns the catalog text", () => {
    expect(t("app.name")).toBe("Seura Photos");
  });

  it("fills placeholders", () => {
    expect(t("app.version", { version: "1.2.3" })).toBe("Version 1.2.3");
  });

  it("leaves unknown placeholders visible", () => {
    expect(t("app.version")).toBe("Version {version}");
  });
});
