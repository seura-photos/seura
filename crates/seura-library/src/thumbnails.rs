//! Thumbnails: made by a pool of worker threads, visible photos first
//! (LIB-3), stored as WebP in the cache (DSP-4; technical design, section 4).
//!
//! Cache layout: `<cache>/thumbs/ab/cd/abcd….webp`, keyed by a hash of the
//! photo's path, size, and modification time, so a changed photo gets a new
//! thumbnail and an unchanged one is never made twice. Everything here is
//! rebuilt from the photos and is safe to delete.

use std::collections::{HashSet, VecDeque};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use seura_image::Rgba8;
use sha2::{Digest, Sha256};

use crate::index::{FileTime, PhotoId};

/// The thumbnail folder inside the cache.
pub const THUMBS_FOLDER: &str = "thumbs";

/// Where thumbnails live, and how to make one.
#[derive(Clone, Debug)]
pub struct ThumbnailCache {
    root: PathBuf,
    edits: Option<ApplyEdits>,
}

/// Renders a photo's edits onto its thumbnail-sized image, in sRGB (BRW-1).
/// Provided by the app, which owns the renderer (decision 0004); without
/// it, edited photos' thumbnails show the photo as it is.
#[derive(Clone)]
pub struct ApplyEdits(pub Arc<EditsFn>);

/// Takes the photo's path and its decoded, thumbnail-sized image in sRGB;
/// returns the image with the photo's edits.
pub type EditsFn = dyn Fn(&Path, Rgba8) -> Rgba8 + Send + Sync;

impl std::fmt::Debug for ApplyEdits {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ApplyEdits")
    }
}

/// What making a thumbnail learned about the photo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Made {
    /// Upright pixel dimensions.
    pub width: u32,
    pub height: u32,
    /// Date taken, from EXIF.
    pub taken: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Made now, or already in the cache (`None` details).
    Ready(Option<Made>),
    /// The photo is only in the cloud: it isn't decoded (LIB-9). The system's
    /// own thumbnail, if any, is used instead.
    CloudOnly,
    /// The system has no decoder for the photo's format: HEIC on Windows
    /// without Microsoft's extensions (decision 0014).
    NeedsCodec,
    Failed(String),
}

impl ThumbnailCache {
    pub fn new(cache: &Path) -> ThumbnailCache {
        ThumbnailCache {
            root: cache.join(THUMBS_FOLDER),
            edits: None,
        }
    }

    /// The same cache, rendering edited photos' thumbnails with their edits.
    pub fn with_edits(self, edits: ApplyEdits) -> ThumbnailCache {
        ThumbnailCache {
            edits: Some(edits),
            ..self
        }
    }

    /// The cache key for a photo as it is now: its path, size, and date,
    /// and for an edited photo its edit file's size and date too, so a new
    /// edit means a new thumbnail. Unedited photos' keys are unchanged.
    pub fn key(
        path: &Path,
        size: u64,
        modified: FileTime,
        edit_file: Option<(u64, FileTime)>,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(path.as_os_str().as_encoded_bytes());
        hasher.update(size.to_le_bytes());
        hasher.update(modified.secs.to_le_bytes());
        hasher.update(modified.nanos.to_le_bytes());
        if let Some((size, modified)) = edit_file {
            hasher.update(b"edits");
            hasher.update(size.to_le_bytes());
            hasher.update(modified.secs.to_le_bytes());
            hasher.update(modified.nanos.to_le_bytes());
        }
        hasher.finalize()[..16]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    /// Where the thumbnail for `key` is, or would be.
    pub fn path(&self, key: &str) -> PathBuf {
        self.root
            .join(&key[..2])
            .join(&key[2..4])
            .join(format!("{key}.webp"))
    }

    /// Makes and stores the thumbnail for a photo, with its edits if it's
    /// `edited` (BRW-1). The photo is only read; the thumbnail is written
    /// atomically (CLAUDE.md rules 1 and 3).
    pub fn make(&self, photo: &Path, key: &str, edited: bool) -> Outcome {
        let decoded = match seura_image::decode(photo) {
            Ok(decoded) => decoded,
            Err(seura_image::DecodeError::HeifUnavailable(_)) => return Outcome::NeedsCodec,
            Err(error) => return Outcome::Failed(error.to_string()),
        };
        let small = seura_image::fit(&decoded.image, seura_image::THUMBNAIL_SIZE);
        let small = match (&self.edits, edited) {
            (Some(ApplyEdits(apply)), true) => apply(photo, small),
            _ => small,
        };
        let bytes = seura_image::thumbnail(&small);
        if let Err(error) = self.store(key, &bytes) {
            return Outcome::Failed(error.to_string());
        }
        Outcome::Ready(Some(Made {
            width: decoded.image.width,
            height: decoded.image.height,
            taken: decoded.taken,
        }))
    }

    /// Stores thumbnail bytes made elsewhere (the system's, for cloud-only photos).
    pub fn store(&self, key: &str, bytes: &[u8]) -> io::Result<()> {
        let path = self.path(key);
        fs::create_dir_all(path.parent().expect("thumbnails are in folders"))?;
        seura_edit::write_atomically(&path, bytes)
    }
}

/// One photo to make a thumbnail for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Job {
    pub id: PhotoId,
    pub path: PathBuf,
    pub key: String,
    /// Cloud-only: never decoded.
    pub placeholder: bool,
    /// Has edits, which its thumbnail shows (BRW-1).
    pub edited: bool,
}

