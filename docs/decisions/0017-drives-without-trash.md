# 0017. Drives with no Recycle Bin or Trash

Status: Accepted · 2026-10-27 · Narrows CLAUDE.md rule 5's exceptions

## Context

CLAUDE.md rule 5 says every deletion goes to the system Recycle Bin or Trash, never permanent, with one exception: a temporary file Seura created during a save that failed.

Some drives have no Recycle Bin or Trash: on Windows, USB sticks, memory cards, and network shares; on the Mac, network shares. Nevada tested a USB stick with 0.2.0 on Windows, and Delete removed a photo permanently, without asking (#115). The `trash` crate passes `FOF_NO_UI`, which suppresses the warning Windows would otherwise show. Slice 9 had assumed that warning, without measuring it. The fix (#113) refuses any path on such a drive before calling the system, so nothing is deleted there at all.

That fix makes two of Seura's own operations fail on those drives:

- **FIL-1:** when a photo's last tag or rating is cleared, its emptied edit file can't go to the Recycle Bin.
- **FIL-4:** Start over moves an unreadable edit file into Seura's folder. Across drives that's a copy, then the original to the Trash, which isn't possible here.

Two more questions came with it: whether to let the user delete photos on such drives after a warning, and whether Seura should import photos from them.

## Decision

**Photos on a drive with no Recycle Bin or Trash are never deleted by Seura, even after a warning.** Delete leaves them out, and the confirmation says why and that File Explorer or Finder can delete them, with their own warning.

**Seura deletes a file directly, without the Trash, only when nothing in it can be lost:**

1. A temporary file Seura created during a save that failed (as before).
2. **An empty edit file (FIL-1)**, where there is no Recycle Bin or Trash. Seura reads it again immediately before deleting. If it holds any edits, tags, rating, unknown operations, fields, or extensions, or can't be read, it's left in place. Its history is lost with it, since a history of changes to nothing doesn't record anything the user would want back.
3. **An unreadable edit file after Start over (FIL-4)**, where there is no Recycle Bin or Trash, only once its copy in Seura's folder has been read back and matches byte for byte. Otherwise Start over stops, and the file is left where it was.

Every other deletion goes to the Recycle Bin or Trash, or doesn't happen. Tests enforce that no other path deletes directly.

## Rejected

- **"Delete anyway" for the user's own photos** on removable media, behind a warning with an unfocused button. It's a real use: clearing a memory card after copying. But permanent deletion of originals is the one thing Seura promises never to do (product brief, principle 1; rule 5). Explorer and Finder already do it with their own warning. Clearing a card is better done in the camera, which also formats it properly.
- **Leaving emptied edit files in place** on such drives. Nothing is lost, but it leaves files that say nothing beside the photos, contrary to FIL-1, and Cleanup would face the same question later.
- **Importing** (copying photos from a card or stick into a folder on the computer). It isn't a managed library, but it copies originals, which principle 1 rules out. The operating system's own import does it well: Windows Photos and Image Capture. The user then adds that folder. Not in v1; revisit only with a change to principle 1.

## Consequences

- CLAUDE.md rule 5 lists the three cases.
- FIL-1, FIL-4, and BRW-9 say how each behaves on such drives.
- 0.2.0 shipped with the permanent-deletion bug. Its release notes carry a warning, and 0.2.1 fixes it.
- Undo of a cleared rating or tag (BRW-11) must still work after the emptied file is gone, by writing a new edit file with the previous values.
