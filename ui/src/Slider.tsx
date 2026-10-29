import { useEffect, useEffectEvent, useId, useRef } from "react";
import { formatSigned } from "./format";

type Props = {
  label: string;
  /** −100 to 100, neutral at 0 (EDT-5). */
  value: number;
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
export function Slider({ label, value, disabled, onInput, onCommit }: Props) {
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

  const shown = Math.round(value);
  return (
    <div className="slider">
      <div className="slider-head">
        <label htmlFor={id}>{label}</label>
        <output htmlFor={id}>{formatSigned(shown)}</output>
      </div>
      <input
        ref={input}
        id={id}
        type="range"
        min={-100}
        max={100}
        step={1}
        value={value}
        disabled={disabled}
        aria-valuetext={formatSigned(shown)}
        onChange={(event) => onInput(Number(event.target.value))}
        onDoubleClick={() => {
          onInput(0);
          onCommit(0);
        }}
      />
    </div>
  );
}
