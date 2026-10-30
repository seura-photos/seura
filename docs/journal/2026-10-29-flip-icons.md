# The flip buttons' icons were the wrong way round

- **Nevada noticed it while trying 5a:** the Flip horizontal button showed a top-to-bottom mirror, and Flip vertical a left-to-right one. The buttons themselves flipped correctly.
- **The cause was in an icon name.** In the installed lucide (1.52), `FlipHorizontal2` is an alias for a drawing named `triangles-centerline-dashed-horizontal`: triangles above and below a horizontal line, which pictures a top-to-bottom flip. Claude Code had chosen the icons by their names without looking at the drawings.
- **The fix** uses the drawings by their own names, `TrianglesCenterlineDashedVertical` for Flip horizontal and `…Horizontal` for Flip vertical. A test checks which drawing each button shows, so a renamed alias can't swap them again.
