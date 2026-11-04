import { LoaderCircle } from "lucide-react";
import { useEffect, useState } from "react";
import { formatBytes } from "./format";
import type { ExportProgress } from "./bindings";
import { t } from "./messages";
import { useStore } from "./store";

/** How long a message shows, in milliseconds (INT-6: about 6 seconds). */
const SHOWN_FOR = 6_000;

/** What an export is doing: one photo by name, a batch by count, and a photo
 * in the cloud as downloading, with its size (EXP-6). */
function exportProgressText(batch: ExportProgress | null): string {
  if (!batch) return t("export-starting");
  const { at, total, name, downloading, size } = batch;
  if (downloading) {
    const values = { name, size: formatBytes(size ?? 0) };
    return total === 1
      ? t("export-downloading-one", values)
      : t("export-downloading", { ...values, at, total });
  }
  return total === 1 ? t("export-exporting-one", { name }) : t("export-exporting", { at, total });
}

/**
 * The last action's result, when it isn't visible on its own: "Moved 3
 * photos to the Recycle Bin.", an undo, photos skipped as read-only
 * (INT-6). A brief message near the bottom of the photo area, centred, for
 * about 6 seconds or until the next one; it never blocks anything, and the
 * region is always there so screen readers announce each one. `since`
 * leaves out messages from before, such as the Library's when the Viewer
 * opens.
 */
export function Message({ since = -1 }: { since?: number }) {
  const notice = useStore((state) => state.notice);
  const noticeId = useStore((state) => state.noticeId);
  // The message whose time is up.
  const [ended, setEnded] = useState(-1);
  useEffect(() => {
    const timer = window.setTimeout(() => setEnded(noticeId), SHOWN_FOR);
    return () => window.clearTimeout(timer);
  }, [noticeId]);
  const shown = notice !== null && noticeId > since && noticeId !== ended;
  // Enhance's downloads: up until they're over, whatever the view (BRW-10).
  const batch = useStore((state) => state.downloadBatch);
  const stopDownloads = useStore((state) => state.stopDownloads);
  const noticeAction = useStore((state) => state.noticeAction);
  // An export's progress, likewise (EXP-6).
  const exportRunning = useStore((state) => state.exportRunning);
  const exporting = useStore((state) => state.exportBatch);
  const stopExport = useStore((state) => state.stopExport);
  return (
    <div className="message-region" role="status">
      {batch && (
        <div className="message message-progress">
          <LoaderCircle className="icon viewer-spinner" aria-hidden />
          <span>{t("enhance-downloading", { at: batch.at, total: batch.total })}</span>
          <button
            type="button"
            className="button"
            onMouseDown={(event) => event.preventDefault()}
            onClick={() => void stopDownloads()}
          >
            {t("enhance-stop")}
          </button>
        </div>
      )}
      {exportRunning && (
        <div className="message message-progress">
          <LoaderCircle className="icon viewer-spinner" aria-hidden />
          <span>{exportProgressText(exporting)}</span>
          <button
            type="button"
            className="button"
            onMouseDown={(event) => event.preventDefault()}
            onClick={() => void stopExport()}
          >
            {t("export-cancel")}
          </button>
        </div>
      )}
      {shown &&
        (noticeAction ? (
          <div key={noticeId} className="message message-progress">
            <span>{notice}</span>
            <button
              type="button"
              className="button"
              onMouseDown={(event) => event.preventDefault()}
              onClick={noticeAction.run}
            >
              {noticeAction.label}
            </button>
          </div>
        ) : (
          <p key={noticeId} className="message">
            {notice}
          </p>
        ))}
    </div>
  );
}
