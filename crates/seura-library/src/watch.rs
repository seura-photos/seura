//! Watching library folders for changes while the app runs (LIB-4): new,
//! changed, deleted, and renamed photos and edit files. Changes made while
//! the app was closed are found by the scan at launch.
//!
//! The operating system reports changes path by path, often many at once
//! (a sync client catching up, a camera import). They're gathered until
//! things are quiet for half a second, or for two seconds at most, then
//! reported as the folders to rescan, so the index catches up well within
//! LIB-4's 5 seconds.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher as _};

/// How long changes must stop before they're reported.
const QUIET: Duration = Duration::from_millis(500);
/// Changes are reported at least this often during a long burst.
const AT_MOST: Duration = Duration::from_secs(2);

/// Watches library folders. Dropping it stops watching.
pub struct Watcher {
    inner: RecommendedWatcher,
    watched: Vec<PathBuf>,
}

impl Watcher {
    /// Starts a watcher that calls `on_change` from its own thread with the
    /// folders that changed, outermost only. Changes under `ignore` (Seura's
    /// own cache, if it's inside a library folder) aren't reported.
    pub fn start<F>(ignore: Vec<PathBuf>, on_change: F) -> notify::Result<Watcher>
    where
        F: Fn(Vec<PathBuf>) + Send + 'static,
    {
        let (send, receive) = mpsc::channel::<PathBuf>();
        let inner = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            let Ok(event) = event else { return };
            if matches!(event.kind, EventKind::Access(_)) {
                return;
            }
            for path in event.paths {
                if !ignore.iter().any(|ignored| path.starts_with(ignored)) {
                    let _ = send.send(path);
                }
            }
        })?;
        std::thread::Builder::new()
            .name("watch".into())
            .spawn(move || {
                // Ends when the watcher, and with it the sender, is dropped.
                while let Ok(first) = receive.recv() {
                    let started = Instant::now();
                    let mut changed = vec![first];
                    loop {
                        let left = AT_MOST.saturating_sub(started.elapsed());
                        match receive.recv_timeout(QUIET.min(left)) {
                            Ok(path) if !left.is_zero() => changed.push(path),
                            Ok(path) => {
                                changed.push(path);
                                break;
                            }
                            Err(RecvTimeoutError::Timeout) => break,
                            Err(RecvTimeoutError::Disconnected) => return,
                        }
                    }
                    on_change(folders_to_scan(&changed));
                }
            })
            .expect("can start the watch thread");
        Ok(Watcher {
            inner,
            watched: Vec::new(),
        })
    }

    /// Watches exactly these library folders. A folder that can't be
    /// watched, such as one on a disconnected drive (LIB-6), is skipped;
    /// calling this again when it's back starts watching it.
    pub fn watch(&mut self, roots: &[PathBuf]) {
        for old in std::mem::take(&mut self.watched) {
            let _ = self.inner.unwatch(&old);
        }
        for root in roots {
            if self.inner.watch(root, RecursiveMode::Recursive).is_ok() {
                self.watched.push(root.clone());
            }
        }
    }

    /// The folders being watched now.
    pub fn watched(&self) -> &[PathBuf] {
        &self.watched
    }
}

/// The folders to rescan for changed paths: a folder that changed itself,
/// or the folder a changed file is in. A path that's gone (deleted, or the
/// old name of a rename) means its folder. Folders inside others in the list
/// are left out, since rescanning the outer one covers them.
pub fn folders_to_scan(paths: &[PathBuf]) -> Vec<PathBuf> {
    let folders: BTreeSet<PathBuf> = paths
        .iter()
        .filter_map(|path| {
            if path.is_dir() {
                Some(path.clone())
            } else {
                path.parent().map(Path::to_path_buf)
            }
        })
        .collect();
    let mut outermost: Vec<PathBuf> = Vec::new();
    // Sorted, so a folder comes before everything inside it.
    for folder in folders {
        if !outermost.iter().any(|outer| folder.starts_with(outer)) {
            outermost.push(folder);
        }
    }
    outermost
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn changed_files_mean_their_folders_and_inner_folders_are_covered() {
        let root = tempfile::tempdir().unwrap();
        let a = root.path().join("a");
        let b = a.join("b");
        fs::create_dir_all(&b).unwrap();
        let folders = folders_to_scan(&[
            b.join("photo.jpg"),
            a.join("gone.jpg"),
            b.clone(),
            root.path().join("other").join("deleted.jpg"),
        ]);
        assert_eq!(folders, [a, root.path().join("other")]);
    }
}
