import type { Size } from "./preview/geometry";

/**
 * Crop mode's box (EDT-2, EDT-3; wireframes, "Crop and straighten"), in
 * pixels of the oriented photo's frame (edit format, "Geometry frame and
 * sampling"): the photo turns inside the frame by the straighten angle, and
 * the box is an upright rectangle that must stay on the turned photo.
 */

/** A box by its edges, in frame pixels. */
export type Edges = { left: number; top: number; right: number; bottom: number };

/** The ratio choices, in the panel's order. */
export const RATIOS = ["original", "freeform", "1:1", "4:3", "3:2", "16:9", "5:4"] as const;
export type Ratio = (typeof RATIOS)[number];

/** The smallest box, in the photo's pixels. */
export const MINIMUM = 16;

/** How close a corner may come to the turned photo's edge and still count
 * as on it, so a box that touches an edge exactly isn't refused. */
const SLACK = 1e-6;

/** Where frame point `x`, `y` was before the photo turned `angle` degrees
 * clockwise about the frame's center. */
function unturn(frame: Size, angle: number, x: number, y: number): [number, number] {
  const theta = (angle * Math.PI) / 180;
  const [cx, cy] = [frame.width / 2, frame.height / 2];
  const [dx, dy] = [x - cx, y - cy];
  return [
    cx + dx * Math.cos(theta) + dy * Math.sin(theta),
    cy - dx * Math.sin(theta) + dy * Math.cos(theta),
  ];
}

/** Whether frame point `x`, `y` is on the turned photo, and in the frame. */
function covered(frame: Size, angle: number, x: number, y: number): boolean {
  const [u, v] = unturn(frame, angle, x, y);
  const inside = (value: number, limit: number) => value >= -SLACK && value <= limit + SLACK;
  return (
    inside(x, frame.width) &&
    inside(y, frame.height) &&
    inside(u, frame.width) &&
    inside(v, frame.height)
  );
}

/** Whether every corner of `box` is on the turned photo: it's convex, so
 * then all of the box is. */
export function fits(box: Edges, frame: Size, angle: number): boolean {
  return [
    [box.left, box.top],
    [box.right, box.top],
    [box.left, box.bottom],
    [box.right, box.bottom],
  ].every(([x, y]) => covered(frame, angle, x ?? 0, y ?? 0));
}

const width = (box: Edges) => box.right - box.left;
const height = (box: Edges) => box.bottom - box.top;

/** `box` scaled by `scale` about its center. */
function scaled(box: Edges, scale: number): Edges {
  const [cx, cy] = [(box.left + box.right) / 2, (box.top + box.bottom) / 2];
  const [a, b] = [(width(box) / 2) * scale, (height(box) / 2) * scale];
  return { left: cx - a, top: cy - b, right: cx + a, bottom: cy + b };
}

/**
 * The largest scale, at most 1, of `box` about its own center that keeps
 * it on the photo turned `angle` degrees (formulas appendix, geometry.md,
 * "Keeping the corners out of view"); 0 if its center isn't on it.
 */
export function fitScale(box: Edges, frame: Size, angle: number): number {
  const theta = (angle * Math.PI) / 180;
  const [cos, sin] = [Math.cos(theta), Math.sin(theta)];
  const back = (dx: number, dy: number): [number, number] => [
    dx * cos + dy * sin,
    -dx * sin + dy * cos,
  ];
  const [halfWidth, halfHeight] = [frame.width / 2, frame.height / 2];
  const [ex, ey] = back(
    (box.left + box.right) / 2 - halfWidth,
    (box.top + box.bottom) / 2 - halfHeight,
  );
  if (Math.abs(ex) > halfWidth || Math.abs(ey) > halfHeight) return 0;
  let scale = 1;
  for (const sx of [-1, 1]) {
    for (const sy of [-1, 1]) {
      const [gx, gy] = back((sx * width(box)) / 2, (sy * height(box)) / 2);
      for (const [g, e, limit] of [
        [gx, ex, halfWidth],
        [gy, ey, halfHeight],
      ] as const) {
        if (g !== 0) scale = Math.min(scale, (limit - Math.sign(g) * e) / Math.abs(g));
      }
    }
  }
  return Math.max(scale, 0);
}

/**
 * The box shown for the box the user set, at `angle`: shrunk about its
 * center only as much as it must be (EDT-3), so it grows back as the angle
 * returns. One whose center has turned off the photo is moved to the
 * frame's center.
 */
