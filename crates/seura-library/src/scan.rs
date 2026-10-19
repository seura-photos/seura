//! Scanning a library folder and reconciling the index with it (LIB-1,
//! LIB-7, LIB-8). Photos are only read, never written (CLAUDE.md rule 1):
//! a new or changed file's first bytes confirm its format, and its edit file
//! is read when that changes.

use std::collections::{HashMap, HashSet};
use std::fs::{self, Metadata};
use std::io;
use std::path::{Path, PathBuf};

use seura_edit::{EditFile, edit_file_path};

use crate::formats::{Format, read_head};
use crate::index::{EditFileState, FileTime, FolderId, Index, Photo, PhotoId};
use crate::platform::{is_placeholder, is_skipped};

/// What a scan found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScanReport {
    /// Photos in the folder after the scan.
    pub photos: usize,
    pub added: usize,
    pub changed: usize,
    pub removed: usize,
    /// Edit files that couldn't be read; their photos show a warning (FIL-4).
    pub unreadable_edit_files: Vec<PathBuf>,
    /// The library folder couldn't be read, for example on a disconnected
    /// drive. Its photos were kept, not removed (LIB-6).
    pub unavailable: bool,
}

impl Index {
    /// Scans one library folder and everything in it, and updates the index:
    /// new and changed photos are read, missing ones removed. Unchanged
    /// photos (same size and modification time) aren't opened at all.
    pub fn scan(&mut self, root: &Path) -> ScanReport {
        let mut report = ScanReport::default();
        if fs::read_dir(root).is_err() {
            report.unavailable = true;
            report.photos = self.photos_under(root).len();
            return report;
        }

        let mut seen_folders = HashSet::new();
        let mut seen_photos = HashSet::new();
        let mut pending = vec![(root.to_path_buf(), None)];
        while let Some((folder_path, parent)) = pending.pop() {
            let Ok(listing) = list(&folder_path) else {
                continue;
            };
            let folder = self.ensure_folder(&folder_path, parent);
            seen_folders.insert(folder);
            for name in &listing.folders {
                pending.push((folder_path.join(name), Some(folder)));
            }
            for (name, metadata) in &listing.files {
                let Some(format) = Format::from_name(name) else {
                    continue;
                };
                let path = folder_path.join(name);
                let edit_file = listing.files.get(&edit_file_name(name));
                if let Some(id) = self.reconcile(
                    folder,
                    &path,
                    name,
                    format,
                    metadata,
                    edit_file,
                    &mut report,
                ) {
                    seen_photos.insert(id);
                }
            }
        }

        // Anything under this root that wasn't seen is gone.
        for id in self.photos_under(root) {
            if !seen_photos.contains(&id) {
                self.remove_photo(id);
                report.removed += 1;
            }
        }
        // Folders that are gone, or left with no photos anywhere inside them,
        // aren't kept. Deepest first, so a parent sees its children removed.
        let mut folders = self.folders_under(root);
        folders.sort_by_key(|(_, path)| std::cmp::Reverse(path.components().count()));
        for (id, _) in folders {
            let empty = self.folder_len(id) == 0 && !self.has_subfolders(id);
            if !seen_folders.contains(&id) || empty {
                self.remove_folder(id);
            }
        }
        report.photos = self.photos_under(root).len();
        report
    }

    /// Brings one photo's record up to date. Returns its id, or `None` if
    /// it isn't a photo after all.
    #[allow(clippy::too_many_arguments)]
    fn reconcile(
        &mut self,
        folder: FolderId,
        path: &Path,
        name: &str,
        format: Format,
        metadata: &Metadata,
        edit_file: Option<&Metadata>,
        report: &mut ScanReport,
    ) -> Option<PhotoId> {
        let size = metadata.len();
        let modified = FileTime::from(metadata.modified().ok()?);
        let placeholder = is_placeholder(metadata);
        let existing = self.find(folder, name);
        let unchanged = existing
            .and_then(|id| self.photo(id))
            .is_some_and(|photo| photo.size == size && photo.modified == modified);

        let id = match existing.filter(|_| unchanged) {
            Some(id) => id,
            None => {
                // A cloud placeholder is trusted by extension: reading it
                // would download it.
                let genuine =
                    placeholder || read_head(path).is_ok_and(|head| format.matches(&head));
                if !genuine {
                    if let Some(id) = existing {
                        self.remove_photo(id);
                        report.removed += 1;
                    }
                    return None;
                }
                let photo = Photo {
                    folder,
                    name: name.into(),
                    size,
                    modified,
                    fingerprint: None,
                    dimensions: None,
                    taken: None,
                    edit_file: EditFileState::None,
                    edit_file_stamp: None,
                    rating: 0,
                    tags: Box::new([]),
                    placeholder,
                };
                match existing {
                    Some(id) => {
                        self.replace_photo(id, photo);
                        report.changed += 1;
                        id
                    }
                    None => {
                        report.added += 1;
                        self.insert_photo(photo)
                    }
                }
            }
        };

        self.refresh_edit_file(id, path, edit_file, report);
        if let Some(photo) = self.photo_mut(id) {
            photo.placeholder = placeholder;
        }
        Some(id)
    }

