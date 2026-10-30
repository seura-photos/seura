# Zoom in the Editor, built (slice 4c)

- **A gap from the start, found by using the app.** Nevada noticed the Editor couldn't zoom, though the Viewer could. Nevada asked the design session, which found it had never been discussed: the Editor's wireframe simply had no zoom. The design session specified it in #165 (its own journal entry, `2026-10-29-editor-zoom.md`), to be built before red-eye, which needs it to reach small eyes.
- **Nevada asked for this to go in the next devlog post:** a gap that had been in the design from the beginning, caught by the person using the app rather than by any review.
- **Claude Code built it next, before crop (slice 5), and Nevada agreed.** Crop must show the whole photo and bring the zoom back when it closes, so building zoom first lets crop be built around it, instead of fitting zoom in afterwards. It's slice 4c in the M4 plan.
- **"Reusing the Viewer's implementation" meant taking it out of the Viewer.**
  - The zoom state, wheel and pinch, drag to pan, and fetching the sharp part are now a hook, `useZoom`, with the toolbar group in `ZoomControls`. Both the Viewer and the Editor use them.
  - In the core, cutting a sharp part from the full-size photo is a small shared type, `Parts`, which each of them holds.
  - The Viewer's own copies are gone, and its tests pass unchanged.
- **What the Editor gained:**
  - Fit, 100%, in and out, from the zoom group after undo and redo, the wheel, pinch, and + and −; dragging pans.
  - Zoomed-in parts are drawn with every edit, highlights and shadows sampling the whole photo's guide, and redrawn as sliders move.
  - Compare keeps the zoom and position, showing the original's sharp part too. Another photo opens fitted.
- **Left to the slices that need them, as the plan now says:** crop and straighten showing the whole photo (slice 5), and Space-drag to pan while red-eye uses the drag (slice 7). Neither has anything to work on yet.
- **Also fixed:** the M4 plan, Claude Code's draft, called Nevada "he" and "his" in several places; it now says "they" and "their".
