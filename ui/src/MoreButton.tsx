import { Ellipsis } from "lucide-react";
import { useRef, useState } from "react";
import { Menu } from "./Menu";
import { t } from "./messages";
import { system, useStore } from "./store";

/**
 * Windows' ⋯ button, at the right end of the toolbar in every view: the
 * commands for the whole app, which macOS has in its menu bar (ACC-4). Each
 * platform has one place for them, so there's none on a Mac. About now;
 * Settings and Clean up once they're built (M7).
 */
export function MoreButton() {
  const setAbout = useStore((state) => state.setAbout);
  const button = useRef<HTMLButtonElement>(null);
  // Where the menu opens: below the button, its right edge on the button's.
  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);
  if (system === "mac") return null;

  return (
    <>
      <button
        ref={button}
        type="button"
        className="more-button"
        aria-label={t("more")}
        title={t("more")}
        aria-haspopup="menu"
        aria-expanded={menu !== null}
        onClick={() => {
          const box = button.current?.getBoundingClientRect();
          if (box) setMenu({ x: box.right, y: box.bottom + 4 });
        }}
      >
        <Ellipsis className="icon" aria-hidden />
      </button>
      {menu && (
        <Menu x={menu.x} y={menu.y} align="end" onClose={() => setMenu(null)}>
          <button
            type="button"
            role="menuitem"
            className="menu-item"
            onClick={() => {
              setMenu(null);
              setAbout(true);
            }}
          >
            {t("more-about")}
          </button>
        </Menu>
      )}
    </>
  );
}
