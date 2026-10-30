# Red-eye mode (M4 slice 7b)

- **Built as the design session settled it** (seura #180, edit-format #14). It confirmed Claude Code's suggestions with four changes:
  - a click's circle is a fixed 24 CSS pixels across, not 2% of the photo;
  - `[` and `]` size the next click's circle, as for a brush, and resize a focused circle, keeping `+` and `−` for zoom;
  - the count says how many the crop hides;
  - the edit format now sets the 64 limit.
- **A mode, like crop:** R or "Fix red-eye…" opens it. The header becomes "Fix red-eye · name", the filmstrip goes, and the panel holds the instruction, the count, Remove all, Cancel and Done. Done or Enter saves one "Red-eye" record, Esc or Cancel discards, and leaving the photo counts as Done. The mode has its own undo, built as crop mode's is.
- **The zoom stays,** unlike crop. The header keeps the zoom group, since the wheel pans in this mode and the user needs a visible way to zoom. That's Claude Code's addition to the wireframe's header, said in the pull request.
- **`useZoom` learned a tool mode** (EDT-1): while a tool uses the drag, the drag pans only with Space held, the wheel and trackpad pan, and Ctrl with the wheel, or a pinch, zooms. Its first Space handler failed when the key event's target was the window, which has no `closest`. The test caught it.
- **Placing the circles** needed the geometry the other way round: from the upright photo to the result, inverting `uprightPosition`. A test round-trips it through every turn and flip, straightened and cropped. Geometry turns and moves the photo but never scales it, so one scale maps the result's pixels to screen pixels.
- **The brush size** is kept while the app runs, as the design asked. Tests reset it after each test, since it would otherwise carry from one test into the next.
- **The core** saves the circles clamped to the format's ranges, at most 64. Each radius stays at least 0.0001: the file rounds to four decimals, and the format requires a radius above 0.
- **Not yet seen in the real app.** Claude Code can't click into the Editor. Placing, moving and resizing circles, and Space-panning, need Nevada's hands on both machines.
