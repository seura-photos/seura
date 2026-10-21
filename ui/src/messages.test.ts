import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { formatBytes, formatDateTime, formatNumber, setRegion } from "./format";
import { hasMessage, setPseudo, t } from "./messages";
import { pseudoTranslate } from "./pseudo";

afterEach(() => {
  setRegion("en-US");
  setPseudo(false);
});

describe("messages (ACC-5)", () => {
  it("returns the catalog text", () => {
    expect(t("app-name")).toBe("Seura Photos");
  });

  it("fills variables", () => {
    expect(t("app-version", { version: "1.2.3" })).toBe("Version 1.2.3");
  });

  it("chooses the plural form from the number", () => {
    expect(t("library-photo-count", { count: 1 })).toBe("1 photo");
    expect(t("library-photo-count", { count: 214 })).toBe("214 photos");
    expect(t("library-photo-count", { count: 0 })).toBe("0 photos");
  });

  it("formats numbers in the region while English picks the words (ACC-6)", () => {
    setRegion("de-DE");
    expect(t("library-photo-count", { count: 1214 })).toBe("1.214 photos");
    setRegion("en-US");
    expect(t("library-photo-count", { count: 1214 })).toBe("1,214 photos");
  });

  it("shows an unknown id rather than failing", () => {
    expect(t("no-such-message")).toBe("no-such-message");
  });

  it("has every message the interface uses", () => {
    const sources: string[] = [];
    const walk = (folder: string) => {
      for (const entry of readdirSync(folder, { withFileTypes: true })) {
        const path = join(folder, entry.name);
        if (entry.isDirectory()) walk(path);
        else if (/\.tsx?$/.test(entry.name) && !/\.test\./.test(entry.name)) {
          sources.push(readFileSync(path, "utf8"));
        }
      }
    };
    walk("src");
    const ids = new Set(
      sources.flatMap((source) =>
        [...source.matchAll(/\bt\("([a-z][a-z0-9-]*)"/g)].map((m) => m[1]),
      ),
    );
    expect(ids.size).toBeGreaterThan(20);
    const missing = [...ids].filter((id) => id !== undefined && !hasMessage(id));
    expect(missing).toEqual([]);
  });
});

describe("pseudo-translation (ACC-7)", () => {
  it("lengthens text by about 40% and accents it", () => {
    const english = "Choose a folder on the left.";
    const pseudo = pseudoTranslate(english);
    expect([...pseudo].length).toBeGreaterThanOrEqual(english.length * 1.3);
    expect(pseudo).not.toContain("o");
    expect(pseudo).toContain("ö");
  });

  it("transforms the catalog's text but not the values put into it", () => {
    setPseudo(true);
    const text = t("library-already-included", { folder: "Pictures" });
    expect(text).toContain("Pictures");
    expect(text).not.toContain("folder");
    expect(text.length).toBeGreaterThan(
      "That folder is already in your library, in Pictures.".length,
    );
  });
});

describe("regional formats (ACC-6)", () => {
  it("formats numbers and sizes by region", () => {
    setRegion("de-DE");
    expect(formatNumber(1234.5)).toBe("1.234,5");
    expect(formatBytes(2_100_000)).toBe("2,1 MB");
    setRegion("en-US");
    expect(formatBytes(2_100_000)).toBe("2.1 MB");
    expect(formatBytes(512)).toBe("512 bytes");
  });

  it("formats a photo's date as recorded, by region", () => {
    // 2026-07-12 18:42 on the camera's clock.
    const taken = Date.UTC(2026, 6, 12, 18, 42) / 1000;
    setRegion("de-DE");
    expect(formatDateTime(taken)).toBe("12.07.2026, 18:42");
    setRegion("en-US");
    // Whether a narrow no-break space comes before "PM" depends on the
    // Unicode data version, which differs between Node and the web views.
    expect(formatDateTime(taken).replace(/\s/g, " ")).toBe("Jul 12, 2026, 6:42 PM");
  });

  it("falls back for a tag that isn't valid", () => {
    setRegion("not a tag!");
    expect(formatNumber(1)).toBe("1");
  });
});
