//! Generates a test library of photo-like JPEGs, for measuring the
//! performance targets without anyone's real photos (M2 slice 10).
//!
//! `cargo run --release -p seura-library --example make_library -- <folder>
//! <photos> <width> <height> [<per folder>]`
//!
//! Photos are smooth gradients with fine noise, so they decode like real
//! ones, each a little different; with `<per folder>`, they're spread over
//! subfolders of that many. The folder must not exist yet.

use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let usage = "usage: make_library <folder> <photos> <width> <height> [<per folder>]";
    let folder = PathBuf::from(args.first().expect(usage));
    let number = |index: usize| -> usize {
        args.get(index)
            .and_then(|value| value.parse().ok())
            .expect(usage)
    };
    let (photos, width, height) = (number(1), number(2), number(3));
    let per_folder = args.get(4).and_then(|value| value.parse().ok());
    assert!(!folder.exists(), "{} already exists", folder.display());

    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    std::thread::scope(|scope| {
        for thread in 0..threads {
            let folder = &folder;
            scope.spawn(move || {
                for n in (thread..photos).step_by(threads) {
                    let path = photo_path(folder, n, per_folder);
                    std::fs::create_dir_all(path.parent().expect("in a folder")).expect("folder");
                    std::fs::write(&path, photo(n as u32, width, height)).expect("photo");
                }
            });
        }
    });
    println!(
        "{photos} photos of {width} x {height} in {}",
        folder.display()
    );
}

fn photo_path(folder: &Path, n: usize, per_folder: Option<usize>) -> PathBuf {
    let name = format!("IMG_{n:05}.JPG");
    match per_folder {
        Some(size) => folder.join(format!("Folder {:03}", n / size)).join(name),
        None => folder.join(name),
    }
}

fn photo(seed: u32, width: usize, height: usize) -> Vec<u8> {
    let mut state = seed.wrapping_mul(2_654_435_761).max(1);
    let mut pixels = Vec::with_capacity(width * height * 3);
    for y in 0..height {
        for x in 0..width {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            let noise = (state & 15) as i32 - 8;
            let base = [
                (x * 255 / width) as i32,
                (y * 255 / height) as i32,
                ((x + y + seed as usize * 97) % 256) as i32,
            ];
            for channel in base {
                pixels.push((channel + noise).clamp(0, 255) as u8);
            }
        }
    }
    let mut bytes = Vec::new();
    jpeg_encoder::Encoder::new(&mut bytes, 90)
        .encode(
            &pixels,
            u16::try_from(width).expect("width"),
            u16::try_from(height).expect("height"),
            jpeg_encoder::ColorType::Rgb,
        )
        .expect("encodes");
    bytes
}
