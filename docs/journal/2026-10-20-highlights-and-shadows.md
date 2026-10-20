# Highlights and shadows join the Light sliders

- A friend of the project, interested in it, suggested adding brightness, shadows, and highlights to exposure and contrast.
- Claude's view, in the design session: shadows and highlights are a strong fit (they fix backlit faces and blown skies, the most common snapshot problems, and Picasa had both). A separate brightness slider would overlap exposure, since both lighten the photo.
- Nevada chose four sliders: Brightness, Contrast, Highlights, Shadows. Brightness is the existing `seura.exposure` under a friendlier label.
- Nevada asked whether to rename `seura.exposure` to `seura.brightness` while the format is still a draft. Claude recommended keeping the name, since the value is in photographic stops and other implementations need that meaning; the label is the app's business. Kept.
- Highlights and shadows are the first neighborhood operations, which shapes the renderer (technical design, section 6). Planned for M4.
- Also settled: the design sessions own the specs and the edit format; Claude Code implements and raises gaps (CLAUDE.md, "Who changes what").
- In the code (Claude Code): `seura-edit` now knows `seura.highlights` and `seura.shadows`, inserts them after white balance and before contrast, and rejects files that put them out of order; the `edit-format` submodule points at the merge of edit-format #2. Rendering them is M4 work. Until then Seura's export ignores them if a file has them, which can only happen if another app wrote them, since Seura's own Editor can't set them yet.
