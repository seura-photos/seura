//! The library as the app holds it: the index, the thumbnail cache, and the
//! workers that fill it (technical design, sections 3 and 4).

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};

use serde::Serialize;
use seura_library::{
    ApplyEdits, Change, ChangeReport, Filter, FolderId, History, Index, Outcome, PhotoId, Problem,
    SWEEP_AFTER, Scope, SettingsFile, ThumbnailCache, ThumbnailWorkers, UndoReport, Watcher,
    move_to_trash,
};
use specta::Type;
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

use crate::downloads::{Downloaded, Downloads};

pub struct Library {
    pub index: RwLock<Index>,
    /// Held while scanning: one scan at a time, so a scan never removes a
    /// photo another just found. The index itself is locked one folder at
    /// a time, so browsing goes on meanwhile.
    scanning: Mutex<()>,
    /// How far the current scan and thumbnail pass has got (LIB-3).
    progress: Mutex<Progress>,
    /// Watches the library folders while the app runs (LIB-4).
    watcher: Mutex<Option<Watcher>>,
    /// Enhance's downloads, one at a time (BRW-10).
    downloads: Downloads,
    /// Where the cache is, and the workers filling it; replaced together
    /// when the cache moves.
    store: RwLock<Store>,
    /// Thumbnails finished since the UI was last told, sent together by
    /// [`Library::send_thumbnails`]: one event per thumbnail flooded the UI
    /// during a large first scan, so clicking a folder waited for them all.
    ready: Mutex<Vec<ThumbnailReady>>,
    /// The session's last actions, for undo and redo (BRW-11). Held while
    /// tags and ratings are written, so two changes never interleave on a
    /// photo.
    history: Mutex<History>,
    /// When decoding first learned something the saved index doesn't have,
    /// such as a date taken; `None` when it's all saved (#99).
    unsaved_details: Mutex<Option<Instant>>,
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
        // Edited photos' thumbnails show their edits (BRW-1).
        let thumbnails = ThumbnailCache::new(cache)
            .with_edits(ApplyEdits(std::sync::Arc::new(crate::gpu::apply_edits)));
        let workers = ThumbnailWorkers::start(thumbnails, threads, move |id, outcome| {
            finished(&handle, id, outcome);
        });
        Store {
            cache: cache.to_path_buf(),
            thumbnails: ThumbnailCache::new(cache),
            workers,
        }
    }
}

/// Thumbnails finished, or that can't be made, since the last of these.
#[derive(Clone, Debug, Serialize, Type, Event)]
pub struct ThumbnailsReady(pub Vec<ThumbnailReady>);

/// The UI hears about finished thumbnails this often at most.
pub const THUMBNAILS_INTERVAL: Duration = Duration::from_millis(100);

/// A thumbnail finished, or can't be made.
#[derive(Clone, Debug, Serialize, Type)]
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

/// Enhance's download of a cloud-only photo started or ended (BRW-10).
#[derive(Clone, Debug, Serialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct EnhanceDownload {
    pub id: u32,
    pub downloading: bool,
    /// Which download of how many, counting from 1.
    pub at: u32,
    pub total: u32,
    /// For a selection, which shows its progress and Stop; one photo's tile
    /// is enough for itself.
    pub batch: bool,
}

/// What Enhance with downloads did, in numbers of photos.
#[derive(Clone, Debug, Default, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EnhanceReport {
    /// In a read-only folder (TAG-7).
    pub read_only: u32,
    /// Their edit file can't be read, so it's left alone (FIL-4).
    pub unreadable: u32,
    /// Couldn't be changed for another reason, such as a full drive.
    pub failed: u32,
    /// In the cloud, and the download failed or took too long.
    pub not_downloaded: u32,
    /// In the cloud, and not enhanced because the user stopped.
    pub stopped: u32,
    /// The user stopped during a download. Seura stopped waiting, but its
    /// read goes on, so the photo may arrive anyway, not enhanced.
    pub still_downloading: bool,
}

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

/// What decoding learns is saved as soon as the thumbnails are done, and at
/// most this often while they're being made, so stopping the app without
/// quitting loses no more than this (#99). A snapshot of a large library
/// takes a moment, so not after every thumbnail.
const SAVE_DETAILS_EVERY: Duration = Duration::from_secs(30);

/// Progress is sent at most this often while thumbnails are being made.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(250);

/// The settings file, as the app holds it (SET-3).
pub struct AppSettings(pub Mutex<SettingsFile>);

