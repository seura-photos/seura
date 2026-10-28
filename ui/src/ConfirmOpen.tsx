import { useEffect, useRef } from "react";
import { dialogKeys } from "./dialogKeys";
import { t } from "./messages";
import { useStore } from "./store";

/**
 * Opening several photos in the default app, confirmed first when it's
 * many or some are in the cloud (BRW-8, LIB-9). Cancel has the focus, so
 * Enter is safe (INT-5).
 */
export function ConfirmOpen() {
  const confirming = useStore((state) => state.confirmingOpen);
  const confirmOpen = useStore((state) => state.confirmOpen);
  const cancel = useRef<HTMLButtonElement>(null);
  useEffect(() => cancel.current?.focus(), [confirming]);
  if (!confirming) return null;
  const { count, cloudOnly } = confirming;

  return (
    <div className="dialog-backdrop">
      <div
        className="dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="open-title"
        aria-describedby="open-body"
        onKeyDown={(event) => dialogKeys(event, () => void confirmOpen(false))}
      >
        <h2 id="open-title" className="dialog-title">
          {t("photo-open-title", { count })}
        </h2>
        <div id="open-body">
          <p>{t("photo-open-windows")}</p>
          {cloudOnly > 0 && <p>{t("photo-open-cloud", { count: cloudOnly })}</p>}
        </div>
        <div className="dialog-actions">
          <button type="button" className="button" onClick={() => void confirmOpen(true)}>
            {t("photo-open-confirm")}
          </button>
          <button
            ref={cancel}
            type="button"
            className="button primary"
            onClick={() => void confirmOpen(false)}
          >
            {t("photo-open-cancel")}
          </button>
        </div>
      </div>
    </div>
  );
}
