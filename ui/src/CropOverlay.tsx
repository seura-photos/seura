import { type KeyboardEvent, type PointerEvent, useRef } from "react";
import { type Edges, type Handle, moveBox, resizeBox } from "./crop";
import { t } from "./messages";
import type { Size } from "./preview/geometry";

type Props = {
  /** Where the frame is on the stage, in CSS pixels. */
  place: { left: number; top: number; width: number; height: number };
  /** The oriented photo's frame, in the photo's pixels. */
  frame: Size;
  angle: number;
  /** The box shown, in frame pixels. */
  box: Edges;
  /** Width over height, or `null` for Freeform. */
  ratio: number | null;
  /** The smallest box, in frame pixels. */
  minimum: number;
  /** "3:2 · 3720 × 2480". */
  label: string;
  /** A change of the box: each movement, then `done` when a drag ends or
   * a key moves it, for crop mode's own undo. */
  onChange: (box: Edges, done: boolean) => void;
};

const CORNERS = ["top-left", "top-right", "bottom-left", "bottom-right"] as const;
const EDGES = ["top", "right", "bottom", "left"] as const;

/**
 * Crop mode's box over the photo (wireframes, "Crop and straighten"): the
 * area outside it darkened, the frame's corners the turned photo leaves
 * uncovered as the dark surround, rule-of-thirds lines, corner handles,
 * and the size. Corners and edges resize, the inside moves it, and it
 * slides along the turned photo's edge. The box and each corner take the
 * focus in turn; arrow keys move the focused one by a screen pixel, ten
 * with Shift.
 */
export function CropOverlay({ place, frame, angle, box, ratio, minimum, label, onChange }: Props) {
  // CSS pixels for each frame pixel.
  const scale = place.width / Math.max(1, frame.width);
  // A drag in progress: where it started, and the box it has made so far.
  const drag = useRef<{
    pointer: number;
    handle: Handle | "box";
    x: number;
    y: number;
    start: Edges;
    last: Edges;
  } | null>(null);

  /** The box with `handle` (or all of it) moved by `dx`, `dy` frame pixels. */
  const moved = (start: Edges, handle: Handle | "box", dx: number, dy: number) =>
    handle === "box"
      ? moveBox(start, dx, dy, frame, angle)
      : resizeBox(start, handle, dx, dy, ratio, frame, angle, minimum);

  /** The handle an event is on, from its element's `data-handle`. */
  const handleOf = (target: EventTarget): Handle | "box" | null => {
    const element = (target as HTMLElement).closest<HTMLElement>("[data-handle]");
    return (element?.dataset.handle as Handle | "box" | undefined) ?? null;
  };

  // One set of handlers for the box and its handles.
  const handlers = {
    onPointerDown(event: PointerEvent<HTMLElement>) {
      const handle = handleOf(event.target);
      if (event.button !== 0 || handle === null) return;
      event.stopPropagation();
      event.currentTarget.setPointerCapture(event.pointerId);
      drag.current = {
        pointer: event.pointerId,
        handle,
        x: event.clientX,
        y: event.clientY,
        start: box,
        last: box,
      };
    },
    onPointerMove(event: PointerEvent<HTMLElement>) {
      const held = drag.current;
      if (!held || held.pointer !== event.pointerId) return;
      const next = moved(
        held.start,
        held.handle,
        (event.clientX - held.x) / scale,
        (event.clientY - held.y) / scale,
      );
      held.last = next;
      onChange(next, false);
    },
    onPointerUp(event: PointerEvent<HTMLElement>) {
      const held = drag.current;
      if (held?.pointer !== event.pointerId) return;
      drag.current = null;
      onChange(held.last, true);
    },
    onPointerCancel(event: PointerEvent<HTMLElement>) {
      const held = drag.current;
      if (held?.pointer !== event.pointerId) return;
      drag.current = null;
      onChange(held.last, true);
    },
    onKeyDown(event: KeyboardEvent<HTMLElement>) {
      const handle = handleOf(event.target);
      const step = (event.shiftKey ? 10 : 1) / scale;
      const [dx, dy] =
        event.key === "ArrowLeft"
          ? [-step, 0]
          : event.key === "ArrowRight"
            ? [step, 0]
            : event.key === "ArrowUp"
              ? [0, -step]
              : event.key === "ArrowDown"
                ? [0, step]
                : [0, 0];
      if (handle === null || (dx === 0 && dy === 0)) return;
      // Not the Editor's keys for the next photo.
      event.preventDefault();
      event.stopPropagation();
      onChange(moved(box, handle, dx, dy), true);
    },
  };

  // The turned photo's corners in the frame, for the uncovered corners.
  const theta = (angle * Math.PI) / 180;
  const [cx, cy] = [frame.width / 2, frame.height / 2];
  const turned = [
    [0, 0],
    [frame.width, 0],
    [frame.width, frame.height],
    [0, frame.height],
  ].map(([x = 0, y = 0]) => {
    const [dx, dy] = [x - cx, y - cy];
    return `${cx + dx * Math.cos(theta) - dy * Math.sin(theta)} ${cy + dx * Math.sin(theta) + dy * Math.cos(theta)}`;
  });
  const whole = `M0 0H${frame.width}V${frame.height}H0Z`;
  const inside = `M${box.left} ${box.top}H${box.right}V${box.bottom}H${box.left}Z`;

  return (
    <div className="crop-overlay" style={place}>
      <svg
        className="crop-shade"
        viewBox={`0 0 ${frame.width} ${frame.height}`}
        preserveAspectRatio="none"
        aria-hidden
      >
        <path className="crop-outside" d={`${whole}${inside}`} fillRule="evenodd" />
        <path className="crop-uncovered" d={`${whole}M${turned.join("L")}Z`} fillRule="evenodd" />
      </svg>
      <div
        className="crop-box"
        role="button"
        tabIndex={0}
        aria-label={t("crop-box")}
        aria-roledescription={t("crop-handle")}
        data-handle="box"
        style={{
          left: box.left * scale,
          top: box.top * scale,
          width: (box.right - box.left) * scale,
          height: (box.bottom - box.top) * scale,
        }}
        {...handlers}
      >
        <div className="crop-thirds" aria-hidden />
        {EDGES.map((edge) => (
          <div key={edge} className={`crop-edge ${edge}`} data-handle={edge} aria-hidden />
        ))}
        {CORNERS.map((corner) => (
          <div
            key={corner}
            className={`crop-corner ${corner}`}
            role="button"
            tabIndex={0}
            aria-label={t(`crop-${corner}`)}
            aria-roledescription={t("crop-handle")}
            data-handle={corner}
          />
        ))}
        <span className="crop-size">{label}</span>
      </div>
    </div>
  );
}
