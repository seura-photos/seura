# M5 planning questions (design session)

- **Claude Code drafted the M5 plan** (#203) with twelve questions and one decision to check. Claude answered, mostly agreeing:
  - **Paste** in three groups: Adjustments, Effects, and Crop and rotation as one choice, as PRE-1 already named it; rotating a selection has its own quick action. A ratio-locked crop keeps its ratio on a photo of another shape.
  - **Other apps' operations** are left alone by paste and presets. That changed the edit format's preset rule (edit-format #17).
  - **Presets in the Library** live in the right-click menu, not the Windows ⋯ menu, which is for app-wide commands.
  - **Keywords, one change:** Claude Code suggested `dc:subject` with the tag as written; Claude asked for each level as its own keyword, as Lightroom writes them, since `family/kids` as one keyword reads oddly in other apps, plus the hierarchy in `lr:hierarchicalSubject`.
  - **XMP** is written by Seura's own code, dropping Adobe's toolkit and its C++ build (technical design updated).
  - **Embedded thumbnails** in exports are dropped, and EXP-4 now says so: an old thumbnail would show what the user cropped out.
