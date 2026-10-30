//! Undo and redo in the Library (BRW-11). The app remembers, in memory only,
//! the session's last [`UNDO_LIMIT`] actions and which photos each touched;
//! the changes themselves are only in each photo's edit file history.
//! Undoing an action is the edit format's own undo on each of those photos:
//! the record is marked undone and the previous values come back, so the
//! Library and the Editor share one history per photo.
//!
//! A photo is left alone when its latest change is no longer the one from
//! that action (it was changed since, in the Editor or by another app), or
//! when its history fails the edit format's consistency check. An edit file
//! that went to the Trash with nothing left in it is kept here, whole, and
//! written back by undo (Nevada, 2026-10-26).

use std::collections::VecDeque;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde_json::Value;
use seura_edit::{EditDocument, EditFile, Step, Writer, edit_file_path};
use time::OffsetDateTime;

use crate::changes::{Change, Written, finish};
use crate::index::{Index, PhotoId};

/// How many actions can be undone (BRW-11).
pub const UNDO_LIMIT: usize = 10;

/// One action: one change to every photo it touched.
#[derive(Debug)]
pub struct Action {
    pub change: Change,
    photos: Vec<Touched>,
}

/// One photo an action changed.
#[derive(Debug)]
struct Touched {
    id: PhotoId,
    photo: PathBuf,
    /// The history record the action wrote, without an `undone` mark.
    record: Value,
    /// The whole document, while its file is in the Trash because nothing
    /// was left in it.
    removed: Option<EditDocument>,
}

impl Action {
    pub(crate) fn new(change: Change) -> Action {
        Action {
            change,
            photos: Vec::new(),
        }
    }

    /// Remembers a photo the action changed.
    pub(crate) fn touched(&mut self, id: PhotoId, photo: &Path, written: &mut Written) {
        if let Some(record) = written.record.take() {
            self.photos.push(Touched {
                id,
                photo: photo.to_path_buf(),
                record,
                removed: written.removed.take(),
            });
        }
    }

    /// How many photos it changed.
    pub fn len(&self) -> usize {
        self.photos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.photos.is_empty()
    }
}

/// What undo or redo did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UndoReport {
    pub change: Change,
    /// Photos undone or redone.
    pub done: usize,
    /// Photos left alone: changed since, or failing the consistency check.
    pub skipped: usize,
}

#[derive(Clone, Copy)]
enum Direction {
    Undo,
    Redo,
}

/// The session's actions, for undo and redo.
#[derive(Debug, Default)]
pub struct History {
    done: VecDeque<Action>,
    undone: Vec<Action>,
}

impl History {
    /// Remembers an action. A new action can't be redone past, so the redo
    /// list is cleared, as everywhere.
    pub fn record(&mut self, action: Action) {
        if action.is_empty() {
            return;
        }
        self.undone.clear();
        self.done.push_back(action);
        if self.done.len() > UNDO_LIMIT {
            self.done.pop_front();
        }
    }

    /// What undo would undo: the change and how many photos it touched.
    pub fn next_undo(&self) -> Option<(&Change, usize)> {
        self.done
            .back()
            .map(|action| (&action.change, action.len()))
    }

    /// What redo would redo.
    pub fn next_redo(&self) -> Option<(&Change, usize)> {
        self.undone
            .last()
            .map(|action| (&action.change, action.len()))
    }

    /// Undoes the last action, whatever is selected now.
    pub fn undo(
        &mut self,
        index: &RwLock<Index>,
        writer: Writer,
        at: OffsetDateTime,
        trash: &dyn Fn(&Path) -> io::Result<()>,
    ) -> Option<UndoReport> {
        let mut action = self.done.pop_back()?;
        let outcome = step(index, &mut action, Direction::Undo, writer, at, trash);
        if !action.is_empty() {
            self.undone.push(action);
        }
        Some(outcome)
    }

    /// Redoes the last action undone.
    pub fn redo(
        &mut self,
        index: &RwLock<Index>,
        writer: Writer,
        at: OffsetDateTime,
        trash: &dyn Fn(&Path) -> io::Result<()>,
    ) -> Option<UndoReport> {
        let mut action = self.undone.pop()?;
        let outcome = step(index, &mut action, Direction::Redo, writer, at, trash);
        if !action.is_empty() {
            self.done.push_back(action);
        }
        Some(outcome)
    }
}

