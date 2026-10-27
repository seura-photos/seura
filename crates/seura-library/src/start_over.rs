//! Starting over from an edit file that can't be read (FIL-4; CLAUDE.md
//! rules 2, 4 and 5). The user confirms with Start over; the old file is
//! then kept, unchanged, in `Unreadable edit files` in Seura's folder, named
//! with the date and time and its own name, and the photo has no edit file.
//!
//! The file is moved there. Where a move isn't possible, as from another
//! drive, it's copied there and the original moved to the Trash, never
//! deleted.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use seura_edit::{EditDocument, edit_file_path};
use time::OffsetDateTime;

use crate::changes::{Problem, io_problem};
use crate::index::{EditFileState, Index, PhotoId};
use crate::scan::ScanReport;

/// The folder in Seura's own folder that unreadable edit files are kept in.
pub const UNREADABLE_FOLDER: &str = "Unreadable edit files";

/// What starting over did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartedOver {
    /// The old file is kept here, and the photo has no edit file.
    Kept(PathBuf),
    /// The file can be read now, such as fixed or synced again since: it's
    /// left as it is, and read again.
    Readable,
    /// There's no edit file any more.
    Missing,
}

/// A move from one path to another; tests pass one that fails.
type Rename = fn(&Path, &Path) -> io::Result<()>;

impl Index {
    /// Starts one photo over from an unreadable edit file (FIL-4), keeping
    /// the old file in `kept_in` (created if needed), named with the time
    /// `at`, in the user's own time zone.
    pub fn start_over(
        index: &RwLock<Index>,
        id: PhotoId,
        kept_in: &Path,
        at: OffsetDateTime,
        trash: &dyn Fn(&Path) -> io::Result<()>,
    ) -> Result<StartedOver, Problem> {
        Index::start_over_with(index, id, kept_in, at, trash, |from, to| {
            fs::rename(from, to)
        })
    }

    fn start_over_with(
        index: &RwLock<Index>,
        id: PhotoId,
        kept_in: &Path,
        at: OffsetDateTime,
        trash: &dyn Fn(&Path) -> io::Result<()>,
        rename: Rename,
    ) -> Result<StartedOver, Problem> {
        let (photo_path, read_only) = {
            let index = index.read().expect("index lock");
            let photo = index.photo(id).ok_or(Problem::Gone)?;
            let read_only = index
                .folder(photo.folder)
                .is_some_and(|folder| folder.read_only);
            (index.path(id).ok_or(Problem::Gone)?, read_only)
        };
        // Moving the file out changes the photo's folder (TAG-7).
        if read_only {
            return Err(Problem::ReadOnly);
        }
        let edit_path = edit_file_path(&photo_path);
        let folder = photo_path.parent().unwrap_or(Path::new(""));
        let bytes = match fs::read(&edit_path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                index.write().expect("index lock").set_edit_file(
                    id,
                    EditFileState::None,
                    None,
                    0,
                    &[],
                );
                return Ok(StartedOver::Missing);
            }
            Err(error) => return Err(io_problem(&error, folder)),
        };
        if EditDocument::parse(&bytes).is_ok() {
            let metadata = fs::metadata(&edit_path).ok();
            index.write().expect("index lock").refresh_edit_file(
                id,
                &photo_path,
                metadata.as_ref(),
                &mut ScanReport::default(),
            );
            return Ok(StartedOver::Readable);
        }

        fs::create_dir_all(kept_in).map_err(|error| Problem::Failed(error.to_string()))?;
        let name = edit_path.file_name().unwrap_or_default().to_string_lossy();
        let kept = free_name(kept_in, &name, at);
        match rename(&edit_path, &kept) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::CrossesDevices => {
                copy_then_trash(&edit_path, &bytes, &kept, trash)
                    .map_err(|error| io_problem(&error, folder))?;
            }
            Err(error) => return Err(io_problem(&error, folder)),
        }
        index
            .write()
            .expect("index lock")
            .set_edit_file(id, EditFileState::None, None, 0, &[]);
        Ok(StartedOver::Kept(kept))
    }
}

/// `2026-10-26 143012 IMG_1234.jpg.seura.json` in `folder`, or with a
/// number after the time if that's taken: `2026-10-26 143012 (2) …`.
fn free_name(folder: &Path, name: &str, at: OffsetDateTime) -> PathBuf {
    let time = format!(
        "{:04}-{:02}-{:02} {:02}{:02}{:02}",
        at.year(),
        u8::from(at.month()),
        at.day(),
        at.hour(),
        at.minute(),
        at.second()
    );
    let mut path = folder.join(format!("{time} {name}"));
    let mut n = 2;
    while path.exists() {
        path = folder.join(format!("{time} ({n}) {name}"));
        n += 1;
    }
    path
}

