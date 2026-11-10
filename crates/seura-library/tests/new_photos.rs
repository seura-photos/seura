//! What a scan says is new (REV-1): the start of the new-photos banner.

use std::fs;
use std::path::{Path, PathBuf};

use seura_library::Index;

/// Enough of a JPEG for a scan to take it for one.
fn photo(path: &Path, content: &[u8]) {
    let mut bytes = vec![0xFF, 0xD8, 0xFF, 0xE0];
    bytes.extend_from_slice(content);
    fs::write(path, bytes).unwrap();
}

fn library() -> (tempfile::TempDir, Index, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let mut index = Index::default();
    index.add_root(dir.path()).unwrap();
    let root = index.roots()[0].clone();
    (dir, index, root)
}

#[test]
fn a_scan_names_the_photos_it_found_for_the_first_time() {
    let (_dir, mut index, root) = library();
    photo(&root.join("a.jpg"), b"a");
    assert_eq!(index.scan(&root).new_photos.len(), 1);
    photo(&root.join("b.jpg"), b"bb");
    let report = index.scan(&root);
    assert_eq!(report.new_photos.len(), 1);
    assert_eq!(index.path(report.new_photos[0]), Some(root.join("b.jpg")));
}

#[test]
fn nothing_is_new_when_nothing_changed() {
    let (_dir, mut index, root) = library();
    photo(&root.join("a.jpg"), b"a");
    index.scan(&root);
    assert!(index.scan(&root).new_photos.is_empty());
}

#[test]
fn a_changed_photo_is_not_a_new_one() {
    let (_dir, mut index, root) = library();
    photo(&root.join("a.jpg"), b"a");
    index.scan(&root);
    photo(&root.join("a.jpg"), b"longer now");
    assert!(index.scan(&root).new_photos.is_empty());
}

#[test]
fn a_renamed_photo_is_not_a_new_one() {
    let (_dir, mut index, root) = library();
    photo(&root.join("a.jpg"), b"a");
    index.scan(&root);
    fs::rename(root.join("a.jpg"), root.join("renamed.jpg")).unwrap();
    let report = index.scan(&root);
    assert_eq!(report.added, 1);
    assert!(report.new_photos.is_empty());
}

#[test]
fn a_copy_of_a_photo_is_a_new_one() {
    let (_dir, mut index, root) = library();
    photo(&root.join("a.jpg"), b"a");
    index.scan(&root);
    fs::copy(root.join("a.jpg"), root.join("copy.jpg")).unwrap();
    assert_eq!(index.scan(&root).new_photos.len(), 1);
}
