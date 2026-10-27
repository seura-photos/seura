# M3 search and browsing targets

Status: recorded 2026-10-27 on both machines · [Functional spec, "Performance targets" and FND-3](../functional-spec.md#performance-targets)

M3 slice 11 runs the measurement script again, now with search as you type (FND-3: "results update as the user types, within 200 milliseconds for a library of 10,000 photos"). The browsing targets are measured again too, to check that M3 hasn't slowed them; they were first recorded in [M2](m2-browsing.md). As there, the targets are for a mid-range laptop from about 2020, and on Nevada's much faster machines the aim is about half of each.

## Search as you type (FND-3)

| Library | Goal | Windows PC | Mac |
| --- | --- | --- | --- |
| 10,000 photos (FND-3's size) | within 200 ms (aim 100 ms) | 46 ms | 79 ms |
| 50,000 photos (a stress test) | not set | 142 ms | 119 ms |

Each figure is the slowest of the nine letters of `img_01234`, typed one at a time. The first letters match every photo, and the last matches one.

Met on both machines, and within the half-target aim at FND-3's size. Even at five times that size, every letter is within 200 ms.

**Measuring search found three slow paths, which slice 11 fixed** (journal, `2026-10-27-wrap-up.md`). Before the fixes, a search matching all 50,000 photos took about 430 ms a letter on Windows. At 10,000 photos the slowest letter took 68 ms, so FND-3 was already met. The index's own search takes under 2 ms. The time went on what's around it:

- sorting the results, which compared names on every comparison instead of only to break a tie;
- making each result's thumbnail cache key out of 16 small strings;
- queuing every result's thumbnail again on each letter. That sent the UI a notice for each thumbnail already made: 50,000 a letter, which the next letter's results waited behind.

What remains at 50,000 is mostly sending the results to the web view: about 8 MB as JSON. It's well within the target, so it's left as it is.

## Browsing again

| Target | Measure | Goal (aim on these machines) | Windows PC | Mac |
| --- | --- | --- | --- | --- |
| PRF-1 | Launch to a usable grid, cache built | under 2 s (1 s) | 0.61 s | 0.48 s |
| PRF-2 | Open a folder of 1,000 photos, cache built | under 1 s (0.5 s) | 0.07 s | 0.06 s |
| PRF-3 | First scan of 1,000 12-megapixel photos, with their thumbnails | at least 30 a second (60) | 133 a second | 90 a second |
| PRF-4 | Scrolling the grid | 60 frames a second | 119 a second; 95% of frames within 8 ms | 59 a second, WebKit's limit; 95% within 17 ms |
| PRF-5 | Next photo in the Viewer | under 150 ms (75 ms) | median 62 ms, slowest 68 ms | median 40 ms, slowest 55 ms |
| PRF-7 | Memory with a 50,000-photo library | under 1 GB (500 MB) | 75 MB (peak 139 MB) | 236 MB (peak; macOS gives only the peak) |

Every target is still met on both machines, and every result is within the half-target aim. Compared with M2, the times are about the same, within the variation between runs.

The **memory peaks are higher**: Windows' peak went from 93 MB to 139 MB, and the Mac's from 170 MB to 236 MB. The current use on Windows barely moved (70 MB to 75 MB). The difference is the search step, which is new in this run: each letter that matches all 50,000 photos builds and sends a full set of results. On the Mac only the peak is reported, so its figure includes it.

**Memory with large photos, not part of PRF-7.** The run with 1,000 12-megapixel photos, through the Viewer, peaked at 284 MB on Windows and 608 MB on the Mac. That's within PRF-7's 1 GB, but more than the 50,000 small photos use. PRF-7 is set for a library's size, and decoding large photos for the Viewer costs more, so it's noted here for M4's Editor, which holds full-size images.

**Windows PC:** AMD Ryzen 7 7800X3D (8 cores), 32 GB, NVIDIA GeForce RTX 4070 SUPER, Windows 11 Home; the app's window 1440 × 900 at 100% scale, on a 120 Hz display. Measured by Claude Code.

**Mac:** 14-inch MacBook Pro (2021), Apple M1 Max (10 cores), 32 GB, macOS 26.6.2; the built-in display (ProMotion, up to 120 Hz); the app's window 1440 × 868 at 2× scale. Measured by the Mac Claude session, with Homebrew's Python 3.13.

## How it's measured

As in [M2](m2-browsing.md#how-its-measured), with `scripts/measure.py` and a release build, on generated libraries and a Seura folder of their own, so no real library is touched. What's new:

- **Search:** at the end of each run, the script types `img_01234` into the search box a letter at a time, 150 ms apart, through the box's own input event, as typing does. Each letter is timed from when it's typed until its results are in the grid and drawn. The command is also timed on its own, after each letter (`commandMs` in `results.json`), to tell the core's part from the UI's. The search table reports the large library: `--small`'s 10,000 photos for FND-3's size, and the full run's 50,000.
- **`--small`'s large library is now 10,000 photos**, up from 5,000, so the quick check covers FND-3's size. Each test library's folder is named by its size, so changing a size makes a new one.
- **On macOS, the display is kept awake** with `caffeinate` while the script runs. WebKit stops drawing frames while the display sleeps, and that once stalled a run for over five minutes.

The numbers above come from the full run, except the 10,000-photo search, which comes from `--small`.
