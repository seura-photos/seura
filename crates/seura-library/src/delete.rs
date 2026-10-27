//! Deleting photos (BRW-9; CLAUDE.md rules 1 and 5): on the user's explicit,
//! confirmed request, photos and their edit files go to the Recycle Bin or
//! Trash, never deleted for good. Not in read-only folders (TAG-7), and not
//! cloud-only photos, which the system can't put there: on Windows they go
//! only to the service's own deleted items, and on the Mac it refuses
//! (journal, `2026-10-26-trash-measurements.md`). Whether to delete those
//! anyway is for a design session.

use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::RwLock;

use seura_edit::edit_file_path;

use crate::index::{Index, PhotoId};
use crate::platform::is_placeholder;

/// Why a photo wasn't deleted. It's where it was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeleteProblem {
    /// Its folder is read-only (TAG-7).
    ReadOnly,
    /// It's only in the cloud, so it can't go to the Recycle Bin or Trash.
    CloudOnly,
    /// It's no longer in the library.
    Gone,
    /// The system didn't move it, such as a file in use.
    Failed(String),
}

/// What deleting did, photo by photo.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeleteReport {
    /// Moved to the Recycle Bin or Trash, with their edit files, and taken
    /// out of the index.
    pub deleted: Vec<PhotoId>,
    pub problems: Vec<(PhotoId, DeleteProblem)>,
}

impl Index {
    /// Moves photos and their edit files to the Recycle Bin or Trash with
    /// `trash_all`, all in one operation, then takes the ones that left
    /// their folders out of the index. What's still there afterwards wasn't
    /// deleted, whatever the system said.
    pub fn delete_photos(
        index: &RwLock<Index>,
        ids: &[PhotoId],
        trash_all: &dyn Fn(&[PathBuf]) -> io::Result<()>,
    ) -> DeleteReport {
        let mut report = DeleteReport::default();
        let mut chosen = Vec::new();
        {
            let index = index.read().expect("index lock");
            for &id in ids {
                let (Some(photo), Some(path)) = (index.photo(id), index.path(id)) else {
                    report.problems.push((id, DeleteProblem::Gone));
                    continue;
                };
                if index
                    .folder(photo.folder)
                    .is_some_and(|folder| folder.read_only)
                {
                    report.problems.push((id, DeleteProblem::ReadOnly));
                    continue;
                }
                // As it is now, not as last scanned.
                match fs::symlink_metadata(&path) {
                    Ok(metadata) if is_placeholder(&metadata) => {
                        report.problems.push((id, DeleteProblem::CloudOnly));
                    }
                    Ok(_) => chosen.push((id, path)),
                    Err(_) => report.problems.push((id, DeleteProblem::Gone)),
                }
            }
        }
        if chosen.is_empty() {
            return report;
        }

        let mut paths = Vec::new();
        for (_, photo) in &chosen {
            paths.push(photo.clone());
            let edit_file = edit_file_path(photo);
            if fs::symlink_metadata(&edit_file).is_ok() {
                paths.push(edit_file);
            }
        }
        let result = trash_all(&paths);

        let mut index = index.write().expect("index lock");
        for (id, photo) in chosen {
            if fs::symlink_metadata(&photo).is_ok() {
                let reason = match &result {
                    Err(error) => error.to_string(),
                    Ok(()) => "the system left it where it was".into(),
                };
                report.problems.push((id, DeleteProblem::Failed(reason)));
            } else {
                index.remove_photo(id);
                report.deleted.push(id);
            }
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    struct Library {
        dir: tempfile::TempDir,
        trash: tempfile::TempDir,
        index: RwLock<Index>,
        ids: Vec<PhotoId>,
    }

    /// Three photos, the first with an edit file.
    fn library() -> Library {
        let dir = tempfile::tempdir().unwrap();
        for n in 1..=3 {
            let mut jpeg = vec![0xFF, 0xD8, 0xFF, 0xE1];
            jpeg.extend([n; 4096]);
            fs::write(dir.path().join(format!("IMG_{n}.jpg")), jpeg).unwrap();
        }
        fs::write(dir.path().join("IMG_1.jpg.seura.json"), b"{ any }").unwrap();
        let mut index = Index::new();
        index.add_root(dir.path()).unwrap();
        let root = index.roots()[0].clone();
        index.scan(&root);
        let mut ids: Vec<_> = index.photo_ids().collect();
        ids.sort_by_key(|id| index.path(*id));
        Library {
            dir,
            trash: tempfile::tempdir().unwrap(),
            index: RwLock::new(index),
            ids,
        }
    }

    impl Library {
        /// A stand-in for the Trash: a folder outside the library.
        fn trash_all(&self, paths: &[PathBuf]) -> io::Result<()> {
            for path in paths {
                fs::rename(path, self.trash.path().join(path.file_name().unwrap()))?;
            }
            Ok(())
        }

        fn names_left(&self) -> Vec<String> {
            let mut names: Vec<_> = fs::read_dir(self.dir.path())
                .unwrap()
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            names
        }
    }

    #[test]
    fn photos_and_their_edit_files_go_to_the_trash_together() {
        let library = library();
        let sent = RefCell::new(Vec::new());
        let trash_all = |paths: &[PathBuf]| {
            sent.borrow_mut().push(paths.len());
            library.trash_all(paths)
        };
        let report = Index::delete_photos(&library.index, &library.ids[..2], &trash_all);

        assert_eq!(report.deleted, library.ids[..2]);
        assert!(report.problems.is_empty());
        // One operation for both photos and the edit file.
        assert_eq!(*sent.borrow(), [3]);
        assert_eq!(library.names_left(), ["IMG_3.jpg"]);
        assert!(library.trash.path().join("IMG_1.jpg.seura.json").exists());
        let index = library.index.read().unwrap();
        assert!(index.photo(library.ids[0]).is_none());
        assert!(index.photo(library.ids[2]).is_some());
    }

    #[test]
    fn what_the_system_left_behind_is_reported_and_kept() {
        let library = library();
        // The system moves the first file, then fails.
        let partly = |paths: &[PathBuf]| {
            library.trash_all(&paths[..1])?;
            Err(io::Error::other("in use"))
        };
        let report = Index::delete_photos(&library.index, &library.ids[1..], &partly);

        assert_eq!(report.deleted, [library.ids[1]]);
        assert_eq!(
            report.problems,
            [(library.ids[2], DeleteProblem::Failed("in use".into()))]
        );
        assert!(library.dir.path().join("IMG_3.jpg").exists());
        assert!(
            library
                .index
                .read()
                .unwrap()
                .photo(library.ids[2])
                .is_some()
        );
    }

    #[test]
    fn a_photo_no_longer_in_the_library_is_not_looked_for() {
        let library = library();
        let gone = PhotoId::new(99);
        let never = |_: &[PathBuf]| -> io::Result<()> { panic!("nothing to move") };
        let report = Index::delete_photos(&library.index, &[gone], &never);
        assert_eq!(report.problems, [(gone, DeleteProblem::Gone)]);
        assert_eq!(library.names_left().len(), 4);
    }
}
