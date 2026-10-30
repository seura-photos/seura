# Straighten and crop in the renderer (slice 5b)

- **The appendix first.** The design session's answers (edit-format #10) asked Claude Code for three formulas. Claude Code wrote them as edit-format #11, which Nevada merged:
  - the transform from an output pixel to a photo position;
  - Catmull-Rom sampling;
  - the largest crop that keeps a straightened photo's corners out of view.
  Their test vectors sit in a folder of their own, because Seura's tests read every file in `tests/formulas/` as color vectors.
- **The vectors caught a mismatch while #11 was being written.** The spec says positions past the photo's edge take the nearest edge pixel. Plain Catmull-Rom just past the edge still blends neighbours: at a corner, it gave −0.0078 where the corner pixel is 0. The appendix now clamps the position to the outermost pixel centers first, which matches the spec exactly.
- **What 5b does:**
  - Every pixel of the result is placed in the upright photo (the crop, the turn back about the photo's center, then the orientation undone) and sampled there with Catmull-Rom, in the shader the preview and export share.
  - Export comes out at the crop's size. Each tile uploads only the part of the photo it samples, with a margin for the filter.
  - Edited thumbnails, the Viewer and the Editor show straighten and crop from the edit file. Crop mode, which writes them, is slice 5c.
  - Zoom cuts the part of the upright photo that a straightened view needs, and Compare keeps straighten and crop, as the design session confirmed.
- **Caught along the way:**
  - The appendix's vectors showed that a crop's numbers need all 64 bits. Seura held them as 32-bit floats, and a 0.7-high crop of a 5-pixel photo is 3.5 pixels in 64-bit, which rounds to 4. In 32-bit it's 3.49999994, which rounds to 3. The settings now keep the crop as the file has it.
  - Export's 2048-pixel tiles no longer fit: a tile's part of the photo grows with the margin, and with the angle. Tiles are now 1440 pixels, so the part fits the 2048 texture limit even at 45°. The GPU test on a 2300 × 2100 photo found it at once.
  - In one of Claude Code's own new tests, the expected decode size for a cropped photo was wrong, not the code: the left half of a landscape photo is tall, so its height limits the fit.
- **Tests:**
  - Core, UI and the shader's CPU twin are each checked against the appendix's vectors.
  - The GPU matches the definitions with every turn, straightened and cropped, with highlights and shadows, and across export's tiles.
  - The render check now straightens and crops its image too, and still matches export exactly on WebGPU and WebGL2, on Windows and on the Mac (difference 0 on all four; the Mac session ran it).
- **Bicubic is fast enough.** The design session allowed a cheaper filter for the preview only if measurement showed a need. Catmull-Rom runs on every pixel of every photo, straightened or not. `measure.py --sliders` on Nevada's Windows PC: the slowest slider holds 116 frames a second on WebGPU and 118 on WebGL2, against 114 and 119 before; the target is 30. The longest frame is still the first drag of the first photo (67 ms). On the MacBook, the Mac session measured 60 frames a second on both backends, WebKit's cap, with no frame over 33 ms (the longest 26 ms) and none of #164's first-use spike; opening took 0.21 to 0.51 s.
- **The technical design** said export tiles were about 2048 pixels; it now says 1440, and why, and that geometry samples with Catmull-Rom as the appendix defines.
