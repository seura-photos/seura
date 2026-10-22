# M2 work plan: Browse

Status: ready to start · Last updated: 2026-10-20

The working plan for milestone M2 from the [milestone plan](milestone-plan.md): Seura replaces Nevada's photo viewer. It breaks the milestone into slices, one pull request each, in order. Drafted by Claude Code from the milestone plan, the [functional spec](functional-spec.md), the [user flows](ux/user-flows.md) and [wireframes](ux/wireframes.md), the [visual design](visual-design.md), and the [technical design](technical-design.md). The questions it raised were answered in a design session (PR #16; [journal](journal/2026-10-20-m2-plan-review.md)), and the plan follows those answers.

**Scope** (milestone plan, M2): LIB-1 to LIB-8 with the first-run Welcome (SET-1) and settings storage (SET-3); BRW-1 to BRW-8; DSP-1 to DSP-4; keyboard access for everything built (ACC-1); the visual design applied throughout; performance for scanning and browsing measured.

**Read-only.** M2 writes no edit files. Ratings, tags, and edit badges read from existing edit files are shown, but nothing in M2 changes them; that's M3. M1's Editor, the one place that writes them, is hidden in installed builds and stays available in development builds.

**Done when:** Nevada points Seura at his real photo library and browses it daily on both machines.

## How to work

As in M1 ([M1 work plan](m1-plan.md), "How to work"): one slice per branch and pull request, branched from `main` and named `m2/<slice>`; Nevada merges. Use current, stable dependencies at least two weeks old, checked against decision 0012. Ask Nevada for anything that needs his machines, his photos, or his eyes. Keep journal notes as you go.

New in M2:

- **GitHub issues.** Each slice has an issue under the "M2" milestone on GitHub, now that the repositories are public and others may follow along. Its pull request closes it.
- **Styling.** Every screen built in M2 is styled from `design/tokens.json` and checked against the mockups in [`docs/ux/mockups/`](ux/mockups/): styled versions of the Library and Viewer, wireframes for the rest. Icons are Lucide, as inline SVG.
- **Screenshots for review.** Each slice that changes a screen attaches screenshots in light and dark mode to its pull request, beside the matching mockup.
- **Only what's built.** Toolbar buttons and sidebar sections appear as their features arrive (Review, rotate, Enhance, Export, the TAGS tree), never as disabled placeholders.

## Slices

### 1. Foundations: Seura's folder, settings, shell, styling

- **Seura's folder (decision 0015):** `Seura Photos` in the home folder (`FOLDERID_Profile` on Windows, the home directory on macOS), replacing `Documents/Seura Photos`. It holds `settings.json`, `Presets/` (empty until M5), and by default `Cache/`, with the web view's data folder inside the cache as before (decision 0013). The app-data audit (`scripts/app_data_audit.py`) and CLAUDE.md rule 2 follow.
- **Settings file (SET-3):** `settings.json`, readable and written atomically. It holds the library folders, so clearing the cache never forgets the library, plus the cache location, the thumbnail size (BRW-3), the sort order (BRW-2), and the update-check choice (SET-4, stored now, used in M7).
- **Carrying over from M1, once:** M1 kept the library folders only in its index snapshot, `Documents/Seura Photos/Cache/index.bin`. On the first run of the new build, if no `settings.json` exists and that snapshot does, its folder list is read into the new settings and the Welcome screen is skipped. The old folder is only read, never changed or removed; Nevada deletes it himself.
- **Text outside the code (ACC-5):** every new piece of UI text in the message catalog from the start, including the native menus' labels, which Rust builds, so the core needs a small catalog of its own.
- **Display settings (ACC-3):** text and control sizes follow the system text size on both platforms, and animations respect reduced-motion settings.
- **Design tokens** as CSS variables for light and dark, generated from `design/tokens.json` and checked in CI like the bindings. Type sizes from the visual design.
- **Main window layout** from the wireframes and the styled Library mockup: toolbar (52 px), sidebar (232 px), main area, Info panel (280 px), status bar (28 px), with the panels widening at 1920 wide (DSP-3). It must work at 1280×720 (DSP-2), already the minimum window size.
- **State store** (Zustand, technical design §8) for the view, selection, and settings.
- **Native menus (ACC-4):** File (Add folder, Remove folder), View (sort, thumbnail size), Help, with each platform's shortcut conventions (ACC-1).
- **M1's Editor** hidden in installed builds; development builds keep it.
- **Done when:** the app shows the styled shell in light and dark mode on both machines, its settings and cache are in `~/Seura Photos`, Nevada's M1 library carries over, and the library survives deleting the cache folder.

### 2. Welcome (SET-1, SET-4)

