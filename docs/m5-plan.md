# M5 work plan: Reuse and share

Status: agreed · Last updated: 2026-11-03

The working plan for milestone M5 from the [milestone plan](milestone-plan.md): getting edited photos out of Seura, and reusing edits on other photos. It breaks the milestone into slices, one pull request each (or a few, lettered), in order. Drafted by Claude Code from the milestone plan, the [functional spec](functional-spec.md), the [user flows](ux/user-flows.md) and [wireframes](ux/wireframes.md), the [visual design](visual-design.md), the [technical design](technical-design.md), and the [edit format](https://github.com/seura-photos/edit-format). The questions it couldn't settle are at the end, with the answers from the design session (#205, edit-format #17) and from Nevada.

**Scope** (milestone plan, M5): copy and paste of edits, and presets (PRE-1 to PRE-5); export (EXP-1 to EXP-6), including sRGB conversion and metadata handling.

**Done when** (milestone plan): Nevada exports real photos for sharing, and the exports look right in other apps on both platforms.

**Also carried in:**
- decoding wide-gamut colors unclipped, and fuller color descriptions (#188), since export is where they would show;
- the one-time message about a cloud service's deletion notice (#200; FIL-1);
- a change made in the last second before quitting isn't saved (#174);
- Node 26 (#156), from 2026-11-11;
- testing Delete on the Mac with Google Drive and Dropbox (#162), which needs Nevada's hands;
- the technical design's sentence that an abandoned read stops a download on macOS (section 4), which #202 corrected in the functional spec but not there.

**Not in M5:** Review and its "Export photos rated 4★ or more" (M6); the Settings screen and its default export options (SET-2), so the dialog remembers its own choices until then; next-photo speed (#190), within its target and left for M7's performance audit, as Nevada decided in M4's wrap-up; HDR display (#60).

**Before M5 starts:** M4's "done when" is still Nevada's to confirm, and #139, #141 and #142 are still open. They'll be closed or carried over against the M4 plan when it's recorded. Nothing in this plan depends on the order, except that the edit format's appendix (#141) isn't touched here.

## How to work

As in M4 ([M4 work plan](m4-plan.md), "How to work"): one slice per branch and pull request, branched from `main` and named `m5/<slice>`; Nevada merges. Every pull request's base is `main` (none stacked on another's branch). Dependencies at least two weeks old, checked against decision 0012. Each slice has an issue under the "M5" milestone on GitHub, and its pull request closes it. Screens are styled from the tokens and checked against the mockups, with screenshots in light and dark mode in the pull request. New text goes in the message catalogs (ACC-5). Journal everything.

New in M5:

- **Export first, then reuse.** The milestone plan lists copy and paste first, but since 0.3 edits can be seen in Seura and can't be taken out of it, so export comes first. The two don't depend on each other.
- **Exports are checked in other apps, not only by Seura's own decoder.** Each export slice ends with a file looked at elsewhere: Photos and File Explorer on Windows; Preview, Photos and Finder on the Mac; and a command-line reader for the metadata. Test photos with metadata are generated, or licensed for the repository, never Nevada's.
- **One render per operation still holds** (decision 0004). Export runs the same shaders, and neither pasting nor presets adds an operation.
- **Data-safety tests grow with each write:** export never replaces a file and never touches the original (EXP-3; CLAUDE.md, rules 1 and 2), and presets are written only to `Seura Photos/Presets` and removed only to the Trash (rule 5). `scripts/app_data_audit.py` runs over a build that has exported and saved a preset.
- **Pasting and applying presets need no pixels,** like rotate, so they work on cloud-only photos without downloading them (question 5).

## Slices

### 0. Carried in (#174, #156, #162)

- **Saving on quit** (#174, EDT-11): a change still waiting when the window closes or the app quits is saved first.
- **Node 26** (#156), once it's allowed on 2026-11-11, so this can land at any point in the milestone.
- **The technical design's sentence** on abandoned downloads, corrected to match LIB-9 as a small factual fix, said plainly in the pull request.
- **Mac, with Nevada** (#162): Delete with Google Drive and Dropbox, on test copies, whenever it suits.
- **Done when:** a slider moved just before Ctrl+Q or Cmd+Q is there after reopening, on both machines.

### 1. Decoding colors in full (#188; edit format's decoding paragraph)

- **Unclipped conversion:** moxcms's extended range, so colors outside Display P3, such as pure green from an Adobe RGB profile, survive to the render. Measured first, since moxcms says it costs time, against PRF-3, PRF-5 and the render check. It's a feature change on an existing dependency.
- **HEIC color descriptions:** the primaries other than Display P3, such as BT.2020, and the transfer characteristics, instead of assuming the sRGB curve.
- **PNG color chunks:** `cICP`, `sRGB`, and `gAMA` with `cHRM`, as well as `iCCP`.
- **16-bit PNGs** keep their precision through the conversion, if the cost is small; otherwise this part stays open in #188, since it isn't against the format's SHOULD.
- **Done when:** generated test images in each color description decode to the expected values, the render check passes on both machines, and the speed targets still hold.

### 2. Rendering and encoding an export (EXP-2, EXP-3, EXP-5)

No screens yet: `seura-render`'s export grows from M1's `export_jpeg` into one function taking the options, used by tests and by the dialog in slice 4.

- **Formats:** JPEG with a quality setting, default 90 (`encode_jpeg` already embeds the sRGB profile), or PNG, with an sRGB profile. HEIC originals export as either.
- **Sizes:** the original, or the long edge limited to 2048, 1600, 1024 or a custom value, applied to the photo as cropped. A limit never enlarges, so a photo smaller than it keeps its size, and a custom value is a whole number of pixels from 16 up (question 7). Resizing happens once, after the render.
- **Color** (EXP-5): the wgpu pipeline's existing sRGB output, checked on an iPhone Display P3 photo and on a wide-gamut generated image, in other apps.
- **Names** (EXP-3): `write_new` is already there and never replaces a file. Tests cover a new extension, taken names, and a name taken meanwhile.
- **The encoder:** the comparison of `jpeg-encoder` with `mozjpeg` the technical design asks for, at quality 90, on file size and time. `mozjpeg` is C with extra build tools, so it needs a clear win to change; the result goes in `docs/benchmarks/`.
- **Done when:** the reference-image tests pass for every format and size, on both machines, and a 48-megapixel export fits in memory.

### 3. Metadata (EXP-4)

- **Kept:** the original's EXIF, read from JPEG, HEIC or PNG, carried into the new file with `img-parts`, as the technical design says.
- **Corrected, since the pixels changed:** orientation reset to 1; pixel dimensions set to the exported size; the color space tag set to sRGB, since iPhone photos say "uncalibrated".
- **Removed:** the thumbnail embedded in EXIF, and any previews, including MPF. They would show the photo unedited and uncropped in some viewers, and can hold a picture the user cropped out. Tested on photos that have both.
- **Remove location data:** every GPS field. Only EXIF is copied and XMP is written new, so location can't arrive through the original's XMP. Tested by reading the result with a tool Seura doesn't use.
- **Include tags and rating as keywords:** in a small XMP packet Seura writes itself, without `xmp_toolkit` (question 9). `dc:subject` gets each level of a tag as its own keyword, without repeats (`family/kids` gives `family` and `kids`); `lr:hierarchicalSubject` gets the whole tag with `|` (`family|kids`); the rating is `xmp:Rating` (question 8).
- **Done when:** date, camera and lens appear in Photos, File Explorer, Preview and Finder for JPEG and PNG exports from JPEG and HEIC originals; location is gone when asked; keywords and rating are read by Windows Explorer, by Finder or Preview, and by one of Lightroom or digiKam (installing either is Nevada's call).

### 4. The Export dialog and batches (EXP-1, EXP-6, LIB-9)

- **The dialog** from the wireframe: Save to (path and Choose…); Format beside Size; Details; the sentence about originals and names; when any photo is only in the cloud, "3 of these photos are in the cloud (24 MB) and will be downloaded." above the buttons, with Export as the confirmation (question 6); Cancel and Export. Export on the toolbars of the Library, Viewer and Editor, and Ctrl+E (Cmd+E). The Editor saves anything waiting first.
- **Remembered in `settings.json`:** the last folder and the dialog's choices. The first export starts in the Pictures folder; a folder in the library is allowed, and the files simply appear there as new photos (question 11). If the last folder is gone, Save to is empty and Export waits for one. M1's development-only export goes.
- **Progress with Cancel,** in the batch's message like Enhance's, and off the main thread. One photo at a time, so a 48-megapixel batch holds one photo in memory. Cancel stops between photos and stops waiting for one in progress; files already written stay, and a partly written temporary file is removed (rule 5).
- **Cloud-only photos** (LIB-9, EXP-6): shown as downloading with their size while the export waits for each, with Stop's behavior already settled in #202: the download may still finish (question 6).
- **The summary:** for a batch, how many were exported, which couldn't be and why, and Show in Explorer or Finder. One photo gets a message with Show in Explorer or Finder instead (question 10).
- **Problems:** a full drive or a folder that can't be written says so and stops; one unreadable photo is reported and the others go on.
- **Done when:** Nevada exports a mixed selection, some in the cloud, to a local folder on both machines, cancels one, and finds the names, summary and originals as they should be.

### 5. Copy and paste edits (PRE-1, PRE-2)

- **Copy edits:** Shift+Ctrl+C (Shift+Cmd+C), in the Library, the Viewer and the Editor's toolbar. It copies from the focused photo, and the message names it ("Copied the edits of IMG_1234."). A photo with no edits is refused with a message, and what was copied earlier stays (question 2). The copy is held in memory in the core: not the system clipboard, and gone when the app quits.
- **Paste edits:** Shift+Ctrl+V (Shift+Cmd+V), then the small dialog: Adjustments (tone and color) and Effects on, Crop and rotation (the whole geometry stage: orient, straighten, crop) off, and red-eye never offered (question 1). Nothing to paste, or nothing in a group, is shown rather than failing silently.
- **A crop on a photo of another shape:** a crop with a ratio lock keeps its ratio, by the rule Claude Code chose, as the design session asked. The center stays where it was and the crop's share of the photo's area stays the same; if that doesn't fit, it shrinks until it does, then moves inward only as far as it must. In the target's frame W × H, after any pasted orient, with ratio `a` and share `s = width × height`: `width = sqrt(s × a × H / W)`, `height = s / width`. A target of the same shape gets exactly the source's crop. If the pasted straighten would then show empty corners, the crop shrinks as it does in the Editor. A crop with no lock keeps its fractions.
- **A target whose dimensions aren't known** (cloud-only, never read) isn't downloaded. If the pasted group holds a ratio-locked crop or a straighten angle, which need its shape, that photo keeps its whole crop and rotation group as it was, the other groups paste as usual, and the message says so: "1 photo is in the cloud, so its crop and rotation weren't changed." Rotation and flips, and a crop with no lock and no angle, need no shape and paste as usual (question 15).
- **Replacing, by stage** (PRE-2; edit format, "Presets"): the chosen stages replace the target's and the others stay. It goes through the one write path, so each photo's history records it, and read-only folders and unreadable edit files are reported as for any change.
- **Undo:** a paste onto a selection is one action to undo in the Library (BRW-11); in the Editor it's a step in the photo's own history (EDT-9). Pasting nothing into a photo with no edits leaves no edit file; pasting that removes the last edit follows FIL-1.
- **Other apps' operations** (FIL-5): within a stage, Seura replaces only what it understands. A target's others stay, and a copied photo's aren't added (question 3; edit-format #17, with the submodule updated in this slice).
- **Done when:** Nevada copies edits, including a ratio-locked crop, from one photo to a selection of various shapes, some in the cloud, on both machines, and one undo reverses it.

### 6. Presets as files (PRE-3, PRE-5)

- **Save as preset** in the Editor: a name and the same choices as pasting, saved as `<name>.seura-preset.json` in `Seura Photos/Presets` (edit format, `kind: "preset"`), atomically. Names are cleaned up for the file system on both platforms, and a name in use asks first; nothing is overwritten silently.
- **The list:** the Presets folder is watched, so files copied in appear without restarting. A file that can't be read is listed with a note and never rewritten (rule 4).
- **Rename and delete** in the app. Delete goes to the Trash or Recycle Bin (rule 5), after a confirmation.
- **Done when:** a preset saved on one machine and copied to the other appears and validates with the format's `check.py`.

### 7. Applying presets (PRE-4, INT-3)

- **The Editor's Presets section,** last before Revert, per the wireframe. Hovering or focusing a preset previews it on the photo, through the same preview renderer, writing nothing; leaving restores the photo; clicking applies it as one step in the photo's history. The preview has to stay smooth, so PRF-6's measurement gets a preset case.
- **In the Library:** a Presets submenu in the photo's right-click menu on both platforms, and in the macOS Edit menu, not in the Windows ⋯ menu (question 4). It acts on the selection, as one action to undo, and needs no download (question 5).
- **Stages:** a preset replaces the stages it holds and leaves the others (edit format), including a geometry stage if the file has one, which follows slice 5's rules for other shapes and unknown dimensions.
- **Done when:** Nevada previews and applies presets on photos of different shapes on both machines, and applies one to a selection.

### 8. Explaining a service's deletion notice (#200, FIL-1)

From the design session's answer in #202: no dialog, no checkbox and no setting.

- **The first time** an edit file goes to the Trash from a folder a cloud service syncs, the message (INT-6) says what happened, once per service: "Removing this photo's last rating moved its small edit file to the Trash. OneDrive may say a file was deleted; the photo itself isn't affected."
- **The action and service are named,** using `sync_service` as Delete and Welcome do, with a general wording for undo or several photos, and one message per batch.
- **Remembered in `settings.json`,** by service.
- **Done when:** Nevada sees it once, on the Mac, with OneDrive, and not again; there's a test for each wording.

### 9. Wrap-up

- **Measure** every target again on both machines, and report export's speed, for which the spec sets no target.
- **`check.py`** over edit files with pasted edits and over presets, in Nevada's library.
- **Build 0.4,** the devlog post, and confirming every "done when". Each outward step needs Nevada's approval.
- **Done when:** Nevada exports real photos for sharing with build 0.4, and they look right in other apps on both platforms.

## Questions

All are answered: 1 to 12 and 15 by the design session on 2026-11-03 (#205, edit-format #17; journal, `2026-11-03-m5-plan-answers.md`), and 13 and 14 by Nevada.

1. **Paste's groups.** Three checkboxes: Adjustments (tone and color) and Effects, on by default, and Crop and rotation (the whole geometry stage), off by default. Red-eye is never pasted. A ratio-locked crop keeps its ratio on a target of another shape, by a rule for Claude Code to choose (slice 5).
2. **What Copy copies from.** The focused photo, and the message names it. A photo with no edits is refused with a message, and the earlier copy stays.
3. **Other apps' operations.** Within a stage, an app replaces only the operations it understands. Others stay on the target, and the source's aren't added (edit-format #17).
4. **The Library's Presets menu.** A Presets submenu in the photo's right-click menu on both platforms, and in the macOS Edit menu. Not in the Windows ⋯ menu, which is for app-wide commands. This differs from the suggestion.
5. **Pasting and presets on cloud-only photos.** No download. LIB-9 is reworded: only changes that need pixels, such as Enhance, download.
6. **Exporting many cloud-only photos.** The line sits above the dialog's buttons, and Export is the confirmation.
7. **Sizes.** Limits never enlarge. A custom value is a whole number of pixels from 16 up. All the choices are remembered.
8. **Slashes in keywords.** `dc:subject` gets each level as its own keyword, without repeats; `lr:hierarchicalSubject` gets the whole tag with `|`; the rating is `xmp:Rating`. This differs from the suggestion.
9. **XMP's library.** Seura writes the small packet itself, and `xmp_toolkit` is out (technical design updated). Tests check that Windows Explorer, macOS (Finder or Preview) and one of Lightroom or digiKam read the keywords and rating.
10. **The result of a one-photo export.** A message with Show in Explorer or Finder, and no separate summary.
11. **An export into a library folder.** Allowed, with nothing special. The first export defaults to the Pictures folder.
12. **Buttons drawn on the photo.** They stay as they are.
13. **An interim build (Nevada).** Yes, as version 0.3.1, now (#204). Another build after slice 4 is still open.
14. **Where #190 goes (Nevada).** M7, as this plan has it.
15. **A ratio-locked crop on a cloud-only photo (raised by Claude Code while folding in the answers).** Claude Code's crop rule is confirmed. A target whose dimensions aren't known isn't downloaded. If the pasted group holds a ratio-locked crop or a straighten angle, that photo keeps its whole crop and rotation group as it was, not just its crop, because straightening without the shape could uncover corners under the old crop. This is wider than the suggestion. The other groups paste as usual, it's one undo action (BRW-11), and presets follow the same rule.
