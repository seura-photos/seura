# Highlights and shadows join the Light sliders

- A friend of the project, interested in it, suggested adding brightness, shadows, and highlights to exposure and contrast.
- Claude's view, in the design session: shadows and highlights are a strong fit (they fix backlit faces and blown skies, the most common snapshot problems, and Picasa had both). A separate brightness slider would overlap exposure, since both lighten the photo.
- Nevada chose four sliders: Brightness, Contrast, Highlights, Shadows. Brightness is the existing `seura.exposure` under a friendlier label.
- Nevada asked whether to rename `seura.exposure` to `seura.brightness` while the format is still a draft. Claude recommended keeping the name, since the value is in photographic stops and other implementations need that meaning; the label is the app's business. Kept.
- Highlights and shadows are the first neighborhood operations, which shapes the renderer (technical design, section 6). Planned for M4.
- Also settled: the design sessions own the specs and the edit format; Claude Code implements and raises gaps (CLAUDE.md, "Who changes what").
