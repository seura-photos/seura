/**
 * Turns off the web view's own menu (Back, Reload, Inspect), which isn't
 * Seura's: right-click opens Seura's menus where it has them, and nothing
 * elsewhere. Text fields keep theirs, for cut, copy, and paste.
 */
export function suppressWebViewMenu(target: Document = document) {
  target.addEventListener("contextmenu", (event) => {
    const element = event.target;
    const editable =
      element instanceof HTMLElement &&
      (element.isContentEditable || element.closest("input, textarea") !== null);
    if (!editable) event.preventDefault();
  });
}
