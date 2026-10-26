//! Changing photos' tags and ratings from the Library (M3 slice 1): the one
//! path every view uses to write edit files, so the rules live in one place
//! (FIL-1, FIL-2, TAG-7; CLAUDE.md rules 1 to 5).
//!
//! Each photo's edit file is read, changed, and saved atomically, or moved to
//! the Trash when nothing is left in it, and the index is updated with what
//! was written. Files are written without holding the index's lock, which is
//! taken for each photo's lookup and record, so browsing goes on meanwhile.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use seura_edit::{
    Action, Edit, EditDocument, EditFile, PhotoIdentity, SaveError, Writer, edit_file_path,
};
use time::OffsetDateTime;

use crate::fingerprint::{fingerprint_file, to_hex};
use crate::index::{EditFileState, FileTime, Index, PhotoId};
use crate::platform::{can_add_files, is_read_only_error};

/// One change, made to every photo in a selection: one action (BRW-11).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    /// 0 to 5 stars; 0 clears the rating (TAG-6).
    Rating(u8),
    /// Adds the tag where it's missing (TAG-3).
    AddTag(String),
    /// Removes the tag where it's present (TAG-4).
    RemoveTag(String),
}

impl Change {
    /// The label recorded in each edit file's history.
    pub fn label(&self) -> &'static str {
        match self {
            Change::Rating(_) => "Rating",
            Change::AddTag(_) => "Add tag",
            Change::RemoveTag(_) => "Remove tag",
        }
    }
}

/// Why a photo couldn't be changed. Its edit file is as it was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    /// The folder is read-only (TAG-7).
    ReadOnly,
    /// The edit file exists but can't be read, so it's never overwritten
    /// (FIL-4; CLAUDE.md rule 4).
    Unreadable,
    /// The edit file changed on disk while this change was being made
    /// (FIL-3); the watcher reads it again.
    ChangedOnDisk,
    /// The photo is no longer in the library.
    Gone,
    /// Anything else, such as a full drive.
    Failed(String),
}

/// What a change did, photo by photo.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChangeReport {
    /// Photos whose edit file was written, created, or moved to the Trash.
    pub changed: Vec<PhotoId>,
    /// Photos the change couldn't be made to.
    pub problems: Vec<(PhotoId, Problem)>,
}

/// A photo's fingerprint, with the size and modification time it's valid for.
type Fingerprint = ([u8; 32], u64, FileTime);

/// What's needed to change one photo, copied from the index.
struct Target {
    path: PathBuf,
    placeholder: bool,
    read_only: bool,
    /// The index's fingerprint, if it has one.
    fingerprint: Option<Fingerprint>,
    dimensions: Option<(u32, u32)>,
}

/// What was written for one photo.
pub(crate) struct Written {
    state: EditFileState,
    stamp: Option<(u64, FileTime)>,
    rating: u8,
    tags: Vec<String>,
    /// The fingerprint computed for a new edit file, to cache.
    fingerprint: Option<Fingerprint>,
    /// The record the change wrote, for undo to find again (BRW-11).
    pub(crate) record: Option<serde_json::Value>,
    /// The document, when the file went to the Trash with nothing left in
    /// it, for undo to bring back (BRW-11; Nevada, 2026-10-26).
    pub(crate) removed: Option<EditDocument>,
}

impl Index {
    /// Makes `change` to each photo in `ids`, writing their edit files.
    /// `trash` moves a file to the Recycle Bin or Trash: [`crate::move_to_trash`],
    /// or a stand-in in tests. One change at a time: the caller sees to that,
    /// so two never interleave on a photo.
    pub fn change_shared(
        index: &RwLock<Index>,
        ids: &[PhotoId],
        change: &Change,
        writer: Writer,
        at: OffsetDateTime,
        trash: &dyn Fn(&Path) -> io::Result<()>,
    ) -> ChangeReport {
        Index::change_recorded(index, ids, change, writer, at, trash).0
    }

