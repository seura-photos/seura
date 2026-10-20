# M2 work plan: Browse

Status: draft for review · Last updated: 2026-10-20

The working plan for milestone M2 from the [milestone plan](milestone-plan.md): Seura replaces Nevada's photo viewer. It breaks the milestone into slices, one pull request each, in order. Drafted by Claude Code from the milestone plan, the [functional spec](functional-spec.md), the [user flows](ux/user-flows.md) and [wireframes](ux/wireframes.md), the [visual design](visual-design.md), and the [technical design](technical-design.md). Where those leave a question about behavior or design, the slice names it and it goes to a design session rather than being decided here.

**Scope** (milestone plan, M2): LIB-1 to LIB-8 with the first-run Welcome (SET-1) and settings storage (SET-3); BRW-1 to BRW-8; DSP-1 to DSP-4; keyboard access for everything built (ACC-1); the visual design applied throughout; performance for scanning and browsing measured.

**Read-only.** M2 writes no edit files beyond M1's test path, the Editor's exposure slider. Ratings, tags, and edit badges read from existing edit files are shown, but nothing in M2 changes them; that's M3.

**Done when:** Nevada points Seura at his real photo library and browses it daily on both machines.

## How to work

As in M1 ([M1 work plan](m1-plan.md), "How to work"): one slice per branch and pull request, branched from `main` and named `m2/<slice>`; Nevada merges. Use current, stable dependencies at least two weeks old, checked against decision 0012. Ask Nevada for anything that needs his machines, his photos, or his eyes. Keep journal notes as you go.

Two things are new in M2:

- **Styling.** Every screen built in M2 is styled from `design/tokens.json` (visual design, "Where to see it"). Slice 1 turns the tokens into CSS variables generated at build time, so the app and the website read the same file. Icons are Lucide, as inline SVG.
- **Screenshots for review.** Each slice that changes a screen ends with screenshots in light and dark mode, attached to the pull request, since the styled mockups live on the design canvas and the result should be compared with them.

## Slices

### 1. Foundations: shell, styling, settings

- **Settings file (SET-3):** a readable JSON file in `Documents/Seura Photos`, written atomically like everything else. It holds the library folders, which today live only in the index snapshot in the cache, so clearing the cache would lose the library. It also holds the cache location, the thumbnail size (BRW-3), the sort order (BRW-2), and the update-check choice (SET-4, stored now, used in M7).
- **Design tokens** as CSS variables for light and dark, generated from `design/tokens.json` and checked in CI like the bindings. System font sizes from the visual design.
- **Main window layout** from the wireframes: toolbar (52 px), sidebar (232 px), main area, Info panel (280 px), status bar (28 px), with the panel widths growing at 1920 wide (DSP-3). It must work at 1280×720 (DSP-2), which is already the minimum window size.
- **State store** (Zustand, technical design §8) for the view, selection, and settings, replacing the component state M1 used.
- **Native menus (ACC-4):** File (Add folder, Remove folder), View (sort, thumbnail size), and Help, with each platform's shortcut conventions (ACC-1).
- **Done when:** the empty app shows the styled shell in light and dark mode on both machines, and its library survives deleting the cache folder.

### 2. Welcome (SET-1, SET-4)

- The first-run card from the wireframes: what the app does; one sentence that originals are never changed and edits live beside each photo; "Which folders hold your photos?" with Pictures pre-checked and Add another folder; "Check for updates automatically", checked; Get started (44 px).
- If the default cache location, `Documents/Seura Photos/Cache`, is inside a cloud-synced folder (`docs/research/cloud-photo-sources.md`), a note offers another location (see the questions below).
- Get started opens the Library on the first folder while the scan runs (LIB-3). Target: under a minute from launch to photos (user flows, flow 1).
- The Welcome screen has no styled mockup. It's styled from the tokens like the rest of M2, and its screenshots go to Nevada for review; a separate design pass seems unnecessary for one card, unless he wants one.
- **Done when:** a first run on each machine, with the settings file absent, ends in a scanning Library.

### 3. Library sidebar and status bar (LIB-1 to LIB-3, LIB-5 to LIB-8)

