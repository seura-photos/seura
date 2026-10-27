/** Space between cells, in logical pixels. */
export const GAP = 12;

/**
 * The grid's layout for a width and the slider's size. Cells are exactly
 * that size, so every move of the slider shows (BRW-3). A wider window gets
 * more columns, not bigger cells (DSP-3), and what's left after the last
 * whole column is spread between them, so rows line up at both edges
 * (Nevada, #73). Rows start in line with the heading, two gaps in, with the
 * same margin on the right. Only a window narrower than one cell shrinks it.
 */
export function gridLayout(width: number, size: number) {
  const inner = width - 4 * GAP;
  const cell = inner > 0 ? Math.min(size, inner) : size;
  const columns = Math.max(1, Math.floor((inner + GAP) / (cell + GAP)));
  const columnGap = columns > 1 ? (inner - columns * cell) / (columns - 1) : GAP;
  return { columns, cell, columnGap };
}
