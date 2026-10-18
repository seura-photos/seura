# 0012. No GPL dependencies; metadata through permissive crates

Status: Accepted · 2026-10-17 · Amended 2026-10-18 (MPL-2.0)

## Context

Seura Photos is Apache-2.0. The Apache License 2.0 is incompatible with GPL version 2, and combining with GPL version 3 code would require the combined app to be distributed under the GPL. The best-known photo metadata library, exiv2 (and its wrapper gexiv2), is GPL, as are some other imaging libraries.

## Decision

- No GPL or AGPL dependencies in anything distributed with the app. LGPL is allowed only for dynamically linked libraries (see 0011).
- MPL-2.0 is allowed for unmodified dependencies (added 2026-10-18, approved by Nevada). Its copyleft applies file by file: it covers changes to the MPL-licensed files themselves, not the code that uses them, so an Apache-2.0 app can include them. Unmodified crates already have their source published on crates.io, and the third-party notices (M7) will list them. Modifying an MPL file, or vendoring a patched copy, needs a fresh review. Tauri itself depends on MPL-2.0 crates (`cssparser`, `cssparser-macros`, `selectors`, and `dtoa-short` through `tauri-utils`; `option-ext` through `dirs`), so excluding MPL would rule out Tauri (decision 0001).
- Metadata uses permissive Rust crates:
  - `kamadak-exif` (BSD-2-Clause) to read EXIF from JPEG, HEIF, and PNG.
  - `img-parts` (MIT or Apache-2.0) to carry metadata blocks into exported files.
  - `little_exif` (MIT or Apache-2.0) to modify EXIF on export: reset orientation, remove GPS.
  - `xmp_toolkit` (MIT or Apache-2.0, wrapping Adobe's BSD-licensed XMP Toolkit) to write tags as `dc:subject` and the rating as `xmp:Rating`.
- A license check (`cargo-deny` for Rust, a license checker for npm packages) runs in CI and fails on disallowed licenses. The allow lists are in `deny.toml` and `ui/scripts/licenses.mjs`.

## Consequences

- Some metadata edge cases handled by exiv2 (maker notes, unusual formats) may be less complete; v1 only needs common EXIF fields and XMP keywords, and only writes metadata to exported copies.
- Supersedes the earlier informal suggestion, made before the license was established, to use exiv2.
