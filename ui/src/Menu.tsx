import { type KeyboardEvent, type ReactNode, useEffect, useRef } from "react";

type Props = {
  x: number;
  y: number;
  onClose: () => void;
  /** Buttons with `role="menuitem"` and the `menu-item` class. */
  children: ReactNode;
};

/**
 * A context menu (ACC-1, following the ARIA menu pattern): the first item
 * takes the focus, ↑ and ↓ move between items, Home and End go to the ends,
 * and Esc or Tab closes it. When it closes, the focus goes back where it
 * was, so the context-menu key works again there. Clicking outside or
 * leaving the window closes it too.
 */
export function Menu({ x, y, onClose, children }: Props) {
  const menu = useRef<HTMLDivElement>(null);
  // The latest onClose, so opening and closing happen once however often
  // the owner renders meanwhile.
  const close = useRef(onClose);
  useEffect(() => {
    close.current = onClose;
  }, [onClose]);

  useEffect(() => {
    const before = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    items(menu.current)[0]?.focus();
    const onBlur = () => close.current();
    window.addEventListener("blur", onBlur);
    return () => {
      window.removeEventListener("blur", onBlur);
      if (before?.isConnected) before.focus();
    };
  }, []);

  function onKeyDown(event: KeyboardEvent) {
    const all = items(menu.current);
    const at = all.indexOf(document.activeElement as HTMLButtonElement);
    let next: number | null = null;
    if (event.key === "ArrowDown") next = (at + 1) % all.length;
    else if (event.key === "ArrowUp") next = (at - 1 + all.length) % all.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = all.length - 1;
    else if (event.key === "Escape" || event.key === "Tab") onClose();
    else return;
    event.preventDefault();
    // The Viewer's keys (Esc, the arrows) mustn't act while the menu is open.
    event.stopPropagation();
    if (next !== null) all[next]?.focus();
  }

  return (
    <div
      className="menu-backdrop"
      onClick={onClose}
      onContextMenu={(event) => event.preventDefault()}
    >
      <div
        ref={menu}
        className="menu"
        role="menu"
        style={{ left: x, top: y }}
        onClick={(event) => event.stopPropagation()}
        onKeyDown={onKeyDown}
      >
        {children}
      </div>
    </div>
  );
}

/** The items that can take the focus: disabled ones are skipped. */
function items(menu: HTMLElement | null): HTMLButtonElement[] {
  return [...(menu?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]') ?? [])].filter(
    (item) => !item.disabled,
  );
}
