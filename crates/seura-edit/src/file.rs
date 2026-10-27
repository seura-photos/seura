//! Edit files on disk: naming, loading, and safe saving (FIL-2, FIL-4;
//! CLAUDE.md rules 3 and 4).

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use time::OffsetDateTime;

use crate::document::{EditDocument, EditError, ReadError};

/// The edit file beside a photo: `IMG_1234.jpg` → `IMG_1234.jpg.seura.json`.
pub fn edit_file_path(photo: &Path) -> PathBuf {
    let mut name = photo.file_name().unwrap_or_default().to_os_string();
    name.push(".seura.json");
    photo.with_file_name(name)
}

/// Why an edit file couldn't be loaded.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("couldn't read the edit file: {0}")]
    Io(#[from] io::Error),
    #[error(transparent)]
    Read(#[from] ReadError),
}

/// Why a save didn't happen. In every case the file on disk is unchanged.
#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("couldn't write the edit file: {0}")]
    Io(#[from] io::Error),
    /// The file on disk can't be read, so it's never overwritten
    /// (CLAUDE.md rule 4; FIL-4).
    #[error("the edit file on disk can't be read, so it wasn't overwritten: {0}")]
    Unreadable(ReadError),
    /// The file changed on disk since it was loaded, for example synced from
    /// another computer (FIL-3). Reload before saving.
    #[error("the edit file changed on disk since it was loaded")]
    ChangedOnDisk,
    #[error(transparent)]
    Invalid(#[from] EditError),
}

/// A Trash function's refusal where there is no Recycle Bin or Trash, such
/// as on a USB stick or a network share (decision 0017), wrapped in the
/// `io::Error` it returns before moving anything.
#[derive(Debug, thiserror::Error)]
#[error("{} is on a drive with no Recycle Bin or Trash, so it's left where it is", .0.display())]
pub struct NoTrash(pub PathBuf);

impl NoTrash {
    pub fn error(path: &Path) -> io::Error {
        io::Error::other(NoTrash(path.to_path_buf()))
    }

    pub fn is(error: &io::Error) -> bool {
        error.get_ref().is_some_and(|inner| inner.is::<NoTrash>())
    }
}

/// The app writing a file, recorded in its `writer` field.
#[derive(Clone, Copy, Debug)]
pub struct Writer<'a> {
    pub app: &'a str,
    pub version: &'a str,
}

/// An edit file and the document in it.
///
/// There is deliberately no delete. When a photo has no edits, tags, or
/// rating left and the file holds nothing else to preserve, the file goes to
/// the system Trash or Recycle Bin with [`EditFile::remove`], and is deleted
/// directly only where there is none and nothing in it can be lost (FIL-1,
/// CLAUDE.md rule 5, decision 0017).
#[derive(Debug)]
pub struct EditFile {
    path: PathBuf,
    /// The bytes on disk when loaded or last saved; `None` if there was no file.
    on_disk: Option<Vec<u8>>,
    pub document: EditDocument,
}

impl EditFile {
    /// Loads the edit file at `path`, or `None` if there isn't one.
    pub fn load(path: &Path) -> Result<Option<EditFile>, LoadError> {
        let Some(bytes) = read_if_exists(path)? else {
            return Ok(None);
        };
        let document = EditDocument::parse(&bytes)?;
        Ok(Some(EditFile {
            path: path.to_path_buf(),
            on_disk: Some(bytes),
            document,
        }))
    }

    /// A new edit file at `path`, not yet written. Saving it fails if a file
    /// appears there first.
    pub fn create(path: &Path, document: EditDocument) -> EditFile {
        EditFile {
            path: path.to_path_buf(),
            on_disk: None,
            document,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The bytes on disk, if they're still those loaded or last saved.
    /// Otherwise the file can't be read, or it changed since (FIL-3, FIL-4),
    /// and it mustn't be replaced.
    fn check_unchanged(&self) -> Result<Option<Vec<u8>>, SaveError> {
        let current = read_if_exists(&self.path)?;
        if current != self.on_disk {
            return Err(match current.map(|bytes| EditDocument::parse(&bytes)) {
                Some(Err(error)) => SaveError::Unreadable(error),
                _ => SaveError::ChangedOnDisk,
            });
        }
        Ok(current)
    }

    /// Writes the document atomically: a temporary file in the same folder,
    /// flushed to disk, then renamed over the old file (FIL-2). Refuses to
    /// replace a file it can't read or one that changed since it was loaded.
    pub fn save(&mut self, writer: Writer, at: OffsetDateTime) -> Result<(), SaveError> {
        self.save_with(writer, at, rename)
    }

    /// `save`, with the final rename passed in so tests can make it fail.
    fn save_with(
        &mut self,
        writer: Writer,
        at: OffsetDateTime,
        rename: Rename,
    ) -> Result<(), SaveError> {
        self.check_unchanged()?;

        let mut document = self.document.clone();
        document.stamp(writer.app, writer.version, at);
        let bytes = document.to_bytes()?;
        write_atomically_with(&self.path, &bytes, rename)?;

        self.document = document;
        self.on_disk = Some(bytes);
        Ok(())
    }

    /// Moves the file to the Trash or Recycle Bin with `trash` (FIL-1,
    /// CLAUDE.md rule 5), after checking it's still the file that was
    /// loaded, as saving does. A file never saved has nothing to move. Where
    /// `trash` refuses with [`NoTrash`], a document with nothing left in it
    /// is deleted directly instead (decision 0017): it's saved as it is now,
    /// read back, and deleted only if the file holds exactly that, and that
    /// is nothing to keep. Otherwise the file stays.
    pub fn remove(
        self,
        trash: impl FnOnce(&Path) -> io::Result<()>,
        writer: Writer,
        at: OffsetDateTime,
    ) -> Result<(), SaveError> {
        if self.check_unchanged()?.is_none() {
            return Ok(());
        }
        match trash(&self.path) {
            Err(refused) if NoTrash::is(&refused) && self.document.is_empty() => {
                self.delete_emptied(refused, writer, at)
            }
            result => Ok(result?),
        }
    }

    fn delete_emptied(
        mut self,
        refused: io::Error,
        writer: Writer,
        at: OffsetDateTime,
    ) -> Result<(), SaveError> {
        self.save(writer, at)?;
        // Read again just before: only these bytes, holding nothing.
        let current = read_if_exists(&self.path)?;
        let holds_nothing = current.as_deref().is_some_and(|bytes| {
            Some(bytes) == self.on_disk.as_deref()
                && EditDocument::parse(bytes).is_ok_and(|document| document.is_empty())
        });
        if !holds_nothing {
            return Err(refused.into());
        }
        // One of the three files Seura deletes directly (CLAUDE.md rule 5;
        // decision 0017): an edit file with nothing in it.
        fs::remove_file(&self.path)?;
        Ok(())
    }
}

fn read_if_exists(path: &Path) -> io::Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// The final step of an atomic write, passed in so tests can make it fail.
type Rename = fn(&Path, &Path) -> io::Result<()>;

fn rename(from: &Path, to: &Path) -> io::Result<()> {
    fs::rename(from, to)
}

/// Writes `bytes` to `path` atomically: a new temporary file beside it,
/// flushed to disk, then renamed over `path` (CLAUDE.md rule 3). On failure
/// `path` is unchanged and the temporary file is removed. Seura uses this for
/// every file it writes: edit files, and the cache's index snapshot.
pub fn write_atomically(path: &Path, bytes: &[u8]) -> io::Result<()> {
    write_atomically_with(path, bytes, rename)
}

fn write_atomically_with(path: &Path, bytes: &[u8], rename: Rename) -> io::Result<()> {
    let temp = temporary_path(path);
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        rename(&temp, path)?;
        sync_folder(path)
    })();
    if result.is_err() {
        // The one file deleted directly rather than sent to the Trash: the
        // temporary file this save just created (CLAUDE.md rule 5).
        let _ = fs::remove_file(&temp);
    }
    result
}

/// `IMG_1234.jpg.seura.json` → `IMG_1234.jpg.seura.json.<pid>-<n>.tmp`, in
/// the same folder so the rename can't cross drives. Visible, not hidden:
/// if a crash leaves one behind, the user can see it.
fn temporary_path(path: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{}-{n}.tmp", std::process::id()));
    path.with_file_name(name)
}

/// Makes the rename itself durable on Unix by flushing the folder. Windows
/// has no portable way to flush a folder; the file's data is already flushed.
#[cfg(unix)]
fn sync_folder(path: &Path) -> io::Result<()> {
    match path.parent() {
        Some(folder) if !folder.as_os_str().is_empty() => fs::File::open(folder)?.sync_all(),
        _ => Ok(()),
    }
}

#[cfg(not(unix))]
fn sync_folder(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_files_keep_the_photo_extension() {
        let path = edit_file_path(Path::new("Summer/IMG_1234.jpg"));
        assert_eq!(path, Path::new("Summer/IMG_1234.jpg.seura.json"));
        let heic = edit_file_path(Path::new("Summer/IMG_1234.heic"));
        assert_ne!(path, heic);
    }

    #[test]
    fn a_failed_write_leaves_no_temporary_file() {
        let dir = tempfile::tempdir().unwrap();
        // A folder where the file should go makes the rename fail on every platform.
        let target = dir.path().join("IMG_1.jpg.seura.json");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("inside"), b"keep").unwrap();

        assert!(write_atomically(&target, b"{}").is_err());

        let names: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(names, ["IMG_1.jpg.seura.json"]);
        assert_eq!(fs::read(target.join("inside")).unwrap(), b"keep");
    }

    #[test]
    fn a_failed_save_leaves_no_temporary_file_and_keeps_the_edit_file() {
        use crate::{Action, Edit, PhotoIdentity};

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("IMG_1.jpg.seura.json");
        let photo = PhotoIdentity {
            file: "IMG_1.jpg".into(),
            size: 1,
            sha256: None,
            width: None,
            height: None,
        };
        let writer = Writer {
            app: "Seura Photos",
            version: "test",
        };
        let rating = |stars| Action {
            label: "Rating".into(),
            edits: vec![Edit::Rating(stars)],
        };

        let mut file = EditFile::create(&path, EditDocument::new_for_photo(&photo));
        file.document
            .apply(&rating(2), OffsetDateTime::now_utc())
            .unwrap();
        file.save(writer, OffsetDateTime::now_utc()).unwrap();
        let saved = fs::read(&path).unwrap();

        // The temporary file is fully written and flushed; only the rename fails.
        file.document
            .apply(&rating(5), OffsetDateTime::now_utc())
            .unwrap();
        let fail: Rename = |_, _| Err(io::Error::other("simulated failure"));
        let result = file.save_with(writer, OffsetDateTime::now_utc(), fail);

        assert!(matches!(result, Err(SaveError::Io(_))));
        assert_eq!(fs::read(&path).unwrap(), saved, "the edit file changed");
        let names: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(
            names,
            ["IMG_1.jpg.seura.json"],
            "a temporary file was left behind"
        );

        // The next save still works, and writes the new rating.
        file.save(writer, OffsetDateTime::now_utc()).unwrap();
        let reloaded = EditFile::load(&path).unwrap().unwrap();
        assert_eq!(reloaded.document.rating(), 5);
    }

    #[test]
    fn removing_goes_through_the_trash_and_only_for_the_file_loaded() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("IMG_1.jpg.seura.json");
        let photo = crate::PhotoIdentity {
            file: "IMG_1.jpg".into(),
            size: 1,
            sha256: None,
            width: None,
            height: None,
        };
        let writer = Writer {
            app: "Seura Photos",
            version: "test",
        };
        let trashed = std::cell::RefCell::new(Vec::new());
        let trash = |path: &Path| {
            trashed.borrow_mut().push(path.to_path_buf());
            Ok(())
        };

        // Never saved: nothing to move.
        let unsaved = EditFile::create(&path, EditDocument::new_for_photo(&photo));
        unsaved
            .remove(trash, writer, OffsetDateTime::now_utc())
            .unwrap();
        assert!(trashed.borrow().is_empty());

        let mut file = EditFile::create(&path, EditDocument::new_for_photo(&photo));
        file.save(writer, OffsetDateTime::now_utc()).unwrap();
        // Changed on disk since: kept, and not sent to the Trash (FIL-3).
        let loaded = EditFile::load(&path).unwrap().unwrap();
        file.save(
            writer,
            OffsetDateTime::now_utc() + time::Duration::seconds(5),
        )
        .unwrap();
        assert!(matches!(
            loaded.remove(trash, writer, OffsetDateTime::now_utc()),
            Err(SaveError::ChangedOnDisk)
        ));
        assert!(trashed.borrow().is_empty());

        file.remove(trash, writer, OffsetDateTime::now_utc())
            .unwrap();
        assert_eq!(*trashed.borrow(), std::slice::from_ref(&path));
        // The trash function moved nothing here, so the file is still there:
        // `remove` itself never deletes.
        assert!(path.exists());
    }

    #[test]
    fn where_there_is_no_trash_only_a_file_holding_nothing_is_deleted() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("IMG_1.jpg.seura.json");
        let photo = crate::PhotoIdentity {
            file: "IMG_1.jpg".into(),
            size: 1,
            sha256: None,
            width: None,
            height: None,
        };
        let writer = Writer {
            app: "Seura Photos",
            version: "test",
        };
        let no_trash = |path: &Path| Err(NoTrash::error(path));
        let rate = |rating: u8| crate::Action {
            label: "Rating".into(),
            edits: vec![crate::Edit::Rating(rating)],
        };
        let rated = |rating: u8| {
            // A fresh file each time, in the test's own folder.
            let _ = fs::remove_file(&path);
            let mut file = EditFile::create(&path, EditDocument::new_for_photo(&photo));
            file.document
                .apply(&rate(rating), OffsetDateTime::now_utc())
                .unwrap();
            file.save(writer, OffsetDateTime::now_utc()).unwrap();
            EditFile::load(&path).unwrap().unwrap()
        };

        // Still holding a rating: refused, and the file is as it was.
        let file = rated(3);
        let before = fs::read(&path).unwrap();
        let error = file.remove(no_trash, writer, OffsetDateTime::now_utc());
        assert!(matches!(error, Err(SaveError::Io(error)) if NoTrash::is(&error)));
        assert_eq!(fs::read(&path).unwrap(), before);

        // Cleared: deleted directly.
        let mut file = rated(3);
        file.document
            .apply(&rate(0), OffsetDateTime::now_utc())
            .unwrap();
        assert!(file.document.is_empty());
        file.remove(no_trash, writer, OffsetDateTime::now_utc())
            .unwrap();
        assert!(!path.exists());

        // Another app's extension: not empty, so kept, holding it.
        rated(3);
        let mut json: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        json["extensions"] = serde_json::json!({ "com.example.notes": { "note": "keep me" } });
        let text = serde_json::to_vec(&json).unwrap();
        fs::write(&path, &text).unwrap();
        let mut file = EditFile::load(&path).unwrap().unwrap();
        file.document
            .apply(&rate(0), OffsetDateTime::now_utc())
            .unwrap();
        assert!(!file.document.is_empty());
        assert!(
            file.remove(no_trash, writer, OffsetDateTime::now_utc())
                .is_err()
        );
        assert_eq!(fs::read(&path).unwrap(), text);
    }

    #[test]
    fn a_write_replaces_the_file_whole() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("IMG_1.jpg.seura.json");
        fs::write(&target, b"old contents that are longer").unwrap();
        write_atomically(&target, b"new").unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"new");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
