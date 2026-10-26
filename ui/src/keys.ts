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

/** The rating a key press asks for, 0 to 5 (TAG-6), or null. */
export function ratingKey(event: KeyboardEvent): number | null {
  if (event.altKey || event.ctrlKey || event.metaKey || isTyping(event)) return null;
  return /^[0-5]$/.test(event.key) ? Number(event.key) : null;
}
