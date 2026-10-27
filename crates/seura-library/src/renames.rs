//! Renamed photos (FIL-6; edit format, "Reattaching"). When a photo is
//! renamed outside Seura, its edit file is left with the old name. If a
//! photo in the same folder has the fingerprint the edit file recorded and
//! no edit file of its own, it's the renamed original: the edit file is
//! renamed to match, and names the photo inside too. Only ever an edit file
//! is renamed, never a photo (CLAUDE.md rule 1).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use seura_edit::{EditFile, Writer, edit_file_path};
use time::OffsetDateTime;

use crate::fingerprint::to_hex;
use crate::index::{EditFileState, Filter, Index, PhotoId, Scope};
use crate::scan::ScanReport;

/// An edit file whose photo is missing, with the photo it recorded.
struct Orphan {
    path: PathBuf,
    size: u64,
    sha256: String,
}

impl Index {
    /// Matches edit files whose photo is missing (from
    /// [`ScanReport::orphan_edit_files`]) back to renamed photos, and
    /// returns the photos that got theirs back.
    ///
    /// Only a photo with the recorded size is fingerprinted, so a leftover
    /// edit file doesn't mean reading every photo in its folder; a
    /// cloud-only photo never is (LIB-9). A match must be the only one both
    /// ways, and an edit file is never put over another.
    pub fn reattach_renamed(
        index: &RwLock<Index>,
        orphans: &[PathBuf],
        writer: Writer,
        at: OffsetDateTime,
    ) -> Vec<PhotoId> {
        let mut by_folder: HashMap<&Path, Vec<Orphan>> = HashMap::new();
        for path in orphans {
            let Ok(Some(file)) = EditFile::load(path) else {
                continue;
            };
            let Some(photo) = file.document.photo() else {
                continue;
            };
            let (Some(sha256), Some(folder)) = (photo.sha256, path.parent()) else {
                continue;
            };
            by_folder.entry(folder).or_default().push(Orphan {
                path: path.clone(),
                size: photo.size,
                sha256: sha256.to_ascii_lowercase(),
            });
        }
        let mut reattached = Vec::new();
        for (folder, orphans) in by_folder {
            reattached.extend(reattach_in(index, folder, &orphans, writer, at));
        }
        reattached
    }
}

fn reattach_in(
    index: &RwLock<Index>,
    folder: &Path,
    orphans: &[Orphan],
    writer: Writer,
    at: OffsetDateTime,
) -> Vec<PhotoId> {
    // Photos here with no edit file, on disk, of a size some orphan recorded.
    let candidates: Vec<(PhotoId, PathBuf)> = {
        let index = index.read().expect("index lock");
        let Some(folder_id) = index.folder_id(folder) else {
            return Vec::new();
        };
        if index
            .folder(folder_id)
            .is_some_and(|folder| folder.read_only)
        {
            return Vec::new();
        }
        index
            .query(Scope::Folder(folder_id), &Filter::default())
            .into_iter()
            .filter(|id| {
                index.photo(*id).is_some_and(|photo| {
                    photo.edit_file == EditFileState::None
                        && !photo.placeholder
                        && orphans.iter().any(|orphan| orphan.size == photo.size)
                })
            })
            .filter_map(|id| Some((id, index.path(id)?)))
            .collect()
    };
    let fingerprints: Vec<(PhotoId, PathBuf, String)> = candidates
        .into_iter()
        .filter_map(|(id, path)| {
            let fingerprint = Index::fingerprint_shared(index, id).ok()?;
            Some((id, path, to_hex(&fingerprint)))
        })
        .collect();

    let mut reattached = Vec::new();
    for orphan in orphans {
        let same_edit_files = orphans
            .iter()
            .filter(|other| other.sha256 == orphan.sha256)
            .count();
        let photos: Vec<_> = fingerprints
            .iter()
            .filter(|(_, _, fingerprint)| *fingerprint == orphan.sha256)
            .collect();
        let ([(id, photo, _)], 1) = (photos.as_slice(), same_edit_files) else {
            continue;
        };
        let Some(name) = photo.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let target = edit_file_path(photo);
        if fs::symlink_metadata(&target).is_ok() || fs::rename(&orphan.path, &target).is_err() {
            continue;
        }
        // The file names its photo too. If this save fails, the file still
        // belongs to the photo by its name.
        if let Ok(Some(mut file)) = EditFile::load(&target)
            && file.document.rename_photo(name).unwrap_or(false)
        {
            let _ = file.save(writer, at);
        }
        let metadata = fs::metadata(&target).ok();
        index.write().expect("index lock").refresh_edit_file(
            *id,
            photo,
            metadata.as_ref(),
            &mut ScanReport::default(),
        );
        reattached.push(*id);
    }
    reattached
}
