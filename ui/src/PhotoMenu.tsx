import { ChevronRight } from "lucide-react";
import { type KeyboardEvent, useRef, useState } from "react";
import { commands, type PresetSummary } from "./bindings";
import { Menu } from "./Menu";
import { t } from "./messages";
import { system, useStore } from "./store";

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
 * or open them in the system's default app (BRW-8), apply a preset to them
 * from the Presets submenu (PRE-4), or move them to the Recycle Bin or
 * Trash (BRW-9). Rust does these (decision 0009).
 */
export function PhotoMenu({ ids, x, y, onClose }: Props) {
  const showInFolder = useStore((state) => state.showInFolder);
  const openInDefaultApp = useStore((state) => state.openInDefaultApp);
  const askDelete = useStore((state) => state.askDelete);
  const applyPreset = useStore((state) => state.applyPreset);
  // The Presets submenu, once opened: the presets, where it goes, and
  // whether it takes the focus (from the keyboard or a click, not a hover).
  const [presets, setPresets] = useState<{
    list: PresetSummary[];
    x: number;
    y: number;
    flipX: number;
    focus: boolean;
  } | null>(null);
  const opening = useRef(false);

  /** Opens the submenu beside its item, as hovering it does in a native
   * menu, or moves the focus into it if it's open. */
  async function openPresets(item: HTMLElement, focus: boolean) {
    if (presets) {
      if (focus) setPresets({ ...presets, focus: true });
      return;
    }
    // Hovering and clicking both open it; the list is read once.
    if (opening.current) return;
    opening.current = true;
    const box = item.getBoundingClientRect();
    const parent = item.closest(".menu")?.getBoundingClientRect() ?? box;
    const result = await commands.presets();
    opening.current = false;
    setPresets({
      list: result.status === "ok" ? result.data : [],
      x: parent.right,
      y: box.top,
      flipX: parent.left,
      focus,
    });
  }

  const closePresets = () => setPresets(null);

  function onPresetsKey(event: KeyboardEvent<HTMLButtonElement>) {
    if (event.key === "ArrowRight" || event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      event.stopPropagation();
      void openPresets(event.currentTarget, true);
    }
  }

  return (
    <Menu x={x} y={y} onClose={onClose}>
      <button
        type="button"
        role="menuitem"
        className="menu-item"
        onPointerEnter={closePresets}
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
        onPointerEnter={closePresets}
        onClick={() => {
          onClose();
          void openInDefaultApp(ids);
        }}
      >
        {t("photo-open", { count: ids.length })}
      </button>
      <button
        type="button"
        role="menuitem"
        className="menu-item menu-item-submenu"
        aria-haspopup="menu"
        aria-expanded={presets !== null}
        onPointerEnter={(event) => void openPresets(event.currentTarget, false)}
        onClick={(event) => void openPresets(event.currentTarget, true)}
        onKeyDown={onPresetsKey}
      >
        {t("photo-presets")}
        <ChevronRight className="icon" aria-hidden />
      </button>
      <button
        type="button"
        role="menuitem"
        className="menu-item"
        onPointerEnter={closePresets}
        onClick={() => {
          onClose();
          void askDelete(ids);
        }}
      >
        {t("photo-delete", { system })}
      </button>
      {presets && (
        <Menu
          x={presets.x}
          y={presets.y}
          flipX={presets.flipX}
          submenu
          focusFirst={presets.focus}
          onClose={closePresets}
        >
          {presets.list.length === 0 ? (
            <button type="button" role="menuitem" className="menu-item" disabled>
              {t("photo-no-presets")}
            </button>
          ) : (
            presets.list.map((preset) => (
              <button
                key={preset.name}
                type="button"
                role="menuitem"
                className="menu-item"
                disabled={preset.problem !== null}
                title={preset.problem ?? undefined}
                onClick={() => {
                  onClose();
                  void applyPreset(ids, preset.name);
                }}
              >
                {preset.name}
              </button>
            ))
          )}
        </Menu>
      )}
    </Menu>
  );
}
