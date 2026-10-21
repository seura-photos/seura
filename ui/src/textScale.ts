import { commands } from "./bindings";

/**
 * Follows the system text size (ACC-3). Every size in the app is in rem, so
 * scaling the root font scales text and controls together. The core reports
 * the system's scale (Windows' "Text size"; always 1 on macOS, which has no
 * system-wide setting apps can read). If the web view already applies it to
 * its default font size, only the remainder is added, so nothing is scaled
 * twice.
 */
export async function applyTextScale(root: HTMLElement = document.documentElement) {
  const system = (await commands.textScale()) ?? 1;
  root.style.removeProperty("--text-scale");
  const probe = document.createElement("div");
  probe.style.fontSize = "medium";
  document.body.append(probe);
  const webViewDefault = parseFloat(getComputedStyle(probe).fontSize) || 16;
  probe.remove();
  const alreadyApplied = webViewDefault / 16;
  root.style.setProperty("--text-scale", String(system / alreadyApplied));
}

/** Applies the scale now, and again whenever the window regains focus, in
 * case the setting changed meanwhile. */
export function followTextScale() {
  void applyTextScale();
  window.addEventListener("focus", () => void applyTextScale());
}
