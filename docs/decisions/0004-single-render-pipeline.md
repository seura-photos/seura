# 0004. One GPU render pipeline for preview and export

Status: Accepted · 2026-10-13

## Context

Edits must preview live while dragging sliders (at least 30 frames per second on a 48 MP photo, PRF-6) and must export at full resolution. A common failure is a fast GPU preview and a separate CPU export that drift apart, so the exported image doesn't match what the user saw. The edit format also promises that a file renders the same everywhere.

## Decision

All edit operations are implemented once, as GPU shaders in WGSL. The same shaders drive the live preview (WebGPU in the web view, with WebGL2 as a fallback if needed) and full-resolution export (wgpu in the Rust core, processing large images in tiles). Tone and color operations work in linear light with Display P3 primaries, in floating point.

The preview renders only a screen-sized image, or the visible region when zoomed in; the original is never displayed at full resolution. Thumbnails in the grid are ordinary images served from the cache.

## Alternatives considered

- **CPU rendering in Rust, pixels sent to the web view.** Copying image buffers across the boundary is too slow for live sliders.
- **Native rendering beneath a transparent web view.** Full control of color and HDR, but fiddly layering, input routing, and platform-specific code. Revisit only if true HDR display becomes a goal.
- **Separate preview and export implementations.** Faster to start, but results drift.

## Consequences

- The shader code is the reference implementation of the edit format and can be published with the conformance tests.
- WebGPU support in each platform's web view sets the minimum OS versions; to be confirmed in the technical design.
- Rendering tests compare output against reference images with a stated tolerance.
