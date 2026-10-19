//! The index benchmarks (decision 0010; M1 slice 3): memory, snapshot load,
//! filtering, search, and scanning, measured on synthetic libraries and
//! checked against the functional spec's targets.
//!
//! Run with `cargo bench -p seura-library`. Prints a Markdown report (also
//! appended to the GitHub Actions job summary) and fails if a target is missed.

#![allow(
    unsafe_code,
    reason = "a counting allocator measures the index's memory"
)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::fmt::Write as _;
use std::fs;
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use seura_library::synthetic::{LARGE_FOLDER, synthetic};
use seura_library::{Filter, Index, Scope};

/// Counts bytes currently allocated, so the index's size can be measured.
struct Counting;

static LIVE: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LIVE.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        LIVE.fetch_add(new_size, Ordering::Relaxed);
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// A target the results are checked against.
struct Target {
    what: &'static str,
    requirement: &'static str,
    limit: f64,
    unit: &'static str,
}

const SIZES: [usize; 3] = [10_000, 50_000, 100_000];

/// Index memory at 100,000 photos. PRF-7 allows the whole app 1 GB at 50,000;
/// the index gets a tenth of that at twice the size.
const MEMORY: Target = Target {
    what: "Index memory, 100,000 photos",
    requirement: "PRF-7",
    limit: 100.0,
    unit: "MB",
};
/// Loading the snapshot at 100,000 photos: a quarter of PRF-1's 2 seconds
/// from launch to a usable grid, leaving the rest for the window and web view.
const LOAD: Target = Target {
    what: "Snapshot load, 100,000 photos",
    requirement: "PRF-1",
    limit: 500.0,
    unit: "ms",
};
/// Listing a 1,000-photo folder: a hundredth of PRF-2's 1 second, which is
/// mostly for thumbnails.
const FOLDER: Target = Target {
    what: "Open a 1,000-photo folder, 100,000 photos",
    requirement: "PRF-2",
    limit: 10.0,
    unit: "ms",
};
/// Search results as the user types, for 10,000 photos (FND-3 exactly).
const SEARCH: Target = Target {
    what: "Search, slowest keystroke, 10,000 photos",
    requirement: "FND-3",
    limit: 200.0,
    unit: "ms",
};

