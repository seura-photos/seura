# M6 work plan: Review

Status: agreed · Last updated: 2026-11-09

The working plan for milestone M6 from the [milestone plan](milestone-plan.md): a guided pass through a batch of photos, one at a time, with one-key actions. It breaks the milestone into slices, one pull request each (or a few, lettered), in order. Drafted by Claude Code from the milestone plan, the [functional spec](functional-spec.md), the [user flows](ux/user-flows.md) and [wireframes](ux/wireframes.md), the [mockups](ux/mockups/), the [visual design](visual-design.md) and the [technical design](technical-design.md). The questions it couldn't settle are at the end, with the design session's answers (#240).

**Scope** (milestone plan, M6): the Review flow, REV-1 to REV-6, combining rating, rotate, Enhance, tagging, marking for deletion and editing; and the new-photos banner after a scan.

**Done when** (milestone plan): Nevada runs their newly added photos through Review end to end.

**Also carried in:**
- downloading cloud-only photos one at a time so they can go to the Trash, which BRW-9 plans "with Review's batched deletion (M6, REV-4)";
- Node 26 (#156), from 2026-11-11.

**Not in M6:** Google Drive and Dropbox (#162), which stays in M7 (question 12); the Settings screen (SET-2); Cleanup (CLN-*); next-photo speed (#190), for M7's performance audit; HDR display (#60).

## How to work

As in M5 ([M5 work plan](m5-plan.md), "How to work"): one slice per branch and pull request, branched from `main` and named `m6/<slice>`; Nevada merges. Every pull request's base is `main` (none stacked on another's branch). Dependencies at least two weeks old, checked against decision 0012. Each slice has an issue under the "M6" milestone on GitHub, and its pull request closes it. Screens are styled from the tokens and checked against the mockups, with screenshots in light and dark mode in the pull request. New text goes in the message catalogs (ACC-5). Journal everything.

New in M6:

- **Review is a new view, not new machinery.** Rating, rotate, Enhance, tags, the Editor, Delete and Export all exist. Review calls the same commands and writes through the same paths, so a rating given in Review is the same change as one given in the Library, with the same undo (BRW-11) and the same edit file. No edit operation is added (decision 0004).
- **The photo is shown by the Viewer's code.** Review needs what the Viewer has: decoding the neighbors ahead (PRF-5), fitting the window, and following edits. That part of `Viewer.tsx` becomes something both use, rather than a copy.
- **Nothing is removed until the one confirmation.** Tests enforce that marking, unmarking, leaving a review, and reaching the summary send nothing to the Trash, and that the confirmed deletion takes the BRW-9 path (CLAUDE.md, rules 1 and 5).
- **Batches need testing by hand.** As M5 found with the Cancel button, the UI tests don't apply the stylesheet, and the automation can't select several photos. Each slice with a screen ends with a run through a real batch on both machines.

## Slices

### 0. Carried in (#156)

- **Node 26** (#156), once it's allowed on 2026-11-11, so this can land at any point in the milestone.

### 1. The Review screen (REV-2; starting from the Library, REV-1)

- **A fourth view,** over the Library as the Viewer and the Editor are: "Exit review", a title that names the batch ("Reviewing new photos in Summer"), "5 of 12" with a progress bar, the photo as large as fits on the dark surround, and one line beneath it with the file name, date, rating and tags.
- **Moving through the batch:** the arrow keys, Back and Next, with the neighbors decoded ahead. A review always starts at its first photo; Home, End and a click on the progress bar jump within it (question 14). Esc leaves and selects the photo the review ended on in the grid. The batch is fixed when the review starts, so a photo that arrives during it doesn't move the count.
- **Starting one:** the toolbar's Review button. With two or more photos selected it reviews the selection; otherwise everything the grid shows, with filters applied, in the Library's current order (question 7).
- **A photo that disappears during a review,** deleted or moved elsewhere, is skipped, and the count follows.
- **Done when:** Nevada moves through a batch of at least 200 photos on both machines, and the next photo feels as quick as in the Viewer.

### 2. Actions (REV-3)

- **The action bar,** 64 px tall, in the wireframe's order (question 2), with each key shown: `[` `]` rotate, `E` Edit, `T` Tag, `X` Mark to delete, `A` Enhance, stars, `→` Next. `0` clears a rating, as everywhere else. A rating stays on the photo and doesn't move on (question 13).
- **The existing actions,** wired to the same store actions and commands as the Library's quick actions. The tag picker opens by its button. Ctrl+Z works as in the Library.
- **Edit and back:** `editedFrom` gains "review". Done or Esc returns to the review, on the photo the Editor ended on. The filmstrip shows the review's batch (question 8).
- **"Enhanced"** on the line beneath the photo, when applied.
- **Done when:** on both machines, Nevada rates, rotates, enhances, tags and edits in one batch with the keyboard alone, and the same with the mouse, and each change is there in the Library afterwards.

### 3. Marking for deletion (REV-4, REV-6)

- **`X` marks and unmarks.** The button reads "Mark to delete", then "Marked ✓". The photo dims with a red outline and the banner "Marked for deletion · deleted only when you finish · X to unmark". Every label says "mark", as REV-4 requires.
- **Marks last** past leaving a review and quitting the app, in a file in `Seura Photos`, never the edit file (question 3). It's written atomically. A mark belongs to the photo, so it's found again by any review that includes that photo (question 5). A photo that has gone is dropped from the file, and a photo that's been renamed keeps its mark where the scan recognizes it.
- **The Delete key does nothing in a review,** so the only way to remove a photo there is the batch at the end.
- **Leaving with marks** says in a message how many are still marked, and that reviewing them again finishes the job (question 5).
- **Marks in the Library and Viewer:** a marked photo is dimmed with a red outline, as in the review, with "Marked for deletion" in its tooltip and accessible label, and a line in the Info panel with Unmark (question 5).
- **Tests:** no file moves on marking, unmarking or leaving, and marks survive a restart.
- **Done when:** on both machines, Nevada marks photos, quits, finds them dimmed in the Library, starts the same review again, finds them marked there too, and unmarks one from the review and one from the Info panel.

### 4. Finishing (REV-5)

- **The summary** after the last photo: the counts (question 4), the sentence that everything is saved and nothing is deleted yet, then Move N marked photos to the Recycle Bin or Trash, Export photos rated at or above a chosen number of stars, and Back to the Library. Next on the last photo opens it, and Back or `←` returns to the last photo, so a mark can be changed before confirming (question 15). Photos marked in earlier reviews are counted and deleted with the rest (REV-5).
- **The one confirmation** is the existing Delete dialog, with everything BRW-9 says, including photos in cloud-synced folders and those that can't reach the Trash. A photo that couldn't be removed stays marked, and the summary says so.
- **Export** opens the existing dialog with the batch's photos at or above the chosen rating. Marked photos are left out.
- **Done when:** Nevada finishes a batch on both machines, the marked photos are in the Recycle Bin or Trash and can be restored with their edit files, and the export has the right photos.

### 5. Cloud-only photos in a review (LIB-9, BRW-9)

- **Starting a review with cloud-only photos** asks once, with how many and their total size, whether to download them as they come up (question 6). If yes, Seura downloads about 3 photos ahead of the current one, not the whole batch, and jumping moves that window. If no, they show as their thumbnails, marked as in the cloud.
- **Actions that need no pixels,** rating, rotate, tags and marking, work without a download, as in the Library. Enhance and Edit download that one photo first, as they do now (BRW-10).
- **Downloading marked cloud-only photos so they can reach the Trash,** one at a time, as BRW-9 plans: the confirmation says how many and how large, with progress and a Stop button. Whatever isn't downloaded stays as it is, and stays marked.
- **Done when:** on both machines, with OneDrive, Nevada reviews a batch that's partly in the cloud, and marked cloud-only photos end up in the Recycle Bin or Trash after downloading. On the Mac this depends on #112's refusal for synced folders, and the slice reports what's possible there.

### 6. The new-photos banner (REV-1)

- **Knowing what's new** (question 10). The index has no record of when a photo arrived, so the set of new photos is kept in memory for the session. It starts with what the launch's scan finds that the loaded snapshot didn't have, and that isn't a recognized rename. It grows with what the folder watcher finds while the app runs.
- **What doesn't count:** Seura's own exports, which the export records as it writes them, and the photos of a folder just added to the library.
- **No banner when the comparison means nothing:** with no usable snapshot, such as after clearing the cache or a build that changes the snapshot's layout, everything would look new. No banner appears on first run either (question 11).
- **The banner** above the grid: "84 new photos in Summer. Sort through them one at a time?", or "84 new photos in Summer and 2 other folders. Sort through them one at a time?", with Review and Not now (question 1). It appears at launch, or during a session once additions have been quiet for about a minute. It stays until Review or Not now, or until the next launch (question 9). Review starts on exactly the new photos, oldest first (question 7).
- **Done when:** on both machines, Nevada copies photos into a library folder with Seura closed, opens it, sees the banner with the right count, and reviews them; copies more with Seura open and sees the banner about a minute after the last one arrives; exports into a library folder and sees no banner for those; and sees none after clearing the cache.

### 7. Wrap-up

- **Measure** next-photo time in Review against PRF-5 on both machines, and rerun the other targets.
- **`scripts/app_data_audit.py`** over a build that has marked photos, and `check.py` over edit files changed in reviews.
- **Build 0.5,** the devlog post, and confirming every "done when". Each outward step needs Nevada's approval.
- **Done when:** Nevada runs newly added photos through Review end to end, with build 0.5.

## Status

**The milestone's "done when"** is met: **M6 is complete** (2026-11-12). Nevada copied five photos into a OneDrive folder on Windows. Build 0.5 on the Mac, already open, offered them in the new-photos banner about a minute after they synced. Nevada enhanced and rated them in a review there. Once the edit files had synced back, Nevada exported the 4-star photos on Windows. Claude Code checked the results read only:
- All the edit files pass `check.py`.
- Every photo still matches the checksum in its edit file.
- The four exports are brighter than their originals, as the Mac's Enhance settings say, so Windows rendered the edits that synced over.
- The exports carry the rating, and no location, as the export settings asked.

This records the milestone, not a fresh check of every slice's "done when"; what each slice found is in `docs/journal/`, and the final measurements are in [m6-wrap-up.md](benchmarks/m6-wrap-up.md). Build 0.5.0 is published as a public pre-release, and devlog part 12 is live.

**Carried forward:**
- **To M7:** Node 26 (#156), slice 0, which was waiting for 2026-11-11 and can be done now.
- **In no milestone yet:**
  - From this milestone: edit files written before a photo's thumbnail never getting its width and height (#264), found in the run above; the Viewer once not drawing a photo during measuring (#258); and a library folder saved through a symbolic link (#255).
  - Older: quitting from the Dock on the Mac (#217), and the GPU shader caches in decision 0013 (#231, for a design session).
- **For a design session:** decision 0015 doesn't list `marks.json` yet; BRW-11 and REV-6 don't say that undo can put back a mark a review made.

## Questions

All are answered, by the design session on 2026-11-09 (#240; journal, `2026-11-09-m6-plan-answers.md`). Answers are Claude Code's suggestions except where they say otherwise.

1. **The banner's words.** The wireframes' wording, a change from the suggestion, since it tells a first-time user what Review is: "84 new photos in Summer. Sort through them one at a time?", with Review and Not now. For several folders: "84 new photos in Summer and 2 other folders. Sort through them one at a time?"
2. **The action bar's order.** The wireframes' order; the user flows are corrected.
3. **Where marks are kept.** A file in Seura's own folder, keyed so a mark follows the photo; not the edit file.
4. **The summary's counts.** Rated, edited, tagged and marked for deletion, for only what this review did. Edited means any change to the look, including rotate and Enhance.
5. **Finding marks again.** A mark belongs to the photo. Any review that includes it shows it marked and counts it, and a message says so on leaving with marks. A change from the suggestion: in the Library and Viewer, a marked photo is dimmed with a red outline, with "Marked for deletion" in its tooltip and accessible label, and a line in the Info panel with Unmark.
6. **Cloud-only photos at the start of a review.** Ask once at the start, with the count and size. A change from the suggestion: if yes, download a few photos ahead of the current one, say 3, not the whole batch. If no, show thumbnails marked as in the cloud; rate, tag, rotate and mark still work, and Enhance or Edit downloads that one photo (BRW-10).
7. **What the Review button reviews, and in what order.** A selection of two or more photos, otherwise everything the grid shows, with filters, in the Library's order. The banner's review goes oldest first.
8. **Moving to another photo in the Editor during a review.** The filmstrip shows the batch, and Done returns to the review on the photo the Editor ended on.
9. **How long the banner lasts.** Until Review or Not now, or the next launch.
10. **What counts as new.** A change from the suggestion: photos found since the previous launch, by the launch's scan or by the watcher during a session, since phone photos often sync while Seura is open. During a session, the banner shows once additions have been quiet for about a minute. Seura's own exports and the photos of a folder just added don't count. Several folders as suggested.
11. **The first-run offer.** None; the user flows are corrected.
12. **Review on a Google Drive folder (#162).** Nothing special, and #162 stays in M7.
13. **Rating and moving on.** A rating doesn't move to the next photo.
14. **Starting a review again.** Always at the first photo. Added to the suggestion: Home and End, and a click on the progress bar, jump within the review.
15. **After the last photo, and going back.** Back or `←` from the summary returns to the last photo.
