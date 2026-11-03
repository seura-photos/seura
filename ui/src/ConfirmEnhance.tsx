import { useEffect, useRef } from "react";
import { dialogKeys } from "./dialogKeys";
import { formatBytes } from "./format";
import { t } from "./messages";
import { useStore } from "./store";

/**
 * Enhance on a selection that includes photos in the cloud, asked once
 * first (BRW-10, LIB-9). Downloading has the focus, since it's what the
 * user asked for and the question gives the count and size. Cancel, Esc and
 * closing it do nothing.
 */
export function ConfirmEnhance() {
  const confirming = useStore((state) => state.confirmingEnhance);
  const confirmEnhance = useStore((state) => state.confirmEnhance);
  const download = useRef<HTMLButtonElement>(null);
  useEffect(() => download.current?.focus(), [confirming]);
  if (!confirming) return null;
  const { ids, cloudOnly, cloudSize, onComputer } = confirming;

  return (
    <div className="dialog-backdrop">
      <div
        className="dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="enhance-title"
        aria-describedby="enhance-body"
        onKeyDown={(event) => dialogKeys(event, () => void confirmEnhance("cancel"))}
      >
        <h2 id="enhance-title" className="dialog-title">
          {t("enhance-confirm-title", { count: ids.length })}
        </h2>
        <p id="enhance-body">
          {t("enhance-confirm-body", { count: cloudOnly, size: formatBytes(cloudSize) })}
        </p>
        <div className="dialog-actions">
          <button type="button" className="button" onClick={() => void confirmEnhance("cancel")}>
            {t("enhance-confirm-cancel")}
          </button>
          {onComputer > 0 && (
            <button
              type="button"
              className="button"
              onClick={() => void confirmEnhance("onComputer")}
            >
              {t("enhance-confirm-on-computer", { count: onComputer })}
            </button>
          )}
          <button
            ref={download}
            type="button"
            className="button primary"
            onClick={() => void confirmEnhance("download")}
          >
            {t("enhance-confirm-download")}
          </button>
        </div>
      </div>
    </div>
  );
}
