import { useEffect, useRef } from "react";
import { dialogKeys } from "./dialogKeys";
import { t } from "./messages";
import { system, useStore } from "./store";

/**
 * Deleting photos, always confirmed first (BRW-9): how many go to the
 * Recycle Bin or Trash, with their edits; that photos in a synced folder
 * leave the cloud and the user's other devices too, naming the service; and
 * which are left as they are. Cancel has the focus, so Enter is safe
 * (INT-5). When none of them can go, it says why, with OK alone.
 */
export function ConfirmDelete() {
  const confirming = useStore((state) => state.confirmingDelete);
  const confirmDelete = useStore((state) => state.confirmDelete);
  const showInFolder = useStore((state) => state.showInFolder);
  const cancel = useRef<HTMLButtonElement>(null);
  useEffect(() => cancel.current?.focus(), [confirming]);
  if (!confirming) return null;
  const {
    count,
    editFiles,
    synced,
    service,
    readOnly,
    cloudOnly,
    noTrash,
    fileProvider,
    fileProviderService,
  } = confirming.summary;
  const nothing = count === 0;
  const leftAlone = readOnly + cloudOnly + noTrash + fileProvider.length;

  return (
    <div className="dialog-backdrop">
      <div
        className="dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="delete-title"
        aria-describedby="delete-body"
        onKeyDown={(event) => dialogKeys(event, () => void confirmDelete(false))}
      >
        <h2 id="delete-title" className="dialog-title">
          {nothing
            ? t("delete-nothing-title", { count: leftAlone, system })
            : t("delete-title", { count, system })}
        </h2>
        <div id="delete-body">
          {/* Each edit file is restored separately, so the text says so
              when there are any (Nevada, 2026-10-26; design answers). */}
          {!nothing && (
            <p>{t("delete-body", { count, system, edited: editFiles > 0 ? "yes" : "no" })}</p>
          )}
          {synced > 0 && service !== null && (
            <p>
              {synced === count
                ? t("delete-synced-all", { count, service })
                : t("delete-synced-some", { count: synced, service })}
            </p>
          )}
          {/* The photos left as they are, set apart from those that go,
              unless they're all there is to say. */}
          {leftAlone > 0 && (
            <div className={nothing ? undefined : "dialog-aside"}>
              {cloudOnly > 0 && <p>{t("delete-cloud-only", { count: cloudOnly, system })}</p>}
              {readOnly > 0 && <p>{t("delete-read-only", { count: readOnly })}</p>}
              {noTrash > 0 && <p>{t("delete-no-trash", { count: noTrash, system })}</p>}
              {/* The Mac refuses Seura these (#112): Finder can delete them. */}
              {fileProvider.length > 0 && (
                <p>
                  {t("delete-file-provider", {
                    count: fileProvider.length,
                    service: fileProviderService ?? t("delete-cloud-service"),
                  })}
                </p>
              )}
            </div>
          )}
        </div>
        <div className="dialog-actions">
          {fileProvider.length > 0 && (
            <button
              type="button"
              className="button"
              onClick={() => {
                void showInFolder(fileProvider);
                void confirmDelete(false);
              }}
            >
              {t("delete-show-in-finder")}
            </button>
          )}
          {!nothing && (
            <button type="button" className="button" onClick={() => void confirmDelete(true)}>
              {t("delete-confirm", { system })}
            </button>
          )}
          <button
            ref={cancel}
            type="button"
            className="button primary"
            onClick={() => void confirmDelete(false)}
          >
            {nothing ? t("delete-ok") : t("delete-cancel")}
          </button>
        </div>
      </div>
    </div>
  );
}
