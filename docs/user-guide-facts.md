# User guide fact sheet

Date: 2026-11-24 · Written by Claude Code for M7 slice 12 (#281)

This is the raw material for the Seura Photos user guide, which a design session will write as a page on the website. It says how the app behaves, as plain facts, each with its source. It describes the code on `main` as of 2026-11-24 (commit 488e1eb). That is build 0.5.0 (published 2026-11-10) plus three changes merged since, which will be in the next build but not in 0.5: a fix for quitting on the Mac (#217), filling in a photo's size later (#264), and third-party notices (#276). Where 0.5 differs, the fact says so. The fix for library folders saved through a symbolic link (#255) is in pull request #288, not yet merged, so it isn't counted as on `main`.

Sources in brackets: requirement IDs are from `docs/functional-spec.md`; "flows" is `docs/ux/user-flows.md`; "decision NNNN" is `docs/decisions/`; "journal" entries are `docs/journal/` by date and name; "code:" names a file; "#N" is a GitHub issue or pull request. UI text is quoted from `ui/src/messages/en.ftl` (the interface) and `src-tauri/messages/en.ftl` (the Mac's menus).

Wording and structure are the design session's. Things not built yet are collected in "Not built yet", and disagreements between sources in "Open questions".

## What Seura is, in one paragraph

- Seura Photos is a desktop app for Windows and macOS that shows, fixes, tags and sorts the photos already on the computer, in the folders where they are. It doesn't import, copy or move photos. [LIB-1; product brief; decision 0017 "Importing"]
- It shows JPEG, HEIC/HEIF and PNG files. Other files are ignored without a message. For an iPhone Live Photo, the still is shown and its paired video ignored. When the same photo exists as HEIC and JPEG (for example from iCloud for Windows), both are shown, as separate photos with their own edits and tags. [LIB-8; code: crates/seura-library/src/formats.rs]
- There are four views in one window: Library (the grid), Viewer (one photo), Editor (one photo with tools) and Review (a guided pass through a batch). Esc goes back one step. The layout of each view is fixed: panels can't be moved, hidden or rearranged; only the sidebar's width can be dragged (180 to 400 px). [flows, "Design approach"; INT-2; code: crates/seura-library/src/settings.rs]
- English only. Dates, times, numbers and file sizes follow the operating system's region settings. [ACC-5; ACC-6]
- No telemetry. In 0.5 Seura makes no network requests of its own; downloads of cloud photos are done by the sync service when Seura reads a file. [CLAUDE.md rule 6; tauri.conf.json's content security policy allows only the app's own pages; LIB-9]

## Installing

### System requirements

- Windows 10 (22H2) and Windows 11, x64. Windows on ARM isn't supported. [technical design §12]
- macOS 13 Ventura and later. The app sets 13.0 as its minimum. [technical design §12; code: src-tauri/tauri.conf.json `minimumSystemVersion`]
- On macOS 13 to 15, the Editor's live preview uses WebGL2 instead of WebGPU, which the web view has only from macOS Tahoe 26. The result is meant to look the same. [technical design §5 "Fallback", §12]
- If the beta finds no testers on macOS 13 to 15, the minimum may be raised to Tahoe 26 before 1.0, in a new decision record. [technical design §12; m7-plan slice 13]
- The 0.5 Mac download is `Seura.Photos_0.5.0_aarch64.dmg`, an Apple Silicon build. There is no Intel build in the release. [gh release view v0.5.0; .github/workflows/release.yml] See open question 2.
- Minimum window size is 1280 × 720 logical pixels. [DSP-2; code: tauri.conf.json `minWidth`, `minHeight`]

### Windows

- The download is `Seura.Photos_<version>_x64-setup.exe`, an NSIS installer. [gh release view v0.5.0; tauri.conf.json `targets`]
- It installs for the current user only, with no administrator rights, in `%LOCALAPPDATA%\Seura Photos`. This was confirmed as the plan for 1.0 too (#70). [decision 0013 "What is still written"; m7-plan question 2, Nevada's answer 2026-11-24; checked on Nevada's PC, where Windows records 0.5.0's install location as `C:\Users\<name>\AppData\Local\Seura Photos`]
- 0.5 isn't signed. SmartScreen may warn; the release notes' steps are: More info → Run anyway. [0.5.0 release notes; .github/workflows/release.yml]
- Signing is planned for the beta (0.6) through SignPath Foundation's program for open-source projects. Windows will then name SignPath Foundation as the publisher, not Nevada. Planned (M7), not in 0.5. [m7-plan slice 1 and question 1; journal 2026-11-12-m7-plan]

### macOS

- The download is a disk image. Open it, drag Seura Photos to Applications, and open it. [0.5.0 release notes]
- 0.5 is only ad-hoc signed. The first time, macOS blocks it "because it isn't from an identified developer"; then open System Settings → Privacy & Security and choose Open Anyway. [0.5.0 release notes; decision 0016; tauri.conf.json `signingIdentity: "-"`]
- Developer ID signing and notarization are planned for the beta, so it opens without Open Anyway. Planned (M7), not in 0.5. [m7-plan slice 1]

### Updates

- There are no automatic updates in 0.5. New builds are downloaded from the website or GitHub by hand and installed over the old one. [0.5.0 release notes: "there are no automatic updates yet"]
- The Welcome screen already asks "Check for updates automatically" (on by default) and saves the answer, but nothing uses it yet. [code: crates/seura-library/src/settings.rs, `check_for_updates` "asked on first run, used from M7"; no update command in src-tauri/src/commands.rs]

## HEIC on Windows

- On Windows, Seura decodes HEIC with Windows' own decoder, which needs two free or inexpensive extensions from the Microsoft Store: HEIF Image Extensions and HEVC Video Extensions. Seura doesn't bundle a decoder of its own. [decision 0014]
- Many PCs already have them. [decision 0014 "Context"]
- Without them, a HEIC photo's tile says "Needs Microsoft's HEIF and HEVC extensions" instead of the photo. Seura checks which one is missing, so a damaged file isn't blamed on a missing extension. [decision 0014 item 3; en.ftl `grid-needs-heif-extensions`; code: crates/seura-image/src/heif/windows.rs `missing_extension`]
- Opening such a photo shows "This photo can't be shown:" with the reason, which names the missing extension, for example "HEIC photos can't be decoded on this computer: Microsoft's HEIF Image Extensions aren't installed". [en.ftl `viewer-failed`; code: crates/seura-image/src/decode.rs, heif/windows.rs] (Assembled from the code; not seen on screen in the sources.)
- Exporting such a photo fails for that photo with "This computer needs Microsoft's HEIF and HEVC extensions to read it." [en.ftl `export-problem-needsCodec`]
- On a Mac, HEIC needs nothing extra: macOS's own decoder (ImageIO) is used. [decision 0014 item 1]
- HEIC photos are uncommon on Windows: iPhone photos usually arrive as JPEG through OneDrive's camera upload, iCloud for Windows, or a USB transfer. They appear through settings that keep originals, Google Photos downloads, and libraries shared with a Mac. [decision 0014 "Context"]
- After installing the extensions, how a photo that already showed the message gets its thumbnail isn't recorded in the sources. Leave out, or check before writing.

## First run (Welcome)

- The first launch shows a Welcome screen: "Welcome to Seura Photos", "Browse, fix, and organize the photos already on your computer.", and "Your originals are never changed. Edits are saved in small files next to each photo, so you can undo them at any time." [SET-1; en.ftl `welcome-*`]
- It names Seura's own folder: "Seura keeps its settings and thumbnails in the Seura Photos folder in your home folder: <path>". [en.ftl `welcome-seura-folder`]
- "Which folders hold your photos?" lists the Pictures folder, ticked, if there is one, with "Add another folder…". [code: ui/src/Welcome.tsx, src-tauri/src/welcome.rs]
- If the Seura Photos folder is synced to a cloud service, it says so ("Thumbnails take up space and would be uploaded too.") and offers "Store thumbnails somewhere else…". [SET-1; en.ftl `welcome-cache-synced`, `welcome-cache-elsewhere`]
- "Check for updates automatically" is ticked by default (unused in 0.5, above). [en.ftl `welcome-check-updates`; Welcome.tsx]
- Get started opens the Library, and thumbnails fill in as the scan runs. No review is offered after the first scan, since every photo is new then. [flows, flow 1; REV-1; journal 2026-11-09-new-photos-build]

## Library folders, scanning and watching

- A library folder includes all its subfolders. Add folder uses the system's folder dialog (on the Mac also File → Add Folder…). Adding a folder already inside the library does nothing and says so; adding a parent of a library folder absorbs it. [LIB-1; en.ftl `library-already-included`, `library-absorbed`; src-tauri/messages/en.ftl `menu-add-folder`]
- The sidebar's FOLDERS tree shows only folders that hold photos, directly or below, each with its photo count. The grid shows only the photos directly in the selected folder, not its subfolders. [LIB-2; BRW-1]
- The first scan shows thumbnails as they're made, with "Making thumbnails · N of M" in the status bar, and the app stays usable meanwhile. [LIB-3; en.ftl `status-thumbnails`]
- While Seura runs, new, changed, deleted and renamed photos are picked up within about 5 seconds; changes made while it was closed are found at the next launch. [LIB-4]
- Hidden files and folders, and system folders such as the Recycle Bin and Trash, are skipped. A name starting with a dot counts as hidden on both systems. [LIB-7; code: crates/seura-library/src/platform.rs]
- A folder on a disconnected drive stays in the library, shown as "Not connected", and comes back when the drive does. [LIB-6; en.ftl `library-unavailable`]
- "Remove from library…" only stops showing the folder: "Its photos stay where they are, and so do their edits. Seura just stops showing them and clears their thumbnails." [LIB-5; en.ftl `library-remove-body`]
- In a read-only folder, photos can be viewed but not tagged, rated or edited, and the Info panel says "This folder is read-only, so tags and ratings can't be saved here." [TAG-7; en.ftl `read-only-folder`, `editor-save-read-only`]
- A renamed photo keeps its edits when it's renamed outside Seura, if another photo in the same folder has the same fingerprint and no edit file of its own. [FIL-6]
- On the Mac, a library folder saved through a symbolic link (such as a path under `/var`) misses live changes until the next launch (#255). The fix is in pull request #288, not yet merged. Windows isn't affected. [#255; journal 2026-11-24-linked-folders]

## "Seura never changes your photos": what that means

- Seura never modifies, moves, renames or copies an original photo. The one exception is Delete, on an explicit, confirmed request, which moves photos to the Recycle Bin or Trash. [CLAUDE.md rule 1; BRW-9]
- Every edit, tag and rating is a setting stored in a small file beside the photo, so any of it can be changed or removed later. [Editing intro; decision 0002; decision 0003]
- Edits leave Seura only through Export, which writes new files. [Export intro; EXP-1]
- The tests enforce this, including that no file is deleted directly except in the few cases listed under "Deleting". [CLAUDE.md rules 1 to 5; journal 2026-10-27-stars-and-drives]

## What Seura writes, and where

### Beside each photo: the edit file

- Named after the photo with `.seura.json` added: `IMG_1234.jpg` → `IMG_1234.jpg.seura.json`. [code: crates/seura-edit/src/file.rs]
- Created the first time the photo gets an edit, a tag or a rating. Holds the photo's edits, tags, rating, a fingerprint of the photo, its size, and the last 50 changes for undo. Plain, readable JSON in an open format (seura-photos/edit-format). [FIL-1; EDT-9; code: crates/seura-edit/src/document.rs `HISTORY_LIMIT`]
- Written safely: to a temporary file in the same folder (`IMG_1234.jpg.seura.json.<number>.tmp`), then renamed over the old one, so a crash never leaves a half-written file. [FIL-2; code: crates/seura-edit/src/file.rs]
- When a photo has no edits, tags or rating left, its edit file goes to the Recycle Bin or Trash. [FIL-1]
- The first time that happens in a folder a cloud service syncs, a message explains it, once per service, for example: "Removing this photo's last rating moved its small edit file to the Recycle Bin. OneDrive may say a file was deleted; the photo itself isn't affected." OneDrive on the Mac, and Dropbox on Windows, have been seen showing their own deletion notice. [FIL-1; en.ftl `deletion-explained`; journal 2026-11-05-deletion-notice; #162 comment 2026-11-06]
- Operations Seura doesn't know, from a newer version or another app, are kept when it saves, and a notice says they aren't shown. [FIL-5; en.ftl `notice-not-shown`]
- An edit file Seura can't read is never overwritten. The photo shows unedited with a warning, and the notice offers Start over (see "Problems with files"). [FIL-4]
- If an edit file changes on disk (for example synced from another computer), Seura reloads it; if that photo is open in the Editor, it asks "Keep yours or load theirs?". [FIL-3; en.ftl `editor-changed-outside`]
- If the photo itself was changed by another program, the edits still apply and a notice says "This photo was changed by another program; edits may not line up." [FIL-7; en.ftl `notice-original-changed`]
- Edit files written before a photo's thumbnail was made could lack the photo's width and height (#264). From the next build they get it the next time the photo is changed. Nothing depends on it. [journal 2026-11-24-fill-in-size; journal 2026-11-12-m6-complete]

### Seura's own folder: `Seura Photos` in the home folder

- Windows: `C:\Users\<name>\Seura Photos`. Mac: `/Users/<name>/Seura Photos`. It's visible, not hidden, and outside Documents because Documents is often synced. [decision 0015; code: src-tauri/src/locations.rs]
- `settings.json`: the library folders, cache location, sort order, thumbnail size, sidebar width, the update-check answer, the Export dialog's last choices, and which cloud services' deletion notice has been explained. Readable JSON. A file Seura can't read is left alone and Seura runs on defaults. [SET-3; code: crates/seura-library/src/settings.rs]
- `Presets/`: one file per preset, `<name>.seura-preset.json`. Created by the first saved preset. [PRE-3; code: crates/seura-edit/src/preset.rs; journal 2026-11-05-presets-as-files]
- `marks.json`: photos marked for deletion in a review. Kept on this computer only, never in the edit file. [REV-4; decision 0015; code: crates/seura-library/src/marks.rs]
- `Seura Photos Cache/`, by default: thumbnails, the index of the library, and on Windows the web view's data (`Seura Photos Cache/WebView`). An existing `Cache/` is renamed to it at the first launch after updating; if that fails, it stays the cache. [decision 0019] About 500 MB per 10,000 photos. Always rebuildable, and safe to delete. The location can be chosen on the Welcome screen when the folder is synced; moving it later needs Settings, which isn't built. [decision 0013; decision 0015; decision 0018; en.ftl `welcome-cache-elsewhere`]
- `Unreadable edit files/`: old edit files set aside by Start over, named with the date and time and the original name, such as `2026-10-26 143012 IMG_1234.jpg.seura.json`. [FIL-4; code: crates/seura-library/src/start_over.rs]
- Clearing the cache never forgets the library, since the folder list is in `settings.json`. [SET-3]

### Export destinations

- Only folders the user picks in the Export dialog. A temporary file (`.seura-export-….tmp`) is written there first and renamed. [EXP-1; code: crates/seura-render/src/export.rs]

### What the system and web view keep on their own

- The program itself: `%LOCALAPPDATA%\Seura Photos` on Windows; on the Mac, wherever the app was put. [decision 0013]
- The web view runs in private mode on both systems and keeps no browsing data. On Windows its working folder is inside Seura's cache, not in AppData. [decision 0013]
- GPU shader caches the graphics system keeps for every program using the GPU: on Windows, Direct3D's `%LOCALAPPDATA%\D3DSCache\<hash>` (about 0.5 MB) and the driver's shared cache (such as `NVIDIA\DXCache`); on the Mac, Metal's cache in the per-user cache folder, named after Seura's identifier (`app.seuraphotos.desktop`). Small, and rebuilt if deleted. [decision 0013; #231]
- System-wide records the OS keeps about every app (Spotlight, recent-programs lists, the Mac's reopen-at-login list). [decision 0013]
- On the Mac, a privacy permission for a cloud provider's folder (System Settings → Privacy & Security → Files & Folders → Seura Photos → OneDrive) was seen granted on Nevada's Mac. Whether and when macOS asks for it isn't recorded. [#112 first comment]
- A check in CI (`scripts/app_data_audit.py`) fails if anything named for the app appears outside Seura's folders. [decision 0013 item 5]

## Browsing: the Library

- Sort by Date taken (the default, oldest first; falls back to the file's date), File name, or Date modified, ascending or descending. One order for the whole app, remembered. [BRW-2; code: settings.rs `Sort::default`; en.ftl `grid-sort-*`]
- Thumbnail size: a slider from 96 to 256 px, 160 by default, remembered; double-click it for the default. [BRW-3; en.ftl `grid-size-reset`]
- Thumbnails show photos with their edits applied, with small badges for edited photos, ratings, photos in the cloud, and marked photos (dimmed, red outline). [BRW-1; LIB-9; REV-6]
- Selection: click, Ctrl/Cmd-click, Shift-click, Select All; the number selected is always shown. From the keyboard, arrow keys move the selection; Space selects the focused photo, Ctrl/Cmd+Space adds or removes it, Shift+Space extends to it. [BRW-4; code: ui/src/Grid.tsx]
- The Info panel is always shown in the Library and Viewer: file name, date taken (or date modified, labeled, when there's none), dimensions, file size, camera, lens, exposure, location as coordinates, folder, edits, rating and tags. For several photos it shows a date range, "Mixed" for differing ratings, and "beach · 3 of 7" for tags only some have. For a cloud-only photo, camera, lens and location appear once it's on the computer. [BRW-6; TAG-8; en.ftl `info-*`, `info-cloud-only`]
- Right-click a photo (or Shift+F10 / the context-menu key) for Show in Explorer/Finder, Open in default app, Presets, and Move to Recycle Bin/Trash. Opening several in the default app asks first, since each may open its own window, and says which will download from the cloud. [BRW-8; en.ftl `photo-*`; code: ui/src/PhotoMenu.tsx]
- Folder and tag trees: Collapse all in the heading; Alt-click (Windows) or Option-click (Mac) on an arrow, or "Expand all beneath" in a folder's right-click menu, opens everything below; `*` opens the focused node and its siblings. There is no expand-all for a whole tree. [LIB-2; TAG-5]

## The Viewer

- Double-click or Enter opens a photo full size; ← → move through the folder in grid order; Esc returns to the grid at the same place, with the last photo selected. [BRW-5; flows, flow 3]
- Zoom with the wheel or pinch, + and −, or Fit and 100%; drag to pan. [BRW-5; code: ui/src/Viewer.tsx]
- Shows the photo with its edits. Comparing with the original is in the Editor. [BRW-5]
- Rating, tagging, rotate, Enhance and Edit work here by key, as in the Library. [code: ui/src/Viewer.tsx; en.ftl `viewer-keys`]
- The Viewer prepares the next and previous photos in advance, except cloud-only ones. [LIB-9 "Nothing reads ahead"; PRF-5]
- If the computer's graphics can't draw photos, it says "This computer's graphics can't show photos here." [en.ftl `viewer-no-preview`]

## Quick actions (without opening the Editor)

- Rotate left and right (`[` `]`), Enhance (A) and rating (0–5) work on the Library's selection and in the Viewer. In the Library, rotate and Enhance also appear on a thumbnail while it's hovered. [BRW-10; code: ui/src/keys.ts `quickKey`]
- Done with a key, they show a brief message ("Rated 3 photos 3 stars."); done with a click, they don't. [INT-6; en.ftl `change-done-*`]
- Rotating needs no download, so it works on cloud-only photos. Enhance on a selection that includes cloud-only photos asks first (see "Cloud-synced folders"). [BRW-10]

## The Editor

- Opened with E or Edit, from the Library, Viewer or Review. Done or Esc returns to where you came from. [EDT-1; flows, flow 4]
- Tools, top to bottom, in process order: Enhance; Crop and straighten; Rotate and flip; Light (Brightness, Contrast, Highlights, Shadows); Color (Temperature, Tint, Saturation); Effects (Black and white, Sepia, Vignette); Red-eye; Presets. Revert to original is pinned at the bottom; Compare and Done at the top. [INT-3; code: ui/src/Editor.tsx; en.ftl `editor-*`]
- Sliders update the photo live and show a signed whole number, −100 to 100, with 0 at neutral; double-click resets one. Brightness is 25 to a photographic stop (100 is +4 stops). Temperature reads Cool to Warm, Tint Green to Magenta. [EDT-5]
- Effects are one-click looks with On/Off; a strength slider appears once one is on. Vignette has Amount (Darker to Lighter) and Size. [EDT-6; INT-4; en.ftl `editor-effect-*`, `editor-vignette-*`]
- Enhance sets the seven light and color sliders from an analysis of the photo, as ordinary slider values you can adjust or undo. It leaves crop, rotation, straightening, red-eye and effects alone, and pressing it again gives the same result, not a stronger one. [EDT-12]
- Crop (C): Freeform, Original, 1:1, 4:3, 3:2, 16:9, 5:4, each landscape or portrait; rule-of-thirds overlay; straighten from −45° to +45° in 0.1° steps, the crop shrinking so no empty corners show. Enter or Done applies, Esc cancels, and leaving any other way counts as Done. Reopening shows the whole photo so the crop can be widened again. Arrow keys move the focused box or corner. [EDT-2; EDT-3; en.ftl `crop-*`; code: ui/src/CropOverlay.tsx]
- Rotate and flip: a crop follows the photo, and a fixed ratio turns with it. [EDT-4]
- Red-eye (R): click or drag over each eye; circles can be dragged or removed (Delete or Backspace on a selected one); `[` `]` change the circle size; at most 64 per photo; Enter applies, Esc cancels. Hold Space and drag to pan while red-eye is open. No automatic detection. [EDT-7; EDT-1; code: ui/src/Editor.tsx, RedEyeOverlay.tsx; en.ftl `red-eye-*`]
- Compare: hold `\` or the Compare button to see the photo without its tone, color, effects and red-eye edits, framed as cropped and turned so the two line up. [EDT-1]
- Zoom as in the Viewer (+ −, wheel, pinch, Fit, 100%). Crop always shows the whole photo. Moving to another photo returns to Fit. [EDT-1]
- The filmstrip under the photo, and ← →, move to the next photo without leaving the Editor. [flows, flow 4; code: ui/src/Editor.tsx]
- Revert to original removes all of Seura's own edits after a confirmation in place ("Remove every edit to this photo? Its tags and rating stay, and you can undo this."); edits made in other apps are kept. Undoable. [EDT-10; en.ftl `editor-revert-*`]
- The Editor has no rating or tag keys; 0–5 and T do nothing there. [code: ui/src/Editor.tsx key handler] See open question 5.

## Autosave

- There is no Save button. Changes are written to the edit file shortly after the last change (300 ms in the code; the spec allows up to 1 second). The Editor shows "Edits saved automatically". [EDT-11; journal 2026-11-03-save-on-quit; en.ftl `editor-saved-automatically`]
- Tags, ratings and quick actions are saved at once. [flows, "Nothing to save"]
- Closing the window, or Quit / Cmd+Q on the Mac, waits for a waiting change to save, up to 3 seconds. [journal 2026-11-03-save-on-quit]
- In 0.5, on the Mac, a change made in the moment before quitting from the Dock or by logging out may not be saved (#217). Fixed on `main` (#286, merged 2026-11-24), so in the next build, not 0.5. Tested with a logout-style quit event; a real logout wasn't tried. [0.5.0 release notes; journal 2026-11-24-mac-quit]

## Undo and redo

- In the Editor: the photo's own history, the last 50 changes, kept in its edit file, so undo works after closing and reopening Seura. [EDT-9; code: crates/seura-edit/src/document.rs]
- In the Library (and Viewer and Review): Ctrl+Z undoes the last action, whatever is selected now, such as a rating on 5 photos; Shift+Ctrl+Z (or Ctrl+Y on Windows) redoes. A message says what was undone ("Undid rating on 5 photos."); on the Mac the Edit menu names it first ("Undo Rating (5 photos)"). [BRW-11; en.ftl `history-*`; src-tauri/messages/en.ftl `menu-undo-action`; code: ui/src/keys.ts]
- The Library remembers the session's last 10 actions, in memory only; after a restart there's nothing to undo in the Library, though each photo's history still works in the Editor. [BRW-11; code: crates/seura-library/src/undo.rs `UNDO_LIMIT`]
- A photo changed since by the Editor or another app is left alone, and the message says how many were skipped. [BRW-11; en.ftl `history-skipped`]
- An Enhance batch that downloaded photos is one undo step. [BRW-10]
- Unmarking photos marked for deletion can be undone, which puts the marks back; undo never marks a photo that had no mark. [BRW-11; REV-6; journal 2026-11-09-undo-unmark]
- Moving photos to the Recycle Bin or Trash is not undoable; restore them from there. [BRW-11; flows, flow 9]

## Tags and ratings

- Stored in each photo's edit file, so they travel with the photo. [Tags intro]
- T opens the tag picker. Typing filters existing tags and highlights the first match; Enter adds it. "Create tag "…"" is listed when the text isn't a tag yet, and Enter chooses it only when nothing matches; choose it with the arrow keys or a click otherwise. The picker stays open for more; Esc closes it. [TAG-3; flows, flow 5; en.ftl `picker-*`]
- Tags are matched without regard to case, for every language; a tag keeps the form first typed. Spaces and slashes at the ends, and empty levels, are removed. [TAG-2]
- A slash makes levels for display: `family/kids` shows as `kids` under `family` in the TAGS tree. `family`, `kids` and `family/kids` are three separate tags. [TAG-1]
- Selecting a tag shows the photos with exactly that tag, from every folder, not the tags beneath it. A parent node no photo has exactly shows no count. [TAG-5; en.ftl `library-tag-empty`]
- × on a chip removes a tag. Tags can't be renamed or merged. A tag no photo uses disappears. [TAG-4]
- Ratings are 0 to 5 stars, by key or by clicking a star; clicking the current rating's star clears it. [TAG-6]

## Search and filters

- The search box (Ctrl+F / Cmd+F) matches file names and tags across the whole library as you type. The grid shows "Results for "…"" from every folder; Esc or choosing a folder or tag ends the search. [FND-2; FND-3; code: ui/src/keys.ts `isFind`]
- Filter: minimum rating, tags (a photo must have all of them), edited or not edited, and a date-taken range. Active filters show as chips with Clear all, the button reads "Filter · 2", and the status bar shows "3 of 214 photos". Filters stay set from folder to folder and reset when Seura restarts. [FND-1; en.ftl `filter-*`]

## Copying edits and presets

- Copy edits: Shift+Ctrl+C (Shift+Cmd+C, or Edit → Copy Edits on the Mac; also a toolbar button in the Editor). With several photos selected it copies the one last clicked or reached with the arrow keys, and the message names it. [PRE-1; journal 2026-11-05-copy-paste-edits]
- Paste edits: Shift+Ctrl+V (Shift+Cmd+V, or Edit → Paste Edits… on the Mac) opens a dialog with three groups: Adjustments (tone and color) and Effects, ticked, and Crop and rotation, not ticked. Red-eye is never pasted. [PRE-1; en.ftl `paste-*`]
- A pasted group replaces the target's settings in that group; a ticked group with nothing copied removes the target's ("None copied, so pasting removes any on these photos."). Undoable. [PRE-2; journal 2026-11-09-paste-wording]
- A ratio-locked crop keeps its ratio on a photo of another shape. [PRE-1]
- Pasting and presets never download a cloud-only photo. If the paste needs its shape (a locked crop or straightening) and its size isn't known yet, its crop and rotation are left alone and the message says so. [PRE-1; en.ftl `paste-cloud-kept`]
- Save as preset ("Save current edits as a preset…") in the Editor asks for a name and the same groups. A name in use asks to Replace. [PRE-3; en.ftl `presets-*`]
- Hovering a preset in the Editor previews it; clicking applies it as one undo step. In the Library, the photo's right-click menu has a Presets submenu for the selection, and on the Mac so does the Edit menu. [PRE-4; journal 2026-11-05-applying-presets]
- Rename and Move to Recycle Bin/Trash are in each preset's right-click menu. Preset files copied into `Seura Photos/Presets` appear in the list within a couple of seconds, so presets can be shared between computers by copying the files. [PRE-5; journal 2026-11-05-presets-as-files]

## Review

- Starts from the Review button or R in the Library: with two or more photos selected it reviews those, otherwise everything the grid shows (folder, tag or search result, with filters), in the grid's order. [REV-1; journal 2026-11-09-review-keys]
- One photo at a time, as large as fits, with "12 of 84" and a progress bar; the file name, date, rating and tags under the photo. Home and End, or a click on the progress bar, jump within the review. It always starts at the first photo. [REV-2]
- Actions, each one key: ← → previous and next; 0–5 rate; `[` `]` rotate; A Enhance; T tag; X mark to delete; E Edit. Rating doesn't move to the next photo. [REV-3; 0.5.0 release notes; code: ui/src/Review.tsx]
- Edit opens the Editor with the review's photos in the filmstrip; Done returns to the review on the photo you ended on. [REV-3]
- Marking: "Mark to delete" moves nothing. The photo shows "Marked for deletion · deleted only when you finish · X to unmark". A mark belongs to the photo, is kept in `marks.json` on this computer, and survives leaving the review and quitting. [REV-4; en.ftl `review-mark*`; journal 2026-11-09-marking]
- Esc leaves at any time; everything is already saved. With marks left, a message says "N photos are still marked for deletion. Review them again to finish." [REV-6; en.ftl `review-left-marked`]
- In the Library and Viewer, marked photos show dimmed with a red outline. X there, or Unmark in the Info panel, takes marks off, but X doesn't mark outside a review ("Marking for deletion happens in Review. To move photos to the Recycle Bin now, use Delete."). [REV-6; en.ftl `mark-outside-review`]
- Finishing: Next on the last photo opens "End of review", with how many were Rated, Edited, Tagged and Marked for deletion by this review, "Everything is saved, and nothing has been deleted yet.", a button to move the marked photos to the Recycle Bin/Trash (one confirmation, the usual Delete dialog), an Export button for photos rated at or above a chosen number of stars (4 to start), and Back to the Library. ← returns to the last photo. Marked photos from earlier reviews that are in this batch are counted and deleted with the rest. [REV-5; en.ftl `review-summary-*`; journal 2026-11-09-finishing-build]
- The Delete key and the Mac's Move to Trash do nothing during a review. [journal 2026-11-09-marking]

### The new-photos banner

- When photos arrive in library folders, the Library shows "84 new photos in Summer. Sort through them one at a time?" with Review and Not now; for several folders, "…in Summer and 2 other folders". [REV-1; en.ftl `review-new-banner`]
- New means found since the previous launch: by the launch's scan, or while Seura runs, a minute after additions settle. [REV-1; code: crates/seura-library/src/new_photos.rs `QUIET`]
- Not counted: Seura's own exports, the photos of a folder just added, renamed photos, and everything after the cache is cleared or on first run. [REV-1; journal 2026-11-09-new-photos-build]
- The banner's review goes oldest first. It stays until Review or Not now, or the next launch, after which those photos aren't new; the Review button still reaches them. [REV-1]
- Tested across machines: photos copied into OneDrive on Windows were offered by Seura on the Mac about a minute after they synced. [journal 2026-11-12-m6-complete]

## Deleting photos

- Select photos and press Delete (Windows) or Cmd+Delete (Mac), or use the right-click menu or File → Move to Trash…, then confirm. Cancel has the focus. [BRW-9; flows, flow 9; code: ui/src/keys.ts, src-tauri/src/menu.rs]
- Photos always go to the Recycle Bin or Trash, never deleted permanently, with their edit files. The confirmation says so: "Its edits, tags, and rating are in a small file beside it, which goes too. To get everything back, restore both from the Recycle Bin." [BRW-9; CLAUDE.md rule 5; en.ftl `delete-body`]
- A message confirms it: "Moved 3 photos to the Recycle Bin." Not undoable with Ctrl+Z. [flows, flow 9; en.ftl `delete-done`]
- Photos in a cloud-synced folder that are on the computer can be deleted, and the confirmation names the service: "These photos are in OneDrive. They'll be removed from OneDrive on all your devices too. OneDrive keeps deleted files in its own recycle bin for a while." [BRW-9; en.ftl `delete-synced-*`]
- Left as they are, with the reason in the confirmation (and an OK-only dialog if none can go):
  - cloud-only photos: "Delete it from its cloud service, or download it first." (Outside a review, Delete never downloads.) [BRW-9; en.ftl `delete-cloud-only`; journal 2026-11-09-review-cloud-build]
  - photos in read-only folders. [en.ftl `delete-read-only`]
  - on Windows, photos on a drive with no Recycle Bin, such as a USB stick, a memory card or a network share: "Deleting it there would be permanent, so it's left as it is. You can delete it in File Explorer." Seura never deletes a photo permanently, even after a warning. Only fixed drives count as having a Recycle Bin. [BRW-9; decision 0017; code: crates/seura-library/src/platform.rs `has_trash`]
  - on the Mac, photos in OneDrive, Google Drive or Dropbox folders (anything under `~/Library/CloudStorage`): "macOS doesn't let Seura Photos move photos to the Trash", with Show in Finder (#112). [en.ftl `delete-file-provider`; code: platform.rs `trash_refused`]
- If the system refuses a move, the message gives the system's own words ("macOS said: …"), and the photo stays where it was. [en.ftl `delete-failed`, `delete-failed-reason`]
- Seura deletes a file directly only when nothing in it can be lost: a temporary file from a save that failed; an emptied edit file on a drive with no Recycle Bin or Trash, checked again just before; an unreadable edit file after Start over, once its copy in Seura's folder matches byte for byte; and Seura's own cache when cleared (Settings, not built). [CLAUDE.md rule 5; decision 0017; decision 0018]
- On the Mac, Finder may offer Put Back for a photo but not its edit file (`.seura.json`) (#254). Move that file back too, to the same folder, to keep the photo's edits, tags and rating. The cause is unknown; it's a known issue for 1.0. [#254; 0.5.0 release notes; m7-plan question 11]

## Export

- Select photos and press Ctrl+E (Cmd+E, or File → Export… on the Mac), or Export on the Library, Viewer or Editor toolbar. The review summary has its own Export button. [EXP-1; journal 2026-11-04-export-dialog]
- The dialog: Save to (a folder picked with Choose…; remembered; the Pictures folder the first time), Format (JPEG with Quality, 90 by default, or PNG), Size, and Details: "Remove location data" (off by default) and "Include tags and rating as keywords" (on by default). Choices are remembered. [EXP-1; EXP-2; code: crates/seura-library/src/settings.rs `ExportSettings::default`; en.ftl `export-*`]
- Size: Original size (the default), "Long edge up to 2048 / 1600 / 1024 px", or "Custom, up to" a whole number from 16. These are limits: "Photos smaller than the limit keep their size and aren't enlarged." [EXP-2; en.ftl `export-size-*`; journal 2026-11-04-export-dialog]
- HEIC photos export as JPEG or PNG. [EXP-2]
- Edits are applied to the exported copies; originals aren't changed. [en.ftl `export-originals`]
- Export never replaces a file. A name already taken gets a number: `IMG_4417 (1).jpg`, then `(2)`, and so on, even for a file that appears during the export. [EXP-3; en.ftl `export-originals`; code: crates/seura-render/src/export.rs `write_new`]
- So running the same batch again (for example after a dropped connection) gives the photos that already worked a second, numbered copy. The summary says so: "N names were already taken in that folder, so those files got a number. Nothing was replaced, so you may have two copies of those photos." [journal 2026-11-04-export-dialog; en.ftl `export-renamed`]
- Metadata: date, camera, lens and other standard EXIF tags are kept. Orientation is applied to the pixels. Maker notes, the embedded thumbnail and previews are never carried over, so nothing cropped out or edited away survives. [EXP-4; journal 2026-11-04-export-metadata]
- Keywords: a tag with levels, such as `family/kids`, is written as `family` and `kids` in the standard keywords, and as `family|kids` in Lightroom's hierarchical keywords. digiKam read them nested, without duplicates. [EXP-4; journal 2026-11-04-export-metadata]
- Colors are converted to sRGB with its profile embedded, so the file looks right everywhere. [EXP-5]
- PNG exports carry the date, camera and lens in XMP as well as EXIF, because File Explorer doesn't read a PNG's EXIF. The location is only in the EXIF, so Explorer shows no location for a PNG that still holds one; it hasn't been removed. On the Mac, Preview and Get Info read it. [journal 2026-11-04-export-metadata]
- A photo with very many tags (well over a thousand) can be too large for a JPEG's metadata; that photo fails, and it fits as a PNG. [journal 2026-11-04-export-metadata; en.ftl `export-problem-failed`]
- Progress shows "Exporting 3 of 20…" with Cancel; a summary at the end with Show in Explorer/Finder. A single photo ends with a message instead. One export runs at a time. [EXP-6; journal 2026-11-04-export-dialog; en.ftl `export-busy`]
- Photos on the computer go first. A photo whose edit file can't be read isn't exported. A full drive or a folder that can't be written stops the batch and says how many were left. [journal 2026-11-04-export-dialog]
- Exporting into a library folder is allowed; the files appear there as photos, and don't trigger the new-photos banner. [EXP-1; REV-1]
- Exported files are ordinary files, readable by other accounts as usual (fixed in M5). [journal 2026-11-04-export-dialog]

## Colors and PNG color descriptions

- Orientation in a photo's metadata is honored, and embedded color profiles are applied, so iPhone photos show their true colors. Wide-gamut colors (Display P3, Adobe RGB) aren't clipped while editing; they're converted to sRGB on export. [BRW-7; EXP-5; journal 2026-11-03-decoding]
- A PNG's color is read from the first of these it has, in the PNG standard's order: a `cICP` chunk; an ICC profile (`iCCP`); an `sRGB` chunk; then gamma (`gAMA`) and chromaticities (`cHRM`). With none, it's sRGB. [code: crates/seura-image/src/decode.rs `png_color`]
- Gamma alone, or chromaticities alone, also count, with sRGB's value for the missing part. Nevada decided Seura should follow what the file says. [journal 2026-11-03-png-color-chunks; code: crates/seura-image/src/color.rs]
- How that can change a PNG's look compared with apps that ignore these chunks: a gamma of 1.0 or 1.8 used to look much too dark and now looks right (a stored 128 shows as about 188 and 146). A gamma of 2.2 alone darkens deep shadows slightly compared with sRGB (a stored 30 shows as about 24). A file labeled wrongly looks a little off, and Seura can't tell. [journal 2026-11-03-png-color-chunks]
- Chunks that make no sense (a zero gamma, primaries in a line) are ignored, and the file is taken as sRGB. [journal 2026-11-03-decoding]
- Still open (#188): 16-bit PNGs are read at 8 bits; profiles built on lookup tables still clip, which is rare for photos. [#188; journal 2026-11-03-decoding]
- On a Mac with an XDR display, iPhone HEIC photos can look brighter in Apple's Photos and Preview than in Seura; Seura shows the standard picture, not the HDR gain map. A design question, not planned for M7 (#60). [#60; m7-plan "Not in M7"]

## Cloud-synced folders

### What downloads a photo, and what doesn't

- Adding, scanning and browsing a synced folder never downloads cloud-only photos: Seura reads only the file's details, and the thumbnail comes from the system (Explorer or Quick Look) or shows "In the cloud". [LIB-9; docs/research/cloud-photo-sources.md]
- A cloud-only photo is downloaded, one at a time, only when you open it in the Viewer or Editor, Enhance it, open it in the default app, or export it, or in a review when you choose "Download as I go". [LIB-9; BRW-10; REV-3; en.ftl `photo-open-cloud`]
- Never downloading: rotating, rating, tagging, pasting edits, applying presets, marking, and Delete outside a review. Rotation and other edits show on the thumbnail anyway. [LIB-9; journal 2026-11-02-cloud-thumbnail-edits; journal 2026-11-05-copy-paste-edits]
- The Viewer's look-ahead skips cloud-only photos, so browsing never downloads a photo you haven't opened. [LIB-9]
- Before its first download, a photo sorts by its file date and the Info panel lacks camera, lens and location. [LIB-9; en.ftl `info-cloud-only`]
- Seura never downloads photos in bulk unasked and never changes whether a file is kept on the device. [LIB-9]

### What you see

- A cloud-only photo has a small cloud badge in the grid (it moves below the hover buttons while hovered). The badge goes away by itself once the photo is downloaded. [LIB-9; journal 2026-11-02-cloud-badge-slice]
- While one downloads: its thumbnail enlarged and blurred, with "Downloading 9.4 MB from the cloud…". There's no progress bar, because neither system tells an app how far a download has got. [LIB-9; en.ftl `viewer-downloading`]
- You can move on or leave at any time; that's how a download is abandoned. Seura stops waiting, but the download may still finish and leave the photo on the computer. [LIB-9; journal 2026-11-02-download-stop]
- A failed download says "This photo is in the cloud, and it couldn't be downloaded. Check your connection." with Try again. [LIB-9; en.ftl `viewer-download-failed`]
- Enhance on a selection that includes cloud-only photos asks once: "3 of these photos are in the cloud (24 MB). Download them to enhance them?" with Download and enhance, "Enhance only the 2 on this computer", and Cancel. A single cloud-only photo downloads without asking, as opening it does. While a batch downloads, "Downloading 2 of 5…" with Stop. [BRW-10; en.ftl `enhance-*`]
- Export's dialog says "3 of these photos are in the cloud (24 MB) and will be downloaded."; Export is the confirmation. [EXP-1; en.ftl `export-cloud`]

### Review's question about cloud photos

- A review whose batch includes cloud-only photos asks once at the start, with the count and size: "Download as I go", "Show thumbnails only", or Cancel. [REV-3; en.ftl `review-cloud-*`; journal 2026-11-09-review-cloud-build]
- Download as I go fetches the next three photos once the current one is showing, not the whole batch. [code: ui/src/Review.tsx `DOWNLOAD_AHEAD`; journal 2026-11-09-review-cloud-build]
- Thumbnails only: the photo shows as its thumbnail with "In the cloud (size)"; rating, tags, rotate and marking work; Enhance or Edit downloads that one photo. [REV-3; en.ftl `review-in-cloud`]
- At the end, marked cloud-only photos are downloaded one at a time after the one confirmation, so they can go to the Recycle Bin or Trash; any not downloaded stay as they are and stay marked. Stop is available. On the Mac, OneDrive, Google Drive and Dropbox photos are refused first (#112) and not downloaded for nothing. [journal 2026-11-09-review-cloud-build; en.ftl `delete-will-download`]

### Freeing the space again

- Seura doesn't free space; it leaves that to the system. Opening or enhancing a cloud photo leaves it downloaded. To make it cloud-only again, use File Explorer's Free up space, Finder's Remove Download, or, for OneDrive on Windows, Storage Sense. [functional spec decisions log "Freeing space after downloads"; journal 2026-11-10-free-space]
- A photo made cloud-only again gets its cloud badge back within seconds, while Seura runs. [journal 2026-11-02-cloud-badge-slice]

### Deleting in synced folders

- Windows: a downloaded photo in a OneDrive or Dropbox folder goes to the Recycle Bin and leaves the service on all devices; cloud-only ones are left as they are (measured on OneDrive: Windows would send them only to the service's online recycle bin, silently). [BRW-9; spec decisions log "Deleting cloud-only photos"]
- Mac: photos in OneDrive, Google Drive and Dropbox folders can't be deleted from Seura (#112). macOS refused every way of moving a synced OneDrive file to the Trash, from Seura and from other tools; a real Apple certificate didn't help. It will be retested with Developer ID signing in M7, and if it still fails this stays. [#112 comments; m7-plan slice 1]
- Google Drive on Windows: deleting a streamed photo put it in the Recycle Bin, restorable from there. [journal 2026-11-06-google-drive-streaming]

### Each service

- **OneDrive.** The service Seura was built and tested against, on both systems: cloud-only detection, badges, downloads, Stop, Review's question, the deletion notice, and offline behavior. Windows: works throughout. Mac: works, except Delete (#112); OneDrive on the Mac shows its own "deleted" notice when an edit file goes to the Trash. [#162 body; journals 2026-11-02, 2026-11-05-deletion-notice, 2026-11-09-review-cloud-*; research doc "Checked on Nevada's PC"]
- **Dropbox.** Windows: tested by Nevada with 0.4: the cloud badge, opening and downloading, and the deletion explanation naming Dropbox all work, since Dropbox uses Windows' Cloud Files system as OneDrive does. Scanning a cloud-only folder, Delete and offline aren't tested yet. Mac: not tested; Delete is refused as for OneDrive. [#162 comment 2026-11-06; journal 2026-11-06-google-drive-streaming; code: platform.rs]
- **Google Drive.** Windows: Drive for desktop "streams" files through its own drive (usually `G:`), and Seura can't tell they're in the cloud: no badge, a streamed photo opens straight away, and Seura's question before downloading can't apply. A first scan likely downloads every photo for its thumbnail (Claude Code's reading, not checked). Treating the whole Drive as cloud-only is planned in M7, not in 0.5 (#162). Mac: not tested; detection should work as for OneDrive (the dataless flag), and Delete is refused. [#162; journal 2026-11-06-google-drive-streaming; m7-plan slice 7; research doc]
- **iCloud Drive.** Expected to work on both systems by the same file flags, but not tested with Seura. On the Mac its photos are not refused the Trash (it isn't under `~/Library/CloudStorage`). [research doc matrix; code: platform.rs, synced.rs; journal 2026-10-29-mac-cloud-delete]
- **iCloud Photos.** Windows: iCloud for Windows shows iCloud Photos in File Explorer, likely detected the same way, not verified. Mac: not a folder; the photos live inside the Photos app's library, which Seura doesn't read. Not supported in v1; export from Photos, or use a folder-based service. [research doc; spec decisions log "iCloud Photos on a Mac"]
- **Google Photos.** No desktop sync at all, and Drive for desktop no longer connects to it. Photos reach a computer only by downloading from photos.google.com or Google Takeout. The practical advice is a camera backup to OneDrive or Dropbox on the phone. [research doc "Two gaps"]
- **Box** registers with Windows' Cloud Files system too, so Seura names it as a synced service; not tested. [code: src-tauri/src/synced.rs]

### What was tested where

| | Windows | Mac |
| --- | --- | --- |
| OneDrive | Tested throughout | Tested; Delete refused (#112) |
| Dropbox | Badge, opening, deletion notice tested; scan, Delete, offline not | Not tested |
| Google Drive | Opening, edit, Delete tested; not detected as cloud | Not tested |
| iCloud Drive | Not tested | Not tested |
| iCloud Photos | Not tested (iCloud for Windows) | Not supported |

[#162; journals cited above; m7-plan slice 7 will fill in the rest]

## Problems with files

- Shown in a notice bar at the top of the photo area, in the Library and Viewer, never a blocking dialog. Results of actions show as a brief message near the bottom for about 6 seconds. [INT-6; flows, flow 11]
- An unreadable edit file: "The edit file for IMG_1234.jpg can't be read, so the photo is shown unedited. Start over keeps that file aside, in Seura's folder, and begins again with no edits, tags, or rating." After Start over, "The old edit file is kept in "Unreadable edit files" in Seura's folder." with Show in folder. [FIL-4; en.ftl `start-over-*`]
- A file that has become readable again by the time Start over runs is left as it is. [en.ftl `start-over-readable`]

## Keyboard shortcuts

Ctrl on Windows, Cmd on the Mac. Single keys work except while typing in a text field. On the Mac, the shortcuts marked "menu" go through the menu bar. [flows, "Keyboard shortcuts"; code: ui/src/keys.ts, src-tauri/src/menu.rs]

| Action | Windows | Mac | Where |
| --- | --- | --- | --- |
| Open photo | Enter, double-click | Enter, double-click | Library |
| Back | Esc | Esc | Everywhere |
| Next, previous photo | → ← | → ← | Viewer, Editor, Review |
| First, last photo | Home, End | Home, End | Review |
| Rotate left, right | `[` `]` | `[` `]` | Library, Viewer, Review, Editor |
| Enhance | A | A | Library, Viewer, Review, Editor |
| Rate | 0–5 | 0–5 | Library, Viewer, Review |
| Tag | T | T | Library, Viewer, Review |
| Edit | E | E | Library, Viewer, Review |
| Start a review | R | R | Library |
| Mark or unmark for deletion | X | X | Review |
| Unmark | X | X | Library, Viewer |
| Zoom in, out | + − | + − | Viewer, Editor |
| Crop | C | C | Editor |
| Fix red-eye | R | R | Editor |
| Red-eye circle smaller, larger | `[` `]` | `[` `]` | Editor, red-eye mode |
| Pan while red-eye is open | Hold Space and drag | Hold Space and drag | Editor |
| Apply, cancel crop or red-eye | Enter, Esc | Enter, Esc | Editor |
| Compare with original | Hold `\` | Hold `\` | Editor |
| Undo | Ctrl+Z | Cmd+Z (menu) | Everywhere |
| Redo | Shift+Ctrl+Z or Ctrl+Y | Shift+Cmd+Z (menu) | Everywhere |
| Copy edits, paste edits | Shift+Ctrl+C, Shift+Ctrl+V | Shift+Cmd+C, Shift+Cmd+V (menu) | Everywhere |
| Export | Ctrl+E | Cmd+E (menu) | Everywhere |
| Search | Ctrl+F | Cmd+F | Library |
| Select all | Ctrl+A | Cmd+A (menu) | Library |
| Delete | Delete | Cmd+Delete (menu) | Library, Viewer |
| Show in Finder | — | Option+Cmd+R (menu) | Library, Viewer |
| Open in default app | — | Option+Cmd+O (menu) | Library, Viewer |
| Photo menu | Shift+F10, context-menu key | Shift+F10 | Library, Viewer |
| Select the focused photo; add or remove it; extend to it | Space; Ctrl+Space; Shift+Space | Space; Shift+Space (Cmd+Space is usually Spotlight's) | Library |
| Open a tree node and its siblings | `*` | `*` | Sidebar |
| Open a node and everything beneath | Alt-click its arrow | Option-click its arrow | Sidebar |
| Sidebar width (divider focused) | ← →, Home, End; Enter resets | same | Sidebar |
| Quit | Close the window | Cmd+Q | — |

Notes:
- Windows has no Ctrl+Q; closing the window quits. [journal 2026-11-03-save-on-quit]
- There is no Settings shortcut yet (Ctrl+, and Cmd+, are planned with Settings). [SET-2; no settings command in src-tauri/src/commands.rs]
- In the Editor, Ctrl+Z steps through the photo's own history; elsewhere it undoes the Library's last action. [flows, "Keyboard shortcuts"; code: ui/src/Editor.tsx]

## Menus and About

- Windows has no menu bar: About is under the ⋯ button at the right end of the toolbar, in every view. [ACC-4; en.ftl `more-about`]
- The Mac has the menu bar: the app menu (About, Third-Party Notices, Hide, Quit), File (Add Folder…, Show in Finder, Open in Default App, Export…, Move to Trash…, Close Window), Edit (Undo, Redo, Cut, Copy, Paste, Select All, Copy Edits, Paste Edits…, Presets) and Window. [code: src-tauri/src/menu.rs; src-tauri/messages/en.ftl]
- About shows the version, "An early build, before the beta." while below 1.0, a Copy button for the version, and links to seuraphotos.app, the project on GitHub, and the Apache License 2.0. On the Mac it's the system's About panel. [SET-5; en.ftl `about-*`]
- Third-party notices: from the next build, not 0.5. On Windows, About → Third-party notices; on the Mac, the app menu → Third-Party Notices. Either opens a bundled text file (`third-party-notices.txt`) in the system's text viewer (TextEdit on the Mac). It lists 322 Rust crates and 11 web packages, generated at build time. [#287, merged 2026-11-24; journal 2026-11-24-license-notices; code: src-tauri/src/commands.rs `open_notices`, tauri.conf.json `resources`]

## Uninstalling

### Windows

- Today: Tauri's standard NSIS uninstaller removes the program from `%LOCALAPPDATA%\Seura Photos`. It offers a checkbox to delete `%APPDATA%\app.seuraphotos.desktop` and `%LOCALAPPDATA%\app.seuraphotos.desktop`, which are normally empty or absent. [decision 0013 "Removal"]
- It doesn't remove the `Seura Photos` folder in the home folder (settings, presets, marks, cache, set-aside edit files), the edit files beside photos, or the GPU shader caches. These can be removed by hand: the `Seura Photos` folder and the cache are safe to delete; the edit files hold your edits, tags and ratings. [decision 0013; decision 0015; m7-plan slice 5 for what's planned]
- Planned (M7), not in 0.5: the uninstaller offers Clean up's choices, defaulting to leaving edit files in place, removes the web view data in the cache, and removes Seura's `D3DSCache` folder if it can be identified. [CLN-5; m7-plan slice 5]

### Mac

- Today: drag Seura Photos from Applications to the Trash. Nothing else is removed. [CLN-5; decision 0013 "Removal"]
- What remains: `~/Seura Photos` (settings, presets, marks, cache, set-aside edit files), the edit files beside photos, Metal's shader cache in the per-user cache folder, and system-wide records. [CLN-5; decision 0013]
- Planned (M7), not in 0.5: Clean up's full reset is the way to remove Seura's data before dragging the app to the Trash; the guide should say to run it first, and list what remains otherwise (the edit files, and anything in decision 0013). [CLN-5; m7-plan slice 5]

## Known issues

Current in 0.5 and still open on `main`:

- **#254, Mac:** Finder may offer Put Back for a deleted photo but not its edit file. Move the `.seura.json` file back too. Known issue for 1.0. [#254; m7-plan question 11]
- **#112, Mac:** photos in OneDrive, Google Drive and Dropbox folders can't be moved to the Trash from Seura; it leaves them, says why, and offers Show in Finder. To retest with Developer ID signing. [#112; 0.5.0 release notes]
- **#162, Google Drive on Windows:** Seura can't tell streamed photos are in the cloud, so it may download them (likely every photo during the first scan, not checked), and shows no cloud badge. Also, Dropbox and Google Drive on the Mac haven't been tested. [#162; journal 2026-11-06-google-drive-streaming]
- **#188:** 16-bit PNGs are shown at 8 bits; rare lookup-table color profiles are clipped. Unlikely to be noticed. [#188]
- **#60, Mac with an XDR display:** iPhone HEIC photos can look less bright than in Apple's Photos and Preview. A design question, not in M7. [#60]
- **Mac, network shares:** where there's no Trash, Delete reports that the photos couldn't be moved, and clearing a photo's last rating or tag fails, since the emptied edit file can't go to the Trash. Not measured. [journal 2026-10-27-stars-and-drives; code: platform.rs `has_trash` on macOS] See open question 4.
- **#255, Mac:** a library folder saved through a symbolic link misses live changes until relaunch (fix in #288, unmerged). [#255]
- **#291, won't fix (Nevada, 2026-11-24):** with #288, one case remains. A folder saved through a symbolic link that is missing when Seura starts comes back once it's there again, but later changes in it show only after Seura is restarted. Add Folder never saves a link, so this needs an older or hand-edited settings file. Probably not worth a user-facing line; if the guide mentions it, the remedy is to quit and reopen Seura. [#291; journal 2026-11-24-linked-folders]
- **#258:** once, during an automated measuring run on Windows, the Viewer never drew a photo and showed no message. Not reproduced. Probably not worth a user-facing line. [#258]
- **#190:** next photo in the Viewer is slower on Windows than the project's own aim, though within the target. Not user-facing. [#190]
- **Mac, first photo in a review:** once, in a debug build, the first cloud photo took 30 seconds or more to appear after its file arrived. Not seen since; probably not worth a line. [journal 2026-11-09-review-cloud-mac]

Fixed after 0.5, in the next build:

- **#217, Mac:** a change made just before quitting from the Dock or by logging out could be lost. Fixed by #286 (merged 2026-11-24). [journal 2026-11-24-mac-quit]
- **#264:** some edit files lacked the photo's width and height. Fixed by #285. Not user-visible. [journal 2026-11-24-fill-in-size]

## Not built yet (planned in M7)

None of these is in 0.5 or on `main`. Evidence: no command for them in `src-tauri/src/commands.rs` (its last commands are for the Welcome screen, About, measuring and the render check), and each is an open M7 slice issue.

- **Settings (SET-2, #273):** library folders, cache location and size (Move, Clear), Seura's folder with Open folder, export defaults, and update checks, on one page; a dialog from ⋯ → Settings… (Ctrl+,) on Windows, a window from the app menu (Cmd+,) on the Mac. Until then, library folders are added from the Library (and removed from a folder's right-click menu), and the cache can be placed elsewhere only on the Welcome screen. [m7-plan slice 3; en.ftl has no settings strings]
- **Clean up (CLN-1 to CLN-4, #274):** removing edit files, the cache, or everything (a full reset that restarts at Welcome), for a folder, the whole library, or orphaned edit files, with a preview; edit files, settings and presets to the Trash, the cache deleted directly. [m7-plan slice 4; decision 0018]
- **Updates (SET-4, #272):** a check at launch at most once a day and Check now, offered as a quiet notice; the installer downloaded to the system's temporary folder and deleted after. [m7-plan slice 2]
- **Signed installers (#271):** SignPath on Windows, Developer ID and notarization on the Mac. [m7-plan slice 1]
- **The uninstaller's choices (CLN-5, #275)** on Windows, and Clean up's full reset as the Mac's way to remove Seura's data. [m7-plan slice 5]
- **Google Drive on Windows treated as cloud-only throughout, and Google Drive and Dropbox tested on both systems (#162, #277).** [m7-plan slice 7]
- **The beta, build 0.6 (#282).** [m7-plan slice 13]
- Third-party notices are built but not in 0.5 (see "Menus and About").

## Open questions for the design session

1. **Windows install folder: settled.** Decision 0013 and the M7 plan say `%LOCALAPPDATA%\Seura Photos`, and that's where 0.5.0 is installed on Nevada's PC. Decision 0015's "Alternatives considered" says a per-user install goes under `AppData\Local\Programs`, which is out of date; a design session may want to correct that sentence.
2. **Intel Macs.** The technical design says macOS 13+ on Apple Silicon and Intel, but 0.5 ships only an `aarch64` disk image. Whether Intel Macs are supported, and whether the guide says so, is open.
3. **Google Drive on Windows.** The research doc and the technical design say Seura treats the whole Drive as cloud-only; the code doesn't (it checks only Windows' Cloud Files attributes), and the M7 plan has it as work to do. The website's home page says cloud photos aren't downloaded, which may not hold there (journal 2026-11-10-build-0.5).
4. **Mac drives with no Trash.** BRW-9 and the Mac wording of `delete-no-trash` say such photos are left with a "can delete it in Finder" message, but on the Mac `has_trash` always answers yes, so the system's refusal shows as a failure instead, and clearing a last rating there fails (journal 2026-10-27). Not measured on a real Mac network share. Filed as #289.
5. **Rating and tag keys in the Editor.** The user flows say single-letter keys work in every view and list 0–5 and T without exceptions; the Editor handles neither. Either the flows or the Editor needs to change, or the guide notes it.
6. **SET-5's wording for the Mac.** SET-5 says the About panel's credits open the notices; what was built, by Nevada's choice, is a Third-Party Notices item in the app menu. The spec is left for a design session.
7. **Windows redo key.** The code also accepts Ctrl+Y for redo on Windows; the user flows' table lists only Shift+Ctrl+Z. Whether the guide mentions Ctrl+Y.
8. **The spec says "within 1 second" for autosave (EDT-11)**; the code waits 300 ms. Not a conflict, but the guide shouldn't promise a number the spec doesn't.
9. **The Viewer's key hint** (`viewer-keys`: "← → next and previous · + − zoom · T tag · 0–5 rate · E edit · Esc back…") leaves out `[` `]`, A, X and Delete, which work there. Minor; the guide's table is the fuller source.
10. **The update installer and rule 5.** Deleting the downloaded update installer isn't yet in rule 5's list of direct deletions (m7-plan, note after the questions). Affects what the guide says Seura writes once updates exist; not current behavior.
11. **iCloud Drive and iCloud for Windows.** Detection is expected, not verified. Whether the guide lists them as supported or as untested.
12. **HEIC after installing the extensions.** The sources don't say whether a tile that showed the "Needs Microsoft's HEIF and HEVC extensions" message refreshes by itself after installing them, or needs a restart. Worth a check before the guide says what to do next.
