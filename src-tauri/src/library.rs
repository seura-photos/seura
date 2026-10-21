//! The library as the app holds it: the index, the thumbnail cache, and the
//! workers that fill it (technical design, sections 3 and 4).

use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};

use serde::Serialize;
use seura_library::{
    Filter, FolderId, Index, Outcome, PhotoId, Scope, SettingsFile, ThumbnailCache,
    ThumbnailWorkers, Watcher,
};
use specta::Type;
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

pub struct Library {
    pub index: RwLock<Index>,
    /// How far the current scan and thumbnail pass has got (LIB-3).
    progress: Mutex<Progress>,
    /// Watches the library folders while the app runs (LIB-4).
    watcher: Mutex<Option<Watcher>>,
    /// Where the cache is, and the workers filling it; replaced together
    /// when the cache moves.
    store: RwLock<Store>,
}

struct Store {
    cache: PathBuf,
    thumbnails: ThumbnailCache,
    workers: ThumbnailWorkers,
}

impl Store {
    fn open<R: Runtime>(app: &AppHandle<R>, cache: &Path) -> Store {
        let threads = std::thread::available_parallelism().map_or(2, |n| n.get().saturating_sub(1));
        let handle = app.clone();
        let workers =
            ThumbnailWorkers::start(ThumbnailCache::new(cache), threads, move |id, outcome| {
                finished(&handle, id, outcome);
            });
        Store {
            cache: cache.to_path_buf(),
            thumbnails: ThumbnailCache::new(cache),
            workers,
        }
    }
}

/// A thumbnail finished, or can't be made.
#[derive(Clone, Debug, Serialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailReady {
    pub id: u32,
    /// The photo is only in the cloud and has no thumbnail yet (LIB-9).
    pub cloud_only: bool,
    /// The system can't decode the photo's format: HEIC on Windows without
    /// Microsoft's extensions (decision 0014).
    pub needs_codec: bool,
    pub failed: bool,
}

/// The library's folders or photos changed, after a scan.
#[derive(Clone, Debug, Serialize, Type, Event)]
pub struct LibraryChanged;

/// How far scanning has got, for the status bar (LIB-3): first looking for
/// photos in a library folder, then making their thumbnails. All empty and
/// zero when there's nothing to do.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    /// The library folder being scanned, by name.
    pub scanning: Option<String>,
    /// Thumbnails made (or found in the cache) in this pass.
    pub done: u32,
    /// Thumbnails asked for in this pass.
    pub total: u32,
}

#[derive(Default)]
struct Progress {
    scanning: Option<String>,
    /// Thumbnail jobs added since the queue was last empty.
    total: usize,
    last_sent: Option<Instant>,
}

/// Progress is sent at most this often while thumbnails are being made.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(250);

/// The settings file, as the app holds it (SET-3).
pub struct AppSettings(pub Mutex<SettingsFile>);

impl Library {
    /// Loads the index snapshot from the cache (PRF-1) and starts the
    /// thumbnail workers, one fewer than the processor's threads. The
    /// library folders come from the settings; a snapshot of other folders
    /// is set aside, which only costs a full scan.
    pub fn open<R: Runtime>(app: &AppHandle<R>, cache: &Path, folders: &[PathBuf]) -> Library {
        let index = Index::load_snapshot(cache)
            .filter(|index| index.roots() == folders)
            .unwrap_or_else(|| Index::with_roots(folders.to_vec()));
        Library {
            index: RwLock::new(index),
            progress: Mutex::new(Progress::default()),
            watcher: Mutex::new(None),
            store: RwLock::new(Store::open(app, cache)),
        }
    }

    /// Starts watching the library folders for changes (LIB-4), ignoring
    /// Seura's own cache in case it's inside one. Called again when the
    /// cache moves, so the new place is the one ignored.
    pub fn start_watching<R: Runtime>(&self, app: &AppHandle<R>) {
        let cache = self.store.read().expect("store lock").cache.clone();
        let handle = app.clone();
        let watcher = Watcher::start(vec![cache], move |folders| {
            if let Some(library) = handle.try_state::<Library>() {
                library.changed(&handle, &folders);
            }
        });
        // Without a watcher, changes still show at the next launch's scan.
        *self.watcher.lock().expect("watcher lock") = watcher.ok();
        self.rewatch();
    }

    /// Watches exactly the current library folders: after one is added or
    /// removed, or a drive comes back (LIB-6).
    pub fn rewatch(&self) {
        let roots = self.index.read().expect("index lock").roots().to_vec();
        if let Some(watcher) = self.watcher.lock().expect("watcher lock").as_mut() {
            watcher.watch(&roots);
        }
    }

    /// Brings the index up to date with folders the watcher reported, then
    /// saves it, queues new thumbnails, and tells the UI (LIB-4).
    fn changed<R: Runtime>(&self, app: &AppHandle<R>, folders: &[PathBuf]) {
        {
            let mut index = self.index.write().expect("index lock");
            let roots = index.roots().to_vec();
            for folder in folders {
                if let Some(root) = roots.iter().find(|root| folder.starts_with(root)) {
                    index.scan_folder(root, folder);
                }
            }
        }
        self.save();
        self.queue_all(app);
        let _ = LibraryChanged.emit(app);
    }

