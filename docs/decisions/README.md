# Decision records

Short records of significant decisions: what was chosen, why, and what was rejected. They exist so that a later change, by a person or an AI coding assistant, doesn't undo a deliberate choice without knowing the reasons.

- Numbered in order; never renumbered or deleted.
- To reverse a decision, add a new record that supersedes the old one, and mark the old one `Superseded by NNNN`.
- Status is one of: Proposed, Accepted, Superseded.

| # | Decision | Status |
| --- | --- | --- |
| [0001](0001-tauri-and-rust.md) | Tauri with a Rust core and a web UI | Accepted |
| [0002](0002-edit-files-beside-photos.md) | Edits stored in visible sidecar files beside each photo | Accepted |
| [0003](0003-settings-not-steps.md) | Edits stored as settings rendered in a fixed order | Accepted |
| [0004](0004-single-render-pipeline.md) | One GPU render pipeline for preview and export | Accepted |
| [0005](0005-no-hidden-app-data.md) | App data in Documents, keys in the OS credential store | Accepted |
| [0006](0006-cross-platform-from-day-one.md) | Cross-platform from the first commit | Accepted |
| [0007](0007-opinionated-interface.md) | An opinionated interface: one layout, ordered by frequency and process | Accepted |
| [0008](0008-react-typescript-ui.md) | React and TypeScript for the web UI | Accepted |
| [0009](0009-rust-owns-all-file-access.md) | Workspace structure; the Rust core owns all file access | Accepted |
| [0010](0010-in-memory-index.md) | An in-memory index with a snapshot file, not a database | Accepted, confirmed by benchmarks |
| [0011](0011-heic-decoding.md) | HEIC decoding: operating system first, libheif as the Windows fallback | Accepted; fallback superseded by 0014 |
| [0012](0012-no-gpl-dependencies.md) | No GPL dependencies; metadata through permissive crates | Accepted (amended for MPL-2.0) |
| [0013](0013-web-view-and-os-data.md) | Data the web view and operating system keep on their own | Accepted |
| [0014](0014-heic-system-decoders-only.md) | HEIC decoding: the operating system's decoders only | Accepted |