export function shrinkToFit(box: Edges, frame: Size, angle: number): Edges {
  const scale = fitScale(box, frame, angle);
  if (scale > 0) return scaled(box, scale);
  const centered: Edges = {
    left: frame.width / 2 - width(box) / 2,
    top: frame.height / 2 - height(box) / 2,
    right: frame.width / 2 + width(box) / 2,
    bottom: frame.height / 2 + height(box) / 2,
  };
  return scaled(centered, fitScale(centered, frame, angle));
}

/**
 * The box closest to `wanted` that fits, starting from `from`, which fits:
 * each group of edges moves as far toward `wanted` as it can, in turn, as
 * a corner's two edges do when it's dragged.
 */
export function slide(
  from: Edges,
  wanted: Edges,
  frame: Size,
  angle: number,
  groups: (keyof Edges)[][],
): Edges {
  // Again while it gains anything, so the order of the groups doesn't
  // matter: moving down can make room to move further across.
  let box = from;
  for (let pass = 0; pass < 8; pass++) {
    const before = box;
    box = slideOnce(box, wanted, frame, angle, groups);
    if (groups.length === 1 || JSON.stringify(box) === JSON.stringify(before)) break;
  }
  return box;
}

function slideOnce(
  from: Edges,
  wanted: Edges,
  frame: Size,
  angle: number,
  groups: (keyof Edges)[][],
): Edges {
  let box = from;
  for (const group of groups) {
    const at = (t: number): Edges => {
      const next = { ...box };
      for (const edge of group) next[edge] = box[edge] + (wanted[edge] - box[edge]) * t;
      return next;
    };
    if (fits(at(1), frame, angle)) {
      box = at(1);
      continue;
    }
    let [low, high] = [0, 1];
    for (let i = 0; i < 30; i++) {
      const middle = (low + high) / 2;
      if (fits(at(middle), frame, angle)) low = middle;
      else high = middle;
    }
    box = at(low);
  }
  return box;
}

type Point = [number, number];

/**
 * Where the center of a box with half-sizes `a` and `b` can be, so that
 * all of it is on the photo turned `angle` degrees and in the frame: a
 * convex polygon, as half-planes n · c ≤ d, each the region's own edge
 * moved in by how far the box reaches toward it.
 */
function centers(frame: Size, angle: number, a: number, b: number): Point[] {
  const theta = (angle * Math.PI) / 180;
  const [cos, sin] = [Math.cos(theta), Math.sin(theta)];
  const [cx, cy] = [frame.width / 2, frame.height / 2];
  // The turned photo's sides: along u, within W / 2 of the center; along
  // v, within H / 2 (the turn back of `unturn`).
  const u: Point = [cos, sin];
  const v: Point = [-sin, cos];
  const dot = (n: Point, p: Point) => n[0] * p[0] + n[1] * p[1];
  const sides: [Point, number][] = [
    [[-1, 0], 0],
    [[1, 0], frame.width],
    [[0, -1], 0],
    [[0, 1], frame.height],
    [u, frame.width / 2 + dot(u, [cx, cy])],
    [[-u[0], -u[1]], frame.width / 2 - dot(u, [cx, cy])],
    [v, frame.height / 2 + dot(v, [cx, cy])],
    [[-v[0], -v[1]], frame.height / 2 - dot(v, [cx, cy])],
  ];
  // A square larger than any frame, cut by each side in turn.
  const far = 1e7;
  let polygon: Point[] = [
    [-far, -far],
    [far, -far],
    [far, far],
    [-far, far],
  ];
  for (const [n, d] of sides) {
    const limit = d - Math.abs(n[0]) * a - Math.abs(n[1]) * b - 1e-7;
    const inside = (p: Point) => dot(n, p) <= limit;
    const next: Point[] = [];
    polygon.forEach((p, k) => {
      const q = polygon[(k + 1) % polygon.length] ?? p;
      if (inside(p)) next.push(p);
      if (inside(p) !== inside(q)) {
        const t = (limit - dot(n, p)) / (dot(n, q) - dot(n, p));
        next.push([p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t]);
      }
    });
    polygon = next;
  }
  return polygon;
}