    /// Re-reads the photo's edit file if it appeared, changed, or vanished.
    fn refresh_edit_file(
        &mut self,
        id: PhotoId,
        photo_path: &Path,
        edit_file: Option<&Metadata>,
        report: &mut ScanReport,
    ) {
        let stamp = edit_file
            .and_then(|metadata| Some((metadata.len(), FileTime::from(metadata.modified().ok()?))));
        let Some(photo) = self.photo(id) else {
            return;
        };
        if photo.edit_file_stamp == stamp {
            if photo.edit_file == EditFileState::Unreadable {
                report
                    .unreadable_edit_files
                    .push(edit_file_path(photo_path));
            }
            return;
        }

        let (state, rating, tags) = match stamp {
            None => (EditFileState::None, 0, Vec::new()),
            Some(_) => match EditFile::load(&edit_file_path(photo_path)) {
                Ok(Some(file)) => {
                    let document = &file.document;
                    let edited = !document.operations().is_empty();
                    (
                        EditFileState::Read { edited },
                        document.rating(),
                        document.tags(),
                    )
                }
                Ok(None) => (EditFileState::None, 0, Vec::new()),
                Err(_) => {
                    report
                        .unreadable_edit_files
                        .push(edit_file_path(photo_path));
                    (EditFileState::Unreadable, 0, Vec::new())
                }
            },
        };
        let tags: Box<[_]> = tags.iter().map(|tag| self.intern_tag(tag)).collect();
        let mut photo = self.photo(id).cloned().expect("checked above");
        photo.edit_file = state;
        photo.edit_file_stamp = stamp;
        photo.rating = rating;
        photo.tags = tags;
        self.replace_photo(id, photo);
    }

    fn folders_under(&self, root: &Path) -> Vec<(FolderId, PathBuf)> {
        self.folders
            .iter()
            .enumerate()
            .filter_map(|(id, folder)| {
                let folder = folder.as_ref()?;
                folder
                    .path
                    .starts_with(root)
                    .then(|| (FolderId(id as u32), folder.path.clone()))
            })
            .collect()
    }

    fn photos_under(&self, root: &Path) -> Vec<PhotoId> {
        let folders: HashSet<FolderId> = self
            .folders_under(root)
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        self.photo_ids()
            .filter(|id| {
                self.photo(*id)
                    .is_some_and(|photo| folders.contains(&photo.folder))
            })
            .collect()
    }

    fn has_subfolders(&self, folder: FolderId) -> bool {
        self.folders
            .iter()
            .flatten()
            .any(|other| other.parent == Some(folder))
    }
}

/// `IMG_1234.jpg` → `IMG_1234.jpg.seura.json`.
fn edit_file_name(photo: &str) -> String {
    format!("{photo}.seura.json")
}

struct Listing {
    folders: Vec<String>,
    files: HashMap<String, Metadata>,
}

/// A folder's visible subfolders and files. Hidden and system items and
/// symbolic links are left out: links could lead outside the library or in
/// circles. Names that aren't valid Unicode are skipped.
fn list(folder: &Path) -> io::Result<Listing> {
    let mut listing = Listing {
        folders: Vec::new(),
        files: HashMap::new(),
    };
    for entry in fs::read_dir(folder)? {
        let Ok(entry) = entry else { continue };
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if metadata.file_type().is_symlink() || is_skipped(&name, &metadata) {
            continue;
        }
        if metadata.is_dir() {
            listing.folders.push(name);
        } else if metadata.is_file() {
            listing.files.insert(name, metadata);
        }
    }
    Ok(listing)
}
