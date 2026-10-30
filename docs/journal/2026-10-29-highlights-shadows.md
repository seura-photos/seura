# Highlights and shadows: choosing the luminance guide (#135)

## The comparison (slice 4a)

The plan asked for the guide's method to be chosen with reference images that have strong edges. Claude Code wrote a throwaway program, outside the repository, that ran three ways of building the guide with the same highlights-and-shadows formula, at the strongest settings (shadows +100, highlights −100), on three generated images: a dark tree with fine branches against a bright sky, a backlit face in front of a window, and a hard step from deep shadow to bright with the same fine texture on both sides. The guide was built at 1024 pixels on the long side from a 2048-pixel image, as the app does.

| Method | Halo at the step | Texture the base follows | On the GPU |
| --- | --- | --- | --- |
| Plain blur (the baseline) | 1.4 stops, about 50 pixels wide | none | simple |
| Guided filter, ε = 0.5 | 0.27 stops on the bright side | about 4% | box blurs and 2D textures |
| Guided filter, ε = 0.1 | 0.08 stops, 4 pixels | about 19% | box blurs and 2D textures |
| Bilateral grid | none | about 10% | 3D textures, and a harder first pass |

- **The plain blur fails as the technical design warns:** a bright glow around the tree, the ground line and the face, and the gaps in the leaves blown to white.
- **The guided filter and the grid both look clean** at normal size on the tree and the face.
- **Claude Code recommended the guided filter at ε = 0.1, and Nevada agreed.** Its halo is too small to see. It needs only box blurs and 2D textures, so the same WGSL runs on WebGPU, on WebGL2 for macOS 13 to 15, and in export. Its formula is a few averages, easy to write in the appendix, where the grid's result depends on how brightness is split into bins and on the photo's darkest pixel. The grid stays as the fallback if Nevada's photos show halos.
- One measurement was wrong at first: summing 400,000 values in single precision gave nonsense for how much of the texture the base follows. Redone in double precision.

## What was built (slice 4b)

- **The guide's image.** Rust scales the decoded photo to fit 1024 pixels, once, beside the preview. The Editor, the Viewer and export all build the guide from that same image, so the preview and export match however large the photo is.
- **The guide is four passes of the one shader** (`shaders/guide.wgsl`), picked by a `draw_pass` uniform, so WebGL2 still runs a single translated program. It stores a and b per guide pixel; a pixel's base is a × l + b with its own full-size log luminance l, which keeps edges sharp however small the guide is.
- **When it's rebuilt:** when the photo or the white balance changes. Exposure shifts every log luminance equally, so it's added to the base as the pixel is drawn, and dragging Brightness, Highlights or Shadows only changes uniforms.
- **The formula:** highlights moves each pixel by up to 2 stops at ±100, weighed by x³, where x is the base as linear light raised to 1/2.2; shadows by (1 − x)³. Highlights runs first and moves the base with its gain, so shadows works on its result. These constants are a first choice, for Nevada to try on real photos.
- **Export's tiles and the Viewer's zoomed-in parts** sample the whole photo's guide where they are, so tiles join without seams. A test renders a photo larger than one tile with highlights and shadows.

## PRF-6 on Windows

`measure.py --sliders` on Nevada's Windows PC, in a 1440 × 900 window, dragging every slider for two seconds on 12, 24 and 48 megapixel photos. The drags leave each slider where it stops, so by Temperature, highlights and shadows are set and the guide is rebuilt on every frame, the worst case.

- **WebGPU:** at least 114 frames a second for every slider at every size; one frame over 33 ms, the first drag of the first photo.
- **WebGL2:** at least 119 frames a second; none over 33 ms.
- **Opening a photo** first took 0.26 to 0.54 s, against 0.14 to 0.32 s before, because the guide's image is a second scaling of the whole photo. Done side by side with the preview's scaling, it's 0.20 to 0.44 s. PRF-6 sets no target for opening.

## PRF-6 and the render check on the Mac

Nevada asked the Mac session to measure, on the MacBook (M1 Max, macOS 26.6.2, a 120 Hz display), with the same script.

- **Both backends hold 60 frames a second,** WebKit's cap, for every slider at 12, 24 and 48 MP. That includes Temperature with highlights and shadows set, whose longest frame was 22 ms.
- **One frame over 33 ms on each backend:** Highlights on the 12 MP photo, the first drag that needs the guide (77 ms on WebGPU, 55 on WebGL2). It looks like one-time setup at first use; the same slider stays at 18 to 21 ms on the larger photos.
- **Opening** took 0.21 to 0.51 s.
- **WebGL2 on the Mac has the extension the guide needs** (`EXT_color_buffer_float`), and the render check matches export with difference 0 on WebGPU and WebGL2.
- Claude Code's request gave the Mac session a contradictory build command (`--no-bundle --bundles app`); on macOS, `--bundles app` replaces `--no-bundle`, as CLAUDE.md says.

## Nevada's photos

Nevada tried both sliders on their own photos: "As far as I can tell, it looks good. They don't *feel* too strong or too weak or too wide." Nevada asked whether the strengths fit generally accepted practice. Claude Code's answer:
- The method is the one established editors use: an edge-keeping brightness map, with a gain that works mainly at one end.
- Up to 2 stops at the extremes and about 1 on a typical backlit face is within the usual range, a little gentler than Lightroom's Shadows at +100, which also keeps noise down.
- No editor can recover a sky that's pure white in a JPEG.

The first-choice constants stay.

## Caught along the way

- **WebGL2 drew nothing at first.** The render check in the real web view passed on WebGPU and failed on WebGL2 by a full 255 levels. The uniform block had grown from 80 to 100 bytes, and GLSL's block layout rounds a struct up to a multiple of 16, so WebGL2 wanted 112 and refused to draw from a 100-byte buffer. WebGPU accepts 100. The preview now rounds the buffer up, with a test. It had never shown because 80 is a multiple of 16.
- **The generated WebGL2 names were wrong too.** `gl-names.json` took the first texture naga listed as the photo; with three textures, that was the guide. It now names each texture by its name in the WGSL.
- `from` is a reserved word in WGSL.

## The appendix

- **The formulas appendix now has the guide, highlights and shadows** (edit-format #9, merged by Nevada). Two points were listed for the design session: whether a fixed 1024-pixel guide fits the specification's "never in pixels", and that the guide covers the photo before the crop, which slice 5 builds.
- **Their example values give each pixel's base** beside its color, since the result depends on the neighborhood. Seura's render tests check its definitions against them by giving that base directly. The shaders can't be handed a base, so for these two the GPU is checked against the definitions by the other render tests.