/** The point of a convex polygon nearest `point`. */
function nearest(polygon: Point[], point: Point): Point | null {
  if (polygon.length === 0) return null;
  const cross = (p: Point, q: Point, r: Point) =>
    (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]);
  // Inside when on the same side of every edge.
  const sides = polygon.map((p, k) => cross(p, polygon[(k + 1) % polygon.length] ?? p, point));
  if (sides.every((side) => side >= 0) || sides.every((side) => side <= 0)) return point;
  let best: Point = polygon[0] ?? point;
  let distance = Infinity;
  polygon.forEach((p, k) => {
    const q = polygon[(k + 1) % polygon.length] ?? p;
    const [ex, ey] = [q[0] - p[0], q[1] - p[1]];
    const length = ex * ex + ey * ey;
    const t =
      length === 0
        ? 0
        : Math.min(1, Math.max(0, ((point[0] - p[0]) * ex + (point[1] - p[1]) * ey) / length));
    const on: Point = [p[0] + ex * t, p[1] + ey * t];
    const away = Math.hypot(on[0] - point[0], on[1] - point[1]);
    if (away < distance) [best, distance] = [on, away];
  });
  return best;
}

/**
 * `box` moved by `dx`, `dy`, or as near there as it can be while it's on
 * the turned photo: against an edge it slides along it, a slanted one too,
 * and where it ends depends only on where the pointer is, not the way it
 * came.
 */
export function moveBox(box: Edges, dx: number, dy: number, frame: Size, angle: number): Edges {
  const [a, b] = [width(box) / 2, height(box) / 2];
  const wanted: Point = [(box.left + box.right) / 2 + dx, (box.top + box.bottom) / 2 + dy];
  const center = nearest(centers(frame, angle, a, b), wanted);
  if (!center) return box;
  return { left: center[0] - a, top: center[1] - b, right: center[0] + a, bottom: center[1] + b };
}

/** A corner or an edge of the box, by the edges it moves. */
export type Handle =
  "top-left" | "top-right" | "bottom-left" | "bottom-right" | "top" | "right" | "bottom" | "left";

const MOVES: Record<Handle, (keyof Edges)[]> = {
  "top-left": ["top", "left"],
  "top-right": ["top", "right"],
  "bottom-left": ["bottom", "left"],
  "bottom-right": ["bottom", "right"],
  top: ["top"],
  right: ["right"],
  bottom: ["bottom"],
  left: ["left"],
};

/**
 * `box` with `handle` dragged by `dx`, `dy`. Freeform, each edge it holds
 * moves on its own. With a ratio (width over height, in pixels), a corner
 * keeps the opposite corner where it is, and an edge resizes both sides
 * about the middle. The box keeps its smallest size, and stops at the
 * turned photo's edge, sliding along it.
 */
export function resizeBox(
  box: Edges,
  handle: Handle,
  dx: number,
  dy: number,
  ratio: number | null,
  frame: Size,
  angle: number,
  minimum: number,
): Edges {
  const moves = MOVES[handle];
  const wanted = { ...box };
  if (moves.includes("left")) wanted.left = Math.min(box.left + dx, box.right - minimum);
  if (moves.includes("right")) wanted.right = Math.max(box.right + dx, box.left + minimum);
  if (moves.includes("top")) wanted.top = Math.min(box.top + dy, box.bottom - minimum);
  if (moves.includes("bottom")) wanted.bottom = Math.max(box.bottom + dy, box.top + minimum);
  if (ratio === null)
    return slide(
      box,
      wanted,
      frame,
      angle,
      moves.map((edge) => [edge]),
    );

  let next: Edges;
  if (moves.length === 2) {
    // The direction that moved more sets the size, so a corner can shrink
    // the box as well as grow it, from the keyboard one direction at a time.
    const widthChange = Math.abs(width(wanted) - width(box));
    const heightChange = Math.abs(height(wanted) - height(box)) * ratio;
    const across = Math.max(
      widthChange >= heightChange ? width(wanted) : height(wanted) * ratio,
      minimum,
      minimum * ratio,
    );
    const down = across / ratio;
    next = {
      left: moves.includes("left") ? box.right - across : box.left,
      right: moves.includes("left") ? box.right : box.left + across,
      top: moves.includes("top") ? box.bottom - down : box.top,
      bottom: moves.includes("top") ? box.bottom : box.top + down,
    };
  } else {
    const sideways = moves.includes("left") || moves.includes("right");
    const across = Math.max(
      sideways ? width(wanted) : height(wanted) * ratio,
      minimum,
      minimum * ratio,
    );
    const down = across / ratio;
    const [cx, cy] = [(box.left + box.right) / 2, (box.top + box.bottom) / 2];
    next = sideways
      ? { left: wanted.left, right: wanted.right, top: cy - down / 2, bottom: cy + down / 2 }
      : { left: cx - across / 2, right: cx + across / 2, top: wanted.top, bottom: wanted.bottom };
    if (moves.includes("left")) next.left = box.right - across;
    if (moves.includes("right")) next.right = box.left + across;
    if (moves.includes("top")) next.top = box.bottom - down;
    if (moves.includes("bottom")) next.bottom = box.top + down;
  }
  // Every edge moves together, so the ratio holds.
  return slide(box, next, frame, angle, [["left", "top", "right", "bottom"]]);
}

