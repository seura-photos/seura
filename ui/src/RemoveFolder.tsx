import { useEffect, useRef } from "react";
import type { FolderInfo } from "./bindings";
import { dialogKeys } from "./dialogKeys";
import { Menu } from "./Menu";
import { t } from "./messages";

type MenuProps = {
  x: number;
  y: number;
  /** Expand all beneath, for a folder with folders in it (LIB-2). */
  onExpand: (() => void) | null;
  /** Remove from library…, for a library folder (LIB-5). */
  onRemove: (() => void) | null;
  onClose: () => void;
};

/** The menu for a folder, opened by right-click or the context-menu key. */
export function FolderMenu({ x, y, onExpand, onRemove, onClose }: MenuProps) {
  return (
    <Menu x={x} y={y} onClose={onClose}>
      {onExpand && (
        <button type="button" role="menuitem" className="menu-item" onClick={onExpand}>
          {t("tree-expand-beneath")}
        </button>
      )}
      {onRemove && (
        <button type="button" role="menuitem" className="menu-item" onClick={onRemove}>
          {t("library-remove-folder")}
        </button>
      )}
    </Menu>
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
        onKeyDown={(event) => dialogKeys(event, onCancel)}
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
