//! Downloading cloud-only photos for Enhance (BRW-10, LIB-9). Reading a
//! cloud-only file is what downloads it; neither system reports how far it
//! has got, so this only waits, and can stop waiting.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::commands::DOWNLOAD_WAIT;

/// How often a wait looks for Stop.
const POLL: Duration = Duration::from_millis(100);

/// How waiting for a download ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Downloaded {
    Yes,
    /// It failed, or took longer than the wait.
    Failed,
    /// The user stopped. The read can't be cancelled and goes on, so the
    /// download may still finish.
    Stopped,
}

/// The photos to fetch ahead of a review (REV-2, LIB-9), by one worker, so
/// they arrive one at a time. A new list replaces the old: moving on, or
/// jumping, moves the window.
#[derive(Debug, Default)]
pub struct Ahead {
    state: Mutex<AheadState>,
}

#[derive(Debug, Default)]
struct AheadState {
    wanted: VecDeque<u32>,
    running: bool,
}

impl Ahead {
    /// Replaces the list. True if the caller should start the worker.
    pub fn want(&self, ids: Vec<u32>) -> bool {
        let mut state = self.state.lock().expect("ahead lock");
        state.wanted = ids.into();
        let start = !state.running && !state.wanted.is_empty();
        state.running |= start;
        start
    }

    /// The next photo for the worker. With none, the worker is over.
    pub fn next(&self) -> Option<u32> {
        let mut state = self.state.lock().expect("ahead lock");
        let next = state.wanted.pop_front();
        state.running = next.is_some();
        next
    }
}

/// Whose turn it is to download, and which batches the user has stopped.
#[derive(Debug, Default)]
pub struct Downloads {
    pub ahead: Ahead,
    turn: Mutex<()>,
    tickets: AtomicU64,
    /// Tickets below this are stopped.
    stopped_below: AtomicU64,
}

impl Downloads {
    pub fn ticket(&self) -> u64 {
        self.tickets.fetch_add(1, Ordering::SeqCst)
    }

    /// Held for a whole batch: photos download one at a time.
    pub fn turn(&self) -> MutexGuard<'_, ()> {
        self.turn.lock().expect("download turn lock")
    }

    /// Stops the batch in progress and any waiting their turn.
    pub fn stop(&self) {
        self.stopped_below
            .store(self.tickets.load(Ordering::SeqCst), Ordering::SeqCst);
    }

    pub fn stopped(&self, ticket: u64) -> bool {
        ticket < self.stopped_below.load(Ordering::SeqCst)
    }
}

/// Reads a whole file, which downloads it, and waits for at most
/// [`DOWNLOAD_WAIT`], or until `stopped`.
pub fn download(path: PathBuf, stopped: impl Fn() -> bool) -> Downloaded {
    wait(
        move || {
            std::io::copy(&mut std::fs::File::open(path)?, &mut std::io::sink())?;
            Ok(())
        },
        stopped,
        DOWNLOAD_WAIT,
    )
}

/// Runs `read` on a thread of its own, so a download that never ends holds
/// nothing up: past the wait, or once stopped, its result is dropped.
fn wait(
    read: impl FnOnce() -> std::io::Result<()> + Send + 'static,
    stopped: impl Fn() -> bool,
    longest: Duration,
) -> Downloaded {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(read());
    });
    let deadline = Instant::now() + longest;
    loop {
        if stopped() {
            return Downloaded::Stopped;
        }
        match receiver.recv_timeout(POLL) {
            Ok(Ok(())) => return Downloaded::Yes,
            Ok(Err(_)) | Err(RecvTimeoutError::Disconnected) => return Downloaded::Failed,
            Err(RecvTimeoutError::Timeout) if Instant::now() >= deadline => {
                return Downloaded::Failed;
            }
            Err(RecvTimeoutError::Timeout) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LONG: Duration = Duration::from_secs(60);

    #[test]
    fn one_worker_fetches_ahead_and_a_new_list_replaces_the_old() {
        let ahead = Ahead::default();
        assert!(ahead.want(vec![1, 2, 3]));
        assert_eq!(ahead.next(), Some(1));
        // Jumping: the worker is still busy, and goes on with the new list.
        assert!(!ahead.want(vec![7, 8]));
        assert_eq!(ahead.next(), Some(7));
        assert_eq!(ahead.next(), Some(8));
        assert_eq!(ahead.next(), None);
        // Over, so the next list needs a worker again.
        assert!(ahead.want(vec![9]));
    }

    #[test]
    fn an_empty_list_starts_no_worker_and_clears_what_was_wanted() {
        let ahead = Ahead::default();
        assert!(!ahead.want(Vec::new()));
        assert!(ahead.want(vec![1, 2]));
        assert!(!ahead.want(Vec::new()));
        assert_eq!(ahead.next(), None);
    }

    /// Stands in for a download that doesn't end while the test runs.
    fn endless() -> std::io::Result<()> {
        std::thread::sleep(LONG);
        Ok(())
    }

    #[test]
    fn a_read_that_finishes_is_downloaded_and_one_that_fails_is_not() {
        assert_eq!(wait(|| Ok(()), || false, LONG), Downloaded::Yes);
        let offline = || Err(std::io::Error::other("offline"));
        assert_eq!(wait(offline, || false, LONG), Downloaded::Failed);
    }

    #[test]
    fn a_download_is_waited_for_only_so_long() {
        let short = Duration::from_millis(10);
        assert_eq!(wait(endless, || false, short), Downloaded::Failed);
    }

    #[test]
    fn stop_ends_the_wait_for_the_photo_in_progress() {
        assert_eq!(wait(endless, || true, LONG), Downloaded::Stopped);
    }

    #[test]
    fn stop_stops_the_batches_so_far_but_not_a_later_one() {
        let downloads = Downloads::default();
        let (running, waiting) = (downloads.ticket(), downloads.ticket());
        assert!(!downloads.stopped(running));
        downloads.stop();
        assert!(downloads.stopped(running) && downloads.stopped(waiting));
        assert!(!downloads.stopped(downloads.ticket()));
    }

    #[test]
    fn a_missing_file_is_not_downloaded() {
        let folder = tempfile::tempdir().unwrap();
        assert_eq!(
            download(folder.path().join("gone.jpg"), || false),
            Downloaded::Failed
        );
    }
}