fn main() {
    let mut report = String::new();
    let mut checks: Vec<(&Target, f64)> = Vec::new();
    writeln!(
        report,
        "## Index benchmarks ({} {}, {} threads)\n",
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::thread::available_parallelism().map_or(1, |n| n.get())
    )
    .unwrap();
    writeln!(report, "| Photos | Memory | Per photo | Snapshot | Save | Load | 1,000-photo folder | Filter all | Search, slowest keystroke |").unwrap();
    writeln!(
        report,
        "| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"
    )
    .unwrap();

    let cache = tempfile::tempdir().expect("a temporary cache folder");
    for count in SIZES {
        let before = LIVE.load(Ordering::Relaxed);
        let index = synthetic(count);
        let built = LIVE.load(Ordering::Relaxed) - before;
        drop(index);

        // Measure the index as it is after a launch: loaded from a snapshot.
        let snapshot = synthetic(count).to_snapshot();
        let before = LIVE.load(Ordering::Relaxed);
        let index = Index::from_snapshot(&snapshot).expect("the snapshot loads");
        let memory = (LIVE.load(Ordering::Relaxed) - before).max(built);

        let save = median(5, || {
            index
                .save_snapshot(cache.path())
                .expect("the snapshot saves")
        });
        let load = median(5, || {
            black_box(Index::load_snapshot(cache.path()).expect("the snapshot loads"));
        });

        let folder = index
            .folder_id(&index.roots()[0].join(LARGE_FOLDER))
            .expect("the large folder exists");
        let open_folder = median(21, || {
            black_box(index.query(Scope::Folder(folder), &Filter::default()));
        });
        let tag = index.tag_id("family/topic 0").expect("a tag");
        let filter = Filter {
            min_rating: 3,
            tags: vec![tag],
            edited: Some(true),
            taken: Some((1_300_000_000, 1_700_000_000)),
        };
        let filter_all = median(21, || {
            black_box(index.query(Scope::All, &filter));
        });
        // Typing "img_12" then "family" (FND-3: results update as the user types).
        let mut search = Duration::ZERO;
        for word in ["img_12", "family"] {
            for end in 1..=word.len() {
                let time = median(5, || {
                    black_box(index.search(&word[..end], &Filter::default()));
                });
                search = search.max(time);
            }
        }

        let megabytes = memory as f64 / 1_000_000.0;
        writeln!(
            report,
            "| {count} | {megabytes:.1} MB | {} B | {:.1} MB | {} | {} | {} | {} | {} |",
            memory / count,
            snapshot.len() as f64 / 1_000_000.0,
            ms(save),
            ms(load),
            ms(open_folder),
            ms(filter_all),
            ms(search)
        )
        .unwrap();

        if count == 100_000 {
            checks.push((&MEMORY, megabytes));
            checks.push((&LOAD, millis(load)));
            checks.push((&FOLDER, millis(open_folder)));
        }
        if count == 10_000 {
            checks.push((&SEARCH, millis(search)));
        }
    }

    let (first, rescan) = scan_benchmark(10_000);
    writeln!(
        report,
        "\nScanning 10,000 files on disk (no thumbnails): first scan {}, rescan with nothing changed {}.\n",
        ms(first),
        ms(rescan)
    )
    .unwrap();

    writeln!(
        report,
        "| Target | Requirement | Limit | Measured | Result |"
    )
    .unwrap();
    writeln!(report, "| --- | --- | ---: | ---: | --- |").unwrap();
    let mut missed = false;
    for (target, measured) in &checks {
        let pass = *measured <= target.limit;
        missed |= !pass;
        writeln!(
            report,
            "| {} | {} | {} {} | {measured:.3} {} | {} |",
            target.what,
            target.requirement,
            target.limit,
            target.unit,
            target.unit,
            if pass { "pass" } else { "**MISSED**" }
        )
        .unwrap();
    }

    print!("{report}");
    if let Ok(summary) = std::env::var("GITHUB_STEP_SUMMARY") {
        let mut existing = fs::read_to_string(&summary).unwrap_or_default();
        existing.push_str(&report);
        fs::write(summary, existing).expect("the job summary is writable");
    }
    if missed {
        eprintln!(
            "An index target was missed. Raise it before switching to SQLite (decision 0010)."
        );
        std::process::exit(1);
    }
}

/// Times a first scan of `count` stand-in photos and a rescan with nothing changed.
fn scan_benchmark(count: usize) -> (Duration, Duration) {
    let library = tempfile::tempdir().expect("a temporary library");
    for n in 0..count {
        let folder = library.path().join(format!("Folder {}", n / 200));
        if n % 200 == 0 {
            fs::create_dir_all(&folder).expect("a folder");
        }
        let mut bytes = vec![0xFF, 0xD8, 0xFF, 0xE1];
        bytes.extend(n.to_le_bytes());
        fs::write(folder.join(format!("IMG_{n:05}.JPG")), bytes).expect("a photo");
    }
    let mut index = Index::new();
    index.add_root(library.path()).expect("the library exists");
    let root = index.roots()[0].clone();
    let start = Instant::now();
    let report = index.scan(&root);
    let first = start.elapsed();
    assert_eq!(report.photos, count);
    let start = Instant::now();
    let report = index.scan(&root);
    let rescan = start.elapsed();
    assert_eq!(report.added + report.changed + report.removed, 0);
    (first, rescan)
}

/// The median of `runs` timings.
fn median(runs: usize, mut work: impl FnMut()) -> Duration {
    let mut times: Vec<Duration> = (0..runs)
        .map(|_| {
            let start = Instant::now();
            work();
            start.elapsed()
        })
        .collect();
    times.sort();
    times[runs / 2]
}

fn millis(time: Duration) -> f64 {
    time.as_secs_f64() * 1000.0
}

fn ms(time: Duration) -> String {
    let millis = millis(time);
    if millis < 1.0 {
        format!("{:.0} µs", millis * 1000.0)
    } else if millis < 10.0 {
        format!("{millis:.2} ms")
    } else {
        format!("{millis:.0} ms")
    }
}
