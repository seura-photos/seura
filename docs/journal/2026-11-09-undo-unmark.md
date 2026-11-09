# 2026-11-09: Undoing an unmark puts a mark back outside a review (M6)

- **Nevada confirmed R, X and undo on Windows and the Mac** before merging #252.
- **Nevada noticed a discontinuity while testing.** In the Library, X takes a mark off and a second X won't put it back, it says marking happens in Review, yet Ctrl+Z does put it back. So there's a way to mark a photo outside a review after all, by undoing an unmark, and Nevada asked whether that should be allowed.
- **It stays, Claude Code's recommendation and Nevada's decision.** The limit exists because a mark made in the Library would wait for a review that might never include it (#251). Undo can only restore a mark a review made, from the session's last 10 actions, and can't mark a photo that never had one. Without it, a stray X on a selection would lose review work for good. In Nevada's words, the key is that the mark initially occurred during review, and it's something of an edge case.
- **Letting X re-mark what it just unmarked was set aside,** since X would then depend on an invisible history, and that really would be marking outside a review.
- **Still for the design session:** BRW-11 describes undo as working only through edit files, and REV-6 doesn't mention undo, so the written spec doesn't yet cover this.
