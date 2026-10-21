# Fluent catalogs, regional formats, pseudo-translation (M2 slice 1c)

Claude Code implemented the internationalization groundwork the design session settled in PR #30.

- **Both catalogs are Project Fluent now** (ACC-5): `ui/src/messages/en.ftl` and `src-tauri/messages/en.ftl`, replacing M1's JSON while there were only about forty strings. "214 photos" and "1 photo" are one message with plural forms, no longer `{count} photos`.
- **A catch in "follow the region" (ACC-6):** the web view's own locale follows the Windows display language, not the region setting, so an English Windows set to Germany's formats would still have formatted the US way. The core now reads the region itself (Windows' `GetUserDefaultLocaleName`; macOS's `CFLocale`, including the `@rg=` override macOS uses for "English, with German formats") and the UI formats with it. In messages, numbers are formatted in the region while English plural rules choose the words: "1.214 photos" in Germany.
- **Pseudo-translation** (ACC-7): `SEURA_LOCALE=pseudo` doubles vowels and accents letters, about 40% longer, in both catalogs, but never in the user's own data such as folder names, which makes untranslated text stand out. Claude captured the app's window with it on; the text fits, and the capture also caught a layout bug from 1b: the developer-switch line pushed the window past its bottom edge.
- **Isolation marks off:** Fluent normally wraps inserted values in invisible bidirectional marks for right-to-left languages. With English only until after v1, they're off, so copied text and tests stay plain; they come back with right-to-left support.
- Tag matching with Unicode case folding (TAG-2) waits for tags in M3.