/// Undoes or redoes an action photo by photo. Photos left alone are dropped
/// from it, so redo doesn't try them again.
fn step(
    index: &RwLock<Index>,
    action: &mut Action,
    direction: Direction,
    writer: Writer,
    at: OffsetDateTime,
    trash: &dyn Fn(&Path) -> io::Result<()>,
) -> UndoReport {
    let before = action.photos.len();
    action.photos.retain_mut(|touched| {
        let Some(written) = step_one(touched, direction, writer, at, trash) else {
            return false;
        };
        index
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .record_written(touched.id, &touched.photo, written);
        true
    });
    UndoReport {
        change: action.change.clone(),
        done: action.photos.len(),
        skipped: before - action.photos.len(),
    }
}

/// Undoes or redoes one photo, or `None` to leave it alone.
fn step_one(
    touched: &mut Touched,
    direction: Direction,
    writer: Writer,
    at: OffsetDateTime,
    trash: &dyn Fn(&Path) -> io::Result<()>,
) -> Option<Written> {
    let path = edit_file_path(&touched.photo);
    let folder = touched.photo.parent().unwrap_or(Path::new(""));
    let mut file = match &touched.removed {
        // Written back only where nothing has appeared since; saving checks
        // that again at the last moment.
        Some(document) if !path.exists() => EditFile::create(&path, document.clone()),
        Some(_) => return None,
        None => EditFile::load(&path).ok()??,
    };
    let (next, applied) = match direction {
        Direction::Undo => (file.document.last_change(), file.document.undo()),
        Direction::Redo => (file.document.next_redo(), file.document.redo()),
    };
    // Checked before the step was taken: `next` was read first.
    if next.as_ref() != Some(&touched.record) || !matches!(applied, Ok(Step::Done(_))) {
        return None;
    }
    let mut written = finish(file, folder, writer, at, trash).ok()?;
    touched.removed = written.removed.take();
    Some(written)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fs;

    use super::*;

    const WRITER: Writer = Writer {
        app: "Seura Photos",
        version: "test",
    };

    /// A folder of stand-in JPEGs, scanned, with their ids in name order.
    fn library(photos: usize) -> (tempfile::TempDir, RwLock<Index>, Vec<PhotoId>) {
        let dir = tempfile::tempdir().unwrap();
        for n in 0..photos {
            let mut bytes = vec![0xFF, 0xD8, 0xFF, 0xE1];
            bytes.extend([n as u8; 4096]);
            fs::write(dir.path().join(format!("IMG_{n}.jpg")), bytes).unwrap();
        }
        let mut index = Index::new();
        index.add_root(dir.path()).unwrap();
        let root = index.roots()[0].clone();
        index.scan(&root);
        let mut ids: Vec<_> = index.photo_ids().collect();
        ids.sort_by_key(|id| index.path(*id));
        (dir, RwLock::new(index), ids)
    }

    /// A stand-in Trash that only remembers what it was given, and deletes
    /// it, as moving it away would. Or none, as on a USB stick, refusing.
    struct Trash(RefCell<Vec<PathBuf>>, std::cell::Cell<bool>);

    impl Trash {
        fn take(&self, path: &Path) -> io::Result<()> {
            if self.1.get() {
                return Err(seura_edit::NoTrash::error(path));
            }
            self.0.borrow_mut().push(path.to_path_buf());
            fs::remove_file(path)
        }
    }

    struct Session {
        dir: tempfile::TempDir,
        index: RwLock<Index>,
        ids: Vec<PhotoId>,
        history: History,
        trash: Trash,
    }

    impl Session {
        fn new(photos: usize) -> Session {
            let (dir, index, ids) = library(photos);
            Session {
                dir,
                index,
                ids,
                history: History::default(),
                trash: Trash(RefCell::new(Vec::new()), std::cell::Cell::new(false)),
            }
        }

        fn change(&mut self, which: &[usize], change: Change) {
            let ids: Vec<_> = which.iter().map(|n| self.ids[*n]).collect();
            let (_, action) = Index::change_recorded(
                &self.index,
                &ids,
                &change,
                WRITER,
                OffsetDateTime::now_utc(),
                &|path| self.trash.take(path),
            );
            self.history.record(action);
        }

        fn undo(&mut self) -> Option<UndoReport> {
            let trash = &self.trash;
            self.history
                .undo(&self.index, WRITER, OffsetDateTime::now_utc(), &|path| {
                    trash.take(path)
                })
        }

        fn redo(&mut self) -> Option<UndoReport> {
            let trash = &self.trash;
            self.history
                .redo(&self.index, WRITER, OffsetDateTime::now_utc(), &|path| {
                    trash.take(path)
                })
        }

        fn edit_file(&self, n: usize) -> PathBuf {
            self.dir.path().join(format!("IMG_{n}.jpg.seura.json"))
        }

        fn rating(&self, n: usize) -> u8 {
            self.index
                .read()
                .unwrap()
                .photo(self.ids[n])
                .unwrap()
                .rating
        }

        fn file_rating(&self, n: usize) -> Option<u8> {
            let bytes = fs::read(self.edit_file(n)).ok()?;
            Some(EditDocument::parse(&bytes).unwrap().rating())
        }
    }

    /// Rotate and Enhance (BRW-10): each photo's action comes from the app,
    /// and the whole is one action to undo, like a rating.
    #[test]
    fn a_change_made_photo_by_photo_is_one_action_to_undo() {
        let mut session = Session::new(2);
        let ids = session.ids.clone();
        let turn = |_: &Path, _: bool, _: Option<&EditFile>| {
            Ok(seura_edit::Action {
                label: "Rotate right".into(),
                edits: vec![seura_edit::Edit::Set(seura_edit::CoreOperation::Orient(
                    seura_edit::Orient {
                        rotate: 90,
                        flip_h: false,
                        flip_v: false,
                    },
                ))],
            })
        };
        let change = Change::Rotate { clockwise: true };
        let (report, action) = Index::change_recorded_with(
            &session.index,
            &ids,
            &change,
            Some(&turn),
            WRITER,
            OffsetDateTime::now_utc(),
            &|path| session.trash.take(path),
        );
        assert_eq!(report.changed.len(), 2);
        session.history.record(action);
        let files = [session.edit_file(0), session.edit_file(1)];
        let rotated = |n: usize| {
            fs::read(&files[n]).is_ok_and(|bytes| {
                EditDocument::parse(&bytes)
                    .unwrap()
                    .core(seura_edit::CoreOpId::Orient)
                    .is_some()
            })
        };
        assert!(rotated(0) && rotated(1));
        let undone = session.undo().unwrap();
        assert_eq!((undone.change, undone.done), (change, 2));
        assert!(!rotated(0) && !rotated(1));
        // Without the app's actions, a rotation changes nothing.
        let (report, _) = Index::change_recorded(
            &session.index,
            &ids,
            &Change::Enhance,
            WRITER,
            OffsetDateTime::now_utc(),
            &|path| session.trash.take(path),
        );
        assert!(report.changed.is_empty());
    }

    #[test]
    fn undo_undoes_the_last_action_on_every_photo_it_touched() {
        let mut session = Session::new(3);
        session.change(&[0, 1, 2], Change::Rating(2));
        session.change(&[0, 1], Change::Rating(5));
        assert_eq!(session.history.next_undo(), Some((&Change::Rating(5), 2)));

        let outcome = session.undo().unwrap();
        assert_eq!(
            outcome,
            UndoReport {
                change: Change::Rating(5),
                done: 2,
                skipped: 0
            }
        );
        assert_eq!((session.rating(0), session.rating(1)), (2, 2));
        assert_eq!(session.file_rating(1), Some(2));
        // The record is marked undone in the file, never a new one added.
        let document = EditDocument::parse(&fs::read(session.edit_file(0)).unwrap()).unwrap();
        assert_eq!(document.history_len(), 2);
        assert_eq!(document.redo_steps(), 1);

        session.redo().unwrap();
        assert_eq!((session.rating(0), session.file_rating(0)), (5, Some(5)));
        assert_eq!(session.rating(2), 2);
    }

    #[test]
    fn a_photo_changed_since_is_left_alone() {
        let mut session = Session::new(2);
        session.change(&[0, 1], Change::Rating(3));
        // Photo 1 is changed again outside this action, as the Editor would.
        let mut file = EditFile::load(&session.edit_file(1)).unwrap().unwrap();
        file.document
            .apply(
                &seura_edit::Action {
                    label: "Rating".into(),
                    edits: vec![seura_edit::Edit::Rating(4)],
                },
                OffsetDateTime::now_utc(),
            )
            .unwrap();
        file.save(WRITER, OffsetDateTime::now_utc()).unwrap();

        let outcome = session.undo().unwrap();
        assert_eq!((outcome.done, outcome.skipped), (1, 1));
        assert_eq!(session.file_rating(0), None, "undone back to nothing");
        assert_eq!(session.file_rating(1), Some(4), "left alone");
        // Redo only tries the photo undo changed.
        assert_eq!(session.redo().unwrap().skipped, 0);
    }

    #[test]
    fn an_edit_file_sent_to_the_trash_comes_back_with_undo() {
        let mut session = Session::new(1);
        session.change(&[0], Change::Rating(3));
        session.change(&[0], Change::Rating(0));
        assert!(!session.edit_file(0).exists(), "emptied, so in the Trash");
        assert_eq!(session.trash.0.borrow().len(), 1);

        // Undo writes it back, with its whole history, the record undone.
        session.undo().unwrap();
        assert_eq!(session.file_rating(0), Some(3));
        assert_eq!(session.rating(0), 3);
        let document = EditDocument::parse(&fs::read(session.edit_file(0)).unwrap()).unwrap();
        assert_eq!((document.history_len(), document.redo_steps()), (2, 1));

        // Redo empties it again, to the Trash; undoing the first rating
        // empties it too, and redo brings that back.
        session.redo().unwrap();
        assert!(!session.edit_file(0).exists());
        session.undo().unwrap();
        session.undo().unwrap();
        assert!(!session.edit_file(0).exists());
        assert_eq!(session.rating(0), 0);
        session.redo().unwrap();
        assert_eq!(session.file_rating(0), Some(3));
    }

    #[test]
    fn with_no_trash_an_emptied_file_is_deleted_and_undo_still_brings_it_back() {
        let mut session = Session::new(1);
        session.trash.1.set(true);
        session.change(&[0], Change::Rating(3));
        session.change(&[0], Change::Rating(0));
        assert!(!session.edit_file(0).exists(), "emptied, so deleted");
        assert!(session.trash.0.borrow().is_empty());

        // Undo writes a new file with the rating that was cleared (BRW-11).
        session.undo().unwrap();
        assert_eq!(session.file_rating(0), Some(3));
        assert_eq!(session.rating(0), 3);
        session.redo().unwrap();
        assert!(!session.edit_file(0).exists());
        assert_eq!(session.rating(0), 0);
    }

    #[test]
    fn a_file_that_appeared_where_one_was_trashed_is_never_replaced() {
        let mut session = Session::new(1);
        session.change(&[0], Change::Rating(3));
        session.change(&[0], Change::Rating(0));
        fs::write(session.edit_file(0), b"{ someone else's").unwrap();

        assert_eq!(session.undo().unwrap().skipped, 1);
        assert_eq!(fs::read(session.edit_file(0)).unwrap(), b"{ someone else's");
    }

    #[test]
    fn only_the_last_ten_actions_are_kept_and_a_new_one_clears_redo() {
        let mut session = Session::new(1);
        for stars in 1..=5 {
            session.change(&[0], Change::Rating(stars));
            session.change(&[0], Change::AddTag(format!("tag {stars}")));
        }
        session.change(&[0], Change::Rating(1));
        let mut undone = 0;
        while session.undo().is_some() {
            undone += 1;
        }
        assert_eq!(undone, UNDO_LIMIT);

        session.change(&[0], Change::Rating(4));
        assert_eq!(session.history.next_redo(), None);
        assert!(session.redo().is_none());
    }

    #[test]
    fn a_change_that_changed_nothing_is_not_an_action() {
        let mut session = Session::new(1);
        session.change(&[0], Change::Rating(0));
        assert_eq!(session.history.next_undo(), None);
    }
}
