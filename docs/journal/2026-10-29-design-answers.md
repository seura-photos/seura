# The design session's answers on M4 slices 1 to 3

- **What was asked.** After slices 1 to 3, Claude Code gathered five items for the design session: the formulas appendix's start (edit-format #5), revert and other apps' operations, two UX choices made where the documents were silent, #112, and a wording fix in the technical design. The design session answered on 2026-10-29. The spec and technical design changes are in its own pull request (#158), with its journal entry `2026-10-29-m4-review.md`.
- **The appendix:** approved with two wording changes, which Claude Code made before Nevada merged it:
  - 1 is the photo's reference white, not the brightest white it holds, since HDR photos (#60) will have meaningful values above it;
  - components below 0 come from colors outside Display P3, every operation accepts them, and a formula says if it makes them.
  - For later: operations that depend on position or on neighboring pixels will need vectors as small test images, not single colors.
- **That answer changed code already merged.** Saturation, from #157, clamped components at 0, which would have clipped a wider-gamut color. It now keeps them, and can make them, since a vivid color made more vivid can go below 0. A new test runs colors outside Display P3 through every operation.
- **Revert (EDT-10)** stays as built. When the photo has other apps' operations, the confirmation now adds "Edits made in other apps are kept."
- **Confirmed as built:**
  - revert's inline confirmation;
  - leaving before answering "Keep yours or load theirs?" keeps theirs. It now says "Your changes to this photo since it changed elsewhere weren't kept."
- **#112: no Finder Apple Events.** Deleting a downloaded OneDrive photo on the Mac stays off. The confirmation will say why and offer Show in Finder. Claude Code may spend up to an hour looking for an entitlement or bundle key; otherwise it's tested again once Developer ID signing is set up before the beta (M7). That work gets its own pull request.
- **The technical design's wording** on skipping neutral operations was fixed in #158.