impl EnhanceReport {
    fn add(&mut self, report: &ChangeReport) {
        for (_, problem) in &report.problems {
            match problem {
                Problem::ReadOnly => self.read_only += 1,
                Problem::Unreadable => self.unreadable += 1,
                _ => self.failed += 1,
            }
        }
    }
}

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
            scanning: Mutex::new(()),
            progress: Mutex::new(Progress::default()),
            watcher: Mutex::new(None),
            downloads: Downloads::default(),
            store: RwLock::new(Store::open(app, cache)),
            ready: Mutex::new(Vec::new()),
            history: Mutex::new(History::default()),
            unsaved_details: Mutex::new(None),
        }
    }

    /// Changes photos' tags or rating, writing their edit files through the
    /// one path for the Library's writes (M3 slice 1), then saves the index
    /// and tells the UI.
    pub fn change<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        ids: &[PhotoId],
        change: &Change,
    ) -> ChangeReport {
        // Rotate and Enhance work out each photo's action from its own
        // edits and pixels (BRW-10).
        let quick =
            |photo: &std::path::Path, cloud_only: bool, loaded: Option<&seura_edit::EditFile>| {
                crate::editor::quick_action(change, photo, cloud_only, loaded)
            };
        let make: Option<seura_library::MakeAction<'_>> =
            matches!(change, Change::Rotate { .. } | Change::Enhance).then_some(&quick);
        let report = {
            let mut history = self.history.lock().expect("history lock");
            let (report, action) = Index::change_recorded_with(
                &self.index,
                ids,
                change,
                make,
                crate::editor::WRITER,
                time::OffsetDateTime::now_utc(),
                &move_to_trash,
            );
            history.record(action);
            crate::menu::history_changed(app, history.next_undo(), history.next_redo());
            report
        };
        if !report.changed.is_empty() || !report.problems.is_empty() {
            self.save();
            let _ = LibraryChanged.emit(app);
        }
        report
    }

    /// After reading a cloud-only photo, which downloads it (LIB-9): the
    /// grid's badge follows at once, without waiting for the watcher, and a
    /// tile with no thumbnail gets one.
    pub fn downloaded<R: Runtime>(&self, app: &AppHandle<R>, id: PhotoId) {
        if Index::refresh_placeholder(&self.index, id) {
            self.save();
            self.queue_all(app);
            let _ = LibraryChanged.emit(app);
        }
    }

    /// Enhances photos, downloading those in the cloud one at a time and
    /// enhancing each as it arrives (BRW-10, LIB-9). Photos already on the
    /// computer go first. The whole is one action to undo, recorded once
    /// it's over, covering the photos it enhanced. One batch at a time: a
    /// later one waits its turn, and Stop stops those waiting too.
    pub fn enhance_downloading<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        ids: &[PhotoId],
    ) -> EnhanceReport {
        let ticket = self.downloads.ticket();
        let _turn = self.downloads.turn();
        let (cloud, here): (Vec<PhotoId>, Vec<PhotoId>) = {
            for &id in ids {
                self.downloaded(app, id);
            }
            let index = self.index.read().expect("index lock");
            ids.iter().partition(|id| {
                index.photo(**id).is_some_and(|photo| {
                    photo.placeholder
                        && !index
                            .folder(photo.folder)
                            .is_some_and(|folder| folder.read_only)
                })
            })
        };
        let mut result = EnhanceReport::default();
        let (mut action, report) = self.enhance_part(app, &here);
        result.add(&report);
        let total = cloud.len() as u32;
        for (done, &id) in cloud.iter().enumerate() {
            if self.downloads.stopped(ticket) {
                result.stopped = total - done as u32;
                break;
            }
            let Some(path) = self.index.read().expect("index lock").path(id) else {
                result.failed += 1;
                continue;
            };
            let event = |downloading| EnhanceDownload {
                id: id.value(),
                downloading,
                at: done as u32 + 1,
                total,
                batch: ids.len() > 1,
            };
            let _ = event(true).emit(app);
            let outcome = crate::downloads::download(path, || self.downloads.stopped(ticket));
            let _ = event(false).emit(app);
            match outcome {
                Downloaded::Yes => {
                    self.downloaded(app, id);
                    let (part, report) = self.enhance_part(app, &[id]);
                    action.absorb(part);
                    result.add(&report);
                }
                Downloaded::Failed => result.not_downloaded += 1,
                Downloaded::Stopped => {
                    result.stopped = total - done as u32;
                    result.still_downloading = true;
                    break;
                }
            }
        }
        let mut history = self.history.lock().expect("history lock");
        history.record(action);
        crate::menu::history_changed(app, history.next_undo(), history.next_redo());
        result
    }

    /// Enhances photos that are on the computer, without recording the
    /// action, then saves the index and tells the UI, so their thumbnails
    /// follow one by one.
    fn enhance_part<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        ids: &[PhotoId],
    ) -> (seura_library::Action, ChangeReport) {
        // A photo just read is on the computer, whatever its attributes say
        // so far.
        let make = |photo: &std::path::Path, _: bool, loaded: Option<&seura_edit::EditFile>| {
            crate::editor::quick_action(&Change::Enhance, photo, false, loaded)
        };
        let (report, action) = {
            let _history = self.history.lock().expect("history lock");
            Index::change_recorded_with(
                &self.index,
                ids,
                &Change::Enhance,
                Some(&make),
                crate::editor::WRITER,
                time::OffsetDateTime::now_utc(),
                &move_to_trash,
            )
        };
        if !report.changed.is_empty() || !report.problems.is_empty() {
            self.save();
            let _ = LibraryChanged.emit(app);
        }
        (action, report)
    }

    /// Stops Enhance's downloads: see [`Downloads::stop`].
    pub fn stop_downloads(&self) {
        self.downloads.stop();
    }

    /// Changes one photo's edits from the Editor (EDT-11), through the same
    /// path, then saves the index and tells the UI, so the grid's badges
    /// follow. Not one of the Library's actions to undo: the photo's own
    /// history has it (EDT-9). The action is made from the edit file as
    /// loaded, for revert (EDT-10); `kept` is the Editor's: see
    /// [`Index::edit_photo_with`]. Returns whether anything changed.
    pub fn edit_with<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        id: PhotoId,
        kept: &mut Option<seura_edit::EditDocument>,
        make: impl FnOnce(Option<&seura_edit::EditFile>) -> seura_edit::Action,
    ) -> Result<bool, Problem> {
        // One write at a time, as with the Library's changes.
        let changed = {
            let _history = self.history.lock().expect("history lock");
            Index::edit_photo_with(
                &self.index,
                id,
                kept,
                make,
                crate::editor::WRITER,
                time::OffsetDateTime::now_utc(),
                &move_to_trash,
            )?
        };
        if changed {
            self.save();
            let _ = LibraryChanged.emit(app);
        }
        Ok(changed)
    }

    /// Undoes or redoes the last change in one photo's own history, from
    /// the Editor (EDT-9), then saves the index and tells the UI. `kept` is
    /// the Editor's: see [`Index::step_photo`].
    pub fn step_edit<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        id: PhotoId,
        undo: bool,
        kept: &mut Option<seura_edit::EditDocument>,
    ) -> Result<seura_edit::Step, Problem> {
        // One write at a time, as with the Library's changes.
        let step = {
            let _history = self.history.lock().expect("history lock");
            Index::step_photo(
                &self.index,
                id,
                undo,
                kept,
                crate::editor::WRITER,
                time::OffsetDateTime::now_utc(),
                &move_to_trash,
            )?
        };
        if step != seura_edit::Step::Nothing {
            self.save();
            let _ = LibraryChanged.emit(app);
        }
        Ok(step)
    }

    /// Names the Library's next undo and redo in the Edit menu again, after
    /// the Editor named its own (BRW-11).
    pub fn show_history<R: Runtime>(&self, app: &AppHandle<R>) {
        let history = self.history.lock().expect("history lock");
        crate::menu::history_changed(app, history.next_undo(), history.next_redo());
    }

    /// Undoes the last action, whatever is selected now (BRW-11). `None`
    /// when there's nothing to undo.
    pub fn undo<R: Runtime>(&self, app: &AppHandle<R>) -> Option<UndoReport> {
        self.step(app, true)
    }

    /// Redoes the last action undone.
    pub fn redo<R: Runtime>(&self, app: &AppHandle<R>) -> Option<UndoReport> {
        self.step(app, false)
    }

    fn step<R: Runtime>(&self, app: &AppHandle<R>, undo: bool) -> Option<UndoReport> {
        let report = {
            let mut history = self.history.lock().expect("history lock");
            let now = time::OffsetDateTime::now_utc();
            let report = if undo {
                history.undo(&self.index, crate::editor::WRITER, now, &move_to_trash)
            } else {
                history.redo(&self.index, crate::editor::WRITER, now, &move_to_trash)
            };
            crate::menu::history_changed(app, history.next_undo(), history.next_redo());
            report
        };
        if report.as_ref().is_some_and(|report| report.done > 0) {
            self.save();
            let _ = LibraryChanged.emit(app);
        }
        report
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
        let mut edit_files_changed = Vec::new();
        {
            let _scanning = self.scanning.lock().expect("scan lock");
            let roots = self.index.read().expect("index lock").roots().to_vec();
            let mut orphans = Vec::new();
            for folder in folders {
                if let Some(root) = roots.iter().find(|root| folder.starts_with(root)) {
                    let report = Index::scan_shared(&self.index, root, folder);
                    orphans.extend(report.orphan_edit_files);
                    edit_files_changed.extend(report.edit_files_changed);
                }
            }
            self.reattach_renamed(&orphans);
        }
        self.save();
        self.queue_all(app);
        let _ = LibraryChanged.emit(app);
        // The photo open in the Editor may be one of them (FIL-3).
        if !edit_files_changed.is_empty()
            && let Some(editor) = app.try_state::<crate::editor::Editor>()
        {
            editor.changed_outside(app, &edit_files_changed);
        }
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

    /// How far the pass has got right now, unthrottled: for the measurement
    /// script, which mustn't depend on an event it may have missed.
    pub fn progress_now(&self) -> ScanProgress {
        let pending = self.store.read().expect("store lock").workers.pending();
        let progress = self.progress.lock().expect("progress lock");
        if pending == 0 && progress.scanning.is_none() {
            return ScanProgress::default();
        }
        ScanProgress {
            scanning: progress.scanning.clone(),
            done: progress.total.saturating_sub(pending) as u32,
            total: progress.total as u32,
        }
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

    /// Tells the UI about the thumbnails finished since last time, if any,
    /// and saves what decoding learned when it's due. Called every
    /// [`THUMBNAILS_INTERVAL`].
    pub fn send_thumbnails<R: Runtime>(&self, app: &AppHandle<R>) {
        let ready = std::mem::take(&mut *self.ready.lock().expect("ready lock"));
        if !ready.is_empty() {
            let _ = ThumbnailsReady(ready).emit(app);
        }
        self.save_details_when_due();
    }

    /// Saves the index if decoding learned something new and the thumbnails
    /// are done, or it has waited [`SAVE_DETAILS_EVERY`].
    fn save_details_when_due(&self) {
        let since = *self.unsaved_details.lock().expect("unsaved lock");
        let Some(since) = since else {
            return;
        };
        let idle = self.store.read().expect("store lock").workers.pending() == 0;
        if idle || since.elapsed() >= SAVE_DETAILS_EVERY {
            *self.unsaved_details.lock().expect("unsaved lock") = None;
            self.save();
        }
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

    /// Deletes thumbnails no photo in the library uses any more, such as
    /// those of photos deleted or folders renamed while the app was closed,
    /// once they've been unused for a week. Photos on a disconnected drive
    /// are still in the library, so theirs are kept (LIB-6).
    pub fn sweep_thumbnails(&self) {
        let keep: HashSet<String> = {
            let index = self.index.read().expect("index lock");
            index
                .photo_ids()
                .filter_map(|id| index.thumbnail_job(id))
                .map(|job| job.key)
                .collect()
        };
        let thumbnails = self.store.read().expect("store lock").thumbnails.clone();
        thumbnails.sweep(&keep, SWEEP_AFTER);
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
        let scanning = self.scanning.lock().expect("scan lock");
        let roots = self.index.read().expect("index lock").roots().to_vec();
        for root in roots {
            let name = root.file_name().map_or_else(
                || root.display().to_string(),
                |name| name.to_string_lossy().into_owned(),
            );
            self.progress.lock().expect("progress lock").scanning = Some(name);
            self.send_progress(app, true);
            let orphans = Index::scan_shared(&self.index, &root, &root).orphan_edit_files;
            self.reattach_renamed(&orphans);
        }
        drop(scanning);
        self.progress.lock().expect("progress lock").scanning = None;
        self.save();
        self.queue_all(app);
    }

    /// Gives renamed photos their edit files back (FIL-6).
    fn reattach_renamed(&self, orphans: &[PathBuf]) {
        if !orphans.is_empty() {
            Index::reattach_renamed(
                &self.index,
                orphans,
                crate::editor::WRITER,
                time::OffsetDateTime::now_utc(),
            );
        }
    }

    pub fn save(&self) {
        let index = self.index.read().expect("index lock");
        // A failed snapshot only costs a full scan next launch.
        let _ = index.save_snapshot(&self.store.read().expect("store lock").cache);
    }

    /// Queues thumbnails for these photos behind whatever is on screen.
    pub fn queue_photos(&self, jobs: Vec<seura_library::Job>) {
        let added = self
            .store
            .read()
            .expect("store lock")
            .workers
            .queue_background(jobs);
        self.progress.lock().expect("progress lock").total += added;
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
        self.queue_photos(jobs);
    }
}

/// Called on a worker thread when a thumbnail job ends.
fn finished<R: Runtime>(app: &AppHandle<R>, id: PhotoId, outcome: Outcome) {
    let Some(library) = app.try_state::<Library>() else {
        return;
    };
    if let Outcome::Ready(Some(made)) = &outcome {
        let new = library
            .index
            .write()
            .expect("index lock")
            .record_details(id, made);
        if new {
            library
                .unsaved_details
                .lock()
                .expect("unsaved lock")
                .get_or_insert_with(Instant::now);
        }
    }
    library.send_progress(app, false);
    library
        .ready
        .lock()
        .expect("ready lock")
        .push(ThumbnailReady {
            id: id.value(),
            cloud_only: outcome == Outcome::CloudOnly,
            needs_codec: outcome == Outcome::NeedsCodec,
            failed: matches!(outcome, Outcome::Failed(_)),
        });
}