    /// [`Index::change_shared`], also returning the action as undo needs
    /// it: what each photo's history recorded (BRW-11).
    pub fn change_recorded(
        index: &RwLock<Index>,
        ids: &[PhotoId],
        change: &Change,
        writer: Writer,
        at: OffsetDateTime,
        trash: &dyn Fn(&Path) -> io::Result<()>,
    ) -> (ChangeReport, crate::undo::Action) {
        let mut report = ChangeReport::default();
        let mut action = crate::undo::Action::new(change.clone());
        // A typed tag takes the form the library already has, whatever its
        // case (TAG-2), so `Family` adds `family`.
        let resolved;
        let change = match change {
            Change::AddTag(tag) => {
                let Some(tag) = seura_edit::clean_tags(std::slice::from_ref(tag)).pop() else {
                    return (report, action);
                };
                let existing = index
                    .read()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .matching_tag(&tag)
                    .map(String::from);
                resolved = Change::AddTag(existing.unwrap_or(tag));
                &resolved
            }
            other => other,
        };
        for &id in ids {
            let target = index
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .target(id);
            let Some(target) = target else {
                report.problems.push((id, Problem::Gone));
                continue;
            };
            if target.read_only {
                report.problems.push((id, Problem::ReadOnly));
                continue;
            }
            match write(&target, change, writer, at, trash) {
                Ok(None) => {}
                Ok(Some(mut written)) => {
                    action.touched(id, &target.path, &mut written);
                    index
                        .write()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .record_written(id, &target.path, written);
                    report.changed.push(id);
                }
                Err(problem) => {
                    if problem == Problem::ReadOnly {
                        index
                            .write()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .mark_read_only(id);
                    }
                    report.problems.push((id, problem));
                }
            }
        }
        (report, action)
    }

    fn target(&self, id: PhotoId) -> Option<Target> {
        let photo = self.photo(id)?;
        let folder = self.folder(photo.folder)?;
        Some(Target {
            path: folder.path.join(&*photo.name),
            placeholder: photo.placeholder,
            read_only: folder.read_only,
            fingerprint: photo
                .fingerprint
                .map(|fingerprint| (fingerprint, photo.size, photo.modified)),
            dimensions: photo.dimensions,
        })
    }

    /// Records what was written for a photo, unless the photo changed or
    /// moved meanwhile, in which case a scan catches up. Its new stamp means
    /// the watcher's report of this write finds nothing to read again.
    pub(crate) fn record_written(&mut self, id: PhotoId, path: &Path, written: Written) {
        if self.path(id).as_deref() != Some(path) {
            return;
        }
        if let Some((fingerprint, size, modified)) = written.fingerprint
            && let Some(photo) = self.photo_mut(id)
            && (photo.size, photo.modified) == (size, modified)
        {
            photo.fingerprint = Some(fingerprint);
        }
        self.set_edit_file(
            id,
            written.state,
            written.stamp,
            written.rating,
            &written.tags,
        );
    }

    fn mark_read_only(&mut self, id: PhotoId) {
        if let Some(folder) = self.photo(id).map(|photo| photo.folder) {
            self.set_read_only(folder, true);
        }
    }
}

/// Changes one photo's edit file. `None` if the change changes nothing.
fn write(
    target: &Target,
    change: &Change,
    writer: Writer,
    at: OffsetDateTime,
    trash: &dyn Fn(&Path) -> io::Result<()>,
) -> Result<Option<Written>, Problem> {
    let edit_path = edit_file_path(&target.path);
    let folder = target.path.parent().unwrap_or(Path::new(""));
    let loaded = EditFile::load(&edit_path).map_err(|error| match error {
        seura_edit::LoadError::Read(_) => Problem::Unreadable,
        seura_edit::LoadError::Io(error) => io_problem(&error, folder),
    })?;
    let current_tags = loaded
        .as_ref()
        .map(|file| file.document.tags())
        .unwrap_or_default();
    let edit = match change {
        Change::Rating(stars) => Edit::Rating((*stars).min(5)),
        Change::AddTag(tag) => {
            let mut tags = current_tags;
            tags.push(tag.clone());
            Edit::Tags(tags)
        }
        Change::RemoveTag(tag) => {
            let key = seura_edit::tag_key(tag);
            Edit::Tags(
                current_tags
                    .into_iter()
                    .filter(|t| seura_edit::tag_key(t) != key)
                    .collect(),
            )
        }
    };
    let action = Action {
        label: change.label().into(),
        edits: vec![edit],
    };

    let mut fingerprint = None;
    let mut file = match loaded {
        Some(mut file) => {
            if !target.placeholder
                && let Some((width, height)) = target.dimensions
                && file
                    .document
                    .photo()
                    .is_some_and(|photo| photo.sha256.is_none())
            {
                // The photo is on disk now: fill in what a cloud-only photo's
                // file was created without (LIB-9).
                let found =
                    photo_fingerprint(target).map_err(|error| io_problem(&error, folder))?;
                file.document
                    .fill_in_photo(&to_hex(&found.0), width, height)
                    .map_err(|error| Problem::Failed(error.to_string()))?;
                fingerprint = Some(found);
            }
            file
        }
        None => {
            let (identity, found) = identity(target).map_err(|error| io_problem(&error, folder))?;
            fingerprint = found;
            EditFile::create(&edit_path, EditDocument::new_for_photo(&identity))
        }
    };
    let changed = file
        .document
        .apply(&action, at)
        .map_err(|error| Problem::Failed(error.to_string()))?;
    if !changed {
        return Ok(None);
    }

    let record = file.document.last_change();
    let mut written = finish(file, folder, writer, at, trash)?;
    written.fingerprint = fingerprint;
    written.record = record;
    Ok(Some(written))
}

