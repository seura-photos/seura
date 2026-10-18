# 0006. Cross-platform from the first commit

Status: Accepted · 2026-10-13

## Context

Development happens on one machine at a time, mostly Windows, with a Mac available. Building Windows-only first and porting later risks Windows assumptions spreading through the code.

## Decision

- The first milestone is a "walking skeleton" running on both Windows and macOS: open a folder, show a thumbnail grid, open a photo in the edit view, and move one slider (exposure) that updates live through the shader pipeline.
- GitHub Actions builds and tests both platforms on every push.
- Platform-specific code lives behind small Rust traits or modules selected with `#[cfg(target_os = "...")]`; everything else stays platform-neutral.
- The Mac build is run by hand at milestones and after UI or rendering changes.

## Consequences

- Risky pieces (WebGPU in both web views, HEIC decoding, file paths, the thumbnail protocol) are proven on both platforms before features are built on them.
- CI catches compile and test failures on the platform not in daily use.
