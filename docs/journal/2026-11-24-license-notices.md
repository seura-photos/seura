# 2026-11-24: Third-party notices (M7 slice 6, SET-5)

- **Nevada said to start #276** while the Mac session tested #217.
- **Generated, not written:** `npm run notices` writes `src-tauri/notices/third-party-notices.txt` from Cargo's and npm's metadata. CI checks that it's up to date, as it does for the bindings, and fails if a package has no license text. It covers the Rust crates linked into the app on the four platforms Seura ships on (normal dependencies only: build scripts and procedural macros don't ship), and the web interface's production packages: 322 crates and 11 packages. Each license text appears once, naming everything that uses it: 228 texts, about 760 KB.
- **32 crates ship no license file,** mostly the `objc2` family, `specta` and `webview2-com`. For those, the standard SPDX text of their declared license is used, in `src-tauri/notices/texts/`, with the copyright holders their metadata names, or "the <crate> authors" where it names none. Claude Code's choice.
- **Two corrections Claude Code made after reading the first output:**
  - Tauri's crates ship a `LICENSE.spdx` inventory beside their licenses, which is now skipped.
  - `self_cell` is "Apache-2.0 OR GPL-2.0-only" and ships the GPL text too. Seura uses it under Apache-2.0 (decision 0012), so GPL texts are left out where a package offers another license. A package with only a GPL text fails the check.
- **Windows:** About has a Third-party notices link, which opens the bundled file in the system's text viewer.
- **The Mac, raised with Nevada:** SET-5 says the About panel's credits open the list, but the panel shows credits as plain text that can't be clicked. Claude Code offered three choices: an app menu item, the whole list in the credits (about 760 KB, which might be slow to open), or a line saying where the file is. **Nevada chose the menu item,** Claude Code's recommendation: Third-Party Notices in the app menu, under About, opening the file in TextEdit as Windows does. SET-5's wording is left for a design session to update.
- **Claude Code also put a literal byte-order mark in the generator's code** by mistake; ESLint caught it, and it's now written as an escape.
