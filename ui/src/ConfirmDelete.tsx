import { useEffect, useRef } from "react";
import { t } from "./messages";
import { system, useStore } from "./store";

/**
 * Deleting photos, always confirmed first (BRW-9): how many go to the
 * Recycle Bin or Trash, with their edits; that photos in a synced folder
 * leave the cloud and the user's other devices too, naming the service; and
 * which are left as they are. Cancel has the focus, so Enter is safe
 * (INT-5).
 */
export function ConfirmDelete() {
  const confirming = useStore((state) => state.confirmingDelete);
  const confirmDelete = useStore((state) => state.confirmDelete);
  const cancel = useRef<HTMLButtonElement>(null);
  useEffect(() => cancel.current?.focus(), [confirming]);
  if (!confirming) return null;
  const { count, editFiles, synced, service, readOnly, cloudOnly } = confirming.summary;

  return (
    <div className="dialog-backdrop">
      <div
        className="dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="delete-title"
        aria-describedby="delete-body"
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.stopPropagation();
            void confirmDelete(false);
          }
        }}
      >
        <h2 id="delete-title" className="dialog-title">
          {t("delete-title", { count, system })}
        </h2>
        <div id="delete-body">
          <p>{t("delete-body", { count, system })}</p>
          {/* Each edit file is restored separately (Nevada, 2026-10-26). */}
          {editFiles > 0 && <p>{t("delete-restore-both", { count, system })}</p>}
          {synced > 0 && service !== null && (
            <p>
              {synced === count
                ? t("delete-synced-all", { count, service })
                : t("delete-synced-some", { count: synced, service })}
            </p>
          )}
          {cloudOnly > 0 && <p>{t("delete-cloud-only", { count: cloudOnly, system })}</p>}
          {readOnly > 0 && <p>{t("delete-read-only", { count: readOnly })}</p>}
        </div>
        <div className="dialog-actions">
          <button type="button" className="button" onClick={() => void confirmDelete(true)}>
            {t("delete-confirm", { system })}
          </button>
          <button
            ref={cancel}
            type="button"
            className="button primary"
            onClick={() => void confirmDelete(false)}
          >
            {t("delete-cancel")}
          </button>
        </div>
      </div>
    </div>
  );
}