/** The whole frame. */
export function wholeFrame(frame: Size): Edges {
  return { left: 0, top: 0, right: frame.width, bottom: frame.height };
}

/**
 * The largest box of `ratio` (width over height, in pixels), centered where
 * `box` is, that fits on the turned photo: what a ratio button sets.
 */
export function ratioBox(box: Edges, ratio: number, frame: Size, angle: number): Edges {
  const [cx, cy] = [(box.left + box.right) / 2, (box.top + box.bottom) / 2];
  const halfWidth = Math.min(cx, frame.width - cx, ratio * Math.min(cy, frame.height - cy));
  const halfHeight = halfWidth / ratio;
  const largest = {
    left: cx - halfWidth,
    top: cy - halfHeight,
    right: cx + halfWidth,
    bottom: cy + halfHeight,
  };
  return shrinkToFit(largest, frame, angle);
}

const gcd = (a: number, b: number): number => (b === 0 ? a : gcd(b, a % b));

/** "a:b" in lowest terms. */
export function reduced(a: number, b: number): string {
  const [x, y] = [Math.round(a), Math.round(b)];
  const divisor = gcd(x, y) || 1;
  return `${x / divisor}:${y / divisor}`;
}

/** A ratio choice as width over height in pixels, for a frame of `frame`
 * size; `null` for Freeform. Landscape is the wider way; Original, the
 * frame's own, turned for the other orientation. */
export function ratioOf(choice: Ratio, portrait: boolean, frame: Size): number | null {
  if (choice === "freeform") return null;
  const [a, b] =
    choice === "original"
      ? [Math.max(frame.width, frame.height), Math.min(frame.width, frame.height)]
      : choice.split(":").map(Number);
  return portrait ? (b ?? 1) / (a ?? 1) : (a ?? 1) / (b ?? 1);
}

/** What's stored as the crop's ratio lock (`seura.crop`'s `aspect`):
 * Original as the frame's own ratio in lowest terms. */
export function aspectOf(choice: Ratio, portrait: boolean, frame: Size): string | null {
  const ratio = ratioOf(choice, portrait, frame);
  if (ratio === null) return null;
  if (choice === "original") {
    const [long, short] = [
      Math.max(frame.width, frame.height),
      Math.min(frame.width, frame.height),
    ];
    return portrait ? reduced(short, long) : reduced(long, short);
  }
  const [a, b] = choice.split(":");
  return portrait ? `${b}:${a}` : `${a}:${b}`;
}

/** The choice a stored ratio lock shows as: Original when it's the frame's
 * own, either way round. */
export function choiceOf(aspect: string | null, frame: Size): { choice: Ratio; portrait: boolean } {
  const tall = frame.height > frame.width;
  if (aspect === null) return { choice: "freeform", portrait: tall };
  const [a = 1, b = 1] = aspect.split(":").map(Number);
  const own = reduced(frame.width, frame.height);
  const turned = reduced(frame.height, frame.width);
  const lowest = reduced(a, b);
  if (lowest === own) return { choice: "original", portrait: tall };
  if (lowest === turned) return { choice: "original", portrait: !tall };
  for (const choice of RATIOS) {
    if (choice === "original" || choice === "freeform") continue;
    const [x, y] = choice.split(":");
    if (lowest === `${x}:${y}`) return { choice, portrait: false };
    if (lowest === `${y}:${x}`) return { choice, portrait: true };
  }
  return { choice: "freeform", portrait: tall };
}

/** Where `box` is once the photo turns a quarter turn: the frame's width
 * and height trade places, and the box turns with it (EDT-4). */
export function turnEdges(box: Edges, frame: Size, clockwise: boolean): Edges {
  return clockwise
    ? {
        left: frame.height - box.bottom,
        top: box.left,
        right: frame.height - box.top,
        bottom: box.right,
      }
    : {
        left: box.top,
        top: frame.width - box.right,
        right: box.bottom,
        bottom: frame.width - box.left,
      };
}

/** One state of crop mode, for its own undo. */
export type CropState = {
  /** The box the user set, which the angle may shrink (EDT-3). */
  box: Edges;
  angle: number;
  choice: Ratio;
  portrait: boolean;
};

/** The box shown for a state. */
export function shownBox(state: CropState, frame: Size): Edges {
  return shrinkToFit(state.box, frame, state.angle);
}

export { height as boxHeight, width as boxWidth };
