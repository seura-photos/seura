import { useEffect, useEffectEvent, useId, useRef } from "react";
import { formatSigned } from "./format";

type Props = {
  label: string;
  /** What the track's ends mean, named under it: Cool and Warm, Green and
   * Magenta (EDT-5). */
  ends?: [string, string];
  /** −100 to 100, neutral at 0 (EDT-5), unless a range is given. */
  value: number;
  /** The range and step, and the step with Shift held; the light and color
   * sliders' by default (EDT-5). Neutral is 0. */
  range?: { min: number; max: number; step: number; shiftStep: number };
  /** The value as shown; a signed whole number by default (EDT-5). */
  format?: (value: number) => string;
  /** What double-click sets it to: neutral, 0, by default. */
  reset?: number;
  disabled: boolean;
  /** Each movement, for the live preview. */
  onInput: (value: number) => void;
  /** A finished change: a drag released, a key pressed, or a reset. */
  onCommit: (value: number) => void;
};

/**
 * One of the Editor's sliders (EDT-5): its name and signed value above the
 * track. The preview follows every movement; a change is committed when the
 * drag ends, so a drag is one record in the photo's history (edit format,
 * "History"). Double-click resets it.
 */
export function Slider({
  label,
  ends,
  value,
  range = { min: -100, max: 100, step: 1, shiftStep: 10 },
  format = formatSigned,
  reset = 0,
  disabled,
  onInput,
  onCommit,
}: Props) {
  const id = useId();
  const input = useRef<HTMLInputElement>(null);
  // The input's own change event is the commit: it fires when a drag is
  // released, and once for each key press, where React's onChange fires for
  // every movement.
  const committed = useEffectEvent((element: HTMLInputElement) => onCommit(Number(element.value)));
  useEffect(() => {
    const element = input.current;
    if (!element) return;
    const changed = () => committed(element);
    element.addEventListener("change", changed);
    return () => element.removeEventListener("change", changed);
  }, []);

  const shown = format(value);
  return (
    <div className="slider">
      <div className="slider-head">
        <label htmlFor={id}>{label}</label>
        <output htmlFor={id}>{shown}</output>
      </div>
      <input
        ref={input}
        id={id}
        type="range"
        min={range.min}
        max={range.max}
        step={range.step}
        value={value}
        disabled={disabled}
        aria-valuetext={shown}
        onChange={(event) => onInput(Number(event.target.value))}
        onKeyDown={(event) => {
          // Shift takes larger steps; the native keys take one step.
          const direction =
            event.key === "ArrowRight" || event.key === "ArrowUp"
              ? 1
              : event.key === "ArrowLeft" || event.key === "ArrowDown"
                ? -1
                : 0;
          if (!event.shiftKey || direction === 0) return;
          event.preventDefault();
          const next = Math.min(
            range.max,
            Math.max(range.min, Number((value + direction * range.shiftStep).toFixed(6))),
          );
          onInput(next);
          onCommit(next);
        }}
        onDoubleClick={() => {
          onInput(reset);
          onCommit(reset);
        }}
      />
      {ends && (
        <div className="slider-ends" aria-hidden>
          <span>{ends[0]}</span>
          <span>{ends[1]}</span>
        </div>
      )}
    </div>
  );
}
