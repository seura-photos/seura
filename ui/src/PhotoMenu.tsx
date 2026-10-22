import { useEffect, useRef } from "react";
import { t } from "./messages";
import { useStore } from "./store";

/** At most this many photos open in the default app at once (the core's `OPEN_AT_ONCE`). */
export const OPEN_AT_ONCE = 20;

const isMac = typeof navigator !== "undefined" && /Mac/.test(navigator.platform);

type Props = {
  /** The photos it acts on: the selection, or the photo in the Viewer. */
  ids: number[];
  x: number;
  y: number;
  onClose: () => void;
};

/**
 * The menu for photos, opened by right-click or the context-menu key
 * (Shift+F10) in the grid and the Viewer: show them in Explorer or Finder,
 * or open them in the system's default app (BRW-8). Rust does both
 * (decision 0009).
 */
export function PhotoMenu({ ids, x, y, onClose }: Props) {
  const showInFolder = useStore((state) => state.showInFolder);
  const openInDefaultApp = useStore((state) => state.openInDefaultApp);
  const first = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    first.current?.focus();
    const close = () => onClose();
    window.addEventListener("blur", close);
    return () => window.removeEventListener("blur", close);
  }, [onClose]);
  const tooMany = ids.length > OPEN_AT_ONCE;

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
          if (event.key === "Escape") {
            event.stopPropagation();
            onClose();
          }
        }}
      >
        <button
          ref={first}
          type="button"
          role="menuitem"
          className="menu-item"
          onClick={() => {
            onClose();
            void showInFolder(ids);
          }}
        >
          {isMac ? t("photo-show-in-finder") : t("photo-show-in-explorer")}
        </button>
        <button
          type="button"
          role="menuitem"
          className="menu-item"
          disabled={tooMany}
          title={tooMany ? t("photo-open-too-many", { count: OPEN_AT_ONCE }) : undefined}
          onClick={() => {
            onClose();
            void openInDefaultApp(ids);
          }}
        >
          {t("photo-open", { count: ids.length })}
        </button>
      </div>
    </div>
  );
}
