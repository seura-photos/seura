# Open elsewhere (M2 slice 9)

- **Right-click a photo**, in the grid or the Viewer, for Show in Explorer (Finder on the Mac) and Open in default app (BRW-8). The context-menu key and Shift+F10 open the same menu, for the keyboard (ACC-1). The design documents don't say where these live; Claude chose the menu every file manager uses and asked Nevada in the PR.
- **Rust does both** (decision 0009): Tauri's opener plugin is used only as a library, never registered, so the UI gains no new permission. It reveals several files at once, selected in Explorer or Finder, which is what "a selection" needs.
- **At most 20 photos open in the default app at once**, so selecting a whole folder and choosing Open doesn't open hundreds of windows. Claude's choice, raised in the PR.
- **A new dependency caught by the two-week rule:** the opener plugin's own dependency `open` had a release published that very day (5.4.5). Claude pinned the previous one, 5.4.4, from September. The other 36 packages the plugin adds to the lock file are Linux-only and never built for Windows or macOS.
- **Cloud-only photos:** showing one in Explorer or Finder reads only its attributes, so nothing downloads. Opening one in another app downloads it, by that app; the user asked for it.
