//! Checks how Seura sees a folder of cloud-synced photos, without
//! downloading any: `cargo run --release -p seura-library --example
//! cloud_check -- <folder> [--scan]`.
//!
//! Without `--scan`, it only lists the folder and reads each file's
//! attributes, which never downloads anything. With `--scan`, it runs the
//! real library scan, which reads the first bytes of files that are on disk
//! and must leave cloud-only files alone. Nothing is written anywhere.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use seura_library::{Filter, Format, Index, Scope, is_placeholder};

fn main() {
    let mut args = std::env::args().skip(1);
    let folder = PathBuf::from(args.next().expect("usage: cloud_check <folder> [--scan]"));
    let scan = args.next().as_deref() == Some("--scan");

    let (photos, cloud_only) = list(&folder);
    println!("Listing only: {photos} photos, {cloud_only} cloud-only");
    if !scan {
        return;
    }

    let mut index = Index::new();
    index.add_root(&folder).expect("the folder exists");
    let root = index.roots()[0].clone();
    let start = Instant::now();
    let report = index.scan(&root);
    let ids = index.query(Scope::All, &Filter::default());
    let placeholders = ids
        .iter()
        .filter(|id| index.photo(**id).is_some_and(|photo| photo.placeholder))
        .count();
    println!(
        "Scan: {} photos in the index, {placeholders} recorded as cloud-only, in {:.0} ms",
        report.photos,
        start.elapsed().as_secs_f64() * 1000.0
    );
}

/// Counts photos and cloud-only photos from the folder listing alone.
fn list(folder: &Path) -> (usize, usize) {
    let (mut photos, mut cloud_only) = (0, 0);
    let mut pending = vec![folder.to_path_buf()];
    while let Some(folder) = pending.pop() {
        let Ok(entries) = fs::read_dir(&folder) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if metadata.is_dir() {
                pending.push(entry.path());
            } else if Format::from_name(&entry.file_name().to_string_lossy()).is_some() {
                photos += 1;
                cloud_only += usize::from(is_placeholder(&metadata));
            }
        }
    }
    (photos, cloud_only)
}
