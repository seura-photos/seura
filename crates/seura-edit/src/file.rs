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
/// the system Trash or Recycle Bin, never `fs::remove_file` (FIL-1, CLAUDE.md
/// rule 5). That arrives with M3.
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
        let current = read_if_exists(&self.path)?;
        if current != self.on_disk {
            return Err(match current.map(|bytes| EditDocument::parse(&bytes)) {
                Some(Err(error)) => SaveError::Unreadable(error),
                _ => SaveError::ChangedOnDisk,
            });
        }

        let mut document = self.document.clone();
        document.stamp(writer.app, writer.version, at);
        let bytes = document.to_bytes()?;
        write_atomically_with(&self.path, &bytes, rename)?;

        self.document = document;
        self.on_disk = Some(bytes);
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
            sha256: "0".repeat(64),
            size: 1,
            width: 1,
            height: 1,
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
    fn a_write_replaces_the_file_whole() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("IMG_1.jpg.seura.json");
        fs::write(&target, b"old contents that are longer").unwrap();
        write_atomically(&target, b"new").unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"new");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
