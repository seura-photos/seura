//! Measures what moving one file to the Recycle Bin or Trash does, with the
//! call Seura uses (BRW-9; M3 work plan, slice 9): `cargo run -p
//! seura-library --example trash_check -- <file> [--yes]`.
//!
//! Without `--yes`, it only says what the file is: on disk, or only in the
//! cloud (LIB-9). With `--yes`, it moves that one file to the Recycle Bin or
//! Trash, never deleting it, and says whether that worked and how long it
//! took. What happens to the file afterwards (the Recycle Bin, the Trash, the
//! cloud service's own deleted items, any dialog) is for the person running
//! it to look at.

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use seura_library::{has_trash, is_placeholder, move_to_trash};

fn main() {
    let mut args = std::env::args().skip(1);
    let file = PathBuf::from(args.next().expect("usage: trash_check <file> [--yes]"));
    let confirmed = args.next().as_deref() == Some("--yes");

    let metadata = fs::symlink_metadata(&file).expect("the file exists");
    assert!(metadata.is_file(), "{} isn't a file", file.display());
    let where_it_is = if is_placeholder(&metadata) {
        "only in the cloud (not downloaded)"
    } else {
        "on disk"
    };
    println!(
        "{}: {} bytes, {where_it_is}, on a drive {} a Recycle Bin or Trash",
        file.display(),
        metadata.len(),
        if has_trash(&file) { "with" } else { "without" }
    );
    if !confirmed {
        println!("Nothing was changed. Add --yes to move it to the Recycle Bin or Trash.");
        return;
    }

    let start = Instant::now();
    let result = move_to_trash(&file);
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    match result {
        Ok(()) => println!("Moved to the Recycle Bin or Trash in {elapsed:.0} ms."),
        Err(error) => println!("Not moved, after {elapsed:.0} ms: {error}"),
    }
    println!(
        "Still in its folder: {}",
        if fs::symlink_metadata(&file).is_ok() {
            "yes"
        } else {
            "no"
        }
    );
}
