# 2026-11-09: Review's actions (M6 slice 2)

- **Nevada tried slice 1 on the Mac and Windows** before merging it (#248): "it works well".
- **The action bar is in the wireframes' order,** as the design session settled: Back, rotate, Edit, Tag, Enhance, the stars, Next. Mark to delete goes between Tag and Enhance in slice 3. Each action shows its key, and they use the same store actions as the Library, so a rating given in Review is the same change, with the same undo.
- **The stars moved into `Stars.tsx`,** shared by the Info panel and Review, rather than written twice.
- **Edit and back:** Done returns to the review on the photo the Editor ended on, and the filmstrip shows the batch. A review now brings its list up to date from `App` when the library changes, because the review screen isn't there while the Editor is open.
- **What "Enhanced" under the photo means, Claude Code's reading:** Enhance is an ordinary set of slider changes (EDT-12), so nothing on the photo records that it was enhanced. The line shows it when an Enhance is in the photo's history and hasn't been undone. Moving a slider afterwards doesn't take it away.
- **A false alarm from Claude Code's own test.** Driving the real app, it pressed Esc at the window rather than in the tag picker, which closed the review and left the picker open. That isn't what a real key press does: the picker has the focus and keeps Esc to itself. Sent to the focused field, Esc closed only the picker. The stale picker also made the next focus check meaningless, until the scratch copy was restarted.
