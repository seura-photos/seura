//! Scanning a library folder and reconciling the index with it (LIB-1,
//! LIB-7, LIB-8). Photos are only read, never written (CLAUDE.md rule 1):
//! a new or changed file's first bytes confirm its format, and its edit file
//! is read when that changes.

use std::collections::{HashMap, HashSet};
use std::fs::{self, Metadata};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use seura_edit::{EditFile, edit_file_path};

use crate::formats::{Format, read_head};
use crate::index::{EditFileState, FileTime, FolderId, Index, Photo, PhotoId};
use crate::platform::{can_add_files, is_placeholder, is_skipped};

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
        self.scan_folder(root, root)
    }

    /// Scans `folder`, inside library folder `root`, and everything below
    /// it, as [`Index::scan`] does for a whole library folder. Used when
    /// watching reports a change there (LIB-4), so one new photo doesn't
    /// mean walking the whole library. A folder that's gone takes its
    /// photos with it, and folders above it left with no photos go too.
    pub fn scan_folder(&mut self, root: &Path, folder: &Path) -> ScanReport {
        let shared = RwLock::new(std::mem::take(self));
        let report = Index::scan_locked(&shared, root, folder, false);
        *self = shared
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        report
    }

    /// [`Index::scan_folder`] for an index others are reading meanwhile:
    /// folders are listed without the lock, which is taken for one folder's
    /// photos at a time, so browsing goes on during a long first scan. One
    /// scan runs at a time (the caller sees to that); if `root` stops being
    /// a library folder meanwhile, the scan stops.
    pub fn scan_shared(index: &RwLock<Index>, root: &Path, folder: &Path) -> ScanReport {
        Index::scan_locked(index, root, folder, true)
    }

    /// The walk, holding the lock for one folder at a time. With
    /// `check_root`, it stops if `root` stops being a library folder.
    fn scan_locked(
        index: &RwLock<Index>,
        root: &Path,
        folder: &Path,
        check_root: bool,
    ) -> ScanReport {
        let write = || {
            let index = index
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            (!check_root || index.roots().iter().any(|other| other == root)).then_some(index)
        };
        let mut report = ScanReport::default();
        if !folder.starts_with(root) {
            return report;
        }
        if fs::read_dir(folder).is_err() {
            if let Some(mut index) = write() {
                index.gone(root, folder, &mut report);
            }
            return report;
        }

        let Some(parent) = write().map(|mut index| index.ensure_folders_above(root, folder)) else {
            return report;
        };
        let mut seen_folders = HashSet::new();
        let mut seen_photos = HashSet::new();
        let mut pending = vec![(folder.to_path_buf(), parent)];
        while let Some((folder_path, parent)) = pending.pop() {
            let Ok(listing) = list(&folder_path) else {
                continue;
            };
            let Some(mut index) = write() else {
                return report;
            };
            let folder = index.ensure_folder(&folder_path, parent);
            index.set_read_only(folder, listing.read_only);
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
                if let Some(id) = index.reconcile(
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

        let Some(mut index) = write() else {
            return report;
        };
        // Anything under this folder that wasn't seen is gone.
        for id in index.photos_under(folder) {
            if !seen_photos.contains(&id) {
                index.remove_photo(id);
                report.removed += 1;
            }
        }
        index.prune(folder, &seen_folders);
        index.prune_above(root, folder);
        report.photos = index.photos_under(root).len();
        report
    }

    /// `folder` can't be read: a library folder is unavailable (LIB-6), and
    /// a folder in one is gone, with everything that was in it.
    fn gone(&mut self, root: &Path, folder: &Path, report: &mut ScanReport) {
        if folder == root {
            report.unavailable = true;
        } else {
            for id in self.photos_under(folder) {
                self.remove_photo(id);
                report.removed += 1;
            }
            self.prune(folder, &HashSet::new());
            self.prune_above(root, folder);
        }
        report.photos = self.photos_under(root).len();
    }

    /// Removes folders under `folder` that are gone (not in `seen`) or left
    /// with no photos anywhere inside them. Deepest first, so a parent sees
    /// its children removed.
    fn prune(&mut self, folder: &Path, seen: &HashSet<FolderId>) {
        let mut folders = self.folders_under(folder);
        folders.sort_by_key(|(_, path)| std::cmp::Reverse(path.components().count()));
        for (id, _) in folders {
            let empty = self.folder_len(id) == 0 && !self.has_subfolders(id);
            if !seen.contains(&id) || empty {
                self.remove_folder(id);
            }
        }
    }

    /// Removes the folders between `folder` and `root` (both included) that
    /// a change below left with no photos anywhere inside them.
    fn prune_above(&mut self, root: &Path, folder: &Path) {
        for above in folder.ancestors().skip(1) {
            if !above.starts_with(root) {
                break;
            }
            if let Some(id) = self.folder_id(above)
                && self.folder_len(id) == 0
                && !self.has_subfolders(id)
            {
                self.remove_folder(id);
            }
        }
    }

    /// Makes sure the folders from `root` down to `folder`'s parent are in
    /// the index, and returns that parent, if `folder` isn't `root` itself.
    fn ensure_folders_above(&mut self, root: &Path, folder: &Path) -> Option<FolderId> {
        if folder == root {
            return None;
        }
        let mut chain: Vec<&Path> = folder
            .ancestors()
            .skip(1)
            .take_while(|above| above.starts_with(root))
            .collect();
        chain.reverse();
        let mut parent = None;
        for above in chain {
            parent = Some(self.ensure_folder(above, parent));
        }
        parent
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
        self.set_edit_file(id, state, stamp, rating, &tags);
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
    /// Seura can't add files here (TAG-7).
    read_only: bool,
}

/// A folder's visible subfolders and files. Hidden and system items and
/// symbolic links are left out: links could lead outside the library or in
/// circles. Names that aren't valid Unicode are skipped.
fn list(folder: &Path) -> io::Result<Listing> {
    let mut listing = Listing {
        folders: Vec::new(),
        files: HashMap::new(),
        read_only: !can_add_files(folder),
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
