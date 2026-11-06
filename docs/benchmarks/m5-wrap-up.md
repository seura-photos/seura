# M5 wrap-up: every target, and exporting

Status: recorded 2026-11-05 on both machines · [Functional spec, "Performance targets"](../functional-spec.md#performance-targets)

M5 slice 9 measures every target again, to check that M5 hasn't slowed them. They were last recorded in [M4](m4-editor.md). PRF-6 gains a preset case: a preset previewed over and over, as hovering along the list does. Exporting is timed too, though the spec sets no target for it; [m5-export.md](m5-export.md) has the encoder's own numbers from slice 2.

The machines are Nevada's Windows PC (Windows 11, NVIDIA GPU) and a MacBook Pro (M1 Max, macOS 26). The Windows run was on a release build at 11226ca, with the PC kept quiet. The Mac's was by the Mac session, on a release build at the same commit. As before, the targets are for a mid-range laptop from about 2020, and on these much faster machines the aim is about half of each.

## Browsing

| Target | Measure | Goal (aim on these machines) | Windows PC | Mac |
| --- | --- | --- | --- | --- |
| PRF-1 | Launch to a usable grid, cache built | under 2 s (1 s) | 0.60 s | 0.51 s |
| PRF-2 | Open a folder of 1,000 photos, cache built | under 1 s (0.5 s) | 0.07 s | 0.06 s |
| PRF-3 | First scan of 1,000 12-megapixel photos, with their thumbnails | at least 30 a second (60) | 130 a second | 97 a second |
| PRF-4 | Scrolling the grid | 60 frames a second | 118 a second; 95% of frames within 8 ms | 60 a second, WebKit's limit; 95% within 17 ms |
| PRF-5 | Next photo in the Viewer | under 150 ms (75 ms) | median 63 ms, slowest 74 ms | median 41 ms, slowest 51 ms |
| PRF-7 | Memory with a 50,000-photo library | under 1 GB (500 MB) | 78 MB (peak 142 MB) | 262 MB (peak; macOS gives only the peak) |
| FND-3 | Search as you type: the slowest letter, 50,000 photos | within 200 ms at 10,000 photos (100 ms) | 120 ms | 92 ms |

All met, and within the aims except FND-3 at 50,000 photos on Windows. FND-3's target is at 10,000; in M4, 10,000 matches took 38 ms there.

- **PRF-5 on Windows is back within the aim:** a median of 63 ms, against 77 ms in M4's final run (#190). Nothing in M5 set out to change it. #190 measured the guide image costing about 18 ms, and that's still fetched, so this may be the computer more than the app. #190 stays open for M7's audit.

## The Editor's live preview (PRF-6)

| Preview | Goal | Windows PC | Mac |
| --- | --- | --- | --- |
| WebGPU, sliders | at least 30 frames a second, 12 to 48 MP | 120 a second, every slider and size | 60 a second, every slider and size (WebKit's limit) |
| WebGL2, sliders | the same | 120 a second | 60 a second |
| WebGPU, a preset previewed over and over | the same | 120 a second; the core answers in 3.4 ms at most | 60 a second; 4 ms at most |
| WebGL2, a preset previewed over and over | the same | 120 a second; 3.2 ms at most | 60 a second; 4 ms at most |

Met on both machines and both paths. For the preset, the script focuses and leaves it every few frames for two seconds, and each time the Editor asks the core for the preview and draws it. No frame took over 33 ms on either machine. The answer times are the slowest of ten asked on their own.

| Opening a photo in the Editor | 12 MP | 24 MP | 48 MP |
| --- | --- | --- | --- |
| Windows, WebGPU | 0.20 s | 0.22 s | 0.38 s |
| Windows, WebGL2 | 0.31 s | 0.24 s | 0.46 s |
| Mac, WebGPU | 0.17 s | 0.26 s | 0.47 s |
| Mac, WebGL2 | 0.19 s | 0.27 s | 0.49 s |

PRF-6 sets no target for opening. As in M4, the first photo on WebGL2 opens more slowly on Windows than the second, larger one, as the web view sets up WebGL2.

## Exporting

| Measure | Windows PC | Mac |
| --- | --- | --- |
| 20 photos of 12 MP, with an applied preset's edits, as JPEG at quality 90 and their own size | 9.4 s, 0.47 s a photo | 7.0 s, 0.35 s a photo |

There's no target. The script saves a preset, applies it to the folder, and exports, with the export folder in the run's own settings: `SEURA_MEASURE=export`, run by [`scripts/measure.py`](../../scripts/measure.py). On the Mac, the core's memory peaked at 679 MB during this run, against 262 MB browsing 50,000 photos. Exporting holds a whole 12-megapixel photo, decoded and rendered at full size, so this is expected; PRF-7 is about browsing.
