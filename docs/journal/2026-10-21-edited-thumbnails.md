# Edited thumbnails and HEIC dates (M2 slice 5b)

- **Thumbnails show edits** (BRW-1). An edited photo's thumbnail is rendered through the same shader as the preview and export (decision 0004), so there's still one implementation of each operation; in M2 that means exposure. Its cache key now includes the edit file's size and date, so changing an edit makes a new thumbnail, while unedited photos' keys didn't change and nothing had to be remade.
- **No extra wiring for updates:** saving an edit writes the edit file, the watcher (slice 4) sees it, the rescan reads the new date, and the new key makes the new thumbnail.
- **A race found by reading, not running:** each render writes its settings into one shared GPU buffer and then draws. With several thumbnail workers rendering at once, one photo's exposure could have been drawn with another's. The renderer is now shared app-wide and used by one caller at a time.
- **HEIC photos sort by date taken now:** the date comes from the file's EXIF item. One of Nevada's iPhone photos (kept local) reads correctly; the committed HEIC test files carry no date, so a JPEG stands in for the test of reading EXIF from a whole file.
