# M6 wrap-up: every target, and next photo in Review

Status: recorded 2026-11-09 on both machines · [Functional spec, "Performance targets"](../functional-spec.md#performance-targets)

M6 slice 7 times next photo in Review against PRF-5, and measures every other target again, to check that M6 hasn't slowed them. They were last recorded in [M5](m5-wrap-up.md).

The machines are Nevada's Windows PC (Windows 11, NVIDIA GPU) and a MacBook Pro (M1 Max, macOS 26). Both ran release builds at f892f36. On Windows a YouTube video was playing during the run. The Mac's was by the Mac session. As before, the targets are for a mid-range laptop from about 2020, and on these much faster machines the aim is about half of each.

## Browsing

| Target | Measure | Goal (aim on these machines) | Windows PC | Mac |
| --- | --- | --- | --- | --- |
| PRF-1 | Launch to a usable grid, cache built | under 2 s (1 s) | 0.64 s | 0.49 s |
| PRF-2 | Open a folder of 1,000 photos, cache built | under 1 s (0.5 s) | 0.07 s | 0.06 s |
| PRF-3 | First scan of 1,000 12-megapixel photos, with their thumbnails | at least 30 a second (60) | 124 a second | 97 a second |
| PRF-4 | Scrolling the grid | 60 frames a second | 118 a second; 95% of frames within 8 ms | 58 a second, under WebKit's limit of 60; 95% within 18 ms |
| PRF-5 | Next photo in the Viewer | under 150 ms (75 ms) | median 56 ms, slowest 60 ms | median 41 ms, slowest 43 ms |
| PRF-5 | Next photo in Review | under 150 ms (75 ms) | median 49 ms, slowest 72 ms | median 30 ms, slowest 35 ms |
| PRF-7 | Memory with a 50,000-photo library | under 1 GB (500 MB) | 68 MB (peak 145 MB) | 285 MB (peak; macOS gives only the peak) |
| FND-3 | Search as you type: the slowest letter, 50,000 photos | within 200 ms at 10,000 photos (100 ms) | 172 ms in the full run; 118 to 134 ms in five repeats | 120 ms |

All are met except PRF-4 on the Mac by two frames a second, and within the aims except FND-3 at 50,000 photos, whose target is at 10,000.

- **PRF-4 on the Mac** has read 59 and 60 in earlier milestones. WebKit limits the page to 60 frames a second, so any dropped frame reads as under the target. This run didn't look at which frames were late.
- **Next photo in Review** is timed as the Viewer's is: 20 moves, half a second on each, until the photo is drawn. It's the same drawing code, and it reads ahead to the photos on either side.
- **The first search number on Windows** was slower than M5's 120 ms, so that library was run five more times, to see whether M6's marks had cost anything. They hadn't: the repeats came in at 118 to 134 ms.
- **In one of those repeats on Windows, the Viewer never drew a photo,** and showed no message. It didn't happen in the five that followed, and the cause is unknown ([#258](https://github.com/seura-photos/seura/issues/258)).

## The Editor's live preview (PRF-6)

| Preview | Goal | Windows PC | Mac |
| --- | --- | --- | --- |
| WebGPU, sliders | at least 30 frames a second, 12 to 48 MP | 118 a second at the slowest | 60 a second at the slowest (WebKit's limit) |
| WebGL2, sliders | the same | 120 a second | 60 a second |

Met on both machines and both paths. On Windows no slider's frame took over 33 ms, and a preset previewed over and over held 120 a second, with the core answering in 3 ms at most.

| Slowest to open a photo in the Editor | Windows PC | Mac |
| --- | --- | --- |
| WebGPU and WebGL2, 12 to 48 MP | 0.44 s | 0.5 s |

## Exporting

| Measure | Windows PC | Mac |
| --- | --- | --- |
| 20 photos of 12 MP, with an applied preset's edits, as JPEG at quality 90 and their own size | 9.7 s, 0.48 s a photo | 6.9 s, 0.34 s a photo |

There's no target.

## Where the app writes

`scripts/app_data_audit.py --release --export` passes on both machines, on a run that now also marks a photo for deletion and rates another. Nothing with Seura's name was written outside the run's own folders, and `marks.json` is in its Seura folder. Metal's shader cache for Seura on the Mac didn't grow; [#231](https://github.com/seura-photos/seura/issues/231) covers these GPU caches.
