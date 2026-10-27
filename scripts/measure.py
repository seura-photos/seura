"""Measures the browsing and scanning targets with a release build (M2
slice 10; functional spec, "Performance targets").

Generates test libraries of photo-like JPEGs, then runs the app with
SEURA_DATA set to a Seura folder of its own (so your real library, settings,
and thumbnails are never touched) and SEURA_MEASURE=1, which makes it
measure itself and quit:

1. 1,000 photos of 12 megapixels in one folder, with an empty cache: the
   first scan and its thumbnails (PRF-3);
2. the same again, with the cache built: launch to a usable grid (PRF-1),
   opening the folder (PRF-2), scrolling (PRF-4), next photo (PRF-5);
3. 50,000 small photos in 50 folders: memory (PRF-7), after the first scan,
   and searching as you type (FND-3).

Build first, from ui/:
    Windows: npm run tauri -- build --no-bundle
    macOS:   npm run tauri -- build --bundles app
Then, from the repository root:
    python3 scripts/measure.py [--work FOLDER] [--small]

The libraries take about 5 GB in the work folder (by default
`target/measure`), are made once, and are reused by later runs; delete the
folder to free the space. --small uses 1,000 photos of 3 megapixels and 10,000
small ones instead, for a quick check. Prints the results as a Markdown
table and saves them, with every number, as results.json in the work folder.
"""

# `int | None` annotations on the Python 3.9 that comes with macOS.
from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import subprocess
import sys
import time
from pathlib import Path

if platform.system() == "Windows":
    APP = Path("target/release/seura-photos.exe")
elif platform.system() == "Darwin":
    # Inside the bundle, so WebKit sees the app's real identifier.
    APP = Path("target/release/bundle/macos/Seura Photos.app/Contents/MacOS/seura-photos")
else:
    sys.exit("Seura runs on Windows and macOS only.")

# Each run must finish in this many seconds.
LIMIT = 60 * 60


def make_library(folder: Path, photos: int, width: int, height: int, per_folder: int | None):
    """Makes a test library once; a finished one is reused."""
    done = folder / ".complete"
    if done.exists():
        return
    if folder.exists():
        shutil.rmtree(folder)
    print(f"Making {photos:,} photos of {width} x {height} in {folder}...", flush=True)
    arguments = [str(folder), str(photos), str(width), str(height)]
    if per_folder:
        arguments.append(str(per_folder))
    subprocess.run(
        ["cargo", "run", "--quiet", "--release", "-p", "seura-library",
         "--example", "make_library", "--", *arguments],
        check=True,
    )
    done.write_text("")


def run(data: Path, library: Path, fresh: bool, label: str) -> dict:
    """Runs the app measuring itself on `library`, with Seura's folder at
    `data`, emptied first when `fresh`."""
    if fresh and data.exists():
        shutil.rmtree(data)
    data.mkdir(parents=True, exist_ok=True)
    (data / "settings.json").write_text(
        json.dumps({"libraryFolders": [str(library.resolve())], "checkForUpdates": False})
    )
    results = data / "measurement.json"
    results.unlink(missing_ok=True)
    print(f"{label}...", flush=True)
    started = time.monotonic()
    env = {**os.environ, "SEURA_DATA": str(data.resolve()), "SEURA_MEASURE": "1"}
    try:
        subprocess.run([str(APP)], env=env, timeout=LIMIT, check=False,
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except subprocess.TimeoutExpired:
        sys.exit(f"{label} didn't finish within {LIMIT // 60} minutes.")
    if not results.exists():
        sys.exit(f"{label}: the app wrote no measurement.json.")
    measured = json.loads(results.read_text())
    measured["wallSeconds"] = round(time.monotonic() - started, 1)
    if "error" in measured:
        sys.exit(f"{label}: {measured['error']}")
    return measured


def table(results: dict) -> str:
    cold, warm, large = results["cold"], results["warm"], results["large"]
    scan = cold.get("firstScan") or {}
    large_scan = large.get("firstScan") or {}
    mb = lambda bytes: f"{bytes / 1e6:,.0f} MB"
    rows = [
        ("PRF-1", "Launch to a usable grid, cache built", "under 2 s (aim 1 s)",
         f"{warm['launchToUsableMs'] / 1000:.2f} s"),
        ("PRF-2", f"Open a folder of {warm['openFolder']['photos']:,} photos, cache built",
         "grid full in under 1 s (aim 0.5 s)", f"{warm['openFolder']['ms'] / 1000:.2f} s"),
        ("PRF-3", f"First scan of {scan.get('photos', 0):,} photos, making thumbnails",
         "at least 30 a second (aim 60)", f"{scan.get('photosPerSecond', 0):.0f} a second"),
        ("PRF-4", "Scrolling the grid", "60 frames a second",
         f"{warm['scroll']['framesPerSecond']:.0f} a second "
         f"(95% of frames within {warm['scroll']['p95FrameMs']:.0f} ms)"),
        ("PRF-5", "Next photo in the Viewer", "under 150 ms (aim 75 ms)",
         f"median {warm['nextPhoto']['medianMs']:.0f} ms, "
         f"slowest {warm['nextPhoto']['slowestMs']:.0f} ms"),
        ("PRF-7", f"Memory with a {large_scan.get('photos', 0):,}-photo library",
         "under 1 GB (aim 500 MB)",
         f"{mb(large['memory']['coreBytes'])} (peak {mb(large['memory']['corePeakBytes'])}), "
         "core process only"),
        ("FND-3", f"Search as you type, {large['search']['photos']:,} photos",
         "within 200 ms", f"slowest letter {large['search']['slowestMs']:.0f} ms"),
    ]
    lines = ["| Target | Measure | Goal | Result |", "| --- | --- | --- | --- |"]
    lines += [f"| {a} | {b} | {c} | {d} |" for a, b, c, d in rows]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description="Measure the performance targets.")
    parser.add_argument("--work", type=Path, default=Path("target/measure"))
    parser.add_argument("--small", action="store_true", help="smaller libraries, for a quick check")
    args = parser.parse_args()
    if not APP.exists():
        sys.exit(f"No release build at {APP}; see the instructions at the top of this script.")

    folder_photos, folder_size = (1_000, (2016, 1512)) if args.small else (1_000, (4032, 3024))
    # At least FND-3's 10,000, for the search.
    large_photos = 10_000 if args.small else 50_000
    tag = "small" if args.small else "full"
    folder_library = args.work / f"folder-{tag}"
    # Named by size, so a change of size makes a new one.
    large_library = args.work / f"large-{large_photos}"
    make_library(folder_library, folder_photos, *folder_size, None)
    make_library(large_library, large_photos, 320, 240, 1_000)

    data = args.work / "data"
    results = {
        "machine": f"{platform.system()} {platform.release()} ({platform.machine()})",
        "cold": run(data, folder_library, True, "First scan, empty cache"),
        "warm": run(data, folder_library, False, "Browsing, cache built"),
        "large": run(args.work / "data-large", large_library, True, "Large library"),
    }
    (args.work / "results.json").write_text(json.dumps(results, indent=2))
    print()
    print(table(results))
    print(f"\nEvery number: {args.work / 'results.json'}")


if __name__ == "__main__":
    main()
