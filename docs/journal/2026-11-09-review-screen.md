# 2026-11-09: The Review screen (M6 slice 1)

- **A fourth view, over the Library,** with a title that names the batch, "1 of 5" and a progress bar, the photo as large as fits, and a line beneath it with the file name, date, rating and tags. Back and Next are the first two of the action bar; slice 2 fills in the rest. Home, End and a click on the progress bar jump, as the design session added (#240).
- **The Viewer's picture code is now shared.** Claude Code moved it into `useShownPhoto`, rather than copying it, as the plan said. The Viewer's tests didn't change and still pass.
- **A review keeps its own list of photos, Claude Code's choice.** Reading the grid's list looks simpler, but with a filter on, rating a photo out of the filter would drop it from the review while it's on screen. So the batch is held by the review, and a small new command, `review_photos`, brings it up to date. A photo that has really gone is skipped, and the count follows.
- **For now, showing a cloud-only photo downloads it,** as the Viewer does. Asking once at the start is slice 5.
- **Checked in the real app** on a scratch build, by page events through the web view's debug port: moving, Home and End, the bar, Esc, and a review of a selection. It looks like the mockups in both modes.
