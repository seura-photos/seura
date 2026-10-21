// @vitest-environment node
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
// @ts-expect-error: a plain JavaScript build script, without type declarations.
import { generate } from "../../scripts/tokens.mjs";

describe("design tokens", () => {
  it("tokens.css is generated from design/tokens.json; run `npm run tokens` if this fails", () => {
    const committed = readFileSync(new URL("./tokens.css", import.meta.url), "utf8");
    expect(committed.replace(/\r\n/g, "\n")).toBe(generate());
  });
});
