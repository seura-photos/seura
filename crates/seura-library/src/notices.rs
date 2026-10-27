//! What the user is told about a photo's files beyond whether its edit file
//! can be read (FIL-4): edits Seura can't show (FIL-5), and an original
//! changed since its edits were made (FIL-7).

use std::sync::RwLock;

use seura_edit::{EditFile, edit_file_path};

use crate::fingerprint::to_hex;
use crate::index::{EditFileState, Index, PhotoId};
use crate::platform::is_placeholder;

/// What to tell the user about one photo's files.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FileNotices {
    /// Operations in its edit file that Seura doesn't render, from a newer
    /// version or another app. They're kept when the file is saved (FIL-5).
    pub not_shown: usize,
    /// The photo's fingerprint no longer matches the one its edit file
    /// recorded, so its edits may not line up (FIL-7).
    pub original_changed: bool,
}

impl Index {
    /// Reads the photo's edit file, and for one with edits, fingerprints the
    /// photo to compare (cached while the photo is unchanged). A cloud-only
    /// photo isn't read, so it isn't downloaded (LIB-9).
    pub fn file_notices(index: &RwLock<Index>, id: PhotoId) -> FileNotices {
        let path = {
            let index = index.read().expect("index lock");
            match (index.photo(id), index.path(id)) {
                (Some(photo), Some(path))
                    if matches!(photo.edit_file, EditFileState::Read { .. }) =>
                {
                    path
                }
                _ => return FileNotices::default(),
            }
        };
        let Ok(Some(file)) = EditFile::load(&edit_file_path(&path)) else {
            return FileNotices::default();
        };
        let operations = file.document.operations();
        let not_shown = operations
            .iter()
            .filter(|operation| operation.enabled && operation.core.is_none())
            .count();
        // Only edits depend on the pixels: tags and a rating still apply.
        let recorded = file.document.photo().and_then(|photo| photo.sha256);
        let cloud_only = std::fs::metadata(&path).is_ok_and(|metadata| is_placeholder(&metadata));
        let original_changed = match recorded {
            Some(recorded) if !operations.is_empty() && !cloud_only => {
                Index::fingerprint_shared(index, id)
                    .is_ok_and(|fingerprint| !to_hex(&fingerprint).eq_ignore_ascii_case(&recorded))
            }
            _ => false,
        };
        FileNotices {
            not_shown,
            original_changed,
        }
    }
}
