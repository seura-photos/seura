# Nevada's first tests of slice 9, and a scan that froze the grid

- **Nevada found four problems on Windows** while #47's checks ran:
  1. The photo menu opened with Shift+F10, but ↑ and ↓ didn't move between its items.
  2. After Esc closed it, Shift+F10 opened the web view's own menu (with Inspect) instead.
  3. Right-clicking almost anywhere opened that web view menu, which gives away that the app is a web page.
  4. With his OneDrive Pictures folder added (5,835 photos), he could select other folders while their thumbnails were being made, but the grid didn't change until all of them were done.
- **1 to 3:** Seura's menus now follow the standard menu pattern: ↑ and ↓, Home and End, and Esc or Tab to close. Closing gives the focus back to where it was, so the menu key works again. The web view's own menu is turned off everywhere except text fields, which keep Cut, Copy, and Paste. Claude checked the keyboard path in the real app.
- **4:** every finished thumbnail sent its own event, 5,835 in a row, and the UI copied its whole list of thumbnail states on each one. The command that lists a folder's photos ran on the app's main thread, which also delivers those events, so a click waited behind them. Now finished thumbnails go to the UI in batches, about ten a second, with one update each, and the quick commands that read the index run off the main thread. Claude couldn't reproduce it with every thumbnail already made, so Nevada will see whether it's fixed the next time a large folder is scanned.
