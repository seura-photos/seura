//! Photos marked for deletion in a review (REV-4). A mark is an intention on
//! this computer, not part of the photo, so it's kept in one file in Seura's
//! own folder, never in the edit file: it mustn't create files beside photos
//! or travel to the user's other computers. Marking moves nothing.
//!
//! A mark belongs to the photo, by its path, and remembers the photo's size
//! and modification time so that it follows a rename or a move, which the
//! index sees as one photo going and another arriving. Comparing those reads
//! no photo, so it never downloads one (LIB-9), and a match must be the only
//! one: when in doubt a mark is dropped, which removes nothing.

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use seura_edit::write_atomically;

use crate::index::{FileTime, Index, PhotoId};

/// The file's name in Seura's folder.
pub const MARKS_FILE: &str = "marks.json";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Mark {
    path: PathBuf,
    size: u64,
    modified: FileTime,
}

#[derive(Debug, Default)]
pub struct Marks {
    file: PathBuf,
    marks: Vec<Mark>,
}

impl Marks {
    /// The marks in `folder`. A missing or damaged file means none: losing
    /// a mark removes nothing.
    pub fn load(folder: &Path) -> Marks {
        let file = folder.join(MARKS_FILE);
        let marks = fs::read(&file)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        Marks { file, marks }
    }

    fn id_of(index: &Index, mark: &Mark) -> Option<PhotoId> {
        let folder = index.folder_id(mark.path.parent()?)?;
        index.find(folder, mark.path.file_name()?.to_str()?)
    }

    /// The marked photos that are in the index now.
    pub fn ids(&self, index: &Index) -> HashSet<PhotoId> {
        self.marks
            .iter()
            .filter_map(|mark| Marks::id_of(index, mark))
            .collect()
    }

    /// Marks or unmarks photos, and saves. Nothing changes if saving fails.
    pub fn set(&mut self, index: &Index, ids: &[PhotoId], marked: bool) -> io::Result<()> {
        let mut next = self.marks.clone();
        for id in ids {
            let (Some(path), Some(photo)) = (index.path(*id), index.photo(*id)) else {
                continue;
            };
            next.retain(|mark| mark.path != path);
            if marked {
                next.push(Mark {
                    path,
                    size: photo.size,
                    modified: photo.modified,
                });
            }
        }
        self.replace(next)
    }

    /// Brings the marks up to date once a scan has finished: a mark follows
    /// a photo that was renamed or moved, and a photo that has gone loses
    /// its mark. Not during a scan, when the index may not hold every photo
    /// yet. A library folder on a drive that isn't connected keeps its marks.
    pub fn reconcile(&mut self, index: &Index) -> io::Result<()> {
        let mut taken = self.ids(index);
        let mut next = Vec::with_capacity(self.marks.len());
        for mark in &self.marks {
            if let Some(id) = Marks::id_of(index, mark) {
                // Still here. Its size or date may be new, for later renames.
                let photo = index.photo(id).expect("the index just found it");
                next.push(Mark {
                    path: mark.path.clone(),
                    size: photo.size,
                    modified: photo.modified,
                });
                continue;
            }
            match index
                .roots()
                .iter()
                .find(|root| mark.path.starts_with(root))
            {
                Some(root) if !root.is_dir() => {
                    next.push(mark.clone());
                    continue;
                }
                None => continue,
                Some(_) => {}
            }
            let mut same = index.photo_ids().filter(|id| {
                !taken.contains(id)
                    && index.photo(*id).is_some_and(|photo| {
                        photo.size == mark.size && photo.modified == mark.modified
                    })
            });
            if let (Some(id), None) = (same.next(), same.next())
                && let Some(path) = index.path(id)
            {
                next.push(Mark {
                    path,
                    ..mark.clone()
                });
                taken.insert(id);
            }
        }
        self.replace(next)
    }

    fn replace(&mut self, next: Vec<Mark>) -> io::Result<()> {
        if next != self.marks {
            let bytes = serde_json::to_vec_pretty(&next).map_err(io::Error::other)?;
            write_atomically(&self.file, &bytes)?;
            self.marks = next;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    struct Library {
        photos: tempfile::TempDir,
        seura: tempfile::TempDir,
        index: Index,
    }

    fn write_photo(path: &Path, n: u8) {
        let mut jpeg = vec![0xFF, 0xD8, 0xFF, 0xE1];
        jpeg.extend([n; 4096]);
        fs::write(path, jpeg).unwrap();
    }

    /// Three photos of the same size, each with its own modification time.
    fn library() -> Library {
        let photos = tempfile::tempdir().unwrap();
        for n in 1..=3 {
            let path = photos.path().join(format!("IMG_{n}.jpg"));
            write_photo(&path, n);
            set_time(&path, u64::from(n));
        }
        let mut index = Index::new();
        index.add_root(photos.path()).unwrap();
        let root = index.roots()[0].clone();
        index.scan(&root);
        Library {
            photos,
            seura: tempfile::tempdir().unwrap(),
            index,
        }
    }

    fn set_time(path: &Path, seconds: u64) {
        fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000 + seconds))
            .unwrap();
    }

    impl Library {
        fn id(&self, name: &str) -> PhotoId {
            let folder = self.index.folder_id(&self.index.roots()[0]).unwrap();
            self.index.find(folder, name).unwrap()
        }

