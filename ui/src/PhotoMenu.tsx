import { Menu } from "./Menu";
import { t } from "./messages";
import { useStore } from "./store";

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

  return (
    <Menu x={x} y={y} onClose={onClose}>
      <button
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
        onClick={() => {
          onClose();
          void openInDefaultApp(ids);
        }}
      >
        {t("photo-open", { count: ids.length })}
      </button>
    </Menu>
  );
}
