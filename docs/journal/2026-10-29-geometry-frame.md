# The crop frame, sampling, and the guide (design session)

- **Before building crop and straighten (M4 slice 5a), Claude Code found three things the edit format left open:** what frame the crop's fractions refer to once a photo is straightened, how a turned photo is sampled, and where the highlights and shadows guide is built. Each fixes what a file means, so another app must draw the same result.
- **The crop frame:** Claude Code proposed the oriented photo's own rectangle, unchanged by straightening, with uncovered corners filled from the nearest edge pixel. Claude agreed: it's the only choice where a crop keeps its meaning as the angle changes, which EDT-3's "grows back" needs. The other two (the box around the turned photo, or the largest rectangle inside it) both move the crop with every change of angle. Claude added that the crop's edges round to whole pixels, so an unstraightened crop is an exact copy.
- **Sampling:** Catmull-Rom bicubic, as Claude Code suggested, but Claude asked for it in the shared shader for preview and export alike (decision 0004), with a faster preview only if measurement shows a need.
- **The guide:** built from the photo before any geometry, Claude Code's proposal, accepted. Turning the photo never rebuilds it, and uncovered corners never enter it.
- Written into the edit format (edit-format #10); the formulas follow in the appendix.
