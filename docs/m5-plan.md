# M5 work plan: Reuse and share

Status: draft for review · Last updated: 2026-11-03

The working plan for milestone M5 from the [milestone plan](milestone-plan.md): getting edited photos out of Seura, and reusing edits on other photos. It breaks the milestone into slices, one pull request each (or a few, lettered), in order. Drafted by Claude Code from the milestone plan, the [functional spec](functional-spec.md), the [user flows](ux/user-flows.md) and [wireframes](ux/wireframes.md), the [visual design](visual-design.md), the [technical design](technical-design.md), and the [edit format](https://github.com/seura-photos/edit-format). The questions it couldn't settle are at the end: most for a design session, and two that Nevada has answered.

**Scope** (milestone plan, M5): copy and paste of edits, and presets (PRE-1 to PRE-5); export (EXP-1 to EXP-6), including sRGB conversion and metadata handling.

**Done when** (milestone plan): Nevada exports real photos for sharing, and the exports look right in other apps on both platforms.

**Also carried in:**
- decoding wide-gamut colors unclipped, and fuller color descriptions (#188), since export is where they would show;
- the one-time message about a cloud service's deletion notice (#200; FIL-1);
- a change made in the last second before quitting isn't saved (#174);
- Node 26 (#156), from 2026-11-11;
- testing Delete on the Mac with Google Drive and Dropbox (#162), which needs Nevada's hands;
- the technical design's sentence that an abandoned read stops a download on macOS (section 4), which #202 corrected in the functional spec but not there.

**Not in M5:** Review and its "Export photos rated 4★ or more" (M6); the Settings screen and its default export options (SET-2), so the dialog remembers its own choices until then; next-photo speed (#190), within its target and left for M7's performance audit, as Nevada decided in M4's wrap-up; HDR display (#60). Hover and pressed states for buttons (#184) wait on question 12.

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
- **Sizes:** the original, or the long edge limited to 2048, 1600, 1024 or a custom value, applied to the photo as cropped. A photo smaller than the limit keeps its size (question 7). Resizing happens once, after the render.
- **Color** (EXP-5): the wgpu pipeline's existing sRGB output, checked on an iPhone Display P3 photo and on a wide-gamut generated image, in other apps.
- **Names** (EXP-3): `write_new` is already there and never replaces a file. Tests cover a new extension, taken names, and a name taken meanwhile.
- **The encoder:** the comparison of `jpeg-encoder` with `mozjpeg` the technical design asks for, at quality 90, on file size and time. `mozjpeg` is C with extra build tools, so it needs a clear win to change; the result goes in `docs/benchmarks/`.
- **Done when:** the reference-image tests pass for every format and size, on both machines, and a 48-megapixel export fits in memory.

### 3. Metadata (EXP-4)

- **Kept:** the original's EXIF, read from JPEG, HEIC or PNG, carried into the new file with `img-parts`, as the technical design says.
- **Corrected, since the pixels changed:** orientation reset to 1; pixel dimensions set to the exported size; the color space tag set to sRGB, since iPhone photos say "uncalibrated".
- **Removed:** the thumbnail embedded in EXIF. It would show the photo unedited and uncropped in some viewers, and it can hold a picture the user cropped out.
- **Remove location data:** every GPS field. Only EXIF is copied and XMP is written new, so location can't arrive through the original's XMP. Tested by reading the result with a tool Seura doesn't use.
- **Include tags and rating as keywords:** `dc:subject` and `xmp:Rating` in XMP (questions 8 and 9).
- **Done when:** date, camera and lens appear in Photos, File Explorer, Preview and Finder for JPEG and PNG exports from JPEG and HEIC originals; location is gone when asked; keywords show in at least one other photo app on each platform.

### 4. The Export dialog and batches (EXP-1, EXP-6, LIB-9)

- **The dialog** from the wireframe: Save to (path and Choose…); Format beside Size; Details; the sentence about originals and names; Cancel and Export. Export on the toolbars of the Library, Viewer and Editor, and Ctrl+E (Cmd+E). The Editor saves anything waiting first.
- **Remembered in `settings.json`:** the last folder and the dialog's choices. If the last folder is gone, Save to is empty and Export waits for one. M1's development-only export goes.
- **Progress with Cancel,** in the batch's message like Enhance's, and off the main thread. One photo at a time, so a 48-megapixel batch holds one photo in memory. Cancel stops between photos and stops waiting for one in progress; files already written stay, and a partly written temporary file is removed (rule 5).
- **Cloud-only photos** (LIB-9, EXP-6): shown as downloading with their size while the export waits for each, with Stop's behavior already settled in #202: the download may still finish (question 6).
- **The summary:** how many were exported, which couldn't be and why, and Show in Explorer or Finder.
- **Problems:** a full drive or a folder that can't be written says so and stops; one unreadable photo is reported and the others go on.
- **Done when:** Nevada exports a mixed selection, some in the cloud, to a local folder on both machines, cancels one, and finds the names, summary and originals as they should be.

### 5. Copy and paste edits (PRE-1, PRE-2)

- **Copy edits:** Shift+Ctrl+C (Shift+Cmd+C), in the Library, the Viewer and the Editor's toolbar. It copies from the focused photo (question 2), into memory in the core: not the system clipboard, and gone when the app quits.
- **Paste edits:** Shift+Ctrl+V (Shift+Cmd+V), then the small dialog, with adjustments and effects on, crop and rotation off, and red-eye not offered (question 1). Nothing to paste, or nothing in a group, is shown rather than failing silently.
- **Replacing, by stage** (PRE-2; edit format, "Presets"): the chosen stages replace the target's and the others stay. It goes through the one write path, so each photo's history records it, and read-only folders and unreadable edit files are reported as for any change.
- **Undo:** a paste onto a selection is one action to undo in the Library (BRW-11); in the Editor it's a step in the photo's own history (EDT-9). Pasting nothing into a photo with no edits leaves no edit file; pasting that removes the last edit follows FIL-1.
- **Other apps' operations** (FIL-5): a target's are kept, and a copied photo's aren't copied (question 3).
- **Done when:** Nevada copies edits from one photo to a selection of various shapes, some in the cloud, on both machines, and one undo reverses it.

### 6. Presets as files (PRE-3, PRE-5)

- **Save as preset** in the Editor: a name and the same choices as pasting, saved as `<name>.seura-preset.json` in `Seura Photos/Presets` (edit format, `kind: "preset"`), atomically. Names are cleaned up for the file system on both platforms, and a name in use asks first; nothing is overwritten silently.
- **The list:** the Presets folder is watched, so files copied in appear without restarting. A file that can't be read is listed with a note and never rewritten (rule 4).
- **Rename and delete** in the app. Delete goes to the Trash or Recycle Bin (rule 5), after a confirmation.
- **Done when:** a preset saved on one machine and copied to the other appears and validates with the format's `check.py`.

### 7. Applying presets (PRE-4, INT-3)

- **The Editor's Presets section,** last before Revert, per the wireframe. Hovering or focusing a preset previews it on the photo, through the same preview renderer, writing nothing; leaving restores the photo; clicking applies it as one step in the photo's history. The preview has to stay smooth, so PRF-6's measurement gets a preset case.
- **In the Library:** the selection's Presets menu (question 4), one action to undo.
- **Stages:** a preset replaces the stages it holds and leaves the others (edit format), including a geometry stage if the file has one.
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

Questions 1 to 12 are for a design session, and each has a suggestion, which the plan assumes until answered. Questions 13 and 14 were Nevada's, and are answered.

1. **Paste's groups.** PRE-1 names "adjustments and effects" and "crop and rotation". The format's stages are tone, effects, geometry and retouch. Suggestion: three checkboxes, Adjustments (tone), Effects, and Crop and rotation (geometry, including straighten), with red-eye (retouch) never offered. Is crop and rotation one choice or two?
2. **What Copy copies from.** With several photos selected, "the current photo" isn't obvious. Suggestion: the focused photo, and the message names it ("Copied the edits of IMG_1234."). A copy of a photo with no edits: suggest it's refused with a message, rather than replacing what was copied earlier.
3. **Other apps' operations.** The format's stage rule would replace a target's non-core operations in a pasted stage, which conflicts in spirit with keeping what Seura doesn't understand (FIL-5). Suggestion: paste replaces only the core operations in a stage, keeps the target's others, and doesn't copy the source's. Does the format need a sentence on this?
4. **The Library's Presets menu.** User flows give it, but the wireframe's toolbar has no place for it. Suggestion: a Presets submenu in the photo's right-click menu and in the ⋯ menu, for the selection.
5. **Pasting and presets on cloud-only photos.** LIB-9 lists editing among what downloads a photo, while BRW-10 says rotating needs no pixels. Pasting and presets need none either. Suggestion: no download, as with rotate, and LIB-9 is worded to say so.
6. **Exporting many cloud-only photos.** Enhance asks once for a selection, with count and size, because unasked bulk downloads are what LIB-9 rules out. Export is also an action on a selection. Suggestion: the dialog says it, above the buttons ("3 of these photos are in the cloud (24 MB) and will be downloaded."), with no further question, since the dialog is already asking.
7. **Sizes.** Suggestion: a limit never enlarges; a photo smaller than it keeps its size. A custom value is a whole number of pixels from 16 upwards, remembered with the other choices.
8. **Slashes in keywords.** A tag is one string, and slashes mark levels for display (TAG-1). As a keyword, `family/kids` could stay as written, or be split for apps that understand levels. Suggestion: `dc:subject` as written, plus Lightroom's hierarchical field (`lr:hierarchicalSubject`, with `|`) if it's cheap, since that's what other apps read for levels.
9. **XMP's library.** The technical design names `xmp_toolkit`, which wraps Adobe's C++ toolkit, so it adds a C++ build on both platforms. Export only writes a new, small packet with a few fields. Suggestion: write that packet in Seura's own code, with tests that other apps read it, and keep `xmp_toolkit` out. That's a change to the technical design, so it's asked rather than done.
10. **Show the result of a one-photo export.** EXP-6 describes a summary for batches. Suggestion: one photo gets the message, with Show in Explorer or Finder, and no separate summary.
11. **An export into a library folder.** It's a new file there, so the watcher adds it to the library as a photo. Suggestion: that's correct, and nothing special is done, but the default for a first export is the Pictures folder rather than the photo's own.
12. **Hover and pressed states (#184).** Export and the new dialogs add buttons. Are the states wanted in M5, and are the tokens designed?
13. **An interim build (Nevada).** The cloud badge, Enhance's downloads and the thumbnail fixes are on `main` but in no release, and export is several slices away. Nevada's answer (2026-11-03): yes, as version 0.3.1, now. Another build after slice 4 is still open.
14. **Where #190 goes (Nevada).** Nevada's answer (2026-11-03): M7, as this plan has it.
