/**
 * Whether a key press is typing into a field. Single-letter and number keys
 * work in every view except while typing (user flows, "Keyboard").
 */
export function isTyping(event: KeyboardEvent): boolean {
  const target = event.target;
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  if (target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement) return true;
  // Buttons, checkboxes, and sliders aren't typed into.
  return (
    target instanceof HTMLInputElement &&
    !["button", "checkbox", "radio", "range"].includes(target.type)
  );
}

const isMac = typeof navigator !== "undefined" && /Mac/.test(navigator.platform);

/** Whether a key press is Select All: Ctrl+A, or Cmd+A on a Mac, outside a
 * text field, where it selects the text as usual. */
export function isSelectAll(event: KeyboardEvent): boolean {
  const command = isMac ? event.metaKey : event.ctrlKey;
  return command && !event.altKey && event.key.toLowerCase() === "a" && !isTyping(event);
}

/** Whether a key press is the Library's Undo or Redo (BRW-11): Ctrl+Z, and
 * Shift+Ctrl+Z or Ctrl+Y, outside a text field. Windows only: on a Mac,
 * Cmd+Z goes to the Edit menu's Undo, which names the action, and the menu
 * passes it on, so it happens once. */
export function historyKey(event: KeyboardEvent): "undo" | "redo" | null {
  if (isMac || !event.ctrlKey || event.altKey || event.metaKey || isTyping(event)) return null;
  const key = event.key.toLowerCase();
  if (key === "z") return event.shiftKey ? "redo" : "undo";
  if (key === "y" && !event.shiftKey) return "redo";
  return null;
}

/** Whether a key press is Find: Ctrl+F, or Cmd+F on a Mac (FND-2). */
export function isFind(event: KeyboardEvent): boolean {
  const command = isMac ? event.metaKey : event.ctrlKey;
  return command && !event.altKey && !event.shiftKey && event.key.toLowerCase() === "f";
}

/** Whether a key press is `T`, for the tag picker (TAG-3). */
export function isTagKey(event: KeyboardEvent): boolean {
  if (event.altKey || event.ctrlKey || event.metaKey || isTyping(event)) return false;
  return event.key === "t" || event.key === "T";
}

/** The rating a key press asks for, 0 to 5 (TAG-6), or null. */
export function ratingKey(event: KeyboardEvent): number | null {
  if (event.altKey || event.ctrlKey || event.metaKey || isTyping(event)) return null;
  return /^[0-5]$/.test(event.key) ? Number(event.key) : null;
}

/** Whether a key press is Delete for photos (BRW-9): the Delete key,
 * outside a text field. Windows only: on a Mac, Cmd+Delete goes to the
 * File menu's Move to Trash, which passes it on, so it happens once. */
export function isDeleteKey(event: KeyboardEvent): boolean {
  if (isMac || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return false;
  return event.key === "Delete" && !isTyping(event);
}

/** The quick action a key press asks for (BRW-10): `[` and `]` rotate
 * left and right, `A` enhances; or null. */
export function quickKey(event: KeyboardEvent): "left" | "right" | "enhance" | null {
  if (event.altKey || event.ctrlKey || event.metaKey || isTyping(event)) return null;
  if (event.key === "[") return "left";
  if (event.key === "]") return "right";
  if (event.key === "a" || event.key === "A") return "enhance";
  return null;
}

/** Whether a key press is `E`, for the Editor (EDT-1). */
export function isEditKey(event: KeyboardEvent): boolean {
  if (event.altKey || event.ctrlKey || event.metaKey || isTyping(event)) return false;
  return event.key === "e" || event.key === "E";
}

/** Whether a key is `\`, held to compare with the original (EDT-1). */
export function isCompareKey(event: KeyboardEvent): boolean {
  if (event.altKey || event.ctrlKey || event.metaKey || isTyping(event)) return false;
  return event.key === "\\";
}
