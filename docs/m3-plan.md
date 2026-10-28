# M3 work plan: Organize

Status: complete · Last updated: 2026-10-27

The working plan for milestone M3 from the [milestone plan](milestone-plan.md): the first milestone that writes user data in earnest, the edit files beside your photos. It breaks the milestone into slices, one pull request each (or a few, lettered), in order. Drafted by Claude Code from the milestone plan, the [functional spec](functional-spec.md), the [user flows](ux/user-flows.md) and [wireframes](ux/wireframes.md), the [technical design](technical-design.md), and the [edit format](https://github.com/seura-photos/edit-format). The eight questions it couldn't settle went to a design session and are answered in the spec (#79); see "Questions, answered" at the end.

**Scope** (milestone plan, M3): edit files FIL-1 to FIL-7; tags and ratings TAG-1 to TAG-8 with the anchored tag picker; rating as a quick action (part of BRW-10); undo in the Library (BRW-11); finding FND-1 to FND-3; delete to Trash (BRW-9). Also carried in: the Library issues found in M2 (#72, #73, #74), Node 26, build 0.2, and the website's early-build section (#76).

**Writing is the point, so safety comes first.** Every slice that writes adds its write path to the data-safety suite (`tests/data_safety`) before it merges: originals byte-identical, nothing written outside an edit file beside its photo or Seura's own folder, writes atomic, unreadable and changed files never overwritten, unknown content kept. M1's Editor already writes edit files this way; M3 adds the Library's writes beside it.

**Not in M3:** rotate and Enhance as quick actions (M4, with the Editor), Review (REV), and anything that changes a photo itself.

**Done when** (milestone plan): Nevada is tagging and rating his own library, the edit files pass the edit format's `check.py`, and the data-safety suite covers every new write path.

## How to work

As in M2 ([M2 work plan](m2-plan.md), "How to work"): one slice per branch and pull request, branched from `main` and named `m3/<slice>`; Nevada merges. Every pull request's base is `main` (none stacked on another's branch). Dependencies at least two weeks old, checked against decision 0012. Each slice has an issue under the "M3" milestone on GitHub, and its pull request closes it. Screens are styled from the tokens and checked against the mockups, with screenshots in light and dark mode in the pull request. New text goes in the message catalogs (ACC-5). Journal everything (README, "Consider everything for the devlog").

New in M3:

- **Edit files written by the Library go through one path in the core**, used by every view (grid, Viewer, Info panel, tag picker), so the rules (lifecycle, atomic writes, never overwriting what can't be read or changed on disk) live in one place, and every change is one action that undo can find again (BRW-11).
- **Real photos for testing writes stay out of the repository**, as before; Nevada checks writes on a copy of a folder first, then on his library.

## Slices

### 1. Writing edit files from the Library (FIL-1, FIL-2, TAG-7)

- One core service applies a tag or rating change to one photo or a selection, as one action: loads each photo's edit file, applies the change (`seura-edit`'s `Edit::Tags` and `Edit::Rating`), saves atomically, and updates the index, so badges, counts, and the Info panel follow. It reports which photos the action changed, for undo (slice 5).
- **Creating** an edit file on a photo's first tag or rating, with the photo's identity. A cloud-only photo's file is created without the fingerprint and dimensions, as the format allows, and filled in when the photo is next read (LIB-9; `EditDocument::fill_in_photo`).
- **Removing:** when a photo has no edits, tags, or rating left and its file holds nothing else to keep, the file goes to the Recycle Bin or Trash (FIL-1), never deleted. A Trash library, chosen and checked here (decision 0012), serves slices 7 and 9 too.
- **Read-only folders** (TAG-7): detected when a folder is scanned, and reported to the UI; the service refuses to write there.
- **Seura's own writes** don't come back as outside changes: the watcher's report of a file Seura just wrote is recognized and ignored.
- Data-safety tests for each path: create, change, remove to Trash, a cloud-only photo, a read-only folder, and a failed write.
- **Done when:** the data-safety suite covers every write path above on both platforms, and a rating set from a test command shows in the grid and survives a restart.

### 2. Ratings (TAG-6, TAG-7, TAG-8, BRW-10)

- `0` to `5`, and the stars in the Info panel, rate the selection in the Library or the photo in the Viewer; `0` clears. The grid's rating badges and the Info panel follow at once.
- **Mixed ratings** in a selection (TAG-8): empty stars with "Mixed" beside them; choosing a rating sets all of them.
- **Read-only folders** (TAG-7): the stars stay in place, disabled, with a line where they are: "This folder is read-only, so tags and ratings can't be saved here." `0` to `5` show the same message as a notice. Not a tooltip alone.
- **Notices:** the brief notice used here is the one undo (slice 5) and file problems (slice 7) use too.
- **Done when:** Nevada rates photos on both machines by key and by clicking, and the ratings are there after a restart and in the edit files.

### 3. Tags (TAG-1 to TAG-4, TAG-7, TAG-8)

- **The tag picker** (`T`, or Add tag in the Info panel), from the wireframe: a popover anchored beside the selection, "Tag 3 photos", chips for tags already there with ×, a text field, existing tags with counts filtered as you type, the first match highlighted, and "Create tag "…"" when nothing matches. Enter adds; the picker stays open for more; Esc closes. Keyboard and screen reader complete (ACC-1, ACC-2). Counts follow TAG-5's rule: a tag no photo has exactly shows no count.
- **Tag chips** in the Info panel, × to remove from the photo or the whole selection (TAG-4).
- **Partial tags** in a selection (TAG-8): a chip with a count, "beach · 3 of 7", whose × removes the tag from the photos that have it; the picker lists such tags first, and Enter adds the tag to the rest.
- **Read-only folders** (TAG-7): Add tag disabled in place, under the same line as the stars; `T` shows the message as a notice.
- **Cleaning and matching** typed tags (TAG-2): trimmed, empty levels removed, and matched with NFC normalization and full Unicode case folding (today's matching only lowercases).
- **Done when:** Nevada tags photos on both machines, one at a time and in a selection, and a tag typed in a different case finds the existing one.

### 4. The tag tree (TAG-5)

- The sidebar's TAGS section, below FOLDERS, with its own sticky heading: `family/kids` shown as `kids` under `family`, each node with its photo count; parent nodes exist even when no photo has the plain tag (TAG-1), and show no count then.
- Selecting a node shows the photos with exactly that tag, from every folder, in the grid and the Viewer. A tag no photo uses disappears (TAG-4).
- **Done when:** Nevada's tags appear as a tree on both machines, and selecting one shows exactly its photos.

### 5. Undo in the Library (BRW-11)

- Ctrl/Cmd+Z undoes the last action, whatever is selected now; Shift+Ctrl/Cmd+Z redoes it. The app keeps, in memory only, which photos each of the session's last 10 actions touched; the changes themselves are only in each edit file's history.
- Undoing is the edit format's own undo on each of those photos (`EditDocument::undo`, since M1: the record is marked `"undone": true` and its previous values restored, never a new change appended); redo clears the mark. A photo is skipped when its latest record that isn't undone isn't the one from that action (changed since in the Editor or by another app), or when it fails the consistency check.
- What was undone is said: a notice on both platforms ("Undid rating on 5 photos", with how many were skipped), and on macOS the Edit menu's label ("Undo Rating (5 photos)").
- When a change sent an emptied edit file to the Trash (FIL-1), the Library keeps that file in memory, for the 10 actions it remembers, and undo or redo writes it back with the record marked (Nevada's decision, 2026-10-26; journal, `2026-10-26-library-writes.md`).
- Nothing to undo after a restart. Moving to the Trash isn't undoable.
- **Done when:** integration tests cover undo and redo across a selection, the skipped cases, and the 10-action limit, and Nevada has undone a rating and a tag on both machines.

### 6. Finding (FND-1 to FND-3)

- **Search** (Ctrl/Cmd+F) in the header: file names and tags across the whole library, results as you type. The grid shows results from every folder, headed "Results for 'beach' · 42 photos", and the sidebar has no selection. Esc clears it and returns to the folder or tag selected before; selecting a folder or tag also ends it (FND-2).
- **Filters** behind a Filter button in the folder header: minimum rating, tags (all of them), edited or unedited, date range. Active filters show as chips with Clear all, always visible. They narrow whatever the grid shows (a folder, a tag, or search results), stay set from one to the next, and reset on restart. When nothing matches: "No photos here match the filters", with Clear filters (FND-1).
- The index already filters and searches (M1, decision 0010: a search keystroke under 8 ms at 100,000 photos); this slice is the UI and wiring.
- **Done when:** Nevada finds photos by name, tag, rating, and date on both machines, and the measurement script shows search within 200 ms at 10,000 photos (FND-3).

### 7. Problems with files (FIL-3, FIL-4, FIL-5, FIL-7)

- A notice bar at the top of the main area, never a blocking dialog (user flows, flow 11):
  - **Changed on disk** (FIL-3): reloaded, and the index follows; a Library write that meets a file changed since it was read leaves it alone. "Keep yours or load theirs?" applies only while a photo is open in the Editor, so it moves to M4, when the Editor ships (Nevada, 2026-10-26).
  - **Unreadable** (FIL-4): the photo shows unedited with its warning badge (since M1); the notice offers Start over. The old file goes, unchanged, to `Unreadable edit files` in Seura's folder, named `2026-10-26 143012 IMG_1234.jpg.seura.json` (moved there, or copied and the original moved to the Trash where a move isn't possible); the notice then says where, with Show in folder.
  - **Unknown content** (FIL-5): kept on save, as since M1, and the user is told it isn't shown.
  - **Original changed** (FIL-7): the fingerprint no longer matches; the edits still apply, with a warning.
- The backup is a new write path in Seura's folder: the data-safety suite and the app-data audit (rule 2) cover it.
- **Done when:** each case is covered by data-safety or integration tests, and Nevada has seen the notices on purpose-made examples.

### 8. Renamed photos (FIL-6)

- Fingerprints computed in the background for photos on disk (never for cloud-only ones), cached in the index.
- A photo renamed outside Seura gets its edit file back automatically when another photo in the same folder has the same fingerprint and no edit file of its own: the edit file is renamed to match (an edit file, never a photo).
- **Done when:** integration tests rename photos with edit files on both platforms, and Nevada has renamed a tagged photo in Explorer and Finder and seen its tags follow.

### 9. Delete to Trash (BRW-9)

- **Already seen:** OneDrive on the Mac shows its own informative dialog (deleting here deletes everywhere; Got it, or Open OneDrive Recycle Bin; no way to refuse), and OneDrive on Windows shows none, when Seura sends an edit file in a OneDrive folder to the Trash (Nevada, 2026-10-26; journal, `2026-10-26-onedrive-trash-notice.md`). Seura doesn't try to avoid it; it's documented instead.
- **First, measure** (as for LIB-9, with Nevada's permission, on a few photos of his own choosing): when Seura moves an online-only OneDrive file to the Trash with the system call, on both platforms, does it reach the local Recycle Bin or Trash, or only OneDrive's own deleted items? Does OneDrive show a dialog of its own? The result goes to the design session for BRW-9's final wording and the decisions log.
- Delete (Windows) or Cmd+Delete (macOS), or the photo menu: after a confirmation that says how many and that they go to the Recycle Bin or Trash, the selected photos and their edit files move there. Never permanent. Not in read-only folders. Not undoable (BRW-11).
- **Cloud-synced photos:** the confirmation names the service: "3 of these photos are in OneDrive. They'll be removed from OneDrive on all your devices too. OneDrive keeps deleted files in its own recycle bin for a while." A cloud-only photo is never downloaded just to delete it.
- The Viewer moves to the next photo; the grid and index follow.
- **Done when:** data-safety tests show photos and edit files reach the Trash and nothing else changes, on both platforms, and Nevada has deleted and restored photos on both machines.

### 10. Library polish (#72, #73, #74)

- The folder title stays on one line, and a long path gives way (#72).
- Thumbnails keep the slider's size; the slider resizes them smoothly; the window changes the columns, with leftover space spread evenly (#73).
- A resizable sidebar, remembered, with full folder names on hover (#74), once the design session has updated the wireframes.
- **Done when:** Nevada has checked each on both machines.

### 11. Wrap-up

- Node 26, a few weeks after it becomes LTS (2026-10-28), if that falls within M3; otherwise early M4.
- `check.py` from the edit format over the edit files Seura wrote in Nevada's library.
- The measurement script again, with search added; results in `docs/benchmarks/`.
- Build 0.2, with the Mac's ad-hoc signing (#71), and the website's early-build section (#76) with a design session.
- Confirm every "done when"; the devlog post, part 9.
- **Done when:** Nevada is tagging and rating his own library with build 0.2 on both machines.

## Questions, answered

The design session answered all eight on 2026-10-26 (#79; journal, `2026-10-26-m3-questions.md`), and the slices above follow:

| Question | Answer | Slice |
| --- | --- | --- |
| Undo in the Library | In M3: the last action, using each edit file's own history (BRW-11) | 5 |
| Deleting cloud photos | Confirmation names the service; never download to delete (BRW-9) | 9 |
| FIL-4's backup | `Unreadable edit files` in Seura's folder, dated (FIL-4) | 7 |
| Search and the current view | Results replace the grid until Esc or a sidebar selection (FND-2) | 6 |
| Filters and scope | Narrow whatever's shown, stay set, reset on restart (FND-1) | 6 |
| Counts on parent tags | No count (TAG-5) | 3, 4 |
| Mixed selections | "Mixed" stars; partial tags with "3 of 7" (TAG-8) | 2, 3 |
| Read-only folders | A line in the Info panel, controls disabled in place, notices for keys (TAG-7) | 2, 3 |

**Still open:** where an online-only file goes when it's moved to the Trash. Claude Code measures it at the start of slice 9 and reports back.

## Status

Reviewed 2026-10-27, at the end of slice 11. Pull requests #78 to #123.

| Slice | "Done when" | Status |
| --- | --- | --- |
| 1. Writing edit files (#90) | The data-safety suite covers every write path on both platforms; a rating shows in the grid and survives a restart | Met. The Mac Claude session checked that the Trash shows no prompt and that Finder's Put Back works. |
| 2. Ratings (#92) | Nevada rates by key and by clicking on both machines; ratings survive a restart and are in the edit files | Met: checked by Nevada on both machines. |
| 3. Tags (#93) | Nevada tags photos one at a time and in a selection; a tag typed in another case finds the existing one | Met: in Nevada's use on both machines; case folding is covered by tests. |
| 4. The tag tree (#94) | Nevada's tags appear as a tree, and selecting one shows exactly its photos | Met: checked by Nevada on his library. |
| 5. Undo (#96) | Tests cover undo and redo; Nevada has undone a rating and a tag on both machines | Met: checked by Nevada on both machines. |
| 6. Finding (#97, #98, #107) | Nevada finds photos by name, tag, rating and date; search within 200 ms at 10,000 photos | Met: in Nevada's use, which found the date filter missing photos (fixed). The slowest letter took 46 ms on Windows and 79 ms on the Mac (`docs/benchmarks/m3-search.md`). |
| 7. Problems with files (#100, #101) | Each case covered by tests; Nevada has seen the notices on purpose-made examples | Tests: met. The notices were shown on a generated sample library; the screenshots of each style for the design session are still to take. FIL-3's Editor part moved to M4, by Nevada's decision. |
| 8. Renamed photos (#102) | Integration tests rename photos on both platforms; Nevada has renamed a tagged photo and seen its tags follow | Tests: met on both platforms in CI. Nevada's own rename isn't recorded in the journal. |
| 9. Delete to Trash (#104, #113, #116) | Data-safety tests on both platforms; Nevada has deleted and restored photos on both machines | Met, with what the measurements changed. Cloud-only photos, and photos on drives with no Recycle Bin, are left out of Delete (decision 0017). On the Mac, the installed app can't yet delete photos in OneDrive folders (#112). |
| 10. Library polish (#105, #110) | Nevada has checked each on both machines | Met: checked by Nevada, including the resizable sidebar. |
| 11. Wrap-up (#107 to #123) | Nevada is tagging and rating his own library with build 0.2 on both machines | Met. Every edit file Seura wrote in Nevada's library passes `check.py`. Builds 0.2.0 and 0.2.1 are published as public pre-releases. Devlog part 9 is published. Nevada is tagging and rating with 0.2.1 on both machines and declared M3 done on 2026-10-27. |

**The milestone's "done when"** (Nevada tags and rates his own library, the edit files pass `check.py`, and the data-safety suite covers every new write path) is met: **M3 is complete** (2026-10-27).

**Also done in M3, beyond the plan:**
- Select All (#95) and the grid's focus after the Viewer (#114).
- The regional formats checked on both machines (ACC-6).
- About and the Windows ⋯ button (#121), and keys kept inside dialogs (#122). Both ship in 0.3.

**Carried forward:**

| Item | Where |
| --- | --- |
| Node 26 | Early M4; it becomes LTS on 2026-10-28 |
| Dates taken lost when the app is stopped without quitting (#99) | M4 |
| On the Mac, deleting photos in OneDrive folders (#112): Finder as a fallback, or Developer ID signing | A decision for Nevada or a design session |
| The website's "Try an early build" section (#76) | A design session; 0.2.1 is out |
| The filter row wrapping at 1440 px (#118) | A design session |
| The date range repeated within one minute (#125) | M4 |
| A drive with no Trash on the Mac (a network volume), for decision 0017 | To measure on the Mac |
| Screenshots of each notice style for the design session | When the design session reviews the notices |
| FIL-3's "Keep yours or load theirs?" in the Editor | M4, with the Editor |
| Undo and redo buttons in the toolbar | M4, with the toolbar's other actions |
| The HEIC colour test failing on Nevada's Mac only (#91) | M4 |
