# The folder tree (M2 slice 3a)

- **Nevada checked slices 1 and 2 by eye** on both machines, in light and dark mode: both look right. Seura follows the system's appearance; there's no switch of its own (ACC-3, SET-2).
- **No menu bar on Windows** (Nevada, 2026-10-21): not needed unless something would live only there. Everything in the macOS menu is also in the window and on the keyboard.
- **The FOLDERS tree** (LIB-2): library folders always, even empty ones, and every subfolder with photos anywhere below it. Each folder shows its total including subfolders, as the mockup's "2026 · 640" (Spring + Summer + Scans) implies; the grid still shows only the folder's own photos (BRW-1). The keyboard follows the ARIA tree pattern, and the selection follows the focus, as in Explorer's navigation pane.
- **Removing a library folder** (LIB-5): right-click or the context-menu key, then a confirmation where Cancel has the focus. The photos and their edit files are untouched; only the index entries and the thumbnails in the cache go.
- **Unavailable drives** (LIB-6): a disconnected library folder is dimmed and says "Not connected"; a check every few seconds notices it coming back and rescans.
- **Caught on screen, not by the tests:** with nothing selected, the main area showed an empty library folder's heading. An empty library folder has no id (`null`), and "nothing selected" is also `null`, so the lookup matched it. Claude saw it in a capture of the app's window; the fix has a test that fails without it.
