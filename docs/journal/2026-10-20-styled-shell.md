# The styled shell (M2 slice 1b)

Claude Code built the second half of M2 slice 1: the Library's frame, styled from the design tokens.

- **Tokens to CSS:** `design/tokens.json` is turned into CSS variables by `npm run tokens`, and a test fails when the generated file is out of date, like the command bindings and the shaders. Every length is in rem, so the interface follows the system text size.
- **System text size (ACC-3):** Windows' "Text size" setting is read from the core (`UISettings.TextScaleFactor`). In case the web view already applies it to its default font, the UI measures that and adds only the rest, so nothing is scaled twice. macOS has no system-wide text size apps can read. Reduced motion turns off animation.
- **Only what's built:** the header has the mark and "Seura"; the sidebar FOLDERS with counts and Add folder; a folder heading; the status bar. Search, Review, quick actions, tags, and the Info panel come with their slices.
- **Menus:** macOS gets a native menu with Seura's commands, its labels from a small catalog in the core (ACC-5). Windows has no menu bar for now, since the mockups show none and modern Windows apps rarely have one; raised as a question for a design session rather than decided.
- **M1's Editor** is available in development builds only, since M2 writes no edit files.
- **A small catch:** the mark first appeared as a broken image in the built app. Vite inlines small images as `data:` URLs, which the app's Content Security Policy doesn't allow; it now always emits them as files. Claude found it by capturing only the app's own window with Windows' `PrintWindow`, the first time it could see the app itself rather than ask Nevada for screenshots.
- **Deferred:** a Vite advisory (its development server's open-in-editor helper, not part of the app) is fixed in Vite 8.3.4, which is under two weeks old until 2026-10-22.
