# M3 work plan: Organize

Status: draft for review · Last updated: 2026-10-26

The working plan for milestone M3 from the [milestone plan](milestone-plan.md): the first milestone that writes user data in earnest, the edit files beside your photos. It breaks the milestone into slices, one pull request each (or a few, lettered), in order. Drafted by Claude Code from the milestone plan, the [functional spec](functional-spec.md), the [user flows](ux/user-flows.md) and [wireframes](ux/wireframes.md), the [technical design](technical-design.md), and the [edit format](https://github.com/seura-photos/edit-format). The questions it couldn't settle are at the end, for a design session.

**Scope** (milestone plan, M3): edit files FIL-1 to FIL-7; tags and ratings TAG-1 to TAG-7 with the anchored tag picker; rating as a quick action (part of BRW-10); finding FND-1 to FND-3; delete to Trash (BRW-9). Also carried in: the Library issues found in M2 (#72, #73, #74), Node 26, build 0.2, and the website's early-build section (#76).

**Writing is the point, so safety comes first.** Every slice that writes adds its write path to the data-safety suite (`tests/data_safety`) before it merges: originals byte-identical, nothing written outside an edit file beside its photo, writes atomic, unreadable and changed files never overwritten, unknown content kept. M1's Editor already writes edit files this way; M3 adds the Library's writes beside it.

**Not in M3:** rotate and Enhance as quick actions (M4, with the Editor), Review (REV), undo of tagging and rating in the Library (see the questions), and anything that changes a photo itself.

**Done when** (milestone plan): Nevada is tagging and rating his own library, the edit files pass the edit format's `check.py`, and the data-safety suite covers every new write path.

## How to work

As in M2 ([M2 work plan](m2-plan.md), "How to work"): one slice per branch and pull request, branched from `main` and named `m3/<slice>`; Nevada merges. Every pull request's base is `main` (none stacked on another's branch). Dependencies at least two weeks old, checked against decision 0012. Each slice has an issue under the "M3" milestone on GitHub, and its pull request closes it. Screens are styled from the tokens and checked against the mockups, with screenshots in light and dark mode in the pull request. Journal everything (README, "Consider everything for the devlog").

New in M3:

- **Edit files written by the Library go through one path in the core**, used by every view (grid, Viewer, Info panel, tag picker), so the rules (lifecycle, atomic writes, never overwriting what can't be read or changed on disk) live in one place.
- **Real photos for testing writes stay out of the repository**, as before; Nevada checks writes on a copy of a folder first, then on his library.

## Slices

### 1. Writing edit files from the Library (FIL-1, FIL-2, TAG-7)

- One core service applies a tag or rating change to one photo or a selection: loads each photo's edit file, applies the action (`seura-edit`'s `Edit::Tags` and `Edit::Rating`), saves atomically, and updates the index, so badges, counts, and the Info panel follow.
- **Creating** an edit file on a photo's first tag or rating, with the photo's identity. A cloud-only photo's file is created without the fingerprint and dimensions, as the format allows, and filled in when the photo is next read (LIB-9; `EditDocument::fill_in_photo`).
- **Removing:** when a photo has no edits, tags, or rating left and its file holds nothing else to keep, the file goes to the Recycle Bin or Trash (FIL-1), never deleted. A Trash library, chosen and checked here (decision 0012), serves slice 8 too.
- **Read-only folders** (TAG-7): detected when a folder is scanned; tagging and rating are off there, and the controls say why.
- **Seura's own writes** don't come back as outside changes: the watcher's report of a file Seura just wrote is recognized and ignored.
- Data-safety tests for each path: create, change, remove to Trash, a cloud-only photo, a read-only folder, and a failed write.
- **Done when:** the data-safety suite covers every write path above on both platforms, and a rating set from a test command shows in the grid and survives a restart.

### 2. Ratings (TAG-6, BRW-10)

- `0` to `5`, and the stars in the Info panel, rate the selection in the Library or the photo in the Viewer; `0` clears. The grid's rating badges and the Info panel follow at once.
- A selection with mixed ratings shows that, and a rating applies to all of them.
- **Done when:** Nevada rates photos on both machines by key and by clicking, and the ratings are there after a restart and in the edit files.

### 3. Tags (TAG-1 to TAG-4)

- **The tag picker** (`T`, or Add tag in the Info panel), from the wireframe: a popover anchored beside the selection, "Tag 3 photos", chips for tags already there with ×, a text field, existing tags with counts filtered as you type, the first match highlighted, and "Create tag "…"" when nothing matches. Enter adds; the picker stays open for more; Esc closes. Keyboard and screen reader complete (ACC-1, ACC-2).
- **Tag chips** in the Info panel, × to remove from the photo or the whole selection (TAG-4).
- **Cleaning and matching** typed tags (TAG-2): trimmed, empty levels removed, and matched with NFC normalization and full Unicode case folding (today's matching only lowercases).
- **Done when:** Nevada tags photos on both machines, one at a time and in a selection, and a tag typed in a different case finds the existing one.

### 4. The tag tree (TAG-5)

- The sidebar's TAGS section, below FOLDERS, with its own sticky heading: `family/kids` shown as `kids` under `family`, each node with its photo count; parent nodes exist even when no photo has the plain tag (TAG-1).
- Selecting a node shows the photos with exactly that tag, from every folder, in the grid and the Viewer. A tag no photo uses disappears (TAG-4).
- **Done when:** Nevada's tags appear as a tree on both machines, and selecting one shows exactly its photos.

### 5. Finding (FND-1 to FND-3)

- **Search** (Ctrl/Cmd+F) in the header: file names and tags across the whole library, results as you type; Esc clears.
- **Filters** behind a Filter button in the folder header: minimum rating, tags (all of them), edited or unedited, date range. Active filters show as chips with Clear all, always visible.
- The index already filters and searches (M1, decision 0010: a search keystroke under 8 ms at 100,000 photos); this slice is the UI and wiring.
- **Done when:** Nevada finds photos by name, tag, rating, and date on both machines, and the measurement script shows search within 200 ms at 10,000 photos (FND-3).

### 6. Problems with files (FIL-3, FIL-4, FIL-5, FIL-7)

- A notice bar at the top of the main area, never a blocking dialog (user flows, flow 11):
  - **Changed on disk** (FIL-3): reloaded, and the index follows; if the change conflicts with a change Seura is making, "Keep yours or load theirs?".
  - **Unreadable** (FIL-4): the photo shows unedited with its warning badge (since M1); the notice offers Start over, keeping the old file as a backup.
  - **Unknown content** (FIL-5): kept on save, as since M1, and the user is told it isn't shown.
  - **Original changed** (FIL-7): the fingerprint no longer matches; the edits still apply, with a warning.
- **Done when:** each case is covered by data-safety or integration tests, and Nevada has seen the notices on purpose-made examples.

### 7. Renamed photos (FIL-6)

- Fingerprints computed in the background for photos on disk (never for cloud-only ones), cached in the index.
- A photo renamed outside Seura gets its edit file back automatically when another photo in the same folder has the same fingerprint and no edit file of its own: the edit file is renamed to match (an edit file, never a photo).
- **Done when:** integration tests rename photos with edit files on both platforms, and Nevada has renamed a tagged photo in Explorer and Finder and seen its tags follow.

### 8. Delete to Trash (BRW-9)

- Delete (Windows) or Cmd+Delete (macOS), or the photo menu: after a confirmation that says how many and that they go to the Recycle Bin or Trash, the selected photos and their edit files move there. Never permanent. Not in read-only folders.
- The Viewer moves to the next photo; the grid and index follow.
- **Done when:** data-safety tests show photos and edit files reach the Trash and nothing else changes, on both platforms, and Nevada has deleted and restored photos on both machines.

### 9. Library polish (#72, #73, #74)

- The folder title stays on one line, and a long path gives way (#72).
- Thumbnails keep the slider's size; the slider resizes them smoothly; the window changes the columns, with leftover space spread evenly (#73).
- A resizable sidebar, remembered, with full folder names on hover (#74), once the design session has updated the wireframes.
- **Done when:** Nevada has checked each on both machines.

### 10. Wrap-up

- Node 26, a few weeks after it becomes LTS (2026-10-28), if that falls within M3; otherwise early M4.
- `check.py` from the edit format over the edit files Seura wrote in Nevada's library.
- The measurement script again, with search added; results in `docs/benchmarks/`.
- Build 0.2, with the Mac's ad-hoc signing (#71), and the website's early-build section (#76) with a design session.
- Confirm every "done when"; the devlog post, part 9.
- **Done when:** Nevada is tagging and rating his own library with build 0.2 on both machines.

## Questions for a design session

1. **Undo in the Library.** The keyboard table lists Ctrl+Z, and the edit format records tag and rating changes in each file's history. Should tagging and rating be undoable from the Library in M3 (one step back for the last change, across the photos it touched), or wait for the Editor's undo in M4?
2. **Deleting cloud-only photos.** Moving a OneDrive or iCloud photo to the Recycle Bin also deletes it from the cloud (into the service's own recycle bin). Should the confirmation say so when the selection includes cloud photos, and should anything else differ?
3. **FIL-4's backup.** Start over keeps the unreadable file as a backup. Where, and named how? Beside the photo (for example `IMG_1234.jpg.seura.json.bak`, which the scanner ignores), or in Seura's folder? The edit format doesn't say.
4. **Search and the current view.** Search covers the whole library (FND-2). While searching, does the grid show results in place of the selected folder, with the sidebar's selection cleared, and does Esc return to the folder?
5. **Filters and scope.** Do filters narrow the current folder or tag only, and do they stay set when moving to another folder?
6. **Tag counts for parent nodes.** TAG-5 counts photos with exactly that tag. A node like `family` with no photos of its own would show 0; should it show nothing instead, or the total beneath it?
7. **Tags and ratings for a mixed selection.** The Info panel shows what a selection shares. With mixed ratings, do the stars show empty, or a "mixed" state, until one is chosen?
8. **Read-only folders.** How should the controls "say why" (TAG-7): a disabled star row with a tooltip, or a line in the Info panel?
