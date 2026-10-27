import { describe, expect, it } from "vitest";
import { gridLayout } from "./gridLayout";

describe("gridLayout (BRW-3, #73)", () => {
  it("keeps cells at the slider's size, and spreads what's left between them", () => {
    // 1000 wide: 952 inside the margins, so five 160 px cells with 12 px
    // gaps (848) fit, and the 104 left over widens the four gaps by 26 each.
    expect(gridLayout(1000, 160)).toEqual({ columns: 5, cell: 160, columnGap: 38 });
  });

  it("changes the size with every move of the slider", () => {
    expect(gridLayout(1000, 161).cell).toBe(161);
    expect(gridLayout(1000, 168).cell).toBe(168);
  });

  it("gives a wider window more columns, not bigger cells", () => {
    const narrow = gridLayout(1000, 160);
    const wide = gridLayout(1400, 160);
    expect(wide.cell).toBe(narrow.cell);
    expect(wide.columns).toBeGreaterThan(narrow.columns);
  });

  it("lines every row up with both edges", () => {
    for (const width of [700, 1000, 1333, 1920]) {
      const { columns, cell, columnGap } = gridLayout(width, 176);
      expect(columns * cell + (columns - 1) * columnGap).toBeCloseTo(width - 48);
    }
  });

  it("shrinks a cell only when the window is narrower than one", () => {
    expect(gridLayout(200, 256)).toEqual({ columns: 1, cell: 152, columnGap: 12 });
  });
});