/// Saves an edit file, or moves it to the Trash when nothing in it is worth
/// keeping (FIL-1), and says what's on disk now. A file that went to the
/// Trash keeps its document in `removed`, for undo to bring back.
pub(crate) fn finish(
    mut file: EditFile,
    folder: &Path,
    writer: Writer,
    at: OffsetDateTime,
    trash: &dyn Fn(&Path) -> io::Result<()>,
) -> Result<Written, Problem> {
    let rating = file.document.rating();
    let tags = file.document.tags();
    if file.document.is_empty() {
        let document = file.document.clone();
        file.remove(trash)
            .map_err(|error| save_problem(error, folder))?;
        return Ok(Written {
            state: EditFileState::None,
            stamp: None,
            rating,
            tags,
            fingerprint: None,
            record: None,
            removed: Some(document),
        });
    }
    let edited = !file.document.operations().is_empty();
    file.save(writer, at)
        .map_err(|error| save_problem(error, folder))?;
    let metadata = std::fs::metadata(file.path()).map_err(|error| io_problem(&error, folder))?;
    let stamp = metadata
        .modified()
        .ok()
        .map(|modified| (metadata.len(), FileTime::from(modified)));
    Ok(Written {
        state: EditFileState::Read { edited },
        stamp,
        rating,
        tags,
        fingerprint: None,
        record: None,
        removed: None,
    })
}

/// The identity for a new edit file (edit format: "Photo identity"). A
/// cloud-only photo has no fingerprint or dimensions yet: reading it would
/// download it (LIB-9). Dimensions only come from decoding, never a guess.
fn identity(target: &Target) -> io::Result<(PhotoIdentity, Option<Fingerprint>)> {
    let size = std::fs::metadata(&target.path)?.len();
    let (sha256, dimensions, found) = if target.placeholder {
        (None, None, None)
    } else {
        let found = photo_fingerprint(target)?;
        (Some(to_hex(&found.0)), target.dimensions, Some(found))
    };
    let identity = PhotoIdentity {
        file: target
            .path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        size,
        sha256,
        width: dimensions.map(|(width, _)| width),
        height: dimensions.map(|(_, height)| height),
    };
    Ok((identity, found))
}

/// The photo's fingerprint: the index's, if the file hasn't changed since,
/// or read now. Returned with the size and time it's valid for.
fn photo_fingerprint(target: &Target) -> io::Result<Fingerprint> {
    let metadata = std::fs::metadata(&target.path)?;
    let stamp = (metadata.len(), FileTime::from(metadata.modified()?));
    if let Some((fingerprint, size, modified)) = target.fingerprint
        && (size, modified) == stamp
    {
        return Ok((fingerprint, size, modified));
    }
    Ok((fingerprint_file(&target.path)?, stamp.0, stamp.1))
}

/// A write the system refused counts as a read-only folder only if the
/// folder itself refuses new files: a single read-only edit file doesn't
/// make its folder read-only.
pub(crate) fn io_problem(error: &io::Error, folder: &Path) -> Problem {
    if is_read_only_error(error) && !can_add_files(folder) {
        Problem::ReadOnly
    } else {
        Problem::Failed(error.to_string())
    }
}

