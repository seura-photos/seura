//! What the scanner skips or treats carefully, per platform (decision 0006):
//! hidden and system items (LIB-7), and cloud placeholders, files that exist
//! only in OneDrive or iCloud until something reads them. Also whether a
//! folder is read-only (TAG-7).

use std::fs::Metadata;
use std::io;
use std::path::Path;

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
pub fn is_placeholder(metadata: &Metadata) -> bool {
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

/// Whether Seura may add files to the folder, such as the edit files beside
/// its photos. False in a read-only folder (TAG-7): one without permission
/// to write, or on a read-only drive. Checked without writing anything there
/// (CLAUDE.md rule 2). When the system doesn't say, the folder counts as
/// writable, and a failed save says otherwise.
pub(crate) fn can_add_files(folder: &Path) -> bool {
    access::can_add_files(folder)
}

/// A write failed because the folder or drive is read-only.
pub(crate) fn is_read_only_error(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::PermissionDenied | io::ErrorKind::ReadOnlyFilesystem
    )
}

#[cfg(target_os = "windows")]
mod access {
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;
    use std::path::Path;

    /// The right to create a file in a folder.
    const FILE_ADD_FILE: u32 = 0x2;
    /// Needed to open a folder at all.
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;

    /// Opens the folder asking for the right to add files: the system checks
    /// the folder's permissions and the drive, and nothing is created.
    pub fn can_add_files(folder: &Path) -> bool {
        match OpenOptions::new()
            .access_mode(FILE_ADD_FILE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
            .open(folder)
        {
            Ok(_) => true,
            Err(error) => !super::is_read_only_error(&error),
        }
    }
}

#[cfg(target_os = "macos")]
mod access {
    #![allow(unsafe_code, reason = "access is a C API")]

    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    /// Asks the system whether this process may write in the folder, which
    /// accounts for its permissions, ACLs, and a read-only drive.
    pub fn can_add_files(folder: &Path) -> bool {
        let Ok(path) = CString::new(folder.as_os_str().as_bytes()) else {
            return true;
        };
        // SAFETY: `path` is a valid NUL-terminated string for the call.
        if unsafe { libc::access(path.as_ptr(), libc::W_OK) } == 0 {
            return true;
        }
        !super::is_read_only_error(&std::io::Error::last_os_error())
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod access {
    use std::path::Path;

    pub fn can_add_files(_folder: &Path) -> bool {
        true
    }
}

/// Whether files on this path's drive can go to the Recycle Bin or Trash
/// (BRW-9). On Windows, only fixed drives have a Recycle Bin: on a USB stick
/// or a network share, the system would offer to delete permanently instead,
/// so their photos are left out of Delete, as in read-only folders. A drive
/// whose type the system doesn't know counts as having none.
pub fn has_trash(path: &Path) -> bool {
    drive::has_trash(path)
}

#[cfg(target_os = "windows")]
mod drive {
    #![allow(unsafe_code, reason = "these are Win32 calls")]

    use std::path::Path;

    use windows::Win32::Storage::FileSystem::{GetDriveTypeW, GetVolumePathNameW};
    use windows::core::HSTRING;

    /// `DRIVE_FIXED`: a hard disk or SSD, inside or attached.
    const FIXED: u32 = 3;

    pub fn has_trash(path: &Path) -> bool {
        // The folder a drive is mounted on, such as `D:\`, `\\server\share\`,
        // or a folder another drive is mounted in.
        let mut root = [0u16; 1024];
        // SAFETY: the path is a NUL-terminated wide string, and `root` is
        // writable for its whole length, which the call is given.
        if unsafe { GetVolumePathNameW(&HSTRING::from(path), &mut root) }.is_err() {
            return false;
        }
        let end = root.iter().position(|&c| c == 0).unwrap_or(root.len());
        let root = HSTRING::from_wide(&root[..end]);
        // SAFETY: `root` is a NUL-terminated wide string.
        unsafe { GetDriveTypeW(&root) == FIXED }
    }
}

/// On the Mac, the system call never deletes permanently: where there's no
/// Trash, it fails and the photo stays where it was.
#[cfg(not(target_os = "windows"))]
mod drive {
    use std::path::Path;

    pub fn has_trash(_path: &Path) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fixed_drive_has_a_recycle_bin_or_trash() {
        assert!(has_trash(&std::env::temp_dir()));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn a_network_share_has_no_recycle_bin() {
        assert!(!has_trash(Path::new(r"\\localhost\no-such-share\photos")));
    }
}
