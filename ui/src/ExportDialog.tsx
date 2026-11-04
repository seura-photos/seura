import { useEffect, useId, useRef, useState } from "react";
import type { ExportChoices } from "./bindings";
import { dialogKeys } from "./dialogKeys";
import { customEdge, MIN_LONG_EDGE } from "./exportMessages";
import { formatBytes } from "./format";
import { t } from "./messages";
import { type State, useStore } from "./store";

/** The long edges offered by name (EXP-2). */
const LONG_EDGES = [2048, 1600, 1024];

/**
 * The Export dialog (EXP-1 to EXP-4; wireframes, "Export"). The core keeps the
 * folder; the dialog only shows it. Export waits for a folder and a usable
 * size. When any photo is only in the cloud, it says so above the buttons, and
 * Export is the confirmation (LIB-9).
 */
export function ExportDialog() {
  const asked = useStore((state) => state.exportAsked);
  if (!asked) return null;
  // Remounted for each question, so it starts from what's remembered.
  return <Dialog key={asked.opened} asked={asked} />;
}

function Dialog({ asked }: { asked: NonNullable<State["exportAsked"]> }) {
  const chooseFolder = useStore((state) => state.chooseExportFolder);
  const cancel = useStore((state) => state.cancelExport);
  const start = useStore((state) => state.runExport);
  const { choices: remembered, cloudOnly, cloudSize, folder, ids } = asked;
  const id = useId();

  const [png, setPng] = useState(remembered.png);
  const [quality, setQuality] = useState(remembered.quality);
  const edge = remembered.longEdge;
  const [size, setSize] = useState<number | "original" | "custom">(
    edge === null ? "original" : LONG_EDGES.includes(edge) ? edge : "custom",
  );
  const [custom, setCustom] = useState(
    edge !== null && !LONG_EDGES.includes(edge) ? String(edge) : "",
  );
  const [removeLocation, setRemoveLocation] = useState(remembered.removeLocation);
  const [includeKeywords, setIncludeKeywords] = useState(remembered.includeKeywords);

  const longEdge = size === "original" ? null : size === "custom" ? customEdge(custom) : size;
  const sizeUsable = size !== "custom" || longEdge !== null;
  const ready = folder !== null && sizeUsable;
  const choices: ExportChoices = {
    png,
    quality,
    longEdge,
    removeLocation,
    includeKeywords,
  };

  // The focus starts where the next step is.
  const exportButton = useRef<HTMLButtonElement>(null);
  const chooseButton = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    (folder === null ? chooseButton : exportButton).current?.focus();
    // Only as it opens: a size being typed keeps the focus.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

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
    <div className="dialog-backdrop">
      <div
        className="dialog export-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby={`${id}-title`}
        onKeyDown={(event) => dialogKeys(event, cancel)}
      >
        <h2 id={`${id}-title`} className="dialog-title">
          {t("export-title", { count: ids.length })}
        </h2>

        <label className="export-label" htmlFor={`${id}-folder`}>
          {t("export-save-to")}
        </label>
        <div className="export-folder">
          <input
            id={`${id}-folder`}
            className="export-field"
            type="text"
            readOnly
            tabIndex={-1}
            value={folder ?? ""}
            placeholder={t("export-no-folder")}
            title={folder ?? undefined}
          />
          <button
            ref={chooseButton}
            type="button"
            className="button"
            onClick={() => void chooseFolder()}
          >
            {t("export-choose")}
          </button>
        </div>

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
              <input
                type="radio"
                name={`${id}-format`}
                checked={png}
                onChange={() => setPng(true)}
              />
              {t("export-png")}
            </label>
          </fieldset>

          <fieldset className="export-group">
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

        <p className="dialog-aside">{t("export-originals")}</p>
        {cloudOnly > 0 && (
          <p className="export-cloud">
            {t("export-cloud", {
              count: cloudOnly,
              total: ids.length,
              size: formatBytes(cloudSize ?? 0),
            })}
          </p>
        )}

        <div className="dialog-actions">
          <button type="button" className="button" onClick={cancel}>
            {t("export-cancel")}
          </button>
          <button
            ref={exportButton}
            type="button"
            className="button primary"
            disabled={!ready}
            onClick={() => void start(choices)}
          >
            {t("export-go")}
          </button>
        </div>
      </div>
    </div>
  );
}