- The first-run card from the wireframe: what the app does; that originals are never changed and edits are saved in small files beside each photo; where Seura keeps its settings and cache, naming the `Seura Photos` folder in the home folder; "Which folders hold your photos?" with Pictures pre-checked and Add another folder; "Check for updates automatically", checked; Get started (44 px). The wireframe predates decision 0015, so its text gains the folder.
- If `~/Seura Photos` is inside a synced or backed-up location (`docs/research/cloud-photo-sources.md`; some tools sync a whole profile), a note offers another place for the cache.
- Get started opens the Library on the first folder while the scan runs (LIB-3). Target: under a minute from launch to photos (user flows, flow 1).
- No styled mockup exists; it's styled from the tokens, and its screenshots go to Nevada for review.
- **Done when:** a first run on each machine, with no `~/Seura Photos`, ends in a scanning Library.

### 3. Library sidebar and status bar (LIB-1 to LIB-3, LIB-5 to LIB-8)

- The FOLDERS tree (LIB-2): library folders and every subfolder with photos, nested, each with its count.
- Remove a folder (LIB-5): from the library only; its cache entries dropped, its edit files left alone. Confirmed before removing.
- Unavailable folders (LIB-6): dimmed, "Drive not connected", and back when the drive returns. M1 already keeps their photos in the index.
- Status bar (LIB-3): photo count, number selected, and scan progress as processed out of found.
- Check LIB-7 (hidden and system folders) and LIB-8 (Live Photos' videos ignored; HEIC and JPEG pairs both shown) against real folders on both platforms; M1 implemented them.
- **Done when:** Nevada's library shows as a tree on both machines, and a disconnected drive dims and returns.

### 4. Watching for changes (LIB-4)

- File system watching (the `notify` crate, or each platform's own API) for new, changed, deleted, and renamed photos and edit files, reflected within 5 seconds. Changes made while the app was closed are found at launch, as M1's rescan already does.
- Watching never reads a cloud-only photo (LIB-9), and a burst of changes, such as a sync client catching up, is batched rather than scanned one file at a time.
- **Done when:** adding, renaming, and deleting photos in Explorer and Finder shows in Seura within 5 seconds, covered by integration tests with temporary folders.

### 5. The grid (BRW-1 to BRW-4, ACC-1, ACC-2)

- Styled grid from the Library mockup: square cells with photos letterboxed, the selection outlined in the accent color; badges for edited (bottom left) and rating (bottom right), read from existing edit files.
- **Edited thumbnails (BRW-1):** a photo with edits shows them in its thumbnail, rendered by the wgpu pipeline (technical design §4). In M2 that's exposure, the only operation built so far; the rest arrive with M4.
- Sorting (BRW-2): date taken (default, falling back to file date), file name, date modified, ascending or descending; one order for the whole app, remembered. Date taken for HEIC needs its EXIF item read, carried from M1.
- Thumbnail size slider (BRW-3): 96 to 256 px per cell, 160 by default, remembered. The cached 512 px thumbnails stay sharp at the largest size on 2× screens (DSP-4).
- Selection (BRW-4): click, Ctrl/Cmd-click, Shift-click, Select All; the count in the status bar.
- Keyboard (ACC-1): the ARIA grid pattern, with arrow keys, Home and End, Page Up and Down, and Enter to open, carried from M1. Screen readers announce each thumbnail's file name, date, and rating (ACC-2).
- **Done when:** a 1,000-photo folder sorts, resizes, and selects, scrolling at 60 fps (PRF-4), by mouse and by keyboard.

### 6. Thumbnails, faster and for the cloud (PRF-3, LIB-9)

Carried from M1:

- Thumbnail speed: smaller decodes (JPEG's scaled decoding, embedded previews where they're good enough), measured in release builds against PRF-3. Measured with a benchmark (`cargo bench -p seura-library --bench thumbnails`; CI reports it without judging it): on Nevada's PC, 12-megapixel JPEGs went from 46 to 67 a second by resizing without first copying the full-size image, and to 105 with scaled decoding (`jpeg-decoder`, for thumbnails only). Embedded previews weren't needed.
- A long scan no longer holds the index: folders are listed without it and it's locked for one folder's photos at a time, so browsing goes on during a large first scan (carried from slice 3b).
- After the launch scan, in the background, thumbnails no indexed photo uses that haven't been touched for about a week are deleted, so renamed or deleted folders don't leave theirs in the cache for good (added by Nevada, 2026-10-21).
- macOS: thumbnails for cloud-only photos from Quick Look, which keeps the sync service's preview, instead of the placeholder M1 shows. Checked with `cloud_check`, as on Windows.
- **Done when:** a first scan of Nevada's library meets the PRF-3 aim (slice 10) on both machines, and his cloud camera roll shows thumbnails on the Mac without downloading anything.

### 7. Viewer (BRW-5, LIB-9)

- One photo full size on the dark surround, from the styled Viewer mockup: back to the folder, file name, "37 of 214", zoom (−, Fit, 100%, +), 48 px previous and next buttons, and the Info panel.
- The photo is drawn with its edits applied, by the preview renderer from M1, like the thumbnails (BRW-5). Zoom to 100% and beyond fetches tiles of the full-resolution image (technical design §6); pan by dragging; wheel and pinch zoom.
- Arrow keys move through the grid's order; the neighbors are decoded ahead, for PRF-5. Esc returns to the grid at the same scroll position with the last photo selected.
- A cloud-only photo is downloaded when opened, with progress and a way to cancel (LIB-9), one photo at a time.
- The preview re-fits when the window is resized (carried from M1).
- Edit (`E`) opens M1's Editor in development builds only.
- **Done when:** Nevada moves through a folder of his photos with the arrow keys and zooms into details, on both machines.

### 8. Photo info (BRW-6, BRW-7)

- The Info panel, always visible in the Library and Viewer, from the styled mockups: file name, date taken, dimensions, file size, camera and lens, location as coordinates, tags, and rating, read-only in M2. Metadata read with `kamadak-exif` (decision 0012), including HEIC's EXIF item.
- With several photos selected, the panel says how many and shows what they share.
- BRW-7's checks against real photos, on both machines: orientation and wide color, including the on-screen Display P3 checks deferred from M1.
- **Done when:** the panel matches what Explorer and Finder report for a sample of Nevada's photos.

### 9. Open elsewhere (BRW-8)

- Show in Explorer or Finder, and open in the system's default app, from the Library and the Viewer, through Rust (decision 0009: no shell plugin in the UI).
- **Done when:** both work for a photo and a selection on both machines.

### 10. Performance and wrap-up

- Measure the browsing and scanning targets on both machines and CI with release builds: PRF-1, PRF-2, PRF-3, PRF-4, PRF-5, and PRF-7. These machines are much faster than the 2020 reference laptop, so the aim is about half each target's time (functional spec, "Performance targets"); a reference-class laptop is tested in the M7 beta. Record the results in `docs/benchmarks/`.
- Node 26, LTS from 2026-10-28, once it has been LTS for a few weeks (planned in M1).
- Pre-release build 0.1 (milestone plan, "Working within a milestone"), unsigned, for Nevada's own use, with M1's Editor hidden.
- Confirm every "done when" above and the milestone's; draft the devlog post, including the move of Seura's folder out of Documents and its reasoning ([journal](journal/2026-10-20-m2-plan-review.md)).
- **Done when:** Nevada has used build 0.1 as his photo viewer on both machines.

## Carried from M1, and where it lands

| From M1 | Slice |
| --- | --- |
| Thumbnail speed in release builds (PRF-3) | 6 |
| Quick Look thumbnails for cloud-only photos on macOS | 6 |
| HEIC and JPEG pairs from iCloud for Windows | Settled: both shown (LIB-8); checked in 3 |
| Edited thumbnails | 5 |
| Keyboard navigation in the grid (ACC-2) | 5 |
| Preview re-fit on window resize | 7 |
| Hands-on checks on the Mac: preview, P3 color, slider smoothness, export, the HEIC "needs extensions" tile, the cloud-only check | 6 and 8, and the wrap-up |
| Export metadata (EXP-4), `mozjpeg` comparison | M5, with export |
| Third-party notices, including the IJG credit | M7 |

## Questions answered in planning

Raised in the first draft of this plan and answered in a design session (PR #16), Nevada deciding:

| Question | Answer |
| --- | --- |
| Toolbar controls for features not built yet | Shown only once built. |
| M1's Editor during M2 | Hidden in installed builds; kept in development builds. |
| Where to suggest the cache when Documents is synced | Seura's whole folder moves to `~/Seura Photos` (decision 0015). |
| Thumbnail size range | 96 to 256 px, 160 by default (BRW-3). |
| Sort order per folder or app-wide | App-wide (BRW-2). |
| HEIC and JPEG pairs | Both shown, as separate files (LIB-8). |
| What the Viewer shows | The photo with its edits (BRW-5). |
| Reference hardware | Measured on Nevada's machines and CI, aiming for about half the target time; a reference laptop in the M7 beta. |
| Styled mockups in the repository | In `docs/ux/mockups/`. |
| GitHub issues | One per slice, under an "M2" milestone. |
