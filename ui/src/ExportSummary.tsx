import { useEffect, useId, useRef } from "react";
import { dialogKeys } from "./dialogKeys";
import { isMac } from "./keys";
import { exportFolderProblem, exportProblem, exportStoppedParts } from "./exportMessages";
import { t } from "./messages";
import { useStore } from "./store";

/**
 * What a batch's export did (EXP-6): how many were exported, which couldn't be
 * and why, with Show in Explorer or Finder. One photo gets a message instead.
 */
export function ExportSummary() {
  const summary = useStore((state) => state.exportSummary);
  const close = useStore((state) => state.closeExportSummary);
  const show = useStore((state) => state.showExported);
  const id = useId();
  const ok = useRef<HTMLButtonElement>(null);
  useEffect(() => ok.current?.focus(), [summary]);
  if (!summary) return null;
  const { count, report } = summary;

  return (
    <div className="dialog-backdrop">
      <div
        className="dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby={`${id}-title`}
        onKeyDown={(event) => dialogKeys(event, close)}
      >
        <h2 id={`${id}-title`} className="dialog-title">
          {report.exported === count
            ? t("export-summary-all", { count })
            : t("export-summary-some", { done: report.exported, count })}
        </h2>
        {report.folderProblem && (
          <p>
            {exportFolderProblem(report.folderProblem)} {t("export-left", { count: report.left })}
          </p>
        )}
        {report.stopped > 0 && <p>{exportStoppedParts(report).join(" ")}</p>}
        {report.failures.length > 0 && (
          <>
            <p>{t("export-summary-failures", { count: report.failures.length })}</p>
            <ul className="export-failures">
              {report.failures.map((failure, at) => (
                <li key={at}>
                  <span className="export-failure-name">{failure.name}</span>:{" "}
                  {exportProblem(failure.problem)}
                </li>
              ))}
            </ul>
          </>
        )}
        <div className="dialog-actions">
          {report.exported > 0 && (
            <button type="button" className="button" onClick={() => void show()}>
              {isMac ? t("photo-show-in-finder") : t("photo-show-in-explorer")}
            </button>
          )}
          <button ref={ok} type="button" className="button primary" onClick={close}>
            {t("export-summary-ok")}
          </button>
        </div>
      </div>
    </div>
  );
}
