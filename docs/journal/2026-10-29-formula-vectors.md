# The formulas appendix, and Seura checked against it

- **The edit format's formulas appendix now defines every operation Seura renders so far:** exposure (edit-format #5), white balance (#6), contrast (#7) and saturation (#8). The design session asked for them one per pull request, after #5's wording changes. Claude Code wrote each from Seura's shaders, and Nevada merged them.
  - Saturation's entry says, as the design session asked, that it accepts components below 0 and can make them: a vivid yellow at 100 comes out with blue at about −0.70.
- **Seura now reads the vectors.** The `edit-format` submodule moves to the merged appendix, and `crates/seura-render/tests/render.rs` gains two tests:
  - The CPU definitions the render tests compare the shaders with must match every vector within the appendix's tolerance. So the definitions can't drift from the published formulas unnoticed.
  - The shaders are run on every vector's input and must give its output within one level once encoded for Display P3.
  - A new vector file for an operation the test doesn't know fails the test, so adding highlights and shadows to the appendix will need the test too.
