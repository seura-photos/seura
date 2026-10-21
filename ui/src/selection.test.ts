import { describe, expect, it } from "vitest";
import { emptySelection, keepExisting, moveFocus, select, selectAll } from "./selection";

const order = [10, 11, 12, 13, 14];
const plain = { toggle: false, range: false };
const toggle = { toggle: true, range: false };
const range = { toggle: false, range: true };

describe("selection (BRW-4)", () => {
  it("selects one photo on a plain click", () => {
    const one = select(emptySelection, order, 12, plain);
    expect([...one.ids]).toEqual([12]);
    expect([...select(one, order, 14, plain).ids]).toEqual([14]);
  });

  it("adds and removes photos with Ctrl or Cmd", () => {
    let selection = select(emptySelection, order, 10, plain);
    selection = select(selection, order, 13, toggle);
    expect([...selection.ids].sort()).toEqual([10, 13]);
    selection = select(selection, order, 10, toggle);
    expect([...selection.ids]).toEqual([13]);
  });

  it("selects a range from the last photo clicked with Shift, either way", () => {
    let selection = select(emptySelection, order, 11, plain);
    selection = select(selection, order, 13, range);
    expect([...selection.ids].sort()).toEqual([11, 12, 13]);
    // The anchor stays, so the range can be changed.
    selection = select(selection, order, 10, range);
    expect([...selection.ids].sort()).toEqual([10, 11]);
  });

  it("adds a range to the selection with Ctrl and Shift", () => {
    let selection = select(emptySelection, order, 14, plain);
    selection = select(selection, order, 10, toggle);
    selection = select(selection, order, 11, { toggle: true, range: true });
    expect([...selection.ids].sort()).toEqual([10, 11, 14]);
  });

  it("selects all, and keeps only photos still there after a rescan", () => {
    const all = selectAll(order);
    expect(all.ids.size).toBe(5);
    const after = keepExisting(all, [10, 12]);
    expect([...after.ids].sort()).toEqual([10, 12]);
    expect(after.anchor).toBe(10);
  });
});

describe("keyboard focus in the grid (ACC-1)", () => {
  // 10 photos in 4 columns:  0 1 2 3 / 4 5 6 7 / 8 9
  const move = (index: number, key: string, ctrl = false) => moveFocus(index, key, ctrl, 4, 10, 2);

  it("moves by cell and by row", () => {
    expect(move(5, "ArrowRight")).toBe(6);
    expect(move(5, "ArrowLeft")).toBe(4);
    expect(move(5, "ArrowDown")).toBe(9);
    expect(move(5, "ArrowUp")).toBe(1);
    // No cell below: stay.
    expect(move(6, "ArrowDown")).toBe(6);
    expect(move(1, "ArrowUp")).toBe(1);
    expect(move(9, "ArrowRight")).toBe(9);
  });

  it("jumps to the row's ends, or the grid's with Ctrl", () => {
    expect(move(5, "Home")).toBe(4);
    expect(move(5, "End")).toBe(7);
    expect(move(8, "End")).toBe(9);
    expect(move(5, "Home", true)).toBe(0);
    expect(move(5, "End", true)).toBe(9);
  });

  it("pages by rows", () => {
    expect(move(1, "PageDown")).toBe(9);
    expect(move(9, "PageUp")).toBe(1);
  });

  it("ignores other keys", () => {
    expect(move(5, "x")).toBeNull();
  });
});
