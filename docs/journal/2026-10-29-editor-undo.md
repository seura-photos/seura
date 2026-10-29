# Undo and redo in the Editor (M4 slice 2a)

- **Slice 2 split in three,** Claude Code's choice to keep each pull request small:
  - undo and redo (this one);
  - revert to original;
  - outside changes (FIL-3).
- **One history per photo, shared.** The Editor's undo is the edit format's own undo on the photo's file, through the same write path the Library's undo uses. So the Editor undoes a rating set in the Library just as it undoes brightness, when that rating is the photo's latest change. The Library's undo leaves a photo alone once its latest change is no longer the Library's. The labels come from the history, so the buttons, the macOS Edit menu and the message say "Undo Brightness" or "Undo Rating".
- **Undoing everything empties the file,** which then goes to the Trash (FIL-1). The Editor keeps the document while the photo is open, so redo writes it back, as the Library's undo does (Nevada's decision for the Library, 2026-10-26). After a restart, redo can't bring back a file that went to the Trash; every other step survives a restart, because the history is in the file.
- **A history that no longer matches** (another app changed the file without recording it) is discarded, as the format requires, and the Editor says so instead of undoing.
- **Saves and undo take turns.** A change waiting to be saved is saved first, then undone. A save that finished after the undo would otherwise have overwritten it.
- **Checked in the real app** through UI Automation on the scratch library: undo, undo to an empty file (it went to the Recycle Bin), and redo writing it back with all three records.
- **A gap for the design session, found while planning revert (2b):** EDT-10 says revert "removes all edits". A history record keeps only an operation's `params`, so undo couldn't bring back another app's operation whole (its `version`, `group` and `required`). Rule 4 also says such operations are kept. Claude Code's plan for 2b: revert removes Seura's own operations and keeps other apps', said plainly in that pull request.
