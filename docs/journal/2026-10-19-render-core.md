# 2026-10-19: The render core (M1 slice 5a)

Slice 5, one edit end to end, is split in two pull requests at Claude's suggestion, which Nevada accepted: 5a is the core (shaders, wgpu export, tests), 5b the Editor, preview, and autosave.

## Decisions

- **Testing the preview against the export:** Claude recommended an in-app check that renders a reference image in the real web view (WebView2, WKWebView) and compares it with the wgpu export, run in CI, over a headless Chromium test that never touches WKWebView. Nevada agreed; it's built in 5b.
- **JPEG encoder:** `jpeg-encoder`, pure Rust, at Claude's recommendation, Nevada agreeing. Its IJG license asks for a credit line in the documentation, which goes in the third-party notices (M7). `mozjpeg` makes smaller files but is C with extra build tools; the comparison the technical design asks for moves to M2's export work.
- **CI gains macOS 15**, the older image, for the WebGL2 path: `macos-latest` became macOS 26 in the summer, and GitHub drops macOS 14 on November 2.

## What 5a found

- **Thumbnails were fine in 8-bit sRGB, editing isn't.** Decoding converted every photo to sRGB straight away, which clips an iPhone photo's wide colors. Color conversion now happens last: thumbnails still get sRGB, while editing and export get the working space (linear light, Display P3, half floats). On the Mac, ImageIO now draws HEIC into Display P3 instead of sRGB, for the same reason.
- **One shader implementation, three files:** WGSL has no includes, so the color functions, the tone operations, and the render pass are joined in a fixed order in one place, and the preview's copies (WGSL, and naga's GLSL ES translation for WebGL2) are generated from that and checked in CI, like the command bindings.
- **Tests that can fail:** the render test compares wgpu's output with the format's definition computed independently on the CPU, within one level. Claude broke the shader on purpose twice (exposure 3% too strong, the sRGB conversion skipped) to see the test fail before trusting it.
- Export never replaces a file: it writes a temporary file and renames it to the first free name (`beach (1).jpg`) without replacing anything, even a file that appears in between. The data-safety suite checks it.
