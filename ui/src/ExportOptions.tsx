import { useEffect, useId, useRef, useState } from "react";
import type { ExportChoices } from "./bindings";
import { customEdge, MIN_LONG_EDGE } from "./exportMessages";
import { t } from "./messages";

/** The long edges offered by name (EXP-2). */
const LONG_EDGES = [2048, 1600, 1024];

type Props = {
  /** What the controls start from. */
  initial: ExportChoices;
  /** Each change: the choices, or null while a custom size isn't usable. */
  onChange: (choices: ExportChoices | null) => void;
};

/**
 * Export's choices: format and quality, size, and details (EXP-2 to EXP-4).
 * The Export dialog asks with them, and Settings sets what it opens with,
 * so the two are the same controls (SET-2; wireframes, "Settings").
 */
export function ExportOptions({ initial, onChange }: Props) {
  const id = useId();
  const [png, setPng] = useState(initial.png);
  const [quality, setQuality] = useState(initial.quality);
  const edge = initial.longEdge;
  const [size, setSize] = useState<number | "original" | "custom">(
    edge === null ? "original" : LONG_EDGES.includes(edge) ? edge : "custom",
  );
  const [custom, setCustom] = useState(
    edge !== null && !LONG_EDGES.includes(edge) ? String(edge) : "",
  );
  const [removeLocation, setRemoveLocation] = useState(initial.removeLocation);
  const [includeKeywords, setIncludeKeywords] = useState(initial.includeKeywords);

  const longEdge = size === "original" ? null : size === "custom" ? customEdge(custom) : size;
  const sizeUsable = size !== "custom" || longEdge !== null;

  // Told of each change, not of what the controls start from.
  const started = useRef(false);
  const told = useRef(onChange);
  useEffect(() => {
    told.current = onChange;
  });
  useEffect(() => {
    if (!started.current) {
      started.current = true;
      return;
    }
    told.current(sizeUsable ? { png, quality, longEdge, removeLocation, includeKeywords } : null);
  }, [png, quality, longEdge, sizeUsable, removeLocation, includeKeywords]);

  const sizeRadio = (value: number | "original", label: string) => (
    <label className="export-choice">
      <input
        type="radio"
        name={`${id}-size`}
        checked={size === value}
        onChange={() => setSize(value)}
      />
      {label}
    </label>
  );

  return (
    <>
      <div className="export-columns">
        <fieldset className="export-group">
          <legend className="export-label">{t("export-format")}</legend>
          <label className="export-choice">
            <input
              type="radio"
              name={`${id}-format`}
              checked={!png}
              onChange={() => setPng(false)}
            />
            {t("export-jpeg")}
          </label>
          <div className="export-quality">
            <label htmlFor={`${id}-quality`}>{t("export-quality")}</label>
            <output htmlFor={`${id}-quality`}>{quality}</output>
            <input
              id={`${id}-quality`}
              type="range"
              min={1}
              max={100}
              step={1}
              disabled={png}
              value={quality}
              onChange={(event) => setQuality(Number(event.target.value))}
            />
          </div>
          <label className="export-choice">
            <input type="radio" name={`${id}-format`} checked={png} onChange={() => setPng(true)} />
            {t("export-png")}
          </label>
        </fieldset>

        <fieldset className="export-group" aria-describedby={`${id}-size-note`}>
          <legend className="export-label">{t("export-size")}</legend>
          {sizeRadio("original", t("export-size-original"))}
          {LONG_EDGES.map((edge) => (
            <div key={edge}>
              {sizeRadio(edge, t("export-size-long-edge", { edge: String(edge) }))}
            </div>
          ))}
          <div className="export-custom">
            <label className="export-choice">
              <input
                type="radio"
                name={`${id}-size`}
                checked={size === "custom"}
                onChange={() => setSize("custom")}
              />
              {t("export-size-custom")}
            </label>
            <input
              className="export-field"
              type="text"
              inputMode="numeric"
              aria-label={t("export-size-custom-pixels")}
              aria-invalid={size === "custom" && !sizeUsable}
              aria-describedby={`${id}-custom-hint`}
              value={custom}
              onFocus={() => setSize("custom")}
              onChange={(event) => setCustom(event.target.value)}
            />
            <span>{t("export-size-px")}</span>
          </div>
          {/* Always shown, so choosing a size doesn't move what's below (Nevada, 2026-11-04). */}
          <p id={`${id}-size-note`} className="export-size-note">
            {t("export-size-note")}
          </p>
          {size === "custom" && !sizeUsable && (
            <p id={`${id}-custom-hint`} className="export-hint">
              {t("export-size-custom-hint", { min: MIN_LONG_EDGE })}
            </p>
          )}
        </fieldset>
      </div>

      <fieldset className="export-group">
        <legend className="export-label">{t("export-details")}</legend>
        <label className="export-choice">
          <input
            type="checkbox"
            checked={removeLocation}
            onChange={(event) => setRemoveLocation(event.target.checked)}
          />
          {t("export-remove-location")}
        </label>
        <label className="export-choice">
          <input
            type="checkbox"
            checked={includeKeywords}
            onChange={(event) => setIncludeKeywords(event.target.checked)}
          />
          {t("export-include-keywords")}
        </label>
      </fieldset>
    </>
  );
}
