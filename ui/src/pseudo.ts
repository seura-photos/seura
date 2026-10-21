/**
 * Text as a translation might run: about 40% longer, and accented, so
 * clipped or untranslated text stands out (ACC-7; `SEURA_LOCALE=pseudo`).
 * Each vowel is doubled and common letters accented, exactly as the core
 * does for its own text (`src-tauri/src/locale.rs`).
 */
export function pseudoTranslate(text: string): string {
  const from = "aceinouyACEINOUY";
  const to = "áçéîñöûýÁÇÉÎÑÖÛÝ";
  let out = "";
  for (const c of text) {
    const i = from.indexOf(c);
    const accented = i >= 0 ? (to[i] ?? c) : c;
    out += "aeiouAEIOU".includes(c) ? accented + accented : accented;
  }
  return out;
}