pub(crate) fn save_problem(error: SaveError, folder: &Path) -> Problem {
    match error {
        SaveError::Unreadable(_) => Problem::Unreadable,
        SaveError::ChangedOnDisk => Problem::ChangedOnDisk,
        SaveError::Io(error) => io_problem(&error, folder),
        SaveError::Invalid(error) => Problem::Failed(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const WRITER: Writer = Writer {
        app: "Seura Photos",
        version: "test",
    };

    fn no_trash(path: &Path) -> io::Result<()> {
        panic!("nothing should go to the Trash: {}", path.display())
    }

    /// A folder with one stand-in JPEG, scanned.
    fn library() -> (tempfile::TempDir, RwLock<Index>, PhotoId) {
        let dir = tempfile::tempdir().unwrap();
        let mut bytes = vec![0xFF, 0xD8, 0xFF, 0xE1];
        bytes.extend([7; 4096]);
        fs::write(dir.path().join("IMG_1.jpg"), bytes).unwrap();
        let mut index = Index::new();
        index.add_root(dir.path()).unwrap();
        let root = index.roots()[0].clone();
        index.scan(&root);
        let id = index.photo_ids().next().unwrap();
        (dir, RwLock::new(index), id)
    }

    fn saved(dir: &tempfile::TempDir) -> EditDocument {
        EditDocument::parse(&fs::read(dir.path().join("IMG_1.jpg.seura.json")).unwrap()).unwrap()
    }

    fn change(index: &RwLock<Index>, id: PhotoId, change: Change) -> ChangeReport {
        Index::change_shared(
            index,
            &[id],
            &change,
            WRITER,
            OffsetDateTime::now_utc(),
            &no_trash,
        )
    }

    #[test]
    fn a_cloud_only_photo_s_file_is_filled_in_once_it_s_on_disk() {
        let (dir, index, id) = library();
        index.write().unwrap().photo_mut(id).unwrap().placeholder = true;

        change(&index, id, Change::Rating(4));
        let photo = saved(&dir).photo().unwrap();
        assert_eq!(photo.file, "IMG_1.jpg");
        assert_eq!(photo.size, 4100);
        // Reading it would download it (LIB-9).
        assert_eq!((photo.sha256, photo.width), (None, None));

        {
            let mut index = index.write().unwrap();
            let photo = index.photo_mut(id).unwrap();
            photo.placeholder = false;
            photo.dimensions = Some((40, 30));
        }
        change(&index, id, Change::AddTag("beach".into()));
        let photo = saved(&dir).photo().unwrap();
        assert_eq!(photo.sha256.unwrap().len(), 64);
        assert_eq!((photo.width, photo.height), (Some(40), Some(30)));
        // The fingerprint is cached for next time.
        assert!(
            index
                .read()
                .unwrap()
                .photo(id)
                .unwrap()
                .fingerprint
                .is_some()
        );
    }

    #[test]
    fn the_index_records_what_was_written_so_the_watcher_finds_nothing_new() {
        let (dir, index, id) = library();
        let report = change(&index, id, Change::Rating(3));
        assert_eq!(report.changed, [id]);
        change(&index, id, Change::AddTag("family/kids".into()));

        let metadata = fs::metadata(dir.path().join("IMG_1.jpg.seura.json")).unwrap();
        let index = index.read().unwrap();
        let photo = index.photo(id).unwrap();
        assert_eq!(photo.rating, 3);
        assert_eq!(index.tag(photo.tags[0]), Some("family/kids"));
        assert_eq!(photo.edit_file, EditFileState::Read { edited: false });
        // The stamp a scan compares against: the same, so a rescan after
        // this write reads nothing again.
        assert_eq!(
            photo.edit_file_stamp,
            Some((metadata.len(), FileTime::from(metadata.modified().unwrap())))
        );
    }

    #[test]
    fn a_change_that_changes_nothing_writes_nothing() {
        let (dir, index, id) = library();
        assert_eq!(
            change(&index, id, Change::Rating(0)),
            ChangeReport::default()
        );
        assert_eq!(
            change(&index, id, Change::RemoveTag("beach".into())),
            ChangeReport::default()
        );
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn a_photo_no_longer_in_the_library_is_reported() {
        let (_dir, index, _id) = library();
        let gone = PhotoId(99);
        assert_eq!(
            change(&index, gone, Change::Rating(2)).problems,
            [(gone, Problem::Gone)]
        );
    }

    #[test]
    fn a_typed_tag_takes_the_library_s_existing_form_tag_2() {
        let (dir, index, id) = library();
        let mut bytes = vec![0xFF, 0xD8, 0xFF, 0xE1];
        bytes.extend([8; 4096]);
        fs::write(dir.path().join("IMG_2.jpg"), bytes).unwrap();
        let root = index.read().unwrap().roots()[0].clone();
        index.write().unwrap().scan(&root);
        let other = index
            .read()
            .unwrap()
            .photo_ids()
            .find(|other| *other != id)
            .unwrap();

        change(&index, id, Change::AddTag(" Straße/ ".into()));
        change(&index, other, Change::AddTag("STRASSE".into()));
        let tags = |photo: &str| {
            EditDocument::parse(&fs::read(dir.path().join(format!("{photo}.seura.json"))).unwrap())
                .unwrap()
                .tags()
        };
        assert_eq!(tags("IMG_1.jpg"), ["Straße"]);
        assert_eq!(tags("IMG_2.jpg"), ["Straße"]);

        // Removing matches the same way.
        change(&index, other, Change::AddTag("beach".into()));
        change(&index, other, Change::RemoveTag("strasse".into()));
        assert_eq!(tags("IMG_2.jpg"), ["beach"]);
        // Nothing to add from only spaces and slashes.
        assert_eq!(
            change(&index, id, Change::AddTag(" / ".into())),
            ChangeReport::default()
        );
    }
}