        fn marks(&self) -> Marks {
            Marks::load(self.seura.path())
        }

        fn rescan(&mut self) {
            let root = self.index.roots()[0].clone();
            self.index.scan(&root);
        }

        fn names(&self) -> Vec<String> {
            let mut names: Vec<_> = fs::read_dir(self.photos.path())
                .unwrap()
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            names
        }

        fn marked_names(&self, marks: &Marks) -> Vec<String> {
            let mut names: Vec<_> = marks
                .ids(&self.index)
                .into_iter()
                .map(|id| self.index.photo(id).unwrap().name.to_string())
                .collect();
            names.sort();
            names
        }
    }

    #[test]
    fn marks_last_past_a_restart() {
        let library = library();
        let mut marks = library.marks();
        let ids = [library.id("IMG_1.jpg"), library.id("IMG_2.jpg")];
        marks.set(&library.index, &ids, true).unwrap();
        assert_eq!(
            library.marked_names(&library.marks()),
            ["IMG_1.jpg", "IMG_2.jpg"]
        );
    }

    #[test]
    fn unmarking_removes_only_that_photo() {
        let library = library();
        let mut marks = library.marks();
        let ids = [library.id("IMG_1.jpg"), library.id("IMG_2.jpg")];
        marks.set(&library.index, &ids, true).unwrap();
        marks.set(&library.index, &ids[..1], false).unwrap();
        assert_eq!(library.marked_names(&library.marks()), ["IMG_2.jpg"]);
    }

    #[test]
    fn marking_twice_keeps_one_mark() {
        let library = library();
        let mut marks = library.marks();
        let ids = [library.id("IMG_1.jpg")];
        marks.set(&library.index, &ids, true).unwrap();
        marks.set(&library.index, &ids, true).unwrap();
        marks.set(&library.index, &ids, false).unwrap();
        assert!(library.marks().ids(&library.index).is_empty());
    }

    /// CLAUDE.md, rules 1 and 2: only Seura's own file changes.
    #[test]
    fn marking_touches_nothing_beside_the_photos() {
        let library = library();
        let before = library.names();
        let mut marks = library.marks();
        let ids = [library.id("IMG_1.jpg")];
        marks.set(&library.index, &ids, true).unwrap();
        marks.set(&library.index, &ids, false).unwrap();
        assert_eq!(library.names(), before);
        let written: Vec<_> = fs::read_dir(library.seura.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(written, [MARKS_FILE]);
    }

    #[test]
    fn a_mark_follows_a_renamed_photo() {
        let mut library = library();
        let mut marks = library.marks();
        marks
            .set(&library.index, &[library.id("IMG_1.jpg")], true)
            .unwrap();
        fs::rename(
            library.photos.path().join("IMG_1.jpg"),
            library.photos.path().join("Renamed.jpg"),
        )
        .unwrap();
        library.rescan();
        marks.reconcile(&library.index).unwrap();
        assert_eq!(library.marked_names(&library.marks()), ["Renamed.jpg"]);
    }

    #[test]
    fn a_photo_that_has_gone_loses_its_mark() {
        let mut library = library();
        let mut marks = library.marks();
        marks
            .set(&library.index, &[library.id("IMG_1.jpg")], true)
            .unwrap();
        fs::remove_file(library.photos.path().join("IMG_1.jpg")).unwrap();
        library.rescan();
        marks.reconcile(&library.index).unwrap();
        assert!(library.marks().marks.is_empty());
    }

    #[test]
    fn a_mark_isnt_guessed_onto_one_of_two_matches() {
        let mut library = library();
        let mut marks = library.marks();
        marks
            .set(&library.index, &[library.id("IMG_1.jpg")], true)
            .unwrap();
        fs::remove_file(library.photos.path().join("IMG_1.jpg")).unwrap();
        for name in ["A.jpg", "B.jpg"] {
            let path = library.photos.path().join(name);
            write_photo(&path, 1);
            set_time(&path, 1);
        }
        library.rescan();
        marks.reconcile(&library.index).unwrap();
        assert!(library.marks().marks.is_empty());
    }

    #[test]
    fn a_marked_photo_keeps_its_mark_when_it_changes() {
        let mut library = library();
        let mut marks = library.marks();
        marks
            .set(&library.index, &[library.id("IMG_1.jpg")], true)
            .unwrap();
        set_time(&library.photos.path().join("IMG_1.jpg"), 99);
        library.rescan();
        marks.reconcile(&library.index).unwrap();
        assert_eq!(library.marked_names(&library.marks()), ["IMG_1.jpg"]);
    }

    #[test]
    fn a_folder_on_a_disconnected_drive_keeps_its_marks() {
        let library = library();
        let mut marks = library.marks();
        marks
            .set(&library.index, &[library.id("IMG_1.jpg")], true)
            .unwrap();
        let root = library.index.roots()[0].clone();
        let moved = library.seura.path().join("unplugged");
        fs::rename(&root, &moved).unwrap();
        marks
            .reconcile(&Index::with_roots(vec![root.clone()]))
            .unwrap();
        assert_eq!(library.marks().marks.len(), 1);
        fs::rename(&moved, &root).unwrap();
    }

    #[test]
    fn a_damaged_file_means_no_marks() {
        let library = library();
        fs::write(library.seura.path().join(MARKS_FILE), b"{ not json").unwrap();
        assert!(library.marks().marks.is_empty());
    }
}
