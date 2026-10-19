//! What the scanner skips or treats carefully, per platform (decision 0006):
//! hidden and system items (LIB-7), and cloud placeholders, files that exist
//! only in OneDrive or iCloud until something reads them.

use std::fs::Metadata;

/// Folders the system owns, skipped even when they aren't marked hidden.
const SYSTEM_FOLDERS: [&str; 9] = [
    "$RECYCLE.BIN",
    "RECYCLER",
    "System Volume Information",
    ".Trash",
    ".Trashes",
    ".Spotlight-V100",
    ".fseventsd",
    ".DocumentRevisions-V100",
    ".TemporaryItems",
];

/// Hidden or system: skipped entirely (LIB-7). Names starting with a dot
/// count as hidden on every platform.
pub(crate) fn is_skipped(name: &str, metadata: &Metadata) -> bool {
    name.starts_with('.')
        || SYSTEM_FOLDERS
            .iter()
            .any(|folder| folder.eq_ignore_ascii_case(name))
        || flags::hidden_or_system(metadata)
}

/// A file whose contents are in the cloud, not on disk. Reading even its
/// first bytes would download it, so the scanner trusts its extension.
pub(crate) fn is_placeholder(metadata: &Metadata) -> bool {
    flags::placeholder(metadata)
}

#[cfg(target_os = "windows")]
mod flags {
    use std::fs::Metadata;
    use std::os::windows::fs::MetadataExt;

    const HIDDEN: u32 = 0x2;
    const SYSTEM: u32 = 0x4;
    const OFFLINE: u32 = 0x1000;
    const RECALL_ON_OPEN: u32 = 0x4_0000;
    const RECALL_ON_DATA_ACCESS: u32 = 0x40_0000;

    pub fn hidden_or_system(metadata: &Metadata) -> bool {
        metadata.file_attributes() & (HIDDEN | SYSTEM) != 0
    }

    pub fn placeholder(metadata: &Metadata) -> bool {
        metadata.file_attributes() & (OFFLINE | RECALL_ON_OPEN | RECALL_ON_DATA_ACCESS) != 0
    }
}

#[cfg(target_os = "macos")]
mod flags {
    use std::fs::Metadata;
    use std::os::macos::fs::MetadataExt;

    /// `UF_HIDDEN`: hidden in Finder.
    const HIDDEN: u32 = 0x8000;
    /// `SF_DATALESS`: an iCloud file whose contents haven't been downloaded.
    const DATALESS: u32 = 0x4000_0000;

    pub fn hidden_or_system(metadata: &Metadata) -> bool {
        metadata.st_flags() & HIDDEN != 0
    }

    pub fn placeholder(metadata: &Metadata) -> bool {
        metadata.st_flags() & DATALESS != 0
    }
}

/// Other platforms aren't supported; this keeps the crate building there.
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod flags {
    use std::fs::Metadata;

    pub fn hidden_or_system(_metadata: &Metadata) -> bool {
        false
    }

    pub fn placeholder(_metadata: &Metadata) -> bool {
        false
    }
}
