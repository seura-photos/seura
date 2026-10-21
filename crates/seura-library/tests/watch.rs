//! Watching a real folder (LIB-4): each kind of change is reported within
//! the 5 seconds LIB-4 allows, and the scan of the reported folder brings
//! the index up to date. Runs on every CI platform, so both Windows'
//! change notifications and macOS's FSEvents are covered.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use seura_library::{Index, Watcher};

const LIMIT: Duration = Duration::from_secs(5);

fn jpeg() -> Vec<u8> {
    let mut bytes = vec![0xFF, 0xD8, 0xFF, 0xE0];
    bytes.extend([0; 60]);
    bytes
}

struct Watched {
    _folder: tempfile::TempDir,
    root: PathBuf,
    index: Index,
    _watcher: Watcher,
    changes: mpsc::Receiver<Vec<PathBuf>>,
}

impl Watched {
    fn new() -> Watched {
        let folder = tempfile::tempdir().unwrap();
        let root = folder.path().join("Pictures");
        fs::create_dir_all(root.join("2026")).unwrap();
        fs::write(root.join("2026/a.jpg"), jpeg()).unwrap();
        let mut index = Index::new();
        index.add_root(&root).unwrap();
        let root = index.roots()[0].clone();
        index.scan(&root);

        let (send, changes) = mpsc::channel();
        let cache = root.join("Seura cache");
        let mut watcher = Watcher::start(vec![cache], move |folders| {
            let _ = send.send(folders);
        })
        .unwrap();
        watcher.watch(std::slice::from_ref(&root));
        assert_eq!(watcher.watched(), std::slice::from_ref(&root));
        // Let the watch settle before changing anything.
        std::thread::sleep(Duration::from_millis(300));
        Watched {
            _folder: folder,
            root,
            index,
            _watcher: watcher,
            changes,
        }
    }

    /// Waits for reported changes and scans them, as the app does, until
    /// `done` holds or LIB-4's limit passes.
    fn catch_up(&mut self, what: &str, done: impl Fn(&Index) -> bool) {
        let started = Instant::now();
        while !done(&self.index) {
            let left = LIMIT.saturating_sub(started.elapsed());
            let folders = self
                .changes
                .recv_timeout(left)
                .unwrap_or_else(|_| panic!("{what}: not reported within {LIMIT:?}"));
            for folder in folders {
                let root = self.root.clone();
                self.index.scan_folder(&root, &folder);
            }
        }
    }

    fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .index
            .photo_ids()
            .map(|id| self.index.photo(id).unwrap().name.to_string())
            .collect();
        names.sort();
        names
    }
}

fn has_folder(index: &Index, path: &Path) -> bool {
    index.folder_id(path).is_some()
}

#[test]
fn a_new_photo_is_found() {
    let mut watched = Watched::new();
    fs::write(watched.root.join("2026/b.jpg"), jpeg()).unwrap();
    watched.catch_up("new photo", |index| index.len() == 2);
    assert_eq!(watched.names(), ["a.jpg", "b.jpg"]);
}

#[test]
fn a_new_folder_of_photos_is_found() {
    let mut watched = Watched::new();
    let summer = watched.root.join("2026/Summer");
    fs::create_dir_all(&summer).unwrap();
    fs::write(summer.join("c.jpg"), jpeg()).unwrap();
    let path = summer.clone();
    watched.catch_up("new folder", move |index| has_folder(index, &path));
    assert_eq!(watched.names(), ["a.jpg", "c.jpg"]);
}

#[test]
fn a_deleted_photo_leaves_and_takes_its_empty_folders_with_it() {
    let mut watched = Watched::new();
    // A photo two levels down, so the folder in between is left empty too.
    let summer = watched.root.join("2026/Summer");
    fs::create_dir_all(&summer).unwrap();
    fs::write(summer.join("deep.jpg"), jpeg()).unwrap();
    watched.catch_up("new photo", |index| index.len() == 2);
    fs::remove_file(watched.root.join("2026/a.jpg")).unwrap();
    watched.catch_up("first deletion", |index| index.len() == 1);

    fs::remove_file(summer.join("deep.jpg")).unwrap();
    let year = watched.root.join("2026");
    watched.catch_up("deletion", move |index| {
        index.is_empty() && !has_folder(index, &summer) && !has_folder(index, &year)
    });
}

#[test]
fn a_renamed_photo_is_followed() {
    let mut watched = Watched::new();
    fs::rename(
        watched.root.join("2026/a.jpg"),
        watched.root.join("2026/renamed.jpg"),
    )
    .unwrap();
    watched.catch_up("rename", |index| {
        index
            .photo_ids()
            .any(|id| &*index.photo(id).unwrap().name == "renamed.jpg")
            && index.len() == 1
    });
}

#[test]
fn changes_in_seura_s_own_cache_are_ignored() {
    let watched = Watched::new();
    let cache = watched.root.join("Seura cache");
    fs::create_dir_all(&cache).unwrap();
    fs::write(cache.join("thumb.webp"), b"not a photo").unwrap();
    match watched.changes.recv_timeout(Duration::from_secs(2)) {
        Err(_) => {}
        // Creating the cache folder itself may be reported for its parent,
        // but nothing inside the cache may be.
        Ok(folders) => assert!(
            folders.iter().all(|folder| !folder.starts_with(&cache)),
            "{folders:?}"
        ),
    }
}

/// Scanning only the folder a deleted photo was in still removes the folders
/// above it left with no photos. Checked without the watcher, since which
/// folders the operating system reports varies.
#[test]
fn scanning_one_folder_prunes_the_empty_folders_above_it() {
    let folder = tempfile::tempdir().unwrap();
    let root = folder.path().join("Pictures");
    let summer = root.join("2026").join("Summer");
    fs::create_dir_all(&summer).unwrap();
    fs::write(root.join("kept.jpg"), jpeg()).unwrap();
    fs::write(summer.join("deep.jpg"), jpeg()).unwrap();
    let mut index = Index::new();
    index.add_root(&root).unwrap();
    let root = index.roots()[0].clone();
    let summer = root.join("2026").join("Summer");
    index.scan(&root);
    assert!(has_folder(&index, &root.join("2026")));

    fs::remove_file(summer.join("deep.jpg")).unwrap();
    index.scan_folder(&root, &summer);

    assert!(!has_folder(&index, &summer));
    assert!(!has_folder(&index, &root.join("2026")));
    // The library folder still has a photo, so it stays.
    assert!(has_folder(&index, &root));
    assert_eq!(index.len(), 1);
}

/// A new photo deep in a folder the index has never seen comes with the
/// folders above it.
#[test]
fn scanning_a_new_deep_folder_adds_the_folders_above_it() {
    let folder = tempfile::tempdir().unwrap();
    let root = folder.path().join("Pictures");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("kept.jpg"), jpeg()).unwrap();
    let mut index = Index::new();
    index.add_root(&root).unwrap();
    let root = index.roots()[0].clone();
    index.scan(&root);

    let trip = root.join("2027").join("Trip");
    fs::create_dir_all(&trip).unwrap();
    fs::write(trip.join("new.jpg"), jpeg()).unwrap();
    index.scan_folder(&root, &trip);

    let year = index
        .folder_id(&root.join("2027"))
        .expect("the folder above is added");
    let trip_id = index.folder_id(&trip).expect("the new folder is added");
    assert_eq!(index.folder(trip_id).unwrap().parent, Some(year));
    assert_eq!(index.len(), 2);
}
