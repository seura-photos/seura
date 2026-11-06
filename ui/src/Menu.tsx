import {
  type CSSProperties,
  type KeyboardEvent,
  type ReactNode,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

/** How close a menu comes to the window's edges. */
const MARGIN = 8;

type Props = {
  x: number;
  y: number;
  /** "end" puts its right edge at `x`, for a menu from a button at the
   * window's right edge. */
  align?: "start" | "end";
  /** A submenu: ← closes it too, back to the item that opened it. It has no
   * backdrop of its own, so the menu that opened it stays under the
   * pointer; that menu's backdrop closes both. */
  submenu?: boolean;
  /** Where a submenu's right edge goes when there's no room at `x`: the left
   * of the menu that opened it. */
  flipX?: number | undefined;
  /** Whether the first item takes the focus: not for a submenu opened by
   * hovering, which leaves the focus where it is. */
  focusFirst?: boolean;
  onClose: () => void;
  /** Buttons with `role="menuitem"` and the `menu-item` class. */
  children: ReactNode;
};

/**
 * A context menu (ACC-1, following the ARIA menu pattern): the first item
 * takes the focus, ↑ and ↓ move between items, Home and End go to the ends,
 * and Esc or Tab closes it. When it closes, the focus goes back where it
 * was, so the context-menu key works again there. Clicking outside or
 * leaving the window closes it too. It stays inside the window: one that
 * would run off the bottom moves up, and one taller than the window scrolls.
 */
export function Menu({
  x,
  y,
  align = "start",
  submenu = false,
  flipX,
  focusFirst = true,
  onClose,
  children,
}: Props) {
  const menu = useRef<HTMLDivElement>(null);
  // The latest onClose, so opening and closing happen once however often
  // the owner renders meanwhile.
  const close = useRef(onClose);
  useEffect(() => {
    close.current = onClose;
  }, [onClose]);
  // Where it goes once it's measured, to stay inside the window.
  const [placed, setPlaced] = useState<CSSProperties | null>(null);

  useLayoutEffect(() => {
    const box = menu.current?.getBoundingClientRect();
    if (!box) return;
    const room = window.innerHeight - 2 * MARGIN;
    const fitted: CSSProperties = {};
    if (box.height > room) {
      fitted.top = MARGIN;
      fitted.maxHeight = room;
      fitted.overflowY = "auto";
    } else if (box.bottom > window.innerHeight - MARGIN) {
      fitted.top = window.innerHeight - MARGIN - box.height;
    }
    if (align === "start" && box.right > window.innerWidth - MARGIN) {
      fitted.left = Math.max(
        MARGIN,
        flipX !== undefined ? flipX - box.width : window.innerWidth - MARGIN - box.width,
      );
    }
    // Set from the first layout, before it's painted.
    setPlaced(fitted);
  }, [align, flipX]);

  useEffect(() => {
    const before = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    if (focusFirst) items(menu.current)[0]?.focus();
    const onBlur = () => close.current();
    window.addEventListener("blur", onBlur);
    return () => {
      window.removeEventListener("blur", onBlur);
      if (focusFirst && before?.isConnected) before.focus();
    };
  }, [focusFirst]);

  function onKeyDown(event: KeyboardEvent) {
    const all = items(menu.current);
    const at = all.indexOf(document.activeElement as HTMLButtonElement);
    let next: number | null = null;
    if (event.key === "ArrowDown") next = (at + 1) % all.length;
    else if (event.key === "ArrowUp") next = (at - 1 + all.length) % all.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = all.length - 1;
    else if (event.key === "Escape" || event.key === "Tab") onClose();
    else if (event.key === "ArrowLeft" && submenu) onClose();
    else return;
    event.preventDefault();
    // The Viewer's keys (Esc, the arrows) mustn't act while the menu is open.
    event.stopPropagation();
    if (next !== null) all[next]?.focus();
  }

  const position: CSSProperties =
    align === "end" ? { left: x, top: y, transform: "translateX(-100%)" } : { left: x, top: y };
  const shown = (
    <div
      ref={menu}
      className="menu"
      role="menu"
      style={{ ...position, ...placed }}
      onClick={(event) => event.stopPropagation()}
      onKeyDown={onKeyDown}
    >
      {children}
    </div>
  );
  if (submenu) return shown;
  return (
    <div
      className="menu-backdrop"
      onClick={onClose}
      onContextMenu={(event) => event.preventDefault()}
    >
      {shown}
    </div>
  );
}

/** The items that can take the focus: disabled ones are skipped. */
function items(menu: HTMLElement | null): HTMLButtonElement[] {
  return [...(menu?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]') ?? [])].filter(
    (item) => !item.disabled,
  );
}
