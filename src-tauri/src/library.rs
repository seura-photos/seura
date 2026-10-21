//! The library as the app holds it: the index, the thumbnail cache, and the
//! workers that fill it (technical design, sections 3 and 4).

use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};

use serde::Serialize;
use seura_library::{
    Filter, FolderId, Index, Outcome, PhotoId, Scope, SettingsFile, ThumbnailCache,
    ThumbnailWorkers,
};
use specta::Type;
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

pub struct Library {
    pub index: RwLock<Index>,
    pub cache: PathBuf,
    pub thumbnails: ThumbnailCache,
    pub workers: ThumbnailWorkers,
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
        let threads = std::thread::available_parallelism().map_or(2, |n| n.get().saturating_sub(1));
        let handle = app.clone();
        let workers =
            ThumbnailWorkers::start(ThumbnailCache::new(cache), threads, move |id, outcome| {
                finished(&handle, id, outcome);
            });
        Library {
            index: RwLock::new(index),
            cache: cache.to_path_buf(),
            thumbnails: ThumbnailCache::new(cache),
            workers,
        }
    }

    /// Rescans every library folder and saves the snapshot. Runs on a
    /// background thread; the grid shows the snapshot meanwhile.
    pub fn rescan(&self) {
        let roots = self.index.read().expect("index lock").roots().to_vec();
        for root in roots {
            // Scanned outside the lock would be better for a large library;
            // M2's scanning work moves the walk off the lock.
            self.index.write().expect("index lock").scan(&root);
        }
        self.save();
    }

    pub fn save(&self) {
        let index = self.index.read().expect("index lock");
        // A failed snapshot only costs a full scan next launch.
        let _ = index.save_snapshot(&self.cache);
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
        self.workers.queue_background(jobs);
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
    let event = ThumbnailReady {
        id: id.value(),
        cloud_only: outcome == Outcome::CloudOnly,
        needs_codec: outcome == Outcome::NeedsCodec,
        failed: matches!(outcome, Outcome::Failed(_)),
    };
    let _ = event.emit(app);
}
