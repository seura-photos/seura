# Scan progress (M2 slice 3b)

- **The whole library gets thumbnails now**, not just the folder on screen: after every scan, every photo's thumbnail is queued behind the visible ones (LIB-3, PRF-3). Ones already in the cache are skipped almost at once.
- **The status bar says how far it's got**: "Looking for photos in Pictures…" while a library folder is scanned, then "Making thumbnails · 1,240 of 3,862" with a bar, and nothing once it's done. Updates go out at most four times a second.
- **A deadlock caught in review, before it ran:** moving the cache (from the Welcome screen) stopped the old thumbnail workers while holding the lock that a finishing worker now reads to report progress. Each would have waited for the other. Claude found it by checking the lock order after adding progress; the old workers now stop after the lock is released.
- **Left for slice 6:** a scan holds the index while it walks one library folder, so on a very large first scan, clicking a folder waits until that folder's walk ends. Moving the walk off the lock belongs with the scanning speed work.
