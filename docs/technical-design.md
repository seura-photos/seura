# Seura Photos — Technical Design (v1)

Status: approved · Last updated: 2026-10-17

This document describes how Seura Photos v1 is built. It implements the [functional spec](functional-spec.md) and follows the [decision records](decisions/); the edit file format is defined in [seura-photos/edit-format](https://github.com/seura-photos/edit-format). Versions are current as of this date and are pinned when the project is scaffolded.

## 1. Overview

A Tauri 2 desktop app (decision 0001). The **Rust core** owns everything that touches the disk or does heavy work: scanning, watching, decoding, thumbnails, metadata, edit files, the cache, and export. The **web UI** (React and TypeScript, decision 0008) owns presentation and the live edit preview, which it renders with WebGPU using the same WGSL shaders the core uses for export (decision 0004).

```
┌──────────────────────────── Web view (WebView2 / WKWebView) ───────────────────────────┐
│ React UI: Library · Viewer · Editor · Review · dialogs                                 │
│ Preview renderer: WebGPU (WebGL2 fallback), shaders from crates/seura-render/shaders   │
└───────▲──────────────────────────────▲───────────────────────────────▲─────────────────┘
        │ commands (invoke)            │ events (scan progress,        │ seura:// protocol
        │ e.g. save_edits, export      │ file changes, notices)        │ thumbnails, previews
┌───────┴──────────────────────────────┴───────────────────────────────┴─────────────────┐
│ Rust core (src-tauri + workspace crates)                                               │
│ seura-library: scan, watch, index, cache   seura-image: decode, color, thumbs, metadata│
│ seura-edit: edit files, history, presets   seura-render: shaders, wgpu export, enhance │
└────────────────────────────────────────────────────────────────────────────────────────┘
          │ reads only                       │ writes only (CLAUDE.md rules 1–2)
     original photos              edit files beside photos · ~/Seura Photos · exports
```

**The single most important rule:** the UI never touches the file system. Every read and write goes through Rust commands, so the data-safety rules in `CLAUDE.md` are enforced in one place and tested there.

## 2. Repository layout

A Cargo workspace plus a Vite frontend (decision 0009):

```
seura/
├─ src-tauri/              App shell: window, menus, commands, events, protocol, updater
├─ crates/
│  ├─ seura-edit/          Edit format model: parse, validate, canonical order, preserve
│  │                       unknowns, history, presets. No other Seura dependencies, so it
│  │                       can be published as the format's reference library.
│  ├─ seura-image/         Decoding (JPEG, PNG, HEIC), color management, orientation,
│  │                       thumbnails, metadata read and write, export encoding
│  ├─ seura-render/        WGSL shaders (the reference implementation of every operation),
│  │                       wgpu export pipeline with tiling, one-click enhance analysis
│  └─ seura-library/       Library folders, scanning, file watching, in-memory index,
│                          cache (thumbnails and index snapshot), fingerprints
├─ ui/                     React + TypeScript (Vite): views, state, WebGPU preview renderer
├─ tests/                  Cross-crate tests: data safety, render references, benchmarks
├─ docs/                   Specs, decisions, UX documents
└─ third_party/edit-format The edit-format repository as a git submodule (schema, examples,
                           invalid cases), pinned to a release, used by seura-edit's tests
```

## 3. Process and threading

- **Main thread:** Tauri event loop and window. Never blocks.
- **Async runtime** (Tokio, via Tauri): command handlers; anything slow is handed off.
- **Worker pool** (Rayon): decoding, thumbnail generation, hashing, export. Jobs carry a priority: visible thumbnails first, then the current folder, then everything else (LIB-3). Jobs for photos that scroll out of view are deprioritized, and a cancelled export stops between photos.
- **GPU:** one wgpu device in the core for export and enhance analysis; the web view has its own WebGPU device for preview. They share shader source, not memory.

## 4. Library, index, and cache

### Scanning and watching (LIB-1 to LIB-8)

- **Scan:** walk each library folder (skipping hidden and system folders, LIB-7), collect supported files by extension and magic bytes, and compare against the index snapshot by path, size, and modification time. Only new or changed files are decoded.
- **Watch:** the `notify` crate (FSEvents on macOS, ReadDirectoryChangesW on Windows), with events debounced over about 500 ms and folded into the same "reconcile this folder" job the scanner uses, so there's one code path. Watchers can miss events (network drives, overflow), so a cheap re-scan of the visible folder runs on window focus.
- **Renames** (FIL-6): when a photo disappears and an unclaimed photo with the same fingerprint appears in the same folder, its edit file is reattached by renaming the edit file (the one file move the app performs, and only on its own files).
- **Unavailable drives** (LIB-6): a missing library root marks the folder unavailable instead of dropping its index entries.

### The index (decision 0010)

An **in-memory index**, persisted as a single snapshot file in the cache, instead of a database.

- One record per photo: path, size, modification time, fingerprint (once computed), dimensions, date taken, camera, rating, tags, edited flag, and cache keys. Roughly 200 to 300 bytes each in memory, so 100,000 photos is about 30 MB, well within PRF-7.
- Secondary maps: folder → photos, tag → photos, and a lowercase file-name and tag index for search (FND-2), which makes filtering 10,000 photos a sub-millisecond scan (FND-3).
- **Snapshot:** written in a compact binary format (`postcard`; `bincode`, first chosen, is unmaintained, see decision 0010) after scans and periodically, atomically (temporary file, then rename). On launch it's loaded before anything else, so the grid appears immediately (PRF-1) while reconciliation runs in the background.
- **Source of truth** remains the photos and edit files. A missing or corrupt snapshot just triggers a full re-scan.
- **Benchmark gate:** the first milestone measures launch time, memory, and query time on synthetic 10,000 and 100,000 photo libraries. If the targets are missed, SQLite (`rusqlite`) replaces the snapshot behind the same interface, with a new decision record superseding 0010. Passed in M1 with a wide margin; see [`benchmarks/m1-index.md`](benchmarks/m1-index.md).

### Thumbnails and the cache (DSP-1, DSP-4)

- Cache folder (default `~/Seura Photos/Cache`, SET-3, decision 0015) contains `index.bin`, `thumbs/`, and a `README.txt` explaining that everything there is safe to delete.
- Thumbnails are WebP, 512 px on the long edge (enough for the largest grid size on 2× screens), stored by fingerprint-independent key (hash of path + size + mtime) in two-level sharded folders, so renames and re-scans don't regenerate them unnecessarily.
- **Fast first scan** (PRF-3): use the thumbnail embedded in the file when it's large enough (JPEG EXIF thumbnails are often 160 px, too small; HEIC files often carry larger ones), otherwise decode and downscale with `fast_image_resize`. Reduced-size JPEG decoding (decoding at 1/2, 1/4, or 1/8 scale directly from the compressed data) would be much faster; whether `zune-jpeg` supports it, or a different decoder is needed for thumbnails, is checked in the first milestone's benchmarks.
- Edited photos get a second thumbnail rendered with their edits by the core's wgpu pipeline, regenerated after each autosave (debounced).
- Cloud-sync detection (SET-1): check whether `~/Seura Photos` or the chosen cache path lies under a synced or backed-up location (the OneDrive folder and its known-folder backups on Windows, iCloud Drive's Desktop and Documents sync on macOS, and the other services in `docs/research/cloud-photo-sources.md`); if so, suggest another location for the cache.

### Cloud-only photos (LIB-9)

Folders synced by a cloud service often hold files whose contents are only in the cloud; reading even their first bytes downloads them. Research and the service matrix: `docs/research/cloud-photo-sources.md`.

- **Detection, per file:** Windows' Cloud Files API marks cloud-only files with `FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS` (OneDrive, iCloud for Windows, very likely Dropbox); macOS marks them `SF_DATALESS` (iCloud Drive and every File Provider client: OneDrive, Google Drive, Dropbox). Listing folders and reading attributes, sizes, and dates never downloads anything. These checks live in `seura-library`'s platform module.
- **Google Drive on Windows** streams through its own virtual drive with no per-file signal, so every file on it is treated as cloud-only. Its mount point is read from Drive for desktop's registry settings.
- **Scanning** records a cloud-only photo from its name, size, and date, without checking its signature.
- **Thumbnails** for cloud-only photos come from the system's thumbnail service (the Windows shell thumbnail cache, macOS Quick Look), which can use the sync service's own preview, and are never generated by decoding. Where none is available without a download, the grid shows a placeholder tile with a cloud marker. Whether each source really avoids a download is verified in M1 slice 4.
- **Downloading on demand:** opening a cloud-only photo in the Viewer or Editor, editing it, and exporting it read the file, one at a time. Neither platform reports progress to the reading app, so the UI shows an indeterminate indicator with the size from the placeholder's metadata (LIB-9). The read runs on a worker; when the user moves on, the UI stops waiting and the worker's result is discarded. On Windows the download is triggered by opening the file and finishes regardless; on macOS it happens during the first read and stops if the read is abandoned. Neither behavior is relied on. Failures (offline, service errors, a timeout with no data) are reported with a retry. The Viewer's look-ahead skips cloud-only photos. Measurements: `docs/research/cloud-photo-sources.md`. Tagging and rating don't: the edit file is created without the fingerprint and dimensions, which the edit format allows, and `EditDocument::fill_in_photo` adds them the next time the photo is read. Until then a renamed photo can't be reattached (FIL-6) and a changed original is detected by size only (FIL-7). Background work (fingerprints, metadata, edited thumbnails) skips cloud-only photos.
- Seura never pins or unpins files (never changes what the service keeps on the device), and re-checks the cloud-only state on every scan, since the system can move files back to the cloud on its own.

### Fingerprints (FIL-6, FIL-7)

SHA-256 of the file bytes, computed lazily: when an edit file is first created, and in the background for photos with edit files. Results are cached in the index with size and mtime, and recomputed only when those change. The `sha2` crate hashes several hundred MB per second, so a 10 MB photo takes tens of milliseconds.

## 5. Image input and output

### Decoding

| Format | Decoder | Notes |
| --- | --- | --- |
| JPEG | `zune-jpeg` | Fast, with scaled decoding for thumbnails. |
| PNG | `png` | Including 16-bit. |
| HEIC/HEIF | See decision 0014 | macOS: the system's ImageIO. Windows: the system's WIC codec, with Microsoft's HEIF and HEVC extensions; without them, HEIC photos show a message. |

All decoded images become linear-light, floating-point pixels with Display P3 primaries, the working space defined by the edit format.

### Color management

`moxcms` (pure Rust, BSD-3-Clause or Apache-2.0) converts from each photo's embedded ICC profile (sRGB, Display P3, Adobe RGB, and so on) into the working space; photos without a profile are treated as sRGB. Export converts to sRGB with the profile embedded (EXP-5). The display side is handled by the web view: previews are delivered tagged as Display P3, and the browser engine maps them to the monitor's profile. True per-monitor color management of the WebGPU canvas is a known limitation to verify in the first milestone.

### Orientation

The EXIF orientation tag is applied once, at decode, so everything downstream (thumbnails, edit coordinates, export) works on upright pixels, matching the edit format's definition of `width` and `height`.

### Metadata (decision 0012)

No GPL code: exiv2 and gexiv2 are excluded because they're GPL-licensed and Seura is Apache-2.0.

- **Read:** `kamadak-exif` for EXIF in JPEG, HEIF, and PNG (date taken, camera, lens, GPS, orientation, dimensions).
- **Write on export** (EXP-4): `img-parts` to carry the original EXIF block into the new file, `little_exif` to reset orientation and remove GPS when requested, and `xmp_toolkit` (Adobe's XMP Toolkit, BSD-licensed, via MIT/Apache bindings) to write tags as `dc:subject` keywords and the rating as `xmp:Rating`, which Lightroom, digiKam, Windows, and macOS all read.
- Originals are never written.

### Export (EXP-1 to EXP-6)

Decode at full resolution → render all operations with the wgpu pipeline in tiles (1440 px squares of the result, each uploading the part of the photo it samples with a few pixels' margin for the filter, which fits the 2048 px texture limit even straightened by 45°; operations that sample neighbors use the guide, below) → convert to sRGB → encode JPEG (`jpeg-encoder` since M1; `mozjpeg` to be benchmarked against it for quality and speed in M2) or PNG → write metadata → write to a temporary file in the destination folder and rename to the final, never-taken name (EXP-3).

## 6. Rendering (decision 0004)

### One implementation, two hosts

- All operations live as WGSL in `crates/seura-render/shaders/`, one file per stage plus shared color functions. These shaders are the reference implementation of the edit format.
- **Export and edited thumbnails:** the core runs them with wgpu (Metal on macOS, Direct3D 12 on Windows; wgpu's software adapter on Windows machines without a usable GPU).
- **Live preview:** the UI runs the same WGSL with WebGPU in a `<canvas>`. A test in `seura-render` writes the shader source, joined in its fixed order, to `ui/src/preview/generated/`, and fails in CI if the committed copy is out of date (`npm run shaders` regenerates it, as `npm run bindings` does for commands).
- **Fallback:** where WebGPU isn't available in the web view (Macs before macOS Tahoe 26), naga translates the WGSL to GLSL ES 3.0, generated and committed the same way, and the preview runs on WebGL2. The source is still a single WGSL implementation; the translation is generated and tested, never hand-edited.

### Pipeline

1. **Geometry** (orient, straighten, crop) is applied as a coordinate transform when sampling, not as a separate pass: each pixel of the result is placed in the upright photo and sampled there with Catmull-Rom, as the edit format's formulas appendix defines (`geometry.md`).
2. **Retouch** (red-eye) samples the photo in its pre-geometry coordinates.
3. **Tone and color**, then **effects**, run as one fused fragment pass over the visible region, with uniforms for each operation's parameters. An operation with neutral settings is skipped by a branch on its own uniform values (for example, contrast only when its amount isn't 0), rather than separate shader variants or separate flags that would have to be kept in step with the values.

### Neighborhood operations: highlights and shadows

Highlights and shadows depend on the brightness of each pixel's surroundings, not just the pixel, so they can't be computed in the fused pass alone. They use a **luminance guide**: an edge-preserving, blurred map of the photo's brightness, computed in a separate pass and sampled by the fused pass.

- **Resolution-independent.** The guide is computed at a fixed size relative to the photo (a long side of 1024 pixels, from the photo as decoded), and its blur radius is a fraction of the photo's size. The preview and the full-resolution export therefore use the same guide and match, as the edit format requires.
- **Tiles without seams.** For tiled export, the guide is computed once for the whole photo before the tiles are rendered, and every tile samples it. No tile needs to see its neighbors.
- **Independent of the crop.** The guide covers the whole oriented, straightened photo, so changing the crop never changes how shadows or highlights look.
- **Fast while dragging.** The guide is rebuilt only when something it depends on changes. Moving the highlights or shadows sliders changes only uniforms, within PRF-6. Exposure shifts every log luminance equally, so it's added to the guide's result as each pixel is drawn, and needs no rebuild; a white balance change rebuilds the guide (M4 slice 4).
- **Edge-preserving.** A plain blur causes halos along strong edges, such as a dark tree against a bright sky. The method is a guided filter, chosen in M4 over a bilateral grid and a plain blur with reference images that include such edges (journal, 2026-10-29), and written into the edit format's formulas appendix.

Like every operation, it's implemented once in WGSL and runs in both hosts, including the WebGL2 fallback.

### Getting pixels into the preview

When a photo opens, the core decodes it at display resolution (the canvas size × device pixel ratio, DSP-1), converts it to the working space, and serves it over the custom `seura://` protocol as half-float RGBA. The UI fetches it as an `ArrayBuffer` and uploads it with `writeTexture`. Moving a slider changes only uniforms, so each frame is a single draw (PRF-6). Zooming beyond display resolution requests tiles of the full-resolution image the same way.

### One-click enhance (EDT-12)

The core analyzes the guide image (the photo at 1024 pixels on its long side), as the photo's geometry crops and turns it, on the CPU, in a few milliseconds (`seura-render`'s `enhance`). Temperature and tint come from the average of near-neutral pixels, found in steps so a cast's own tint doesn't hide them. Exposure moves the log-average luminance toward middle gray, without pushing the 99.5th percentile far past white. Contrast comes from the spread between the 0.5th and 99.5th percentiles, highlights and shadows from how much of the photo sits near the top and bottom of the range, and saturation from how colorful it is on average. The result is returned as ordinary operation parameters. It never reads the tone settings it replaces, so pressing it again gives the same result (EDT-12).

## 7. Edit files (seura-edit)

- **Model:** each file is kept as both a typed view (the operations Seura knows) and the original JSON value. Saving writes known fields from the typed view and copies every unknown operation, field, and extension back exactly (FIL-5).
- **Validation:** against the edit-format JSON Schema (`jsonschema` crate) plus the rules the schema can't express (canonical order, uniqueness, tag case), mirroring `tools/check.py` in the format repository. Files that fail are never overwritten without confirmation (FIL-4).
- **Writes:** debounced to within one second of the last change (EDT-11); temporary file in the same folder, `fsync`, then rename (FIL-2). Numbers are rounded to four decimals.
- **External changes** (FIL-3): the watcher reports changes to edit files; if the content differs from what Seura last wrote, the index updates and the UI shows the notice.
- **History:** capped at 50 records, with the consistency check before undo, exactly as the format specifies.
- **Presets:** the same code, in `kind: "preset"` mode.

## 8. UI architecture (decision 0008)

- **React with TypeScript**, built with Vite. A small state store (Zustand) holds the current view, selection, and the open photo's edit state.
- **Grid virtualization** (TanStack Virtual) renders only visible thumbnails, so 100,000-photo folders scroll at 60 fps (PRF-4). Thumbnails are `<img>` elements pointing at `seura://thumb/...` URLs.
- **Commands** are typed on both sides: the Rust command signatures generate TypeScript bindings (`tauri-specta` or equivalent), so the UI can't call a command with the wrong arguments.
- **Keyboard:** one keymap module implementing the shortcuts in the user flows document, with the tier rules (single letters ignored while typing in a field).
- **Text** lives in message catalogs from the start (ACC-5), English only in v1, in **Project Fluent** (`.ftl` files): `fluent-bundle` in the core for menus, dialogs, and notifications, and `@fluent/bundle` in the UI, so both sides use one format and one set of translator tools. Fluent was designed with Mozilla's translators for plurals, grammatical variation, and word order, which simple placeholder strings can't express. Both are MIT or Apache-2.0 (decision 0012). It replaces M1's JSON catalogs during M2, while there are only a few dozen strings; if implementation finds a blocker, raise it before choosing another format.
- **Regional formats** (ACC-6): the UI formats dates, times, numbers, and file sizes with the web view's `Intl` APIs in the operating system's region. The core passes raw values (timestamps, byte counts), never preformatted text.
- **Pseudo-translation** (ACC-7): a developer switch, `SEURA_LOCALE=pseudo`, renders every string lengthened by about 40% and with accented letters, for screenshots and checks of each screen.
- **Tag matching** (TAG-2): Unicode NFC normalization and full case folding in the core (`unicode-normalization` and `caseless`, or equivalent), never ASCII-only lowercasing.
- **Accessibility:** native elements wherever possible; the grid implements the ARIA grid pattern; every action reachable by keyboard (ACC-1, ACC-2).

## 9. Platform specifics

Kept behind small traits in `src-tauri/src/platform/` (decision 0006):

| Concern | Windows | macOS |
| --- | --- | --- |
| HEIC decoding | WIC, with Microsoft's HEIF and HEVC extensions | ImageIO |
| Trash | `trash` crate (Recycle Bin) | `trash` crate (Finder's Trash, with Put Back) |
| Credentials (AI, later) | `keyring` (Credential Manager) | `keyring` (Keychain) |
| Cloud-sync detection | OneDrive known folders | iCloud Desktop and Documents |
| Seura's folder (decision 0015) | `Seura Photos` in the profile folder, via the Known Folder API (`FOLDERID_Profile`) | `~/Seura Photos`, the home directory via NSFileManager |
| Web view data (decision 0013) | WebView2 InPrivate, data folder in the cache (`<cache>/WebView`) | WKWebView with a non-persistent data store |
| Installer | NSIS or MSI via Tauri bundler, signed | DMG, signed and notarized |

## 10. Security

- Tauri capabilities allow the UI to call only Seura's own commands; the default file system and shell plugins are not enabled.
- A strict Content Security Policy: no remote scripts or content; network access only for the updater endpoint (and, later, the user's chosen AI provider from Rust, never from the web view).
- The `seura://` protocol serves only thumbnails and previews for photos in the index, never arbitrary paths.
- Updates are signed with the Tauri updater key and verified before install (SET-4); the private key lives in GitHub Actions secrets.

## 11. Testing

| Layer | What | How |
| --- | --- | --- |
| Data safety | Originals are byte-identical after every operation; nothing is written outside allowed locations; edit writes are atomic; unknown content survives | Integration tests running real operations on temporary libraries, with a file-system audit before and after. Required to pass on both platforms. |
| Edit format | Every example and invalid case from the format repository | `seura-edit` tests against the submodule |
| Rendering | Each operation matches reference images within a stated tolerance; WebGPU preview matches wgpu export | Golden-image tests in Rust; a headless browser parity test for the preview renderer |
| Library | Scan, watch, rename reattachment, unavailable drives | Integration tests with temporary folders |
| UI | Components and keyboard behavior | Vitest and React Testing Library |
| End to end | Main flows (Review, fix a photo, tag, export) | WebDriver via `tauri-driver` on Windows; manual checklist on macOS, where Tauri's WebDriver support is not available |
| Performance | PRF-1 to PRF-7 | Benchmarks on generated 10,000 and 100,000 photo libraries, tracked per release |

CI (GitHub Actions) builds and tests on `windows-latest` and `macos-latest` on every push; release builds are produced by `tauri-action` with signing.

## 12. Minimum OS versions and test coverage

- **Windows 10 (22H2) and 11**, x64. WebView2 is preinstalled on both, and WebGPU is enabled in it on x64. Windows on ARM is a later target; WebGPU there is still behind a flag in Chromium.
- **macOS 13 Ventura and later**, Apple Silicon and Intel. WebGPU preview on macOS Tahoe 26; WebGL2 fallback on 13 to 15. Tauri 2 itself supports macOS 10.15, but older versions are not worth the testing cost for this audience.

The developer's own machines (one Windows desktop, one MacBook Pro, both kept current) cover Windows 11 and the newest macOS. The rest is covered in tiers:

| Tier | Covers | How |
| --- | --- | --- |
| Hands-on | Windows 11, current macOS (WebGPU path) | Daily development and milestone checks |
| Automated | Older macOS (WebGL2 fallback), Windows 10-era builds | CI on GitHub's older macOS images (14, 15) and Windows Server images; build, unit, data-safety, and rendering reference tests |
| Beta | Real older Macs and Windows 10 PCs | A short beta with volunteers before v1, following a written checklist |

If the beta finds no testers for macOS 13 to 15, the macOS minimum is raised to Tahoe 26 before release rather than shipping a path nobody has seen run, recorded in a decision record.

## 13. First milestone: walking skeleton

Proves every risky piece on both platforms before features are built on them:

1. Scaffold the workspace and Tauri app; CI green on Windows and macOS.
2. Add a folder; scan; write and load the index snapshot; generated thumbnails in a virtualized grid via `seura://`.
3. Decode a JPEG, a PNG, and an iPhone HEIC on both platforms, with correct orientation and color.
4. Open the Editor with one operation, exposure: WebGPU preview (and the WebGL2 fallback on an older Mac), autosave to a valid edit file, export through wgpu, and a reference-image test that preview and export match.
5. The data-safety test suite running in CI.
6. Benchmarks for the index decision (0010).

## 14. Decisions log

| Question | Decision |
| --- | --- |
| HEIC decoding and patents | Operating system decoders only, so Seura ships no HEVC decoder; documentation for installing Microsoft's extensions (decisions 0011 and 0014). |
| Minimum OS versions | Windows 10 22H2+, macOS 13+, with tiered test coverage and a beta gate for older macOS (section 12). |
| UI framework | React and TypeScript (decision 0008). |
