import type { KeyboardEvent } from "react";

/**
 * A dialog's keys (ACC-1): Esc closes it, and Tab and Shift+Tab go round
 * its buttons, so the focus stays in it. No key reaches the views behind,
 * which listen on the window: there a number would rate the selected
 * photos, Ctrl+Z undo, and Esc close the Viewer (#120).
 */
export function dialogKeys(event: KeyboardEvent<HTMLElement>, close: () => void) {
  event.stopPropagation();
  if (event.key === "Escape") {
    event.preventDefault();
    close();
  } else if (event.key === "Tab") {
    event.preventDefault();
    const all = [...event.currentTarget.querySelectorAll<HTMLButtonElement>("button")].filter(
      (button) => !button.disabled,
    );
    const at = all.indexOf(document.activeElement as HTMLButtonElement);
    const next = event.shiftKey ? (at <= 0 ? all.length - 1 : at - 1) : (at + 1) % all.length;
    all[next]?.focus();
  }
}
