//! Library folders, scanning, the in-memory index and its snapshot, the
//! cache, fingerprints, and the settings file (technical design, section 4;
//! decisions 0010 and 0015).
//!
//! Also the one path that writes edit files from the Library (M3), starting
//! over from unreadable ones (FIL-4), matching renamed photos back to their
//! edit files (FIL-6), and the Trash.

mod changes;
mod delete;
mod fingerprint;
mod formats;
mod index;
mod notices;
mod platform;
mod renames;
mod scan;
mod settings;
mod snapshot;
mod sort;
mod start_over;
#[doc(hidden)]
pub mod synthetic;
mod tag_tree;
mod thumbnails;
mod trash;
mod tree;
mod undo;
mod watch;

pub use changes::{Change, ChangeReport, Problem};
pub use delete::{DeleteProblem, DeleteReport};
pub use fingerprint::{fingerprint_file, to_hex};
pub use formats::Format;
pub use index::{
    AddFolder, EditFileState, FileTime, Filter, Folder, FolderId, Index, Photo, PhotoId, Scope,
    TagId,
};
pub use notices::FileNotices;
pub use platform::{has_trash, is_placeholder};
pub use scan::ScanReport;
pub use settings::{
    DEFAULT_THUMBNAIL_SIZE, SETTINGS_FILE, SIDEBAR_WIDTHS, Settings, SettingsError, SettingsFile,
    Sort, SortBy, THUMBNAIL_SIZES,
};
pub use snapshot::SNAPSHOT_FILE;
pub use sort::natural;
pub use start_over::{StartedOver, UNREADABLE_FOLDER};
pub use tag_tree::TagEntry;
pub use thumbnails::{
    ApplyEdits, Job, Made, Outcome, SWEEP_AFTER, THUMBS_FOLDER, ThumbnailCache, ThumbnailWorkers,
};
pub use trash::{move_all_to_trash, move_to_trash};
pub use tree::FolderEntry;
pub use undo::{Action, History, UNDO_LIMIT, UndoReport};
pub use watch::{Watcher, folders_to_scan};
