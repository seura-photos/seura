import { type KeyboardEvent, type PointerEvent, useRef } from "react";
import { X } from "lucide-react";
import { t } from "./messages";
import type { Geometry, Size } from "./preview/geometry";
import { type Circle, MAX_CORRECTIONS, type Placed, screenMapping, stepped } from "./redeye";

type Props = {
  /** Where the result is on the stage, in CSS pixels (`useZoom`'s `whole`). */
  place: Placed;
  geometry: Geometry;
  /** The upright photo's full size. */
  full: Size;
  circles: Circle[];
  /** A click's circle, in CSS pixels across. */
  brush: number;
  /** The selected circle, by index. */
  selected: number | null;
  /** Space is held: the drag pans, so the circles let it through. */
  panning: boolean;
  /** A change of the circles: each movement, then `done` when a drag ends,
   * a key moves one, or one is added or removed, for the mode's undo. */
  onChange: (circles: Circle[], done: boolean) => void;
  onSelect: (index: number | null) => void;
};

/** How far a press may move and still be a click, in CSS pixels. */
const CLICK = 3;
/** The smallest circle a drag or a key makes, in CSS pixels across. */
const SMALLEST = 4;

/**
 * Red-eye mode's circles over the photo (wireframes, "Red-eye mode"): a
 * thin outline with a handle on its edge for each, drawn where its eye is
 * as the photo is turned, cropped and zoomed. A click away from every
 * circle adds one of the brush's size; a drag adds one drawn out from its
 * center. Dragging inside a circle moves it, and its handle resizes it.
 * Clicking a circle selects it, with a × to remove it. Each circle takes
 * the focus in turn: the arrow keys move it a screen pixel (ten with
 * Shift), `[` and `]` resize it, and Delete removes it.
 */
