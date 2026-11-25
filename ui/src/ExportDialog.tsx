import { useEffect, useId, useRef, useState } from "react";
import type { ExportChoices } from "./bindings";
import { dialogKeys } from "./dialogKeys";
import { ExportOptions } from "./ExportOptions";
import { formatBytes } from "./format";
import { t } from "./messages";
import { type State, useStore } from "./store";

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

  // The remembered choices until the user changes one; null while a custom
  // size isn't usable.
  const [choices, setChoices] = useState<ExportChoices | null>(remembered);
  const ready = folder !== null && choices !== null;

  // The focus starts where the next step is.
  const exportButton = useRef<HTMLButtonElement>(null);
  const chooseButton = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    (folder === null ? chooseButton : exportButton).current?.focus();
    // Only as it opens: a size being typed keeps the focus.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

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

        <ExportOptions initial={remembered} onChange={setChoices} />

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
            onClick={() => {
              if (choices) void start(choices);
            }}
          >
            {t("export-go")}
          </button>
        </div>
      </div>
    </div>
  );
}
