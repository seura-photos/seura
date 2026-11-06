//! Whether a folder is synced to a cloud service (SET-1; decision 0015;
//! `docs/research/cloud-photo-sources.md`). Seura's cache shouldn't be: it's
//! large, rewritten constantly, and specific to one computer. Also the
//! one-time explanation when an edit file first goes to the Trash from such
//! a folder (FIL-1, #200).

use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

use crate::library::AppSettings;

/// What the change that removed the edit files was, for the message's
/// words: "Removing this photo's last rating…".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Removal {
    /// A rating cleared.
    Rating,
    /// A tag removed.
    Tag,
    /// Anything else: an edit reset, undo, redo, revert, or a paste.
    Other,
}

impl Removal {
    pub fn of(change: &seura_library::Change) -> Removal {
        match change {
            seura_library::Change::Rating(0) => Removal::Rating,
            seura_library::Change::RemoveTag(_) => Removal::Tag,
            _ => Removal::Other,
        }
    }
}

/// The first time edit files went to the Trash from a folder this service
/// syncs (FIL-1): the message says the service may report a deletion, and
/// the photos aren't affected. Once per service, ever, and one for a batch.
#[derive(Clone, Debug, Serialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct DeletionExplained {
    pub service: String,
    pub removal: Removal,
    /// The photos in that service's folders whose edit files went.
    pub count: u32,
}

/// The edit files a change moved to the Trash, as it moves them.
#[derive(Default)]
pub struct Trashed(Mutex<Vec<PathBuf>>);

impl Trashed {
    /// Moves `path` to the Recycle Bin or Trash, and notes it if it went.
    pub fn trash(&self, path: &Path) -> io::Result<()> {
        seura_library::move_to_trash(path)?;
        self.0
            .lock()
            .expect("trashed lock")
            .push(path.to_path_buf());
        Ok(())
    }

    /// Explains, if any of them was in a folder a cloud service syncs that
    /// hasn't been explained yet (FIL-1). The service is remembered in the
    /// settings, so it's said once.
    pub fn explain<R: Runtime>(self, app: &AppHandle<R>, removal: Removal) {
        let paths = self.0.into_inner().expect("trashed lock");
        if paths.is_empty() {
            return;
        }
        let services: Vec<String> = paths.iter().filter_map(|path| sync_service(path)).collect();
        let Some(settings) = app.try_state::<AppSettings>() else {
            return;
        };
        let mut file = settings.0.lock().expect("settings lock");
        let Some(service) = first_unexplained(&services, &file.settings.explained_services) else {
            return;
        };
        let count = services.iter().filter(|each| **each == service).count() as u32;
        file.settings.explained_services.push(service.clone());
        // A settings file that can't be saved says so elsewhere; then it's
        // explained again after a restart, which does no harm.
        let _ = file.save();
        drop(file);
        let _ = DeletionExplained {
            service,
            removal,
            count,
        }
        .emit(app);
    }
}

/// The first of `services` not yet in `explained`: one message per batch,
/// and another service waits for its own next time.
fn first_unexplained(services: &[String], explained: &[String]) -> Option<String> {
    services
        .iter()
        .find(|service| !explained.contains(service))
        .cloned()
}

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
    fn each_service_is_explained_once_and_one_per_batch() {
        let names = |list: &[&str]| list.iter().map(|each| each.to_string()).collect::<Vec<_>>();
        let batch = names(&["OneDrive", "OneDrive", "iCloud Drive"]);
        assert_eq!(first_unexplained(&batch, &[]).as_deref(), Some("OneDrive"));
        assert_eq!(
            first_unexplained(&batch, &names(&["OneDrive"])).as_deref(),
            Some("iCloud Drive")
        );
        assert_eq!(
            first_unexplained(&batch, &names(&["OneDrive", "iCloud Drive"])),
            None
        );
        assert_eq!(first_unexplained(&[], &[]), None);
    }

    #[test]
    fn the_message_says_what_removed_the_files() {
        use seura_library::Change;
        assert_eq!(Removal::of(&Change::Rating(0)), Removal::Rating);
        assert_eq!(Removal::of(&Change::Rating(3)), Removal::Other);
        assert_eq!(Removal::of(&Change::RemoveTag("x".into())), Removal::Tag);
        assert_eq!(Removal::of(&Change::Paste), Removal::Other);
    }

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
