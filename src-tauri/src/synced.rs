//! Whether a folder is synced to a cloud service (SET-1; decision 0015;
//! `docs/research/cloud-photo-sources.md`). Seura's cache shouldn't be: it's
//! large, rewritten constantly, and specific to one computer.

use std::path::Path;

/// The sync service that holds `path`, such as "OneDrive", or `None` if it
/// isn't synced. A path that doesn't exist yet is judged by the nearest
/// folder above it that does.
pub fn sync_service(path: &Path) -> Option<String> {
    let existing = path.ancestors().find(|folder| folder.exists())?;
    platform_service(existing)
}

/// Windows: any folder inside a Cloud Files sync root, which OneDrive,
/// iCloud for Windows, Dropbox, and Box all register, names its provider.
#[cfg(target_os = "windows")]
fn platform_service(path: &Path) -> Option<String> {
    use windows::Win32::Storage::CloudFilters::{
        CF_SYNC_ROOT_INFO_PROVIDER, CF_SYNC_ROOT_PROVIDER_INFO, CfGetSyncRootInfoByPath,
    };
    use windows::core::HSTRING;

    let mut info = CF_SYNC_ROOT_PROVIDER_INFO::default();
    #[allow(
        unsafe_code,
        reason = "a Win32 call into a buffer of its own type and size"
    )]
    let result = unsafe {
        CfGetSyncRootInfoByPath(
            &HSTRING::from(path),
            CF_SYNC_ROOT_INFO_PROVIDER,
            (&raw mut info).cast(),
            size_of::<CF_SYNC_ROOT_PROVIDER_INFO>() as u32,
            None,
        )
    };
    // An error means the path isn't in a sync root.
    result.ok()?;
    let length = info
        .ProviderName
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(info.ProviderName.len());
    let name = String::from_utf16_lossy(&info.ProviderName[..length]);
    Some(if name.is_empty() {
        "a cloud service".into()
    } else {
        name
    })
}

/// macOS: sync services keep their folders in `~/Library/CloudStorage`
/// (OneDrive, Google Drive, Dropbox) and `~/Library/Mobile Documents`
/// (iCloud Drive, including Desktop and Documents when those are synced,
/// which macOS then shows in their usual places).
#[cfg(target_os = "macos")]
fn platform_service(path: &Path) -> Option<String> {
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from)?;
    let path = path.canonicalize().ok()?;
    let library = home.join("Library");
    if let Ok(rest) = path.strip_prefix(library.join("CloudStorage")) {
        let folder = rest.components().next()?.as_os_str().to_string_lossy();
        // Folders are named like "OneDrive-Personal" or "GoogleDrive-name@…".
        let service = folder.split('-').next().unwrap_or(&folder);
        return Some(match service {
            "GoogleDrive" => "Google Drive".into(),
            other => other.to_string(),
        });
    }
    path.starts_with(library.join("Mobile Documents"))
        .then(|| "iCloud Drive".into())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn platform_service(_path: &Path) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_temporary_folder_isn_t_synced() {
        let folder = tempfile::tempdir().unwrap();
        assert_eq!(sync_service(folder.path()), None);
        assert_eq!(sync_service(&folder.path().join("not/made/yet")), None);
    }

    /// Run on a PC with OneDrive set up: `OneDrive` points at its folder.
    #[test]
    #[ignore = "needs OneDrive set up on this computer"]
    fn onedrive_is_recognized() {
        let onedrive = std::env::var_os("OneDrive").expect("OneDrive is set up");
        let service = sync_service(&Path::new(&onedrive).join("Seura Photos"));
        assert!(
            service
                .as_deref()
                .is_some_and(|name| name.contains("OneDrive")),
            "{service:?}"
        );
    }
}
