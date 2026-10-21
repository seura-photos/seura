# Built to be translated, English first

- Nevada asked whether internationalization had been discussed: translating the interface and letting the user choose the display language. Only ACC-5 existed ("English only in v1, text kept separate from code"); a language choice had never come up.
- Claude's view, in the design session: keep v1 English-only, but settle now the choices that are cheap early and expensive later. Nevada agreed.
  - **One catalog format with plurals and word order** (ACC-5). M2's first slice already had simple JSON catalogs, and its English file showed why: `"{count} photos"` would say "1 photos", and many languages have several plural forms. The technical design now names Project Fluent for both the interface and the Rust core, replacing the JSON while there are only a few dozen strings.
  - **Dates and numbers follow the region** (ACC-6), even in English: a German user sees `12.07.2026, 18:42`.
  - **Room for 40% longer text** (ACC-7), checked with a pseudo-translated build, because the layout is fixed by design (INT-2) and translated labels mustn't be clipped.
  - **Tag matching across languages** (TAG-2): Unicode case folding, so `Ärzte` matches `ärzte`.
- After v1: translations (likely volunteers on a free translation platform, or AI drafts reviewed by native speakers), the language following the operating system with a setting to override it (Windows has no per-app language choice), and right-to-left languages later still.
- The website stays English-only until the app has translations.