export function RedEyeOverlay({
  place,
  geometry,
  full,
  circles,
  brush,
  selected,
  panning,
  onChange,
  onSelect,
}: Props) {
  // Positions within the overlay, which covers the result.
  const mapping = screenMapping(geometry, full, { ...place, left: 0, top: 0 });
  const drag = useRef<{
    pointer: number;
    kind: "add" | "move" | "resize";
    index: number;
    x: number;
    y: number;
    start: Circle[];
    moved: boolean;
    last: Circle[];
  } | null>(null);

  /** A pointer's position within the overlay. */
  const local = (event: PointerEvent<HTMLElement>) => {
    const rect = event.currentTarget.getBoundingClientRect();
    return { x: event.clientX - rect.left, y: event.clientY - rect.top };
  };

  /** `circles` with the one at `index` replaced. */
  const replaced = (from: Circle[], index: number, circle: Circle) =>
    from.map((other, i) => (i === index ? circle : other));

  const handlers = {
    onPointerDown(event: PointerEvent<HTMLElement>) {
      if (panning || event.button !== 0) return;
      const target = event.target as HTMLElement;
      if (target.closest("[data-remove]")) return;
      // Not the stage's pan.
      event.stopPropagation();
      event.currentTarget.setPointerCapture(event.pointerId);
      const { x, y } = local(event);
      const circle = target.closest<HTMLElement>("[data-index]");
      const index = circle ? Number(circle.dataset.index) : circles.length;
      const kind = !circle ? "add" : target.closest("[data-handle]") ? "resize" : "move";
      drag.current = {
        pointer: event.pointerId,
        kind,
        index,
        x,
        y,
        start: circles,
        moved: false,
        last: circles,
      };
    },
    onPointerMove(event: PointerEvent<HTMLElement>) {
      const held = drag.current;
      if (!held || held.pointer !== event.pointerId) return;
      const { x, y } = local(event);
      const distance = Math.hypot(x - held.x, y - held.y);
      if (!held.moved && distance < CLICK) return;
      held.moved = true;
      let next: Circle[];
      if (held.kind === "add") {
        // At the format's limit, nothing more is added.
        if (held.start.length >= MAX_CORRECTIONS) return;
        const circle = mapping.fromScreen(held.x, held.y, Math.max(SMALLEST / 2, distance));
        next = [...held.start, circle];
      } else {
        const before = held.start[held.index];
        if (!before) return;
        const shown = mapping.toScreen(before);
        const circle =
          held.kind === "move"
            ? mapping.fromScreen(shown.x + x - held.x, shown.y + y - held.y, shown.radius)
            : {
                ...before,
                radius: mapping.fromScreen(
                  shown.x,
                  shown.y,
                  Math.max(SMALLEST / 2, Math.hypot(x - shown.x, y - shown.y)),
                ).radius,
              };
        next = replaced(held.start, held.index, circle);
      }
      held.last = next;
      onChange(next, false);
    },
    onPointerUp(event: PointerEvent<HTMLElement>) {
      const held = drag.current;
      if (held?.pointer !== event.pointerId) return;
      drag.current = null;
      if (held.moved) {
        onChange(held.last, true);
        onSelect(held.index);
        return;
      }
      if (held.kind !== "add") {
        // A click on a circle selects it.
        onSelect(held.index);
        return;
      }
      if (held.start.length >= MAX_CORRECTIONS) return;
      const circle = mapping.fromScreen(held.x, held.y, brush / 2);
      onChange([...held.start, circle], true);
      onSelect(held.start.length);
    },
    onPointerCancel(event: PointerEvent<HTMLElement>) {
      const held = drag.current;
      if (held?.pointer !== event.pointerId) return;
      drag.current = null;
      if (held.moved) onChange(held.last, true);
    },
    onKeyDown(event: KeyboardEvent<HTMLElement>) {
      const element = (event.target as HTMLElement).closest<HTMLElement>("[data-index]");
      if (!element || event.altKey || event.ctrlKey || event.metaKey) return;
      const index = Number(element.dataset.index);
      const circle = circles[index];
      if (!circle) return;
      const step = event.shiftKey ? 10 : 1;
      const shown = mapping.toScreen(circle);
      const moves: Record<string, [number, number]> = {
        ArrowLeft: [-step, 0],
        ArrowRight: [step, 0],
        ArrowUp: [0, -step],
        ArrowDown: [0, step],
      };
      const move = moves[event.key];
      let next: Circle[] | null = null;
      if (move) {
        next = replaced(
          circles,
          index,
          mapping.fromScreen(shown.x + move[0], shown.y + move[1], shown.radius),
        );
      } else if (event.key === "[" || event.key === "]") {
        const across = stepped(shown.radius * 2, event.key === "[" ? -1 : 1, SMALLEST, 4000);
        next = replaced(circles, index, {
          ...circle,
          radius: mapping.fromScreen(shown.x, shown.y, across / 2).radius,
        });
      } else if (event.key === "Delete" || event.key === "Backspace") {
        next = circles.filter((_, i) => i !== index);
        onSelect(null);
      }
      if (!next) return;
      // Not the Editor's keys: the next photo, the brush.
      event.preventDefault();
      event.stopPropagation();
      onChange(next, true);
    },
  };

  return (
    <div
      className={`red-eye-overlay${panning ? " panning" : ""}`}
      style={place}
      data-testid="red-eye-overlay"
      {...handlers}
    >
      {circles.map((circle, index) => {
        const shown = mapping.toScreen(circle);
        if (!shown.inside) return null;
        return (
          <div
            // A circle's place in the list is what names and changes it.
            key={index}
            className={`red-eye-circle${index === selected ? " selected" : ""}`}
            role="button"
            tabIndex={0}
            aria-label={t("red-eye-correction", { number: index + 1 })}
            aria-pressed={index === selected}
            data-index={index}
            style={{
              left: shown.x - shown.radius,
              top: shown.y - shown.radius,
              width: shown.radius * 2,
              height: shown.radius * 2,
            }}
            onFocus={() => onSelect(index)}
          >
            <div className="red-eye-handle" data-handle aria-hidden />
            {index === selected && (
              <button
                type="button"
                className="red-eye-remove"
                data-remove
                aria-label={t("red-eye-remove", { number: index + 1 })}
                tabIndex={-1}
                onClick={() => {
                  onSelect(null);
                  onChange(
                    circles.filter((_, i) => i !== index),
                    true,
                  );
                }}
              >
                <X className="icon" aria-hidden />
              </button>
            )}
          </div>
        );
      })}
    </div>
  );
}
