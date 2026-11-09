# M6 work plan: Review

Status: draft for review · Last updated: 2026-11-09

The working plan for milestone M6 from the [milestone plan](milestone-plan.md): a guided pass through a batch of photos, one at a time, with one-key actions. It breaks the milestone into slices, one pull request each (or a few, lettered), in order. Drafted by Claude Code from the milestone plan, the [functional spec](functional-spec.md), the [user flows](ux/user-flows.md) and [wireframes](ux/wireframes.md), the [mockups](ux/mockups/), the [visual design](visual-design.md) and the [technical design](technical-design.md). The questions it couldn't settle are at the end, for a design session, each with a suggestion.

**Scope** (milestone plan, M6): the Review flow, REV-1 to REV-6, combining rating, rotate, Enhance, tagging, marking for deletion and editing; and the new-photos banner after a scan.

**Done when** (milestone plan): Nevada runs their newly added photos through Review end to end.

**Also carried in:**
- downloading cloud-only photos one at a time so they can go to the Trash, which BRW-9 plans "with Review's batched deletion (M6, REV-4)";
- Node 26 (#156), from 2026-11-11.

**Not in M6:** Google Drive and Dropbox (#162), which is in M7 unless the design session brings it forward (question 12); the Settings screen (SET-2); Cleanup (CLN-*); next-photo speed (#190), for M7's performance audit; HDR display (#60).

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
- **Moving through the batch:** the arrow keys, Back and Next, with the neighbors decoded ahead. Esc leaves and selects the photo the review ended on in the grid. The batch is fixed when the review starts, so a photo that arrives during it doesn't move the count.
- **Starting one:** the toolbar's Review button, for the folder, tag or search on screen, or the selection (question 7).
- **A photo that disappears during a review,** deleted or moved elsewhere, is skipped, and the count follows.
- **Done when:** Nevada moves through a batch of at least 200 photos on both machines, and the next photo feels as quick as in the Viewer.

### 2. Actions (REV-3)

- **The action bar,** 64 px tall, in the wireframe's order (question 2), with each key shown: `[` `]` rotate, `E` Edit, `T` Tag, `X` Mark to delete, `A` Enhance, stars, `→` Next. `0` clears a rating, as everywhere else.
- **The existing actions,** wired to the same store actions and commands as the Library's quick actions. The tag picker opens by its button. Ctrl+Z works as in the Library.
- **Edit and back:** `editedFrom` gains "review". Done or Esc returns to the review, on the photo the Editor ended on. The filmstrip shows the review's batch (question 8).
- **"Enhanced"** on the line beneath the photo, when applied.
- **Done when:** on both machines, Nevada rates, rotates, enhances, tags and edits in one batch with the keyboard alone, and the same with the mouse, and each change is there in the Library afterwards.

### 3. Marking for deletion (REV-4, REV-6)

- **`X` marks and unmarks.** The button reads "Mark to delete", then "Marked ✓". The photo dims with a red outline and the banner "Marked for deletion · deleted only when you finish · X to unmark". Every label says "mark", as REV-4 requires.
- **Marks last** past leaving a review and quitting the app, in a file in `Seura Photos` (question 3). It's written atomically. A photo that has gone is dropped from it, and a photo that's been renamed keeps its mark where the scan recognizes it.
- **The Delete key does nothing in a review,** so the only way to remove a photo there is the batch at the end.
- **Leaving with marks** says so in a message (question 5).
- **Tests:** no file moves on marking, unmarking or leaving, and marks survive a restart.
- **Done when:** on both machines, Nevada marks photos, quits, starts the same review again, finds them marked, and unmarks one.

### 4. Finishing (REV-5)

- **The summary** after the last photo: the counts (question 4), the sentence that everything is saved and nothing is deleted yet, then Move N marked photos to the Recycle Bin or Trash, Export photos rated at or above a chosen number of stars, and Back to the Library.
- **The one confirmation** is the existing Delete dialog, with everything BRW-9 says, including photos in cloud-synced folders and those that can't reach the Trash. A photo that couldn't be removed stays marked, and the summary says so.
- **Export** opens the existing dialog with the batch's photos at or above the chosen rating. Marked photos are left out.
- **Done when:** Nevada finishes a batch on both machines, the marked photos are in the Recycle Bin or Trash and can be restored with their edit files, and the export has the right photos.

### 5. Cloud-only photos in a review (LIB-9, BRW-9)

- **Starting a review with cloud-only photos** (question 6).
- **Actions that need no pixels,** rating, rotate, tags and marking, work without a download, as in the Library. Enhance and Edit download first, as they do now.
- **Downloading marked cloud-only photos so they can reach the Trash,** one at a time, as BRW-9 plans: the confirmation says how many and how large, with progress and a Stop button. Whatever isn't downloaded stays as it is, and stays marked.
- **Done when:** on both machines, with OneDrive, Nevada reviews a batch that's partly in the cloud, and marked cloud-only photos end up in the Recycle Bin or Trash after downloading. On the Mac this depends on #112's refusal for synced folders, and the slice reports what's possible there.

### 6. The new-photos banner (REV-1)

- **Knowing what's new.** The index has no record of when a photo arrived, but the launch's scan can compare what it finds with the snapshot it loaded. A photo the snapshot didn't have, and that isn't a recognized rename, is new.
- **No banner when the comparison means nothing:** with no usable snapshot, such as after clearing the cache or a build that changes the snapshot's layout, everything would look new. The same goes for a folder the user has just added (question 10).
- **The banner** above the grid, with Review and Not now (question 9). Review starts on exactly the new photos. The first-run offer in the user flows, a review of the most recent folder, comes with it (question 11).
- **Done when:** Nevada copies photos into a library folder with Seura closed, opens it, sees the banner with the right count on both machines, and reviews them; and no banner appears after clearing the cache.

### 7. Wrap-up

- **Measure** next-photo time in Review against PRF-5 on both machines, and rerun the other targets.
- **`scripts/app_data_audit.py`** over a build that has marked photos, and `check.py` over edit files changed in reviews.
- **Build 0.5,** the devlog post, and confirming every "done when". Each outward step needs Nevada's approval.
- **Done when:** Nevada runs newly added photos through Review end to end, with build 0.5.

## Questions

For a design session. Each has Claude Code's suggestion. Questions 1, 2 and 4 are places where the documents disagree.

1. **The banner's words.** REV-1 and the user flows say "84 new photos in Summer. Review them?"; the wireframes say "12 new photos in Summer. Sort through them one at a time?". *Suggestion: the spec's, which is shorter.*
2. **The action bar's order.** The user flows have previous, rotate, Enhance, stars, Tag, Mark for deletion, Edit, next. The wireframes and mockups have Back, rotate, Edit, Tag, Mark to delete, Enhance, stars, Next, and give the reason (INT-5). *Suggestion: the wireframes and mockups, and correct the user flows.*
3. **Where marks are kept.** REV-6 says they last, but not where. CLAUDE.md's rule 2 allows the edit file or the `Seura Photos` folder. *Suggestion: a file in `Seura Photos`, not the edit file. A mark is an intention on this computer, not part of the photo. In an edit file it would create and then remove files beside every marked photo, which brings back the cloud services' deletion notices (FIL-1), and a pending deletion would travel to the user's other computers. The cache folder won't do, since it can be cleared. The cost is that marks don't travel, which seems right.*
4. **The summary's counts.** REV-5 and the user flows say rated, edited, tagged and marked; the wireframes say rated, enhanced, tagged and to delete. And what counts as edited: a rotate, an Enhance, or only a visit to the Editor? *Suggestion: rated, edited, tagged and marked, counting a photo as edited if the review changed its look in any way, including rotate and Enhance, and counting only what this review did.*
5. **What "that batch" means in REV-6, and finding marks again.** Marks stay "until the next review of that batch", but a folder's contents change, and a selection has no name. There's also no way to see marks outside a review. *Suggestion: a mark belongs to the photo. Any review that includes a marked photo shows it marked and counts it in the summary. On leaving with marks, a message says how many remain and that reviewing those photos again finishes the job. No badge in the Library for now.*
6. **Cloud-only photos at the start of a review.** Showing one large needs a download, and LIB-9 rules out unasked bulk downloads. *Suggestion: ask once at the start, as Enhance does for a selection, with the count and size. If the user agrees, download ahead of where they are. If not, show those photos by their thumbnail, marked as in the cloud, with the no-pixel actions still working.*
7. **What the Review button reviews, and in what order.** REV-1 lists any folder, tag, search result or selection. *Suggestion: two or more selected photos means the selection; otherwise everything on screen, with filters applied. The Library's current sort order, except that the banner's review goes oldest first, as photos were taken.*
8. **Moving to another photo in the Editor during a review.** The Editor's filmstrip can move on. *Suggestion: the filmstrip shows the review's batch, and Done returns to the review on the photo the Editor ended on, as it does for the Viewer.*
9. **How long the banner lasts.** *Suggestion: until Review or Not now, or until the next launch. "Since the previous launch" then needs no new stored state, and the photos stop being new when the app is next opened, reviewed or not. The Review button still reaches them.*
10. **What counts as new.** Photos added while the app is running? A newly added folder of 20,000 photos? Files Seura exported into a library folder (EXP-1)? Several folders at once? *Suggestion: only what the launch's scan finds. That leaves out exports and photos added during a session. Not a folder added since the last launch. For several folders, name the one with the most and give the total: "84 new photos in Summer and 2 other folders."*
11. **The first-run offer.** The user flows have the Library offering a review of the most recent folder when the first scan finishes. REV-1 doesn't mention it, and at that moment every photo is new. *Suggestion: leave it out for now. A first review of a whole library is a large thing to offer someone who has only just arrived.*
12. **Review on a Google Drive folder (#162).** Seura can't tell that a streamed photo is in the cloud, so a review reads each one, with Drive streaming it as needed. The banner's scan has already read them all. *Suggestion: nothing special in M6, and #162 stays in M7, unless the design session wants the drive-level detection sooner.*
13. **Rating and moving on.** Should choosing a rating go to the next photo? The wireframes put the stars beside Next so the pointer has little to travel, which suggests not. *Suggestion: no; a rating stays on the photo, and Next is its own action, so a photo can be rated and then tagged or marked.*
14. **Starting a review again.** Starting over at the first photo means walking past finished ones. *Suggestion: always start at the first photo. Remembering a place adds stored state for little gain, and the banner's review only happens once.*
15. **After the last photo, and going back.** Next on the last photo opens the summary. Can the user go back from it? *Suggestion: yes, with Back or `←`, to the last photo, so a mark can be changed before confirming.*
