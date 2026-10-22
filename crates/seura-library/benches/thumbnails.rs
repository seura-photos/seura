//! The thumbnail benchmark (PRF-3: a first scan makes at least 30 thumbnails
//! a second on the reference laptop; on faster machines the aim is twice
//! that, functional spec "Performance targets").
//!
//! Generates 12-megapixel JPEGs with photo-like detail, makes their
//! thumbnails with the real worker pool, and reports photos per second and
//! where one photo's time goes. Run with
//! `cargo bench -p seura-library --bench thumbnails`; fails below PRF-3.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use seura_library::{FileTime, Job, Outcome, PhotoId, ThumbnailCache, ThumbnailWorkers};

const PHOTOS: usize = 48;
const WIDTH: u16 = 4032;
const HEIGHT: u16 = 3024;
/// PRF-3, on the reference laptop.
const TARGET: f64 = 30.0;
/// Half the target time on faster machines (functional spec).
const AIM: f64 = 60.0;

/// A photo-like image: smooth gradients with fine, varied detail, so JPEG
/// decoding costs what it costs for real photos (flat images decode faster).
fn photo(seed: u32) -> Vec<u8> {
    let (w, h) = (u32::from(WIDTH), u32::from(HEIGHT));
    let mut state = seed.wrapping_mul(2_654_435_761).max(1);
    let mut pixels = Vec::with_capacity((w * h * 3) as usize);
    for y in 0..h {
        for x in 0..w {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            let noise = (state & 31) as i32 - 16;
            let base = [
                (x * 255 / w) as i32,
                (y * 255 / h) as i32,
                ((x + y + seed * 97) % 256) as i32,
            ];
            for channel in base {
                pixels.push((channel + noise).clamp(0, 255) as u8);
            }
        }
    }
    let mut bytes = Vec::new();
    jpeg_encoder::Encoder::new(&mut bytes, 90)
        .encode(&pixels, WIDTH, HEIGHT, jpeg_encoder::ColorType::Rgb)
        .expect("encodes");
    bytes
}

fn main() {
    let folder = tempfile::tempdir().expect("a temporary folder");
    let started = Instant::now();
    let paths: Vec<PathBuf> = (0..PHOTOS)
        .map(|n| {
            let path = folder.path().join(format!("photo-{n:03}.jpg"));
            std::fs::write(&path, photo(n as u32)).expect("writes");
            path
        })
        .collect();
    let size_mb = std::fs::metadata(&paths[0]).map_or(0.0, |m| m.len() as f64 / 1e6);
    eprintln!(
        "Generated {PHOTOS} photos in {:.1} s",
        started.elapsed().as_secs_f64()
    );

    // One photo's stages, on one thread.
    let stages = {
        let t = Instant::now();
        let decoded = seura_image::decode(&paths[0]).expect("decodes");
        let decode = t.elapsed();
        let t = Instant::now();
        let small = seura_image::fit(&decoded.image, seura_image::THUMBNAIL_SIZE);
        let resize = t.elapsed();
        let t = Instant::now();
        let bytes = seura_image::thumbnail(&small);
        let encode = t.elapsed();
        assert!(!bytes.is_empty());
        (decode, resize, encode)
    };

    // The real worker pool, as the app runs it.
    let cache_folder = tempfile::tempdir().expect("a temporary folder");
    let threads = std::thread::available_parallelism()
        .map_or(2, |n| n.get().saturating_sub(1))
        .max(1);
    let done = Arc::new((Mutex::new(0usize), Condvar::new()));
    let failed = Arc::new(AtomicUsize::new(0));
    let workers = {
        let (done, failed) = (Arc::clone(&done), Arc::clone(&failed));
        ThumbnailWorkers::start(
            ThumbnailCache::new(cache_folder.path()),
            threads,
            move |_, outcome| {
                if !matches!(outcome, Outcome::Ready(_)) {
                    failed.fetch_add(1, Ordering::Relaxed);
                }
                let (count, wake) = &*done;
                *count.lock().expect("lock") += 1;
                wake.notify_all();
            },
        )
    };
    let jobs = paths
        .iter()
        .enumerate()
        .map(|(n, path)| Job {
            id: PhotoId::new(n as u32),
            path: path.clone(),
            key: ThumbnailCache::key(
                path,
                0,
                FileTime {
                    secs: n as i64,
                    nanos: 0,
                },
                None,
            ),
            placeholder: false,
            edited: false,
        })
        .collect();
    let started = Instant::now();
    workers.queue_background(jobs);
    {
        let (count, wake) = &*done;
        let mut count = count.lock().expect("lock");
        while *count < PHOTOS {
            count = wake.wait(count).expect("lock");
        }
    }
    let elapsed = started.elapsed();
    drop(workers);
    let rate = PHOTOS as f64 / elapsed.as_secs_f64();

    let ms = |d: Duration| d.as_secs_f64() * 1000.0;
    let mut report = String::new();
    let _ = writeln!(report, "## Thumbnails (PRF-3)\n");
    let _ = writeln!(
        report,
        "{PHOTOS} JPEGs, {WIDTH} × {HEIGHT}, about {size_mb:.1} MB each, {threads} worker threads.\n"
    );
    let _ = writeln!(report, "| Measure | Result | Target |");
    let _ = writeln!(report, "| --- | --- | --- |");
    let _ = writeln!(
        report,
        "| Thumbnails per second | {rate:.0} | ≥ {TARGET:.0} (aim {AIM:.0}) |"
    );
    let _ = writeln!(report, "| One photo: decode | {:.0} ms | |", ms(stages.0));
    let _ = writeln!(
        report,
        "| One photo: resize to {} px | {:.0} ms | |",
        seura_image::THUMBNAIL_SIZE,
        ms(stages.1)
    );
    let _ = writeln!(
        report,
        "| One photo: encode WebP | {:.0} ms | |",
        ms(stages.2)
    );
    println!("{report}");
    if let Some(summary) = std::env::var_os("GITHUB_STEP_SUMMARY") {
        let _ = std::fs::OpenOptions::new()
            .append(true)
            .open(summary)
            .and_then(|mut file| std::io::Write::write_all(&mut file, report.as_bytes()));
    }
    let failures = failed.load(Ordering::Relaxed);
    if failures > 0 || rate < TARGET {
        eprintln!(
            "FAILED: {rate:.0} thumbnails per second (target {TARGET:.0}), {failures} failed"
        );
        std::process::exit(1);
    }
}
