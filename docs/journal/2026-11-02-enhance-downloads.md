# Enhance downloads cloud-only photos (M4 slice 13)

- **The design session's decision, in #195,** then three questions from Claude Code that went back to it: whether a batch can be stopped, what the question's buttons and Esc do, and what the finished message says. It answered: a Stop button with progress, three buttons with Esc as Cancel, and a message always shown when a download was involved. Claude Code had suggested Stop and the message; it had suggested two buttons with Esc as declining, and the design session chose three instead, so Esc does nothing.
- **One undo step, with photos arriving over minutes:** each photo is enhanced as it arrives, so a stopped batch keeps what it did, and the parts add up to one action, recorded when the batch ends. Until then, undo acts on the action before it. Claude Code's choice while building.
- **Choices the decision left open, all Claude Code's:**
  - Photos on the computer are enhanced first, then the downloads run.
  - A second Enhance during a batch waits its turn, as hovering down a row and clicking would, and Stop stops those waiting too.
  - Photos in read-only folders aren't downloaded or counted in the question, since Enhance can't change them.
  - A selection with only one photo in the cloud still shows progress and Stop; one photo alone just shows it on its tile.
- **Found in the real app, on scratch copies marked offline:** after Enhance, such a tile says "In the cloud" in place of its thumbnail. That comes from the test: the mark stays on after reading, where a real download clears it, so Seura asks the system for a thumbnail, and the system has none for a file marked that way. Automation here can't select several photos or slow a download, so the question, progress and Stop are checked by tests only, until Nevada tries them.
- **To check with real cloud photos:** rotate needs no download, so a rotated cloud-only photo gets its thumbnail from the system, which may not show the turn. It would be the same in 0.3, and isn't part of this slice.
