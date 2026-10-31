# M4 Editor and browsing targets

Status: recorded 2026-10-30 on both machines · [Functional spec, "Performance targets"](../functional-spec.md#performance-targets)

M4 slice 11 measures PRF-6, the Editor's live preview, on both machines and on both of the preview's paths: WebGPU, and the WebGL2 that macOS 13 to 15 use. Every earlier target is measured again, to check that M4 hasn't slowed them; they were last recorded in [M3](m3-search.md). As before, the targets are for a mid-range laptop from about 2020, and on Nevada's much faster machines the aim is about half of each.

## The Editor's live preview (PRF-6)

| Preview | Goal | Windows PC | Mac |
| --- | --- | --- | --- |
| WebGPU | at least 30 frames a second, 12 to 48 MP | 120 a second, every slider and size | 60 a second, every slider and size (WebKit's limit) |
| WebGL2 | the same | 120 a second, every slider and size | 60 a second, every slider and size (WebKit's limit) |

Met on both machines and on both paths, with every operation M4 built turned on by the end of the drags. Each figure is the slowest of every slider at every size.

- **Every frame the display shows.** Windows' display runs at 120 Hz. WebKit gives a page at most 60 animation frames a second, even on the Mac's 120 Hz display, so 60 is the Mac's ceiling, not its GPU's limit.
- **Long frames.** On Windows, no frame took over 33 ms; the longest took 9 ms. On the Mac, one frame on each path did, both on the 12 MP photo's Saturation slider: 36 ms on WebGPU and 39 ms on WebGL2. Elsewhere the Mac's longest frame was 31 ms, and 95% of its frames were within 17 to 19 ms.
- **The photo's size shows only in opening it,** since the preview is decoded to the window's size:

| Opening a photo in the Editor | 12 MP | 24 MP | 48 MP |
| --- | --- | --- | --- |
| Windows, WebGPU | 0.21 s | 0.24 s | 0.41 s |
| Windows, WebGL2 | 0.33 s | 0.26 s | 0.43 s |
| Mac, WebGPU | 0.20 s | 0.30 s | 0.50 s |
| Mac, WebGL2 | 0.23 s | 0.31 s | 0.52 s |

PRF-6 sets no target for opening. On Windows, the first photo on WebGL2 opens more slowly than the second, larger one, as the web view sets up WebGL2.

## Browsing again

| Target | Measure | Goal (aim on these machines) | Windows PC | Mac |
| --- | --- | --- | --- | --- |
| PRF-1 | Launch to a usable grid, cache built | under 2 s (1 s) | 0.57 s | 0.49 s |
| PRF-2 | Open a folder of 1,000 photos, cache built | under 1 s (0.5 s) | 0.06 s | 0.05 s |
| PRF-3 | First scan of 1,000 12-megapixel photos, with their thumbnails | at least 30 a second (60) | 118 a second | 92 a second |
| PRF-4 | Scrolling the grid | 60 frames a second | 117 a second; 95% of frames within 8 ms | 60 a second, WebKit's limit; 95% within 17 ms |
| PRF-5 | Next photo in the Viewer | under 150 ms (75 ms) | median 77 ms, slowest 85 ms | median 40 ms, slowest 43 ms |
| PRF-7 | Memory with a 50,000-photo library | under 1 GB (500 MB) | 80 MB (peak 144 MB) | 272 MB (peak; macOS gives only the peak) |
| FND-3 | Search as you type: the slowest letter | within 200 ms at 10,000 photos (100 ms) | 38 ms at 10,000 matches; 152 ms at 50,000 | 31 ms at 10,000 matches; 112 ms at 50,000 |

Every target is still met on both machines.

- **The Mac:** every result is within the half-target aim, and about the same as in M3.
- **Windows:** every result is within the aim but PRF-5's.

**PRF-5 on Windows is slower than in M3** (#190). The median rose from 62 ms in M3 to 77 to 85 ms over three full runs, just over the 75 ms aim; the Mac didn't change.

- **The cause:** since slice 4 (highlights and shadows), the Viewer also fetches each photo's guide image, about 6 MB, and large transfers through WebView2 are slow.
- **The test:** two `--small` builds measured back to back. With the guide fetched, the median was 70 ms; with the photo reused as its own guide, 52 ms.
- **Fixing it later:** Nevada decided to record PRF-5 as it is and fix it in M5 or in M7's performance audit. Two possible fixes are to fetch the guide only for photos with highlights or shadows set, or after the photo is drawn.

**Search is measured differently now.** M3's table gave the slowest of all nine letters, which at 50,000 photos is the first letter, matching every photo. Here FND-3's row gives both:

- the slowest letter at 10,000 matches, FND-3's size;
- the slowest overall.

On Windows the 50,000-photo letters varied between runs, from 124 to 173 ms. On the Mac the slowest was 112 ms, against 119 ms in M3.

**Memory** is about the same as in M3 on Windows. The Mac's peak went from 236 MB to 272 MB, well within PRF-7.

**The first scan on Windows** (PRF-3) is lower than in M3's 133 a second: 104, 112 and 118 a second over three runs.

- The scan's code didn't change in M4.
- The first two runs had other work on the computer, which Nevada pointed out. The scan uses every core, so it's the figure most affected.
- The third run, recorded here, was on a mostly idle computer, with a video playing.

## Machines

**Windows PC:** AMD Ryzen 7 7800X3D (8 cores), 32 GB, NVIDIA GeForce RTX 4070 SUPER, Windows 11 Home; the app's window 1440 × 900 at 100% scale, on a 120 Hz display. Measured by Claude Code, on a mostly idle computer with a video playing.

- The build was main after #189, with #191, which makes the Editor available in release builds; that changes no measured path.

**Mac:** 14-inch MacBook Pro (2021), Apple M1 Max (10 cores), 32 GB, macOS 26.6.2; the built-in display (ProMotion, up to 120 Hz); the app's window 1440 × 868 at 2× scale. Measured by the Mac Claude session, with a release build of main after #189 (`--bundles app`).

## How it's measured

As in [M2](m2-browsing.md#how-its-measured) and [M3](m3-search.md#how-its-measured), with `scripts/measure.py`, a release build, and generated libraries in a Seura folder of their own, so no real library is touched. PRF-6 is measured with `SEURA_MEASURE=sliders`, in two runs: one on WebGPU, and one with `SEURA_PREVIEW=webgl2`.

- **The library:** one generated photo each of 12, 24 and 48 megapixels (4032 × 3024, 6000 × 4000 and 8064 × 6048), each in a folder of its own.
- **Effects on:** each photo is opened in the Editor and every effect is turned on.
- **The drags:** each slider of the panel is dragged back and forth for two seconds, a step every frame, timing the frames.
  - Each slider stays where its drag left it, so the last drags have every operation on.
  - The drag sends only the sliders' input events, which redraw the photo, never their change events, which save it.
- **Effects off:** after the drags, the effects are turned off again. Turning them on and off saves, in the edit files beside the generated photos, before and after the drags.

The numbers above come from the full run.
