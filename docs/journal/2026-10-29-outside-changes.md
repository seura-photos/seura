# Outside changes in the Editor (M4 slice 2c)

- **How Seura tells another app's change from its own.** Seura's own writes record the edit file's size and time in the index, so the watcher's rescan doesn't read them again. Any edit file the rescan does read again was changed outside Seura: by another app, or synced from Nevada's other computer. The scan now lists those photos, and the Editor checks whether the one it has open is among them (FIL-3).
- **A conflict only when Seura's operations changed.** The Editor keeps Seura's operations as it last saved or loaded them. A change from outside to the tags, rating or history alone isn't a conflict: the Editor just follows the new history. If the operations differ, it shows flow 11's notice, "This photo's edits were changed outside Seura Photos. Keep yours or load theirs?", in the caution style above the photo.
- **Until the user chooses,** the photo stays as they had it. They can go on editing: each change is saved onto the file as it is on disk, and added to their own version. Undo, redo and revert are off, because they act on the file's history, which is the other app's now.
- **Keep yours** writes back only Seura's operations that differ, as one change ("Keep my edits") that can be undone. The other app's operations, tags and rating stay as they are on disk. **Load theirs** writes nothing and shows the file as it is.
- **Checked in the real app** on the scratch library, by changing an open photo's edit file from PowerShell:
  - the notice appeared within the watcher's delay, and the photo stayed at +40;
  - Keep yours wrote +40 back as "Keep my edits";
  - after a second change, Load theirs showed −25.
- **Left as it is:** moving to another photo or closing the Editor before choosing leaves the file as it is on disk, the other app's version. The user's version wasn't saved anywhere else, so nothing of the file is lost. But their unsaved view is, and the pull request says so for Nevada to judge.