- The FOLDERS tree (LIB-2): library folders and every subfolder with photos, nested, each with its count. The TAGS tree joins in M3.
- Remove a folder (LIB-5): from the library only; its cache entries dropped, its edit files left alone. Confirmed before removing.
- Unavailable folders (LIB-6): dimmed, "Drive not connected", and back when the drive returns. M1 already keeps their photos in the index.
- Status bar (LIB-3): photo count, number selected, and scan progress as processed out of found.
- Check LIB-7 (hidden and system folders) and LIB-8 (Live Photos' videos ignored) against real folders on both platforms; M1 implemented both.
- **Done when:** Nevada's library shows as a tree on both machines, and a disconnected drive dims and returns.

### 4. Watching for changes (LIB-4)

- File system watching (the `notify` crate, or each platform's own API) for new, changed, deleted, and renamed photos and edit files, reflected within 5 seconds. Changes made while the app was closed are found at launch, as M1's rescan already does.
- Watching never reads a cloud-only photo (LIB-9), and a burst of changes, such as a sync client catching up, is batched rather than scanned one file at a time.
- **Done when:** adding, renaming, and deleting photos in Explorer and Finder shows in Seura within 5 seconds, covered by integration tests with temporary folders.

### 5. The grid (BRW-1 to BRW-4, ACC-1, ACC-2)

- Styled grid from the wireframes: square cells with photos letterboxed, the selection outlined in the accent color; badges for edited (bottom left) and rating (bottom right), read from existing edit files.
- **Edited thumbnails (BRW-1):** a photo with edits shows them in its thumbnail, rendered by the wgpu pipeline (technical design §4). In M2 that's exposure, the only operation built so far; the rest arrive with M4.
- Sorting (BRW-2): date taken (default, falling back to file date), file name, date modified, ascending or descending. Date taken for HEIC needs its EXIF item read, carried from M1.
- Thumbnail size slider (BRW-3), remembered in the settings file.
- Selection (BRW-4): click, Ctrl/Cmd-click, Shift-click, Select All; the count in the status bar.
- Keyboard (ACC-1): the ARIA grid pattern, with arrow keys, Home and End, Page Up and Down, and Enter to open, carried from M1. Screen readers announce each thumbnail's file name, date, and rating (ACC-2).
- **Done when:** a 1,000-photo folder sorts, resizes, and selects at 60 fps scrolling (PRF-4), by mouse and by keyboard.

### 6. Thumbnails, faster and for the cloud (PRF-3, LIB-9)

Carried from M1:

- Thumbnail speed: smaller decodes (JPEG's scaled decoding, embedded previews where they're good enough), measured in release builds against PRF-3, at least 30 photos per second.
- macOS: thumbnails for cloud-only photos from Quick Look, which keeps the sync service's preview, instead of the placeholder M1 shows. Checked with `cloud_check`, as on Windows.
- **Done when:** a first scan of Nevada's library meets PRF-3 on both machines, and his cloud camera roll shows thumbnails on the Mac without downloading anything.

### 7. Viewer (BRW-5, LIB-9)

- One photo full size on the dark surround, from the wireframes: back to the folder, file name, "37 of 214", zoom (−, Fit, 100%, +), 48 px previous and next buttons, and the Info panel.
- The photo is drawn by the preview renderer from M1, so edits show. Zoom to 100% and beyond fetches tiles of the full-resolution image (technical design §6); pan by dragging; wheel and pinch zoom.
- Arrow keys move through the grid's order; the neighbors are decoded ahead, for PRF-5 (under 150 ms). Esc returns to the grid at the same scroll position with the last photo selected.
- A cloud-only photo is downloaded when opened, with progress and a way to cancel (LIB-9), one photo at a time.
- The preview re-fits when the window is resized (carried from M1).
- **Done when:** Nevada moves through a folder of his photos with the arrow keys and zooms into details, on both machines.

### 8. Photo info (BRW-6, BRW-7)

- The Info panel, always visible in the Library and Viewer: file name, date taken, dimensions, file size, camera and lens, location as coordinates, tags, and rating, read-only in M2. Metadata read with `kamadak-exif` (decision 0012), including HEIC's EXIF item.
- With several photos selected, the panel says how many and shows what they share.
- BRW-7's checks against real photos, on both machines: orientation and wide color, including the on-screen Display P3 checks deferred from M1.
- **Done when:** the panel matches what Explorer and Finder report for a sample of Nevada's photos.

### 9. Open elsewhere (BRW-8)

- Show in Explorer or Finder, and open in the system's default app, from the Library and the Viewer, through Rust (decision 0009: no shell plugin in the UI).
- **Done when:** both work for a photo and a selection on both machines.

### 10. Performance and wrap-up

- Measure the browsing and scanning targets on both machines with release builds: PRF-1, PRF-2, PRF-3, PRF-4, PRF-5, and PRF-7. Record the results in `docs/benchmarks/`.
- Node 26, LTS from 2026-10-28, once it has been LTS for a few weeks (planned in M1).
- Pre-release build 0.1 (milestone plan, "Working within a milestone"), unsigned, for Nevada's own use.
- Confirm every "done when" above and the milestone's; draft the devlog post.
- **Done when:** Nevada has used build 0.1 as his photo viewer on both machines.

## Carried from M1, and where it lands

| From M1 | Slice |
| --- | --- |
| Thumbnail speed in release builds (PRF-3) | 6 |
| Quick Look thumbnails for cloud-only photos on macOS | 6 |
| HEIC and JPEG pairs from iCloud for Windows | Question 6 below |
| Edited thumbnails | 5 |
| Keyboard navigation in the grid (ACC-2) | 5 |
| Preview re-fit on window resize | 7 |
| Hands-on checks on the Mac: preview, P3 color, slider smoothness, export, the HEIC "needs extensions" tile, the cloud-only check | 6 and 8, and the wrap-up |
| Export metadata (EXP-4), `mozjpeg` comparison | M5, with export |
| Third-party notices, including the IJG credit | M7 |

## Questions for a design session

These came up while planning. Each is about behavior or design, so it's listed here rather than decided; the slice that needs it is named.

1. **Toolbar controls for features that don't exist yet** (slices 1 and 5). The Library toolbar has Review, rotate, Enhance, Edit, and Export, and the sidebar a TAGS tree; most arrive in M3 to M6. Show them only as they're built (the proposal), or show them disabled?
2. **The M1 Editor during M2** (slice 7). It works (exposure, autosave, export) but is unstyled, and M4 replaces it. Keep it reachable from the Viewer with E and Edit, unstyled, or hide it until M4?
3. **Where to suggest the cache when Documents is cloud-synced** (slice 2). Rule 2 rules out hidden app-data folders, so the suggestion has to be a visible folder the user agrees to. A folder in the user's home, such as `Seura Photos Cache`? On Windows with OneDrive's folder backup, Desktop and Pictures are usually synced too.
4. **Thumbnail size range** (slice 5). BRW-3 has a slider but no range. The wireframes show cells of about 140 to 175 px. The largest size decides how big cached thumbnails must be for 2× screens (DSP-4): today they're 512 px, enough for cells up to 256 px.
5. **Sort order: one for the app, or remembered per folder?** (slice 5). BRW-2 doesn't say.
6. **HEIC and JPEG pairs** (slice 3). iCloud for Windows, with "Keep high efficiency original if available", leaves both `IMG_1234.HEIC` and `IMG_1234.JPG`. Show both (as now), or treat them as one photo, the way LIB-8 treats a Live Photo's video?
7. **What the Viewer shows** (slice 7). Presumably the photo with its edits, like the thumbnails. BRW-5 doesn't say, so it's confirmed here.
8. **Reference hardware** (slice 10). The targets are for a mid-range 2020 laptop with integrated graphics, and both of Nevada's machines are much faster. Measure on his machines and CI's runners, and treat a reference laptop or a volunteer's as part of M7's beta?
9. **Styled mockups in the repository.** The styled Library, Viewer, Editor, and Review screens live on the design canvas. Exported images, or at least the Viewer's, in `docs/ux/` would let each slice be checked against them; today only the Library, Editor, and Review appear, in devlog part 6.
10. **GitHub issues.** The milestone plan says each milestone is broken into issues, one per requirement or slice. M1 used pull requests only. Open an issue per slice for M2, or keep the plan as the list?