#[derive(Default)]
struct Queue {
    visible: VecDeque<Job>,
    background: VecDeque<Job>,
    queued: HashSet<PhotoId>,
    stopping: bool,
}

impl Queue {
    fn next(&mut self) -> Option<Job> {
        let job = self
            .visible
            .pop_front()
            .or_else(|| self.background.pop_front())?;
        self.queued.remove(&job.id);
        Some(job)
    }

    /// The photos now on screen go first; earlier visible jobs not yet
    /// started drop back to the front of the background queue.
    fn show(&mut self, jobs: Vec<Job>) -> usize {
        let previous: Vec<Job> = self.visible.drain(..).collect();
        let shown: HashSet<PhotoId> = jobs.iter().map(|job| job.id).collect();
        self.background.retain(|job| !shown.contains(&job.id));
        for job in previous
            .into_iter()
            .rev()
            .filter(|job| !shown.contains(&job.id))
        {
            self.background.push_front(job);
        }
        let mut added = 0;
        for job in jobs {
            added += usize::from(self.queued.insert(job.id));
            self.visible.push_back(job);
        }
        added
    }

    fn queue_background(&mut self, jobs: Vec<Job>) -> usize {
        let mut added = 0;
        for job in jobs {
            if self.queued.insert(job.id) {
                self.background.push_back(job);
                added += 1;
            }
        }
        added
    }
}

/// The worker pool. Dropping it stops the workers after their current job.
pub struct ThumbnailWorkers {
    shared: Arc<(Mutex<Queue>, Condvar)>,
    threads: Vec<JoinHandle<()>>,
}

impl ThumbnailWorkers {
    /// Starts `threads` workers. `done` is called from a worker thread for
    /// each finished job.
    pub fn start<F>(cache: ThumbnailCache, threads: usize, done: F) -> ThumbnailWorkers
    where
        F: Fn(PhotoId, Outcome) + Send + Sync + 'static,
    {
        let shared = Arc::new((Mutex::new(Queue::default()), Condvar::new()));
        let done = Arc::new(done);
        let threads = (0..threads.max(1))
            .map(|n| {
                let (shared, done, cache) = (Arc::clone(&shared), Arc::clone(&done), cache.clone());
                std::thread::Builder::new()
                    .name(format!("thumbnails-{n}"))
                    .spawn(move || work(&shared, &cache, &*done))
                    .expect("can start a worker thread")
            })
            .collect();
        ThumbnailWorkers { shared, threads }
    }

    /// The photos now on screen, in the order they should appear. They go
    /// first; earlier visible jobs not yet started drop back to the front
    /// of the background queue. Returns how many weren't queued already.
    pub fn show(&self, jobs: Vec<Job>) -> usize {
        let (lock, wake) = &*self.shared;
        let added = lock
            .lock()
            .expect("the queue lock isn't poisoned")
            .show(jobs);
        wake.notify_all();
        added
    }

    /// Photos to do when nothing visible is waiting. Returns how many
    /// weren't queued already.
    pub fn queue_background(&self, jobs: Vec<Job>) -> usize {
        let (lock, wake) = &*self.shared;
        let added = lock
            .lock()
            .expect("the queue lock isn't poisoned")
            .queue_background(jobs);
        wake.notify_all();
        added
    }

    /// Jobs waiting, visible and background.
    pub fn pending(&self) -> usize {
        let queue = self.shared.0.lock().expect("the queue lock isn't poisoned");
        queue.visible.len() + queue.background.len()
    }
}

impl Drop for ThumbnailWorkers {
    fn drop(&mut self) {
        {
            let (lock, wake) = &*self.shared;
            lock.lock().expect("the queue lock isn't poisoned").stopping = true;
            wake.notify_all();
        }
        for thread in self.threads.drain(..) {
            let _ = thread.join();
        }
    }
}

