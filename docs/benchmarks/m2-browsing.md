# M2 browsing and scanning targets

Status: Windows recorded 2026-10-22; the Mac to come · [Functional spec, "Performance targets"](../functional-spec.md#performance-targets)

The browsing targets measured on a release build, as M2 slice 10 asks: PRF-1, PRF-2, PRF-3, PRF-4, PRF-5, and PRF-7. (PRF-6, the Editor's live preview, belongs to M4.) The targets are for a mid-range laptop from about 2020; on Nevada's much faster machines the aim is about half each target, and a reference-class laptop is tested in the M7 beta.

## Results

| Target | Measure | Goal (aim on these machines) | Windows PC | Mac |
| --- | --- | --- | --- | --- |
| PRF-1 | Launch to a usable grid, cache built | under 2 s (1 s) | 0.53 s | |
| PRF-2 | Open a folder of 1,000 photos, cache built, until every thumbnail on screen is in | under 1 s (0.5 s) | 0.05 s | |
| PRF-3 | First scan of 1,000 12-megapixel photos, with their thumbnails | at least 30 a second (60) | 147 a second | |
| PRF-4 | Scrolling the grid | 60 frames a second | 120 a second, the display's rate; 95% of frames within 8 ms, none over 33 ms | |
| PRF-5 | Next photo in the Viewer, until it's drawn | under 150 ms (75 ms) | median 48 ms, slowest 52 ms | |
| PRF-7 | Memory with a 50,000-photo library | under 1 GB (500 MB) | 70 MB (peak 93 MB) | |

Every target is met on the PC, the closest by a factor of three (PRF-5, against the 150 ms target). The 50,000 small photos were scanned with their thumbnails at 745 a second.

**Windows PC:** AMD Ryzen 7 7800X3D (8 cores), 32 GB, NVIDIA GeForce RTX 4070 SUPER, Windows 11 Home; the app's window 1440 × 900 at 100% scale, on a 120 Hz display.

## How it's measured

`scripts/measure.py`, with a release build (see CLAUDE.md). It generates two test libraries of photo-like JPEGs (smooth gradients with fine noise, so they decode like real photos), sets up a Seura folder of its own with `SEURA_DATA` so no real library is touched, and runs the app three times with `SEURA_MEASURE=1`, which makes it run a fixed script on itself (`ui/src/measure.ts`), write `measurement.json`, and quit:

1. **First scan:** 1,000 photos of 4032 × 3024 in one folder, with an empty cache. PRF-3 is the photos divided by the time from the launch scan starting until it and all thumbnails are done.
2. **Browsing**, with that cache built:
   - PRF-1: from the process starting until the folder tree is on screen;
   - PRF-2: from clicking the 1,000-photo folder until every cell on screen shows its thumbnail;
   - PRF-4: the grid scrolled steadily for 5 seconds (about 1,800 px a second), counting frames;
   - PRF-5: the Viewer moved to the next photo 20 times, pausing half a second on each as a person would (so the neighbors are decoded ahead, as they are in use), each move timed until the photo is drawn.
3. **Memory:** 50,000 photos of 320 × 240 in 50 folders, scanned with their thumbnails, then browsed as in 2; PRF-7 is the core process's memory at the end. Small photos keep the library to 1.4 GB on disk; the index and the app's memory depend on the number of photos, not their size.

What these numbers leave out:

- **PRF-7 counts the core process only**, not the web view's own processes (WebView2's or WebKit's), which the operating system runs separately.
- **PRF-3 here includes the scan**, unlike the thumbnail benchmark (`cargo bench -p seura-library --bench thumbnails`, 105 a second on the PC), which times thumbnails alone with a different number of workers; generated photos also compress differently from real ones.
- **The window size matters** for PRF-2, PRF-4, and PRF-5; these used the app's default window.

In CI, the Measure workflow (run by hand) does the same with smaller libraries on GitHub's runners, for comparing runs rather than judging the targets.
