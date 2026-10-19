//! Commands the UI can call. Each one is also listed in `build.rs`, which
//! generates the permission the UI's capability grants for it.

use std::path::Path;

use serde::Serialize;
use seura_library::{AddFolder, Filter, FolderId, PhotoId, Scope};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use crate::library::{Library, LibraryChanged};

/// The app's version, from `Cargo.toml`.
#[tauri::command]
#[specta::specta]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// A folder with photos directly in it.
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FolderInfo {
    pub id: u32,
    pub path: String,
    pub name: String,
    pub photos: u32,
}

/// A photo, as the grid needs it.
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PhotoInfo {
    pub id: u32,
    pub name: String,
    /// Upright dimensions, once the photo has been decoded.
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// Only in the cloud: never downloaded for browsing (LIB-9).
    pub cloud_only: bool,
}

/// What adding a folder did (LIB-1).
#[derive(Clone, Debug, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AddFolderResult {
    /// The user closed the folder dialog.
    Cancelled,
    Added,
    /// Already in the library, itself or inside this library folder.
    AlreadyIncluded {
        folder: String,
    },
    /// Added, replacing these library folders inside it.
    Absorbed {
        folders: Vec<String>,
    },
}

/// Asks for a folder with the system's dialog (ACC-4), adds it to the
/// library, and scans it. The UI never sees a path it could open itself
/// (decision 0009).
#[tauri::command]
#[specta::specta]
pub async fn add_folder(app: AppHandle) -> Result<AddFolderResult, String> {
    // Native dialogs must run on the main thread (macOS requires it).
    let (send, receive) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = send.send(rfd::FileDialog::new().pick_folder());
    })
    .map_err(|error| error.to_string())?;
    let picked = tauri::async_runtime::spawn_blocking(move || receive.recv().ok().flatten())
        .await
        .map_err(|error| error.to_string())?;
    let Some(path) = picked else {
        return Ok(AddFolderResult::Cancelled);
    };

    tauri::async_runtime::spawn_blocking(move || {
        let library = app.state::<Library>();
        let added = library
            .index
            .write()
            .expect("index lock")
            .add_root(&path)
            .map_err(|error| error.to_string())?;
        let result = match added {
            AddFolder::AlreadyIncluded(folder) => {
                return Ok(AddFolderResult::AlreadyIncluded {
                    folder: display(&folder),
                });
            }
            AddFolder::Added => AddFolderResult::Added,
            AddFolder::Absorbed(folders) => AddFolderResult::Absorbed {
                folders: folders.iter().map(|folder| display(folder)).collect(),
            },
        };
        library.rescan();
        let _ = LibraryChanged.emit(&app);
        Ok(result)
    })
    .await
    .map_err(|error| error.to_string())?
}

/// The folders that have photos directly in them, by path.
#[tauri::command]
#[specta::specta]
pub fn folders(library: State<'_, Library>) -> Vec<FolderInfo> {
    let index = library.index.read().expect("index lock");
    let mut folders: Vec<FolderInfo> = index
        .folders()
        .filter_map(|(id, folder)| {
            let photos = index.folder_len(id);
            (photos > 0).then(|| FolderInfo {
                id: id.value(),
                path: display(&folder.path),
                name: folder.path.file_name().map_or_else(
                    || display(&folder.path),
                    |name| name.to_string_lossy().into_owned(),
                ),
                photos: photos as u32,
            })
        })
        .collect();
    folders.sort_by_key(|folder| folder.path.to_lowercase());
    folders
}

/// A folder's own photos (BRW-1), by file name for now; the sort options
/// of BRW-2 arrive in M2. Their thumbnails are queued behind the visible ones.
#[tauri::command]
#[specta::specta]
pub fn photos(library: State<'_, Library>, folder: u32) -> Vec<PhotoInfo> {
    let folder = FolderId::new(folder);
    let mut photos: Vec<PhotoInfo> = {
        let index = library.index.read().expect("index lock");
        index
            .query(Scope::Folder(folder), &Filter::default())
            .into_iter()
            .filter_map(|id| {
                let photo = index.photo(id)?;
                Some(PhotoInfo {
                    id: id.value(),
                    name: photo.name.to_string(),
                    width: photo.dimensions.map(|(width, _)| width),
                    height: photo.dimensions.map(|(_, height)| height),
                    cloud_only: photo.placeholder,
                })
            })
            .collect()
    };
    photos.sort_by_key(|photo| photo.name.to_lowercase());
    library.queue_folder(folder);
    photos
}

/// The photos now on screen, in order: their thumbnails are made first (LIB-3).
#[tauri::command]
#[specta::specta]
pub fn show_thumbnails(library: State<'_, Library>, ids: Vec<u32>) {
    let jobs = {
        let index = library.index.read().expect("index lock");
        ids.into_iter()
            .filter_map(|id| index.thumbnail_job(PhotoId::new(id)))
            .collect()
    };
    library.workers.show(jobs);
}

fn display(path: &Path) -> String {
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn app_version_matches_the_package() {
        assert_eq!(super::app_version(), env!("CARGO_PKG_VERSION"));
    }
}