/// A cloud-only photo's thumbnail comes from the system, which keeps the
/// sync service's preview; the photo itself is never read (LIB-9).
fn from_system(cache: &ThumbnailCache, job: &Job) -> Outcome {
    let Some(image) = seura_image::system_thumbnail(&job.path, seura_image::THUMBNAIL_SIZE) else {
        return Outcome::CloudOnly;
    };
    match cache.store(&job.key, &seura_image::thumbnail(&image)) {
        Ok(()) => Outcome::Ready(None),
        Err(error) => Outcome::Failed(error.to_string()),
    }
}

fn work(shared: &(Mutex<Queue>, Condvar), cache: &ThumbnailCache, done: &dyn Fn(PhotoId, Outcome)) {
    let (lock, wake) = shared;
    loop {
        let job = {
            let mut queue = lock.lock().expect("the queue lock isn't poisoned");
            loop {
                if queue.stopping {
                    return;
                }
                if let Some(job) = queue.next() {
                    break job;
                }
                queue = wake.wait(queue).expect("the queue lock isn't poisoned");
            }
        };
        let outcome = if cache.path(&job.key).is_file() {
            Outcome::Ready(None)
        } else if job.placeholder {
            from_system(cache, &job)
        } else {
            cache.make(&job.path, &job.key, job.edited)
        };
        done(job.id, outcome);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(n: u32) -> Job {
        Job {
            id: PhotoId(n),
            path: PathBuf::from(format!("{n}.jpg")),
            key: format!("{n:032}"),
            placeholder: false,
            edited: false,
        }
    }

    fn order(queue: &mut Queue) -> Vec<u32> {
        std::iter::from_fn(|| queue.next())
            .map(|job| job.id.0)
            .collect()
    }

    /// Scan progress counts the jobs each call adds (LIB-3), so a photo
    /// queued twice must count once.
    #[test]
    fn only_new_jobs_are_counted() {
        let mut queue = Queue::default();
        assert_eq!(queue.queue_background(vec![job(1), job(2)]), 2);
        assert_eq!(queue.queue_background(vec![job(2), job(3)]), 1);
        // Showing a queued photo moves it up without counting it again.
        assert_eq!(queue.show(vec![job(3), job(4)]), 1);
        assert_eq!(order(&mut queue), [3, 4, 1, 2]);
    }

    #[test]
    fn visible_photos_go_first() {
        let mut queue = Queue::default();
        queue.queue_background((1..=5).map(job).collect());
        queue.show(vec![job(4), job(9)]);
        assert_eq!(order(&mut queue), [4, 9, 1, 2, 3, 5]);
    }

    #[test]
    fn photos_scrolled_away_are_deprioritized_not_dropped() {
        let mut queue = Queue::default();
        queue.queue_background(vec![job(1), job(2)]);
        queue.show(vec![job(10), job(11)]);
        queue.show(vec![job(20)]);
        // 20 is on screen now; 10 and 11 are next, before older background work.
        assert_eq!(order(&mut queue), [20, 10, 11, 1, 2]);
    }

    #[test]
    fn a_photo_is_queued_once() {
        let mut queue = Queue::default();
        queue.queue_background(vec![job(1), job(1), job(2)]);
        queue.show(vec![job(2)]);
        queue.queue_background(vec![job(2)]);
        assert_eq!(order(&mut queue), [2, 1]);
    }

    #[test]
    fn keys_change_when_the_photo_changes() {
        let path = Path::new("/photos/a.jpg");
        let time = FileTime {
            secs: 100,
            nanos: 0,
        };
        let key = ThumbnailCache::key(path, 10, time, None);
        assert_eq!(key.len(), 32);
        assert_ne!(key, ThumbnailCache::key(path, 11, time, None));
        // An edited photo's key follows its edit file too (BRW-1).
        let edited = ThumbnailCache::key(path, 10, time, Some((200, time)));
        assert_ne!(key, edited);
        assert_ne!(
            edited,
            ThumbnailCache::key(path, 10, time, Some((201, time)))
        );
        assert_ne!(
            key,
            ThumbnailCache::key(
                path,
                10,
                FileTime {
                    secs: 101,
                    nanos: 0
                },
                None
            )
        );
        assert_ne!(
            key,
            ThumbnailCache::key(Path::new("/photos/b.jpg"), 10, time, None)
        );
        let cache = ThumbnailCache::new(Path::new("cache"));
        let file = cache.path(&key);
        assert!(
            file.starts_with(
                Path::new("cache")
                    .join("thumbs")
                    .join(&key[..2])
                    .join(&key[2..4])
            )
        );
    }
}
