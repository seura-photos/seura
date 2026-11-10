import { useEffect, useRef } from "react";
import { dialogKeys } from "./dialogKeys";
import { formatBytes } from "./format";
import { t } from "./messages";
import { useStore } from "./store";

/**
 * Starting a review with photos in the cloud, asked once (REV-2, LIB-9): how
 * many and how large, and whether to download them as they're reached.
 * Downloading has the focus, since a review is for looking at the photos.
 * Cancel, Esc and closing it start nothing.
 */
export function ConfirmReview() {
  const confirming = useStore((state) => state.confirmingReview);
  const confirmReview = useStore((state) => state.confirmReview);
  const download = useRef<HTMLButtonElement>(null);
  useEffect(() => download.current?.focus(), [confirming]);
  if (!confirming) return null;
  const { photos, cloudOnly, cloudSize } = confirming;

  return (
    <div className="dialog-backdrop">
      <div
        className="dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="review-cloud-title"
        aria-describedby="review-cloud-body"
        onKeyDown={(event) => dialogKeys(event, () => confirmReview("cancel"))}
      >
        <h2 id="review-cloud-title" className="dialog-title">
          {t("review-cloud-title", { count: photos.length })}
        </h2>
        <p id="review-cloud-body">
          {t("review-cloud-body", { cloudOnly, size: formatBytes(cloudSize) })}
        </p>
        <div className="dialog-actions">
          <button type="button" className="button" onClick={() => confirmReview("cancel")}>
            {t("review-cloud-cancel")}
          </button>
          <button type="button" className="button" onClick={() => confirmReview("thumbnails")}>
            {t("review-cloud-thumbnails")}
          </button>
          <button
            ref={download}
            type="button"
            className="button primary"
            onClick={() => confirmReview("download")}
          >
            {t("review-cloud-download")}
          </button>
        </div>
      </div>
    </div>
  );
}
