/**
 * Selection in the grid (BRW-4), following each platform's conventions:
 * click selects one photo, Ctrl-click (Cmd-click on a Mac) adds or removes
 * one, Shift-click selects the range from the last photo clicked. Photos
 * are kept by id, so the selection survives a rescan.
 */
export type Selection = { ids: Set<number>; anchor: number | null };

export const emptySelection: Selection = { ids: new Set(), anchor: null };

export type Modifiers = { toggle: boolean; range: boolean };

/** The selection after clicking (or moving the keyboard focus to) `id`. */
export function select(
  selection: Selection,
  order: number[],
  id: number,
  { toggle, range }: Modifiers,
): Selection {
  if (range && selection.anchor !== null) {
    const from = order.indexOf(selection.anchor);
    const to = order.indexOf(id);
    if (from >= 0 && to >= 0) {
      const [start, end] = from <= to ? [from, to] : [to, from];
      const ids = toggle ? new Set(selection.ids) : new Set<number>();
      for (const inRange of order.slice(start, end + 1)) ids.add(inRange);
      return { ids, anchor: selection.anchor };
    }
  }
  if (toggle) {
    const ids = new Set(selection.ids);
    if (!ids.delete(id)) ids.add(id);
    return { ids, anchor: id };
  }
  return { ids: new Set([id]), anchor: id };
}

export function selectAll(order: number[]): Selection {
  return { ids: new Set(order), anchor: order[0] ?? null };
}

/** Keeps only photos still in the folder, after a rescan. */
export function keepExisting(selection: Selection, order: number[]): Selection {
  const present = new Set(order);
  const ids = new Set([...selection.ids].filter((id) => present.has(id)));
  const anchor =
    selection.anchor !== null && present.has(selection.anchor) ? selection.anchor : null;
  return ids.size === selection.ids.size && anchor === selection.anchor
    ? selection
    : { ids, anchor };
}

/**
 * Where the keyboard focus goes in a grid of `count` photos in `columns`
 * columns (the ARIA grid pattern): arrows by one cell, Home and End to the
 * row's ends (with Ctrl, the grid's), Page Up and Down by `pageRows` rows.
 * `null` for keys that don't move it.
 */
export function moveFocus(
  index: number,
  key: string,
  ctrl: boolean,
  columns: number,
  count: number,
  pageRows: number,
): number | null {
  const last = count - 1;
  const clamp = (value: number) => Math.max(0, Math.min(last, value));
  const rowStart = index - (index % columns);
  switch (key) {
    case "ArrowRight":
      return clamp(index + 1);
    case "ArrowLeft":
      return clamp(index - 1);
    case "ArrowDown":
      return index + columns <= last ? index + columns : index;
    case "ArrowUp":
      return index - columns >= 0 ? index - columns : index;
    case "Home":
      return ctrl ? 0 : rowStart;
    case "End":
      return ctrl ? last : clamp(rowStart + columns - 1);
    case "PageDown":
      return clamp(index + columns * pageRows);
    case "PageUp":
      return clamp(index - columns * pageRows);
    default:
      return null;
  }
}
