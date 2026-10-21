import { useEffect, useRef } from "react";
import type { FolderInfo } from "./bindings";
import { t } from "./messages";

type MenuProps = {
  folder: FolderInfo;
  x: number;
  y: number;
  onRemove: () => void;
  onClose: () => void;
};

/** The menu for a library folder, opened by right-click or the context-menu key. */
export function FolderMenu({ x, y, onRemove, onClose }: MenuProps) {
  const first = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    first.current?.focus();
    const close = () => onClose();
    window.addEventListener("blur", close);
    return () => window.removeEventListener("blur", close);
  }, [onClose]);
  return (
    <div
      className="menu-backdrop"
      onClick={onClose}
      onContextMenu={(event) => event.preventDefault()}
    >
      <div
        className="menu"
        role="menu"
        style={{ left: x, top: y }}
        onClick={(event) => event.stopPropagation()}
        onKeyDown={(event) => {
          if (event.key === "Escape") onClose();
        }}
      >
        <button ref={first} type="button" role="menuitem" className="menu-item" onClick={onRemove}>
          {t("library-remove-folder")}
        </button>
      </div>
    </div>
  );
}

type ConfirmProps = {
  folder: FolderInfo;
  onConfirm: () => void;
  onCancel: () => void;
};

/**
 * Removing a library folder, confirmed first (LIB-5). Nothing on disk
 * changes, so it's said plainly. Cancel has the focus, so Enter is safe
 * (INT-5).
 */
export function ConfirmRemove({ folder, onConfirm, onCancel }: ConfirmProps) {
  const cancel = useRef<HTMLButtonElement>(null);
  useEffect(() => cancel.current?.focus(), []);
  return (
    <div className="dialog-backdrop">
      <div
        className="dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="remove-title"
        aria-describedby="remove-body"
        onKeyDown={(event) => {
          if (event.key === "Escape") onCancel();
        }}
      >
        <h2 id="remove-title" className="dialog-title">
          {t("library-remove-title", { name: folder.name })}
        </h2>
        <p id="remove-body">{t("library-remove-body")}</p>
        <div className="dialog-actions">
          <button type="button" className="button" onClick={onConfirm}>
            {t("library-remove-confirm")}
          </button>
          <button ref={cancel} type="button" className="button primary" onClick={onCancel}>
            {t("library-remove-cancel")}
          </button>
        </div>
      </div>
    </div>
  );
}
