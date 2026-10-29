# Light and color (M4 slice 3a)

- **Slice 3 in two parts,** Claude Code's choice. 3a is the operations: contrast, temperature and tint, and saturation, with their shaders, tests and sliders. 3b is PRF-6's measurement, dragging a slider on 12, 24 and 48 megapixel photos. The M4 plan expected that measurement to exist already, but it was never built, and every later slice needs it.
- **The formulas, Claude Code's choices, for the design session to review in the appendix:**
  - **White balance:** a gain on each channel. Temperature moves red up and blue down by up to half a stop each, and tint moves green down (toward magenta) by up to half a stop. The gains are then scaled so gray keeps its brightness.
  - **Contrast:** an S-curve around middle gray, on luminance, so hues don't shift. Black and white stay where they are, and +100 doubles the slope at middle gray.
  - **Saturation:** each channel moves away from the pixel's luminance. −100 is gray and +100 doubles the color.
  - Luminance uses Display P3's own coefficients, since that's the working space.
- **The appendix starts in the edit-format repository** (edit-format #5): conventions, test vectors that the format's own check keeps current, and exposure, which M1 built and nobody had written up. The guide asks for one operation per pull request, and the project doesn't stack pull requests, so white balance, contrast and saturation follow one at a time as each merges. Once they're all in, Seura's render tests will read the vectors from the submodule.
- **The render check now covers every operation built so far.** Preview and export matched exactly on WebGPU and WebGL2 on Windows: the largest difference was 0.
- **A slip:** Claude Code's first commit to the edit-format clone used Nevada's private email address, and GitHub refused the push. The commit was redone with the noreply address the app repository uses, and nothing was published with the private one.
