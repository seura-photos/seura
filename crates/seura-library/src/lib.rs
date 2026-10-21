//! Library folders, scanning, the in-memory index and its snapshot, the
//! cache, fingerprints, and the settings file (technical design, section 4;
//! decisions 0010 and 0015).
//!
//! Watching for changes (LIB-4) and renamed photos (FIL-6) come later.

mod fingerprint;
mod formats;
mod index;
mod platform;
mod scan;
mod settings;
mod snapshot;
#[doc(hidden)]
pub mod synthetic;
mod thumbnails;
mod tree;

pub use fingerprint::{fingerprint_file, to_hex};
pub use formats::Format;
pub use index::{
    AddFolder, EditFileState, FileTime, Filter, Folder, FolderId, Index, Photo, PhotoId, Scope,
    TagId,
};
pub use platform::is_placeholder;
pub use scan::ScanReport;
pub use settings::{
    DEFAULT_THUMBNAIL_SIZE, SETTINGS_FILE, Settings, SettingsError, SettingsFile, Sort, SortBy,
    THUMBNAIL_SIZES,
};
pub use snapshot::SNAPSHOT_FILE;
pub use thumbnails::{Job, Made, Outcome, THUMBS_FOLDER, ThumbnailCache, ThumbnailWorkers};
pub use tree::FolderEntry;