    /// Moves the cache to `cache`: the workers stop and start again there.
    /// Thumbnails already made stay where they were; they're rebuilt as
    /// needed (SET-1, when the default place is synced).
    pub fn move_cache<R: Runtime>(&self, app: &AppHandle<R>, cache: &Path) {
        let new = Store::open(app, cache);
        let old = std::mem::replace(&mut *self.store.write().expect("store lock"), new);
        // Dropping the old workers waits for them to stop. That happens after
        // the lock is released: a worker finishing a job reads the store to
        // report progress, and would otherwise wait for this forever.
        drop(old);
        self.start_watching(app);
    }

    /// The photos now on screen: their thumbnails are made first (LIB-3).
    pub fn show(&self, jobs: Vec<seura_library::Job>) {
        let added = self.store.read().expect("store lock").workers.show(jobs);
        self.progress.lock().expect("progress lock").total += added;
    }

    /// Queues thumbnails for every photo in the library, behind whatever is
    /// on screen, so the whole library is ready to browse (LIB-3, PRF-3).
    /// Thumbnails already in the cache are skipped almost at once.
    pub fn queue_all<R: Runtime>(&self, app: &AppHandle<R>) {
        let index = self.index.read().expect("index lock");
        let jobs: Vec<_> = index
            .photo_ids()
            .filter_map(|id| index.thumbnail_job(id))
            .collect();
        drop(index);
        let added = self
            .store
            .read()
            .expect("store lock")
            .workers
            .queue_background(jobs);
        self.progress.lock().expect("progress lock").total += added;
        self.send_progress(app, true);
    }

    /// Says how far the pass has got, at most every 250 ms unless `now`.
    fn send_progress<R: Runtime>(&self, app: &AppHandle<R>, now: bool) {
        let pending = self.store.read().expect("store lock").workers.pending();
        let mut progress = self.progress.lock().expect("progress lock");
        let due = progress
            .last_sent
            .is_none_or(|sent| sent.elapsed() >= PROGRESS_INTERVAL);
        let finished = pending == 0 && progress.scanning.is_none();
        if !(now || due || finished) {
            return;
        }
        let event = if finished {
            progress.total = 0;
            ScanProgress {
                scanning: None,
                ..ScanProgress::default()
            }
        } else {
            ScanProgress {
                scanning: progress.scanning.clone(),
                done: progress.total.saturating_sub(pending) as u32,
                total: progress.total as u32,
            }
        };
        progress.last_sent = Some(Instant::now());
        drop(progress);
        let _ = event.emit(app);
    }

    /// Removes thumbnails from the cache, for photos no longer in the
    /// library. The cache is Seura's own and rebuilt from the photos, so
    /// they're deleted, not sent to the Trash; only files in the cache.
    pub fn drop_thumbnails(&self, keys: &[String]) {
        let store = self.store.read().expect("store lock");
        for key in keys {
            let _ = std::fs::remove_file(store.thumbnails.path(key));
        }
    }

    /// Which library folders' drives are connected now (LIB-6).
    pub fn availability(&self) -> Vec<bool> {
        let roots = self.index.read().expect("index lock").roots().to_vec();
        roots.iter().map(|root| root.is_dir()).collect()
    }

    /// Where a thumbnail is, or would be.
    pub fn thumbnail_path(&self, key: &str) -> PathBuf {
        self.store.read().expect("store lock").thumbnails.path(key)
    }

    /// Rescans every library folder, saves the snapshot, and queues the
    /// thumbnails. Runs on a background thread; the grid shows the snapshot
    /// meanwhile, and the status bar says how far it's got (LIB-3).
    pub fn rescan<R: Runtime>(&self, app: &AppHandle<R>) {
        let roots = self.index.read().expect("index lock").roots().to_vec();
        for root in roots {
            let name = root.file_name().map_or_else(
                || root.display().to_string(),
                |name| name.to_string_lossy().into_owned(),
            );
            self.progress.lock().expect("progress lock").scanning = Some(name);
            self.send_progress(app, true);
            // The walk holds the index for one library folder at a time; a
            // very large first scan should move off the lock (M2 slice 6).
            self.index.write().expect("index lock").scan(&root);
        }
        self.progress.lock().expect("progress lock").scanning = None;
        self.save();
        self.queue_all(app);
    }

    pub fn save(&self) {
        let index = self.index.read().expect("index lock");
        // A failed snapshot only costs a full scan next launch.
        let _ = index.save_snapshot(&self.store.read().expect("store lock").cache);
    }

    /// Queues thumbnails for a folder's photos behind whatever is on screen.
    pub fn queue_folder(&self, folder: FolderId) {
        let index = self.index.read().expect("index lock");
        let jobs = index
            .query(Scope::Folder(folder), &Filter::default())
            .into_iter()
            .filter_map(|id| index.thumbnail_job(id))
            .collect();
        drop(index);
        let added = self
            .store
            .read()
            .expect("store lock")
            .workers
            .queue_background(jobs);
        self.progress.lock().expect("progress lock").total += added;
    }
}

/// Called on a worker thread when a thumbnail job ends.
fn finished<R: Runtime>(app: &AppHandle<R>, id: PhotoId, outcome: Outcome) {
    let Some(library) = app.try_state::<Library>() else {
        return;
    };
    if let Outcome::Ready(Some(made)) = &outcome {
        library
            .index
            .write()
            .expect("index lock")
            .record_details(id, made);
    }
    library.send_progress(app, false);
    let event = ThumbnailReady {
        id: id.value(),
        cloud_only: outcome == Outcome::CloudOnly,
        needs_codec: outcome == Outcome::NeedsCodec,
        failed: matches!(outcome, Outcome::Failed(_)),
    };
    let _ = event.emit(app);
}
