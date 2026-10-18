# 0009. Workspace structure; the Rust core owns all file access

Status: Accepted · 2026-10-17

## Context

The rules that protect users' data (never modify originals, write only to allowed places, atomic writes, preserve unknown edit content) are the project's most important guarantees. They're only enforceable if there's exactly one place that touches the disk. Separately, the edit format's implementation should be reusable as a reference library by other apps.

## Decision

- The web UI never accesses the file system. Tauri's file system and shell plugins are not enabled; the UI calls Seura's own commands, and the Rust core performs every read and write.
- The core is a Cargo workspace: `seura-edit` (the edit format, with no dependencies on other Seura crates), `seura-image`, `seura-render`, and `seura-library`, with `src-tauri` as the app shell.
- Thumbnails and preview pixels reach the UI only through the read-only `seura://` protocol, limited to photos in the index.

## Consequences

- The data-safety rules are implemented and tested in Rust, in one layer.
- `seura-edit` can later be published as a crate and serve as the format's reference implementation alongside the shaders in `seura-render`.
- Some UI features need a command added rather than calling a browser or plugin API directly; that cost is intentional.
