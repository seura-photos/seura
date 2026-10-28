# M4 work plan: Fix

Status: draft for review · Last updated: 2026-10-28

The working plan for milestone M4 from the [milestone plan](milestone-plan.md): the full Editor. It breaks the milestone into slices, one pull request each (or a few, lettered), in order. Drafted by Claude Code from the milestone plan, the [functional spec](functional-spec.md), the [user flows](ux/user-flows.md) and [wireframes](ux/wireframes.md), the [visual design](visual-design.md), the [technical design](technical-design.md), and the [edit format](https://github.com/seura-photos/edit-format). The questions it couldn't settle are at the end, for a design session.

**Scope** (milestone plan, M4): EDT-1 to EDT-12 (crop, straighten, rotate and flip, adjustments, effects, red-eye, reopening, undo across sessions, revert, autosave, and Enhance); rotate and Enhance as quick actions in the Library and the Viewer (BRW-10); each operation implemented once as WGSL (decision 0004), with a reference-image test showing preview and export match.

**Done when** (milestone plan): Nevada fixes his own photos in Seura instead of another app, and the edit format is proven by a full implementation, so it can be frozen at 1.0 in M7.

**Also carried in** (M3 plan, "Carried forward", and the design answers since):
- the notice levels, messages after a key, date ranges, and the filter row (#128);
- Collapse all and expanding beneath a node (#124);
- the system's error text when a photo can't be moved to the Trash, and a signing test on the Mac (#112);
- dates taken lost when the app is stopped without quitting (#99);
- the HEIC colour test failing on Nevada's Mac only (#91);
- Node 26;
- the edit-format submodule moved to its latest `main` (presets in the home folder);
- FIL-3's "keep yours or load theirs" in the Editor;
- the Editor's downloading state for cloud-only photos (LIB-9);
- undo and redo buttons in the toolbars.

Already built for 0.3, on `main`: About and the Windows ⋯ button (#121), and keys kept inside dialogs (#122).

**Not in M4:** copy and paste of edits, and presets (PRE-*, M5); export as a feature (EXP-*, M5; M1's development-only export stays as it is); Review (M6); HDR display (#60, a design question).

## How to work

As in M3 ([M3 work plan](m3-plan.md), "How to work"): one slice per branch and pull request, branched from `main` and named `m4/<slice>`; Nevada merges. Every pull request's base is `main` (none stacked on another's branch). Dependencies at least two weeks old, checked against decision 0012. Each slice has an issue under the "M4" milestone on GitHub, and its pull request closes it. Screens are styled from the tokens and checked against the mockups, with screenshots in light and dark mode in the pull request. New text goes in the message catalogs (ACC-5). Journal everything.

New in M4:

- **One implementation per operation** (decision 0004). Each operation is one WGSL function in `crates/seura-render/shaders/`, run by wgpu for export and edited thumbnails, by WebGPU in the preview, and through the generated GLSL on WebGL2. No operation is written twice, in Rust or TypeScript.
- **A reference-image test for every operation.** Each slice that adds an operation adds reference images and expected results to `seura-render`'s tests, and the render check (`scripts/render_check.py`) compares the preview with the export in the real web view, on both platforms and on WebGL2. Reference images are generated, or photos with a licence that allows them in the repository, never Nevada's.
- **Formulas written as they're built.** Each operation's exact formula goes into the edit format's formulas appendix as it's implemented, so the appendix and the shaders never disagree (question 1).
- **PRF-6 measured as operations are added.** The measurement script drags a slider on 12, 24 and 48 megapixel photos and reports frames per second, so a slow operation is caught in its own slice, not at the end.
- **The Editor stays development-only** until slice 11, as in M1 to M3, so a half-built Editor never reaches a public build.
- **Edits go through the one write path** the Library uses (M3, slice 1): atomic saves, history, never overwriting what can't be read or changed on disk. An edit changes how a photo looks, never the photo (CLAUDE.md, rule 1).

## Slices

### 0. Carried in, for 0.3 (#128, #124, #112, #99, #91)

- **Notice levels** (visual design; #128): information on `accentSubtle` with the info icon in `accentText` (FIL-5, the kept file); caution on `cautionSubtle` with the warning icon in `caution` (FIL-7); a problem on `dangerSubtle` with the warning icon in `danger` (FIL-4). Text in the primary colour. Icons on `accentSubtle` use `accentText` everywhere, not `accent`.
- **Messages after a key** (INT-6): a rating, tag or other change made with a key shows a message ("Rated 3 photos 3 stars", "Cleared the rating on 2 photos", "Added the tag beach to 5 photos"). The same change made with a click doesn't. A new message replaces the one before.
- **Date ranges** (BRW-6; #125): a selection's range is formatted with `Intl.DateTimeFormat.prototype.formatRange` in the region (ACC-6), so shared parts aren't repeated, and one date shows when both ends are the same at the displayed precision.
- **The filter row** (#118): the Tags control capped at 200 px, a long tag cut short with "…" and named in full in its tooltip; the row wraps only when it must.
- **The sidebar trees** (LIB-2, TAG-5; #124): Collapse all in each heading, keeping the way to the selection open; Alt-click (Option-click) on an arrow and "Expand all beneath" in the menu to open a branch; `*` to open the focused node's siblings.
- **Delete on the Mac** (#112): the "couldn't be moved" message includes the system's own reason. Separately, on the Mac, a local build signed with a free Apple Development certificate is tested on a downloaded OneDrive photo (question 8).
- **Dates taken** (#99): saved as they're learned, so stopping the app without quitting loses none.
- **The HEIC test on Nevada's Mac** (#91): found and fixed, or the test made to say why it differs.
- **Node 26** and the edit-format submodule moved to its latest `main`.
- **Done when:** Nevada has seen the new notice levels, a message after a key, a date range and the trees' new controls on both machines, and the error text appears for a photo the Mac refuses.

### 1. The Editor's frame (EDT-1, EDT-8, EDT-11, LIB-9)

- **Opening and closing:** `E` or Edit from the Library (the selection's first photo) and the Viewer opens the Editor; Done or Esc goes back where the user came from, at the same place (user flows, flow 4).
- **The layout** from the wireframe: the toolbar (file name, "Edits saved automatically", undo, redo); the 300 px tool panel on the right in process order (INT-3), with Hold to compare and Done (44 px) on its first row; the photo on the dark surround; the filmstrip (84 px) beneath, moving to the next photo without leaving the Editor. Toolbar and panel items for M5 (Copy edits, Export, presets) are left out until they're built (question 2).
- **Compare** (EDT-1): holding `\` or the Compare button shows the original.
- **A cloud-only photo** shows the Viewer's downloading state (LIB-9).
- **Reopening** (EDT-8): every control shows its stored value.
- **Autosave** (EDT-11): a change is written within 1 second of the last one, through the one write path; a slider drag is one change when it ends (edit format, "History").
- **The preview pipeline** grows from M1's single exposure uniform to a settings block with every operation's parameters and its neutral flags (technical design, section 6), so later slices only add a function and its uniforms.
- **Done when:** Nevada opens photos in the Editor from the Library and the Viewer on both machines, moves through the filmstrip, compares, and finds M1's exposure setting saved and shown again after a restart.

### 2. Undo, redo, and revert (EDT-9, EDT-10, FIL-3)

- **Undo and redo** in the Editor work on the photo's own history, the last 50 changes, across sessions (EDT-9), with the toolbar's buttons and the platform's keys. The Library's undo (BRW-11) and the Editor's share each photo's history, so a change made in one can be undone in the other when it's the latest.
- **Revert to original** (EDT-10): after a confirmation with the safe choice where the pointer is (INT-5), removes every operation and keeps tags and rating; it's one record in the history, so it can be undone.
- **Outside changes** (FIL-3): when the open photo's edit file changes on disk, the Editor keeps the user's current work and says the file changed, with "Keep yours or load theirs?" (user flows, flow 11).
- Data-safety tests for undo, redo and revert, and for a file changed on disk while the Editor has unsaved work.
- **Done when:** Nevada undoes and redoes changes on both machines, including after a restart, reverts a photo and undoes the revert, and sees the outside-change notice when the file is changed from the other machine.

### 3. Light and color (EDT-5, except highlights and shadows)

- Brightness (`seura.exposure`, built in M1), contrast, temperature and tint (`seura.whiteBalance`), and saturation, as point operations in the fused tone pass.
- Sliders start at a neutral centre and show signed values; double-click resets one (EDT-5). Neutral settings remove the operation, as the format advises.
- Each operation's formula is written into the formulas appendix as it's built, with reference images and expected results.
- **Done when:** each slider moves the preview live within PRF-6 on both machines and on WebGL2, the render check shows preview and export match, and Nevada has adjusted real photos with them.

### 4. Highlights and shadows (EDT-5)

- **The luminance guide** (technical design, "Neighborhood operations"): an edge-preserving, blurred map of the photo's brightness at a fixed size relative to the photo, computed once and sampled by the fused pass, so preview, tiles and export match.
- **The method is chosen with reference images** that include strong edges, such as a dark tree against a bright sky, comparing a guided filter, a bilateral grid or similar for halos, speed and how simply it can be written as a formula. The comparison goes in the journal, and the choice into the formulas appendix.
- **When the guide is rebuilt:** moving highlights or shadows changes only uniforms; how changes to exposure and white balance reach the guide is measured against PRF-6.
- **Done when:** the render check matches on both platforms and on WebGL2, PRF-6 holds on 48 megapixel photos, and Nevada has recovered a sky and lifted a backlit face on his own photos without halos he can see.

### 5. Rotate and flip, straighten, and crop (EDT-2, EDT-3, EDT-4)

- **Rotate and flip** (`seura.orient`): rotate left and right, flip horizontally and vertically, in the tool panel.
- **Crop mode** (`C`, or Crop and straighten in the panel; wireframe, "Crop and straighten"): the whole photo with the crop box, rule-of-thirds lines, corner handles and the darkened outside; the ratio grid (Original, Freeform, 1:1, 4:3, 3:2, 16:9, 5:4), Landscape or Portrait, the size label; Reset, Cancel, Done; Enter applies and Esc cancels. Reopening shows the whole photo with the current box (EDT-2).
- **Straighten** (`seura.straighten`): −45° to 45° in 0.1° steps, in crop mode; the crop shrinks so no empty corners show (EDT-3).
- **Geometry is a coordinate transform when sampling** (technical design, "Pipeline"), never a separate pass. Rotating or flipping a cropped photo moves the crop with it (question 3).
- **Done when:** Nevada fixes a crooked photo in under five minutes (user flows, flow 4's design test), the render check matches with geometry and tone together, and edited thumbnails show the rotation and crop.

### 6. Effects (EDT-6)

- Black and white, sepia, and vignette (amount and size) as on and off buttons in the panel, each one's strength slider appearing when it's on (INT-4). They combine with the adjustments in the format's fixed order. The vignette is measured on the cropped frame.
- **Done when:** the render check matches with every effect, and Nevada has used each on his own photos.

### 7. Red-eye (EDT-7)

- **Red-eye mode:** the user clicks or drags over each eye; each correction is a circle that can be moved, resized or removed. Regions are stored on the photo before geometry, so they stay on the eyes when the crop changes. No automatic detection.
- **Done when:** the render check matches with red-eye and a crop together, and Nevada has corrected red eyes in a real photo (one with red eyes he's happy to use, or a licensed sample).

### 8. Enhance (EDT-12)

- **The analysis** (technical design, "One-click enhance"): a compute pass in the core over a downscaled copy sets brightness, contrast, highlights, shadows, temperature, tint and saturation, as ordinary slider settings in one history record.
- **Tuned on a set of reference photos** of common faults (dark, bright, low contrast, a colour cast, backlit), whose results Nevada judges by eye; the numbers and thresholds go in the journal.
- **Done when:** Enhance improves most of Nevada's test photos in his judgement, never makes one unusable, and its result can be adjusted or undone like any other.

### 9. Quick actions in the Library and the Viewer (BRW-10)

- **Rotate left and right, and Enhance,** for the selection in the Library's toolbar and for the photo in the Viewer's, with their keys (`[`, `]`, and `A`); and on a thumbnail while it's hovered (32 px, INT-5).
- Each is one action for the Library's undo (BRW-11), and a key shows a message (INT-6). Thumbnails are redrawn with the new edits.
- **Done when:** Nevada rotates and enhances photos from the grid, by hovering and by key, and in the Viewer, on both machines, and undoes each.

### 10. The formulas appendix and the format's freeze (edit format)

- **The appendix completed** in the `edit-format` repository: every core operation's formula, the luminance guide, the order of the fused pass, the coordinate rules for geometry and red-eye, and the reference images with expected results, so another app could render an edit file the way Seura does.
- **What else 1.0 needs:** anything the full implementation found unclear or missing in the specification, raised with the design session.
- **Done when:** a design session has reviewed the appendix, and the format's own tests run against the reference images.

### 11. Wrap-up

- **Measure** PRF-6 on both machines, on WebGPU and WebGL2, and every earlier target again; results in `docs/benchmarks/`.
- `check.py` over the edit files written by the Editor in Nevada's library.
- **The Editor in release builds,** then build 0.3, the devlog post, part 10, and confirming every "done when".
- **Done when:** Nevada fixes his own photos with build 0.3 on both machines, instead of another app.

## Questions for a design session

1. **Who writes the formulas appendix?** The edit format belongs to the design sessions, but the formulas come from the implementation. Claude Code's suggestion: it writes each formula into the appendix in its own pull request to `edit-format` as the operation is built, and the design session reviews them there, with the whole appendix reviewed again in slice 10.
2. **Toolbar and panel items not built yet.** The Editor's wireframe has Copy edits, Export and the presets list, which come in M5. Suggestion: leave them out until they're built, as About's ⋯ menu leaves out Settings and Clean up, rather than showing them disabled.
3. **Rotating a cropped photo.** When a cropped photo is rotated 90° or flipped, should the crop follow the photo (the same part of it stays, and the box turns with it), or reset? Suggestion: it follows, as in Photos and Lightroom; the ratio's orientation swaps with it.
4. **Straighten and the crop.** As the angle grows, the crop shrinks so no corners show (EDT-3). When the angle goes back, should the crop grow again to what the user had set, or stay as small as it got? Suggestion: grow back to the user's crop, as long as straighten mode is open; the user's last crop is what's stored.
5. **How sliders show their values.** "Sliders show signed values" (wireframes). Suggestion: brightness in stops with two decimals (+0.50), as the format stores it; the others as whole numbers from −100 to 100; temperature and tint with words at the ends (Cool, Warm; Green, Magenta).
6. **Enhance on an edited photo.** Suggestion: Enhance replaces the seven settings it computes and leaves crop, rotation, effects and red-eye as they are; pressing it again gives the same result, not a stronger one.
7. **When 0.3 ships.** Decision 0016 makes each milestone's build public. 0.3 is M4's, but slice 0 and About are ready long before the Editor. Suggestion: an interim 0.2.2 with slice 0, About and the dialog keys, so testers can say which build they have, and 0.3 with the Editor at the end of M4. Nevada's call.
8. **Signing on the Mac (#112).** The test needs Xcode, a large install, on Nevada's MacBook, and his free Apple account as a Personal Team. Nevada's OK is needed before the Mac session installs anything.