/// Where a move isn't possible: the bytes as read go to a new file, and
/// once it's on disk the original goes to the Trash. If that fails, the
/// copy Seura just made is deleted, and the original stays where it was.
fn copy_then_trash(
    from: &Path,
    bytes: &[u8],
    to: &Path,
    trash: &dyn Fn(&Path) -> io::Result<()>,
) -> io::Result<()> {
    let written = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(to)
        .and_then(|mut file| {
            file.write_all(bytes)?;
            file.sync_all()
        });
    if let Err(error) = written {
        let _ = fs::remove_file(to);
        return Err(error);
    }
    // Only the file that was read: one changed since stays where it is.
    let unchanged = fs::read(from).is_ok_and(|current| current == bytes);
    let trashed = if unchanged {
        trash(from)
    } else {
        Err(io::Error::other(
            "the edit file changed while it was being kept",
        ))
    };
    if let Err(error) = trashed {
        let _ = fs::remove_file(to);
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use seura_edit::PhotoIdentity;
    use time::{Date, Month, Time, UtcOffset};

    use super::*;

    fn at() -> OffsetDateTime {
        let date = Date::from_calendar_date(2026, Month::October, 26).unwrap();
        let time = Time::from_hms(14, 30, 12).unwrap();
        OffsetDateTime::new_in_offset(date, time, UtcOffset::from_hms(2, 0, 0).unwrap())
    }

    struct Library {
        dir: tempfile::TempDir,
        index: RwLock<Index>,
        id: PhotoId,
    }

    /// A folder with one photo and an edit file that can't be read.
    fn library() -> Library {
        let dir = tempfile::tempdir().unwrap();
        let photos = dir.path().join("Photos");
        fs::create_dir(&photos).unwrap();
        let mut jpeg = vec![0xFF, 0xD8, 0xFF, 0xE1];
        jpeg.extend([1; 4096]);
        fs::write(photos.join("IMG_1234.jpg"), jpeg).unwrap();
        fs::write(photos.join("IMG_1234.jpg.seura.json"), b"{ not json").unwrap();
        let mut index = Index::new();
        index.add_root(&photos).unwrap();
        let root = index.roots()[0].clone();
        index.scan(&root);
        let id = index.photo_ids().next().unwrap();
        assert_eq!(
            index.photo(id).unwrap().edit_file,
            EditFileState::Unreadable
        );
        Library {
            dir,
            index: RwLock::new(index),
            id,
        }
    }

    impl Library {
        fn edit_file(&self) -> PathBuf {
            self.dir
                .path()
                .join("Photos")
                .join("IMG_1234.jpg.seura.json")
        }

        fn kept_in(&self) -> PathBuf {
            self.dir.path().join("Seura Photos").join(UNREADABLE_FOLDER)
        }

        fn state(&self) -> EditFileState {
            self.index.read().unwrap().photo(self.id).unwrap().edit_file
        }
    }

    fn no_trash(_: &Path) -> io::Result<()> {
        panic!("nothing goes to the Trash here")
    }

    #[test]
    fn the_old_file_is_kept_unchanged_named_with_the_time() {
        let library = library();
        let result = Index::start_over(
            &library.index,
            library.id,
            &library.kept_in(),
            at(),
            &no_trash,
        );
        let kept = library
            .kept_in()
            .join("2026-10-26 143012 IMG_1234.jpg.seura.json");
        assert_eq!(result, Ok(StartedOver::Kept(kept.clone())));
        assert_eq!(fs::read(&kept).unwrap(), b"{ not json");
        assert!(!library.edit_file().exists());
        assert_eq!(library.state(), EditFileState::None);

        // Another in the same second takes a number, not the same name.
        fs::write(library.edit_file(), b"{ again").unwrap();
        library
            .index
            .write()
            .unwrap()
            .scan(&library.dir.path().join("Photos"));
        let again = Index::start_over(
            &library.index,
            library.id,
            &library.kept_in(),
            at(),
            &no_trash,
        );
        let second = library
            .kept_in()
            .join("2026-10-26 143012 (2) IMG_1234.jpg.seura.json");
        assert_eq!(again, Ok(StartedOver::Kept(second.clone())));
        assert_eq!(fs::read(&kept).unwrap(), b"{ not json");
        assert_eq!(fs::read(&second).unwrap(), b"{ again");
    }

    #[test]
    fn a_file_that_reads_now_is_left_alone() {
        let library = library();
        let identity = PhotoIdentity {
            file: "IMG_1234.jpg".into(),
            size: 4100,
            sha256: None,
            width: None,
            height: None,
        };
        let readable = EditDocument::new_for_photo(&identity).to_bytes().unwrap();
        fs::write(library.edit_file(), &readable).unwrap();
        let result = Index::start_over(
            &library.index,
            library.id,
            &library.kept_in(),
            at(),
            &no_trash,
        );
        assert_eq!(result, Ok(StartedOver::Readable));
        assert_eq!(fs::read(library.edit_file()).unwrap(), readable);
        assert!(!library.kept_in().exists());
        assert_eq!(library.state(), EditFileState::Read { edited: false });
    }

    #[test]
    fn across_drives_it_is_copied_then_the_original_trashed() {
        let library = library();
        let across: Rename = |_, _| Err(io::ErrorKind::CrossesDevices.into());
        let trashed = std::cell::RefCell::new(Vec::new());
        let trash = |path: &Path| {
            trashed.borrow_mut().push(path.to_path_buf());
            fs::rename(path, path.with_extension("trashed"))
        };
        let result = Index::start_over_with(
            &library.index,
            library.id,
            &library.kept_in(),
            at(),
            &trash,
            across,
        );
        let kept = library
            .kept_in()
            .join("2026-10-26 143012 IMG_1234.jpg.seura.json");
        assert_eq!(result, Ok(StartedOver::Kept(kept.clone())));
        assert_eq!(fs::read(&kept).unwrap(), b"{ not json");
        assert_eq!(*trashed.borrow(), [library.edit_file()]);
    }

    #[test]
    fn if_the_trash_fails_the_copy_goes_and_the_original_stays() {
        let library = library();
        let across: Rename = |_, _| Err(io::ErrorKind::CrossesDevices.into());
        let refuse = |_: &Path| Err(io::Error::other("no Trash here"));
        let result = Index::start_over_with(
            &library.index,
            library.id,
            &library.kept_in(),
            at(),
            &refuse,
            across,
        );
        assert!(matches!(result, Err(Problem::Failed(_))), "{result:?}");
        assert_eq!(fs::read(library.edit_file()).unwrap(), b"{ not json");
        assert_eq!(fs::read_dir(library.kept_in()).unwrap().count(), 0);
        assert_eq!(library.state(), EditFileState::Unreadable);
    }
}
