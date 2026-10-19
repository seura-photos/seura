# M1 index benchmarks

Status: recorded 2026-10-18 · Decision [0010](../decisions/0010-in-memory-index.md)

The benchmark gate for decision 0010: does an in-memory index with a snapshot file meet the functional spec's targets, or does the index need SQLite? Run with `cargo bench -p seura-library --bench index`; CI runs it on every pull request and fails if a target is missed.

## Result

Every target passes on every machine, by a factor of at least 5: the closest is the snapshot load on CI's Windows runner, 95 ms against 500 ms. **The in-memory index holds; decision 0010 is confirmed.** Memory came out at 340 bytes per photo, close to the decision's estimate of about 300.

## What is measured

Synthetic libraries of 10,000, 50,000, and 100,000 photos, generated in memory and shaped like a family library: dated folders of about 200 photos plus one of 1,000, names like `IMG_4821.JPG`, 300 tags in a hierarchy with 40% of photos tagged, a third rated, 15% edited.

- **Memory:** bytes held by the index after loading it from a snapshot, counted exactly by a counting allocator.
- **Snapshot:** its size; saving it (encode and atomic write); loading it (read, decode, rebuild the folder, tag, and search maps). Medians of 5 runs.
- **1,000-photo folder:** listing the folder's photos, the index's part of opening it. Median of 21.
- **Filter all:** the whole library filtered by rating, a tag, edited, and a date range. Median of 21.
- **Search:** typing `img_12` and then `family` one letter at a time; the slowest keystroke. Medians of 5 per keystroke.
- **Scan:** 10,000 small stand-in files on disk, a first scan and a rescan with nothing changed. No thumbnails or metadata yet (slice 4), so this isn't PRF-3.

## Targets

The spec's performance targets are for the whole app, so the index gets a budget within each. These budgets are Claude's proposal, for Nevada to accept or change.

| Target | Requirement | Index budget |
| --- | --- | --- |
| Index memory, 100,000 photos | PRF-7: the app under 1 GB at 50,000 | ≤ 100 MB |
| Snapshot load, 100,000 photos | PRF-1: launch to a usable grid under 2 s | ≤ 500 ms |
| Open a 1,000-photo folder, 100,000 photos | PRF-2: grid full in under 1 s | ≤ 10 ms |
| Search, slowest keystroke, 10,000 photos | FND-3: under 200 ms | ≤ 200 ms |

## Results

The reference hardware in the spec is a mid-range laptop from about 2020 with integrated graphics. None of these machines is that; Nevada's are faster and GitHub's runners are shared virtual machines. The margins are large enough that the conclusion doesn't depend on it, and the M2 performance work measures on reference-class hardware.

### Nevada's Windows PC

AMD Ryzen 7 7800X3D (8 cores, 16 threads), Windows 11.

| Photos | Memory | Per photo | Snapshot | Save | Load | 1,000-photo folder | Filter all | Search, slowest keystroke |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10,000 | 4.1 MB | 412 B | 0.7 MB | 4.08 ms | 4.79 ms | 3 µs | 25 µs | 267 µs |
| 50,000 | 17.0 MB | 340 B | 3.8 MB | 14 ms | 22 ms | 3 µs | 152 µs | 1.34 ms |
| 100,000 | 34.0 MB | 340 B | 7.5 MB | 24 ms | 48 ms | 3 µs | 374 µs | 2.70 ms |

Scan of 10,000 files: first 330 ms, rescan 11 ms.

### Nevada's MacBook Pro

MacBookPro18,2, Apple M1 Max (10 cores), 32 GB, macOS 26.6.2. Run by the Claude Code session on the Mac.

| Photos | Memory | Per photo | Snapshot | Save | Load | 1,000-photo folder | Filter all | Search, slowest keystroke |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10,000 | 4.1 MB | 412 B | 0.7 MB | 14 ms | 5.04 ms | 5 µs | 46 µs | 501 µs |
| 50,000 | 17.0 MB | 340 B | 3.8 MB | 21 ms | 17 ms | 3 µs | 221 µs | 1.86 ms |
| 100,000 | 34.0 MB | 340 B | 7.5 MB | 29 ms | 36 ms | 3 µs | 457 µs | 3.68 ms |

Scan of 10,000 files: first 162 ms, rescan 19 ms.

### GitHub Actions runners

Shared virtual machines: `windows-latest` (x86-64, 4 threads) and `macos-latest` (Apple silicon, 3 threads). CI run 39182460226, commit `4cd7ba5`.

| Runner | Photos | Memory | Per photo | Snapshot | Save | Load | 1,000-photo folder | Filter all | Search, slowest keystroke |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Windows | 10,000 | 4.1 MB | 412 B | 0.7 MB | 27 ms | 7.97 ms | 7 µs | 47 µs | 455 µs |
| Windows | 50,000 | 17.0 MB | 340 B | 3.8 MB | 47 ms | 43 ms | 5 µs | 270 µs | 2.24 ms |
| Windows | 100,000 | 34.0 MB | 340 B | 7.5 MB | 68 ms | 95 ms | 6 µs | 654 µs | 7.23 ms |
| macOS | 10,000 | 4.1 MB | 412 B | 0.7 MB | 10 ms | 4.22 ms | 4 µs | 44 µs | 639 µs |
| macOS | 50,000 | 17.0 MB | 340 B | 3.8 MB | 22 ms | 29 ms | 4 µs | 256 µs | 2.04 ms |
| macOS | 100,000 | 34.0 MB | 340 B | 7.5 MB | 41 ms | 71 ms | 4 µs | 928 µs | 5.49 ms |

Scan of 10,000 files: Windows first 306 ms, rescan 18 ms; macOS first 203 ms, rescan 41 ms.

## Against the targets

| Target | Budget | Windows PC | MacBook Pro | CI Windows | CI macOS |
| --- | ---: | ---: | ---: | ---: | ---: |
| Index memory, 100,000 photos | 100 MB | 34.0 MB | 34.0 MB | 34.0 MB | 34.0 MB |
| Snapshot load, 100,000 photos | 500 ms | 48 ms | 36 ms | 95 ms | 71 ms |
| Open a 1,000-photo folder, 100,000 photos | 10 ms | 0.003 ms | 0.003 ms | 0.006 ms | 0.004 ms |
| Search, slowest keystroke, 10,000 photos | 200 ms | 0.27 ms | 0.50 ms | 0.46 ms | 0.64 ms |

## What this doesn't cover yet

- Thumbnail generation and the first scan's real speed (PRF-3), with decoding: slice 4.
- The whole launch (PRF-1) and memory of the whole app (PRF-7), measured with the app running: M2.
- Watching for changes (LIB-4) and the cost of keeping the index current while the app runs: M2.
