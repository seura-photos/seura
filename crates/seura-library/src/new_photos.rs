//! Which photos are new, for the banner that offers to review them (REV-1,
//! M6 plan's question 10). Kept in memory for the session: the index has no
//! record of when a photo arrived, and after the next launch these photos
//! are no longer new.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::index::PhotoId;

/// How [`NewPhotos::settle`] ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Settled {
    /// Additions are still coming: look again after this long.
    Wait(Duration),
    /// They've settled. `changed` if the banner's photos are different now.
    Done { changed: bool },
}

#[derive(Debug, Default)]
pub struct NewPhotos {
    /// The banner's photos, until Review or Not now.
    shown: Vec<PhotoId>,
    /// Found since additions last settled.
    arriving: Vec<PhotoId>,
    /// When a photo last arrived.
    last: Option<Instant>,
    /// Someone is waiting for additions to settle.
    waiting: bool,
    /// Files Seura exported, which don't count (REV-1).
    exported: HashSet<PathBuf>,
}

/// So that two spellings of one path, such as `/var` and `/private/var` on
/// the Mac, are the same file.
fn resolved(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

impl NewPhotos {
    /// Notes photos a scan found. True if the caller should start waiting
    /// for additions to settle; false if someone already is, or none came.
    pub fn arrived(&mut self, ids: &[PhotoId], now: Instant) -> bool {
        if ids.is_empty() {
            return false;
        }
        self.arriving.extend_from_slice(ids);
        self.last = Some(now);
        let start = !self.waiting;
        self.waiting = true;
        start
    }

    /// Notes a file Seura exported. An export can land in a library folder,
    /// and it may be seen there before this is told, so the check is made
    /// when additions settle.
    pub fn exported(&mut self, path: &Path) {
        self.exported.insert(resolved(path));
    }

    /// Moves what arrived onto the banner, once nothing has arrived for
    /// `quiet`. `path_of` finds a photo's file, to leave out exports.
    pub fn settle(
        &mut self,
        now: Instant,
        quiet: Duration,
        path_of: impl Fn(PhotoId) -> Option<PathBuf>,
    ) -> Settled {
        if let Some(last) = self.last {
            let since = now.saturating_duration_since(last);
            if since < quiet {
                return Settled::Wait(quiet - since);
            }
        }
        self.waiting = false;
        let before = self.shown.len();
        for id in std::mem::take(&mut self.arriving) {
            let exported = path_of(id).is_some_and(|path| self.exported.contains(&resolved(&path)));
            if !exported && !self.shown.contains(&id) {
                self.shown.push(id);
            }
        }
        Settled::Done {
            changed: self.shown.len() != before,
        }
    }

    /// The banner's photos.
    pub fn shown(&self) -> &[PhotoId] {
        &self.shown
    }

    /// Review or Not now: these photos are no longer offered.
    pub fn dismiss(&mut self) {
        self.shown.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const QUIET: Duration = Duration::from_secs(60);

    fn ids(values: &[u32]) -> Vec<PhotoId> {
        values.iter().map(|value| PhotoId::new(*value)).collect()
    }

    fn no_path(_: PhotoId) -> Option<PathBuf> {
        None
    }

    #[test]
    fn nothing_shows_until_additions_have_been_quiet_for_a_minute_rev_1() {
        let mut new = NewPhotos::default();
        let start = Instant::now();
        assert!(new.arrived(&ids(&[1]), start));
        let later = start + Duration::from_secs(40);
        assert!(!new.arrived(&ids(&[2]), later), "already waiting");
        assert_eq!(
            new.settle(start + QUIET, QUIET, no_path),
            Settled::Wait(Duration::from_secs(40))
        );
        assert!(new.shown().is_empty());
        assert_eq!(
            new.settle(later + QUIET, QUIET, no_path),
            Settled::Done { changed: true }
        );
        assert_eq!(new.shown(), ids(&[1, 2]));
    }

    #[test]
    fn a_scan_that_found_nothing_starts_no_wait() {
        assert!(!NewPhotos::default().arrived(&[], Instant::now()));
    }

    #[test]
    fn more_can_join_the_banner_and_a_new_wait_starts() {
        let mut new = NewPhotos::default();
        let start = Instant::now();
        new.arrived(&ids(&[1]), start);
        new.settle(start + QUIET, QUIET, no_path);
        assert!(new.arrived(&ids(&[2]), start + QUIET));
        new.settle(start + QUIET * 2, QUIET, no_path);
        assert_eq!(new.shown(), ids(&[1, 2]));
    }

    #[test]
    fn the_same_photo_found_twice_counts_once() {
        let mut new = NewPhotos::default();
        let start = Instant::now();
        new.arrived(&ids(&[1, 1]), start);
        new.settle(start + QUIET, QUIET, no_path);
        assert_eq!(new.shown(), ids(&[1]));
    }

    #[test]
    fn seura_s_own_exports_dont_count_even_if_seen_before_they_re_recorded_rev_1() {
        let folder = tempfile::tempdir().unwrap();
        let export = folder.path().join("export.jpg");
        std::fs::write(&export, b"x").unwrap();
        let mut new = NewPhotos::default();
        let start = Instant::now();
        new.arrived(&ids(&[1, 2]), start);
        new.exported(&export);
        let settled = new.settle(start + QUIET, QUIET, |id| {
            (id == PhotoId::new(1)).then(|| export.clone())
        });
        assert_eq!(settled, Settled::Done { changed: true });
        assert_eq!(new.shown(), ids(&[2]));
    }

    #[test]
    fn not_now_clears_them_and_later_arrivals_are_a_new_offer() {
        let mut new = NewPhotos::default();
        let start = Instant::now();
        new.arrived(&ids(&[1]), start);
        new.settle(start + QUIET, QUIET, no_path);
        new.dismiss();
        assert!(new.shown().is_empty());
        new.arrived(&ids(&[2]), start + QUIET);
        new.settle(start + QUIET * 2, QUIET, no_path);
        assert_eq!(new.shown(), ids(&[2]));
    }
}
