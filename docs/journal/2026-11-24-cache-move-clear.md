# 2026-11-24: Moving and clearing the cache (M7 slice 3b, #273)

- **Found while planning: the cache could share a folder with the user's files.** The Welcome screen saved the folder a user picked *as* the cache, so Seura's `thumbs`, `index.bin`, `README.txt` and `WebView` went straight into it. Claude Code raised it, and **Nevada decided (decision 0019) that a picked place gets a folder of its own, `Seura Photos Cache`,** for both the Welcome screen and Move…. Caches already set up stay where they are.
- **Only Seura's own entries, by name, are ever touched.** The names are listed in one place (`CACHE_ENTRIES`, plus the web view's folder), and the one function that deletes for Clear and Move refuses any other name. A test puts the user's own files beside a cache and checks that they survive clearing, moving and launch. The rule-5 test lists the new file with its three deletions:
  - the entries themselves;
  - a web view folder Seura marked;
  - the folder Seura made for the cache, once it's empty.
- **Clear** asks first, with Cancel focused, and says the cache is rebuilt automatically (decision 0018). It then stops the thumbnail workers, deletes the thumbnails and the index snapshot, restarts the workers, and queues every thumbnail again. Library folders and settings aren't touched (SET-3).
- **Move…**
  - asks for a place and copies the thumbnails there, showing a progress bar. The index snapshot and README are written afresh at the new place.
  - then switches the app to the new place and deletes the old copy directly.
  - If the copy or saving the setting fails, only what it copied is deleted, and the cache stays where it was.
  - Moving into the cache itself is refused.
- **The web view's folder on Windows** (decision 0013): WebView2 holds it open while Seura runs, so neither Clear nor Move can remove it then.
  - Seura leaves a mark file in it, and the next launch removes it before the web view starts: in the cache that was cleared, or in caches moved away from, which the settings file lists (`leftoverCaches`).
  - A folder named `WebView` without the mark is never touched.
  - A cache on a drive that isn't fixed (a network share or a USB stick) keeps the web view's folder in the usual `Seura Photos/Cache` instead, since WebView2 doesn't support network drives. Decision 0013 had left that for SET-3.
- **Not run in the real app yet.** It's covered by unit tests, and Nevada's tests on both machines come before merging.
