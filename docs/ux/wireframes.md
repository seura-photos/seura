# Seura Photos — Wireframes (v1)

Status: approved · Last updated: 2026-10-17

The wireframes live on a design canvas in the project owner's Claude account. This document records what they establish, screen by screen, so the decisions are available in the repository. They define layout and flow only; visual styling (color, type, iconography) comes later. Requirement IDs refer to the [functional spec](../functional-spec.md); flows are in [user-flows.md](user-flows.md).

All screens were drawn at 1280×800 logical pixels, close to the 1280×720 minimum (DSP-2), plus one Library at 1920×1080 to show how extra space is used (DSP-3).

## Shared structure

- **Toolbar** (52 px) across the top of every view; **status bar** (28 px) at the bottom of the Library and Viewer.
- **Sidebar** (232 px; 260 px at 1920 wide): FOLDERS tree, then TAGS tree, each row with a count.
- **Info panel** (280 px; 320 px at 1920 wide), always shown in Library and Viewer: preview, file name, date, size, camera, lens, location, edit state with Revert, rating stars, tag chips with Add tag.
- Dark surround (#2b2d30 in the wireframes) behind photos in the Viewer, Editor, and Review, so photos aren't judged against a bright frame.
- Control sizes are fixed across window sizes; only photo areas and panel widths grow.

## 1. Welcome (SET-1, SET-4)

A single centered card: title; two sentences (what the app does; originals are never changed and edits live beside each photo); "Which folders hold your photos?" with Pictures pre-checked and Add another folder; a cloud-sync note for the cache when relevant; "Check for updates automatically," checked; Get started (44 px, bottom right).

## 2. Library (LIB-*, BRW-1–10, REV-1)

- **Toolbar:** app name; search; Review; then, on the right, the selection's quick fixes (rotate left, rotate right, Enhance), Edit (primary), Export.
- **Review banner** above the grid when new photos are found: "12 new photos in Summer. Sort through them one at a time?" with Review and Not now.
- **Folder header row:** name, path, count; then Filter, sort, and thumbnail size on the right.
- **Grid:** square cells, photos letterboxed inside; selected cell outlined in the accent color. Badges: edited (bottom left), rating (bottom right), unreadable edit file (top left, red). Hovering a cell shows rotate left, rotate right, and Enhance (32 px) on the photo (BRW-10, INT-5).
- At 1920×1080: 7 columns of ~175 px thumbnails instead of 5 of ~140 px; same controls.

## 3. Review (REV-*)

- **Header:** Exit review; title; "5 of 12" with a progress bar.
- **Photo:** as large as fits, on the dark surround. Below it, one line: file name, date, rating, tags, and "Enhanced" when applied.
- **Action bar** (64 px tall), ordered from least to most likely so the pointer moves right through the flow (INT-5): Back · rotate left · rotate right · Edit · Tag · Mark to delete · Enhance · five 44 px stars · Next (112 px, primary). Each shows its key (`[`, `]`, `E`, `T`, `X`, `A`, `1`–`5`).
- **Marking:** the button reads "Mark to delete," becoming "Marked ✓"; the photo dims with a red outline and a banner, "Marked for deletion · deleted only when you finish · X to unmark" (REV-4).
- **Summary:** counts of rated, enhanced, tagged, and to delete; a sentence that everything is saved and nothing is deleted yet; Move N marked photos to the Recycle Bin…; Export photos rated 4★ or more; Back to the Library.

## 4. Viewer (BRW-5, BRW-6, BRW-10)

- **Toolbar:** back to the folder; file name; "37 of 214"; zoom group (−, Fit, 100%, +); rotate left and right; Enhance; Edit (primary); Export.
- **Photo** centered on the dark surround, with 48 px previous and next buttons at the edges.
- **Info panel** as in the Library, plus the folder as a link.
- **Status bar:** zoom level and the main keys.

## 5. Editor (EDT-*, PRE-*)

- **Toolbar:** file name, "Edits saved automatically," undo, redo, Copy edits, Export. No Done here.
- **Tool panel** (300 px, right), one scrolling column in process order (INT-3):
  1. Hold to compare · Done (44 px each, first row, where the pointer works)
  2. Enhance (40 px, prominent)
  3. Crop and straighten
  4. Rotate left, rotate right, flip horizontal, flip vertical
  5. LIGHT: brightness, contrast, highlights, shadows
  6. COLOR: temperature, tint, saturation
  7. EFFECTS: black and white, sepia, vignette as on/off buttons; each one's strength slider appears only when it's on (INT-4)
  8. RED-EYE: Fix red-eye…
  9. PRESETS: list, then "Save current edits as a preset…"
  10. Revert to original… (last, styled as cautionary)
- **Photo** on the dark surround with a **filmstrip** (84 px) beneath; the current photo outlined.
- Sliders show signed values; double-click resets.

## 6. Crop and straighten (EDT-2, EDT-3)

- The whole photo shows, rotated by the straighten angle, with the crop box, rule-of-thirds lines, corner handles, and the area outside darkened; a label shows ratio and resulting pixel size.
- **Panel:** aspect ratios (Original, Freeform, 1:1, 4:3, 3:2, 16:9, 5:4) as a 3-column grid; Landscape or Portrait; Straighten slider with its value and a note that the crop shrinks so no corners show; Reset crop; Cancel and Done at the bottom right. Enter applies, Esc cancels.

## 7. Tag picker (TAG-3)

- A popover anchored beside the selection (or the Add tag control), with a pointer to it (INT-5).
- Title "Tag 3 photos" and the key hints; chips for tags already added, each with ×; a text field; the list of existing tags with counts, filtered as the user types, first match highlighted; "Create tag "…"" at the top when the text matches nothing.

## 8. Export (EXP-*)

- A modal over the dimmed Library: Save to (path and Choose…); Format (JPEG with quality, or PNG) beside Size (Original, 2048, 1600, 1024, Custom px); Details (Remove location data; Include tags and rating as keywords); a sentence that originals aren't changed and names that are taken get a number; Cancel and Export.
