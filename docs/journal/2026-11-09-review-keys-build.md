# 2026-11-09: R and X (M6, between slices 3 and 4)

- **Nevada confirmed #250 on Windows and the Mac** before merging it, including the fix for marks not showing in Review on the Mac.
- **R starts a review in the Library, and X only unmarks there and in the Viewer,** as Nevada decided with the design session (#251). In Review, X still toggles. With no marked photo chosen, X says where marking happens and where Delete sends photos.
- **Unmarking is one step to undo,** which the design session's note asked for and #251's spec text doesn't mention. Marks aren't in edit files, and the undo history only knew edit files, so Claude Code added marks to it: undo puts the marks back, and redo takes them off again. Unmark in the Info panel works the same way.
- **Marking and unmarking inside a review aren't in the history, Claude Code's choice,** since X is its own undo there. Nevada was told and didn't object.
