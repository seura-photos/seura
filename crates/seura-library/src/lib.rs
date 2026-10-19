//! Library folders, scanning, the in-memory index and its snapshot, the
//! cache, and fingerprints (technical design, section 4; decision 0010).
//!
//! Watching for changes (LIB-4), renamed photos (FIL-6), and removing
//! folders (LIB-5) come in later milestones.

mod fingerprint;
mod formats;
mod index;
mod platform;
mod scan;
mod snapshot;
#[doc(hidden)]
pub mod synthetic;
mod thumbnails;

pub use fingerprint::{fingerprint_file, to_hex};
pub use formats::Format;
pub use index::{
    AddFolder, EditFileState, FileTime, Filter, Folder, FolderId, Index, Photo, PhotoId, Scope,
    TagId,
};
pub use platform::is_placeholder;
pub use scan::ScanReport;
pub use snapshot::SNAPSHOT_FILE;
pub use thumbnails::{Job, Made, Outcome, THUMBS_FOLDER, ThumbnailCache, ThumbnailWorkers};
