# Seura Photos — Wireframes (v1)

Status: approved · Last updated: 2026-10-27

The wireframes live on a design canvas in the project owner's Claude account. This document records what they establish, screen by screen, so the decisions are available in the repository. They define layout and flow only; visual styling (color, type, iconography) comes later. Requirement IDs refer to the [functional spec](../functional-spec.md); flows are in [user-flows.md](user-flows.md).

All screens were drawn at 1280×800 logical pixels, close to the 1280×720 minimum (DSP-2), plus one Library at 1920×1080 to show how extra space is used (DSP-3).

## Shared structure

- **Toolbar** (52 px) across the top of every view; **status bar** (28 px) at the bottom of the Library and Viewer. On Windows, the toolbar ends with a ⋯ button (32 px, quiet style, tooltip and accessible name "More"), opening a small menu of app-wide commands: About Seura Photos now, and Settings… and Clean up… once they're built (ACC-4). On macOS these are in the menu bar, and there's no ⋯.
- **About** (Windows; SET-5): a small dialog: the mark and "Seura Photos"; the version on its own line ("Version 0.3.0"), selectable, with Copy beside it; for 0.x, "An early build, before the beta."; links to seuraphotos.app, the GitHub project, and the Apache 2.0 license; OK. On macOS, the system's About panel shows the same.
- **Sidebar** (232 px by default; 260 px at 1920 wide): FOLDERS tree, then TAGS tree, each row with a count. The FOLDERS heading has an Add folder button (+, 32 px, accent color) at its right and stays in view while the tree scrolls. A link at the end of the tree scrolled out of sight with many folders, and a button pinned to the bottom of the sidebar would end up below TAGS. TAGS can follow the same pattern if tags get an add action (Nevada, 2026-10-22). A name that doesn't fit ends with "…" and shows in full in a tooltip. Each heading also has a Collapse all button (Lucide "chevrons-down-up", 32 px, quiet style, tooltip and accessible name "Collapse all"), to the left of FOLDERS' +; it's disabled when nothing but the way to the selection is open (LIB-2, TAG-5).
- **Resizable sidebar** (Nevada, 2026-10-23; #74): drag the divider between the sidebar and the grid, from 180 to 400 px. 180 fits the FOLDERS heading with its + and a couple of levels, even with text 40% longer (ACC-7); 400 still leaves the grid 600 px, three columns at the default thumbnail size, in a 1280-wide window (DSP-2). The width is remembered; until the user drags it, the default follows the window as above. At rest the divider is the existing 1 px border; its grab area is 8 px wide, centered on it. On hover and while dragging, the resize cursor shows and the line becomes 2 px in the accent text color, with the tooltip "Drag to resize. Double-click to reset." Double-click resets the default. From the keyboard it's a focusable separator ("Sidebar width", with its value), in the Tab order between the sidebar and the grid, showing the same 2 px line and the focus ring: ← → move it 16 px, Home and End go to the minimum and maximum, and Enter resets it. The Info panel stays a fixed width.
- **Selected row** in either tree: the accent-subtle fill, a 3 px bar on its left edge and the text and icon in the accent text color, so the selection shows at well over 3:1 in both modes (the fill alone is about 1.1:1).
- **Info panel** (280 px; 320 px at 1920 wide), always shown in Library and Viewer: preview, file name, date (date modified when there's no date taken), size, camera, lens, exposure, location, edit state with Revert, rating stars, tag chips with Add tag.
- Dark surround (#2b2d30 in the wireframes) behind photos in the Viewer, Editor, and Review, so photos aren't judged against a bright frame.
- Control sizes are fixed across window sizes; only photo areas and panel widths grow.

## 1. Welcome (SET-1, SET-4)

A single centered card: title; two sentences (what the app does; originals are never changed and edits live beside each photo); "Which folders hold your photos?" with Pictures pre-checked and Add another folder; a cloud-sync note for the cache when relevant; "Check for updates automatically," checked; Get started (44 px, bottom right).

## 2. Library (LIB-*, BRW-1–10, REV-1)

- **Toolbar:** app name; search; Review; then, on the right, the selection's quick fixes (rotate left, rotate right, Enhance), Edit (primary), Export.
- **Review banner** above the grid when new photos are found: "12 new photos in Summer. Sort through them one at a time?" with Review and Not now.
- **Folder header row:** name, path, count; then Filter, sort, and thumbnail size on the right.
- **Filter controls** (FND-1), under the heading when Filter is open, always on two lines: Rating, Tags and Edits; then Taken from and to. At the default window size in English, the five controls on one line measured about 970 px against about 880 available, so they wrapped (#118). Two fixed lines keep the area's height the same as the window or the sidebar is resized (Nevada, 2026-10-28). The Tags control is at most 200 px wide, with a long tag cut off with "…" and shown in full in its list and tooltip. Where a line still doesn't fit, as with longer translated text (ACC-7) or a narrow window, it wraps rather than clipping anything.
- **Notice bar** at the top of the photo area, above the heading, for problems with the selected photos' files (flow 11; INT-6).
- **Message** near the bottom of the photo area, centered, for the last action's result: "Added. Scanning for photos…", "Moved 3 photos to the Recycle Bin.", "Undid rating on 5 photos" (INT-6).
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
- **Notice bar and message** as in the Library, over the photo area: the notice bar pushes the photo down rather than covering it.
- **Status bar:** zoom level and the main keys only.

## 5. Editor (EDT-*, PRE-*)

- **Toolbar:** file name, "Edits saved automatically," undo, redo, Copy edits, Export. No Done here. Copy edits, Export, and the presets section arrive with M5; until then they're left out, not shown disabled.
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
- Sliders show signed whole numbers (EDT-5): "+12", "0", "−30"; brightness on the same scale, 25 to a stop. Temperature and tint name their ends under the track: Cool and Warm, Green and Magenta. Double-click resets.

## 6. Crop and straighten (EDT-2, EDT-3)

- The whole photo shows, rotated by the straighten angle, with the crop box, rule-of-thirds lines, corner handles, and the area outside darkened; a label shows ratio and resulting pixel size.
- **Panel:** aspect ratios (Original, Freeform, 1:1, 4:3, 3:2, 16:9, 5:4) as a 3-column grid; Landscape or Portrait; Straighten slider with its value and a note that the crop shrinks so no corners show; Reset crop; Cancel and Done at the bottom right. Enter applies, Esc cancels.

## 7. Tag picker (TAG-3)

- A popover anchored beside the selection (or the Add tag control), with a pointer to it (INT-5).
- Title "Tag 3 photos" and the key hints; chips for tags already added, each with ×; a text field; the list of existing tags with counts, filtered as the user types, first match highlighted; "Create tag "…"" above them whenever the text isn't already a tag, highlighted only when nothing matches (TAG-3).

## 8. Export (EXP-*)

- A modal over the dimmed Library: Save to (path and Choose…); Format (JPEG with quality, or PNG) beside Size (Original, 2048, 1600, 1024, Custom px); Details (Remove location data; Include tags and rating as keywords); a sentence that originals aren't changed and names that are taken get a number; Cancel and Export.
