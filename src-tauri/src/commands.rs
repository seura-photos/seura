//! Commands the UI can call. Each one is also listed in `build.rs`, which
//! generates the permission the UI's capability grants for it.

use std::path::{Path, PathBuf};

use serde::Serialize;
use seura_library::{AddFolder, Filter, FolderId, PhotoId, Scope};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use crate::editor::Editor;
use crate::library::{AppSettings, Library, LibraryChanged};

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
    let Some(path) = pick_folder(&app).await? else {
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
        // The settings file holds the library folders (SET-3).
        let roots = library.index.read().expect("index lock").roots().to_vec();
        let settings = app.state::<AppSettings>();
        let mut settings = settings.0.lock().expect("settings lock");
        settings.settings.library_folders = roots;
        settings.save().map_err(|error| error.to_string())?;
        drop(settings);
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

/// A photo opened in the Editor.
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenedPhoto {
    /// The preview's size, fetched from `seura://…/preview`.
    pub width: u32,
    pub height: u32,
    pub exposure_ev: f64,
    /// Why the photo can't be edited, if it can't: its edit file can't be
    /// read, and is never overwritten (FIL-4).
    pub problem: Option<String>,
}

/// Opens a photo in the Editor, with a preview `longEdge` pixels on its long
/// side (the canvas size × the device pixel ratio, DSP-1).
#[tauri::command]
#[specta::specta]
pub async fn open_photo(app: AppHandle, id: u32, long_edge: u32) -> Result<OpenedPhoto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (photo, dimensions) = {
            let library = app.state::<Library>();
            let index = library.index.read().expect("index lock");
            let photo = index
                .path(PhotoId::new(id))
                .ok_or("that photo isn't in the library")?;
            let dimensions = index
                .photo(PhotoId::new(id))
                .and_then(|photo| photo.dimensions);
            (photo, dimensions)
        };
        let opened = app
            .state::<Editor>()
            .open(id, photo, dimensions, long_edge)?;
        Ok(OpenedPhoto {
            width: opened.width,
            height: opened.height,
            exposure_ev: opened.exposure_ev,
            problem: opened.problem,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
#[specta::specta]
pub fn close_photo(editor: State<'_, Editor>) {
    editor.close();
}

/// Sets the open photo's exposure (EDT-5) and saves its edit file (EDT-11).
#[tauri::command]
#[specta::specta]
pub async fn set_exposure(app: AppHandle, id: u32, ev: f64) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || app.state::<Editor>().set_exposure(id, ev))
        .await
        .map_err(|error| error.to_string())?
}

/// What exporting did.
#[derive(Clone, Debug, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ExportResult {
    /// The user closed the folder dialog.
    Cancelled,
    Exported {
        file: String,
        folder: String,
    },
}

/// Asks for a folder and exports the open photo there as JPEG (EXP-1, EXP-5).
#[tauri::command]
#[specta::specta]
pub async fn export_photo(app: AppHandle, id: u32) -> Result<ExportResult, String> {
    let Some(folder) = pick_folder(&app).await? else {
        return Ok(ExportResult::Cancelled);
    };
    tauri::async_runtime::spawn_blocking(move || {
        let path = app.state::<Editor>().export(id, &folder)?;
        Ok(ExportResult::Exported {
            file: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            folder: display(&folder),
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

/// The system's text size as a scale, 1 at the default (ACC-3). Windows
/// reads it from "Text size" in Accessibility settings. macOS has no
/// system-wide text size that apps can read, so it's always 1 there.
#[tauri::command]
#[specta::specta]
pub fn text_scale() -> f64 {
    #[cfg(target_os = "windows")]
    {
        windows::UI::ViewManagement::UISettings::new()
            .and_then(|settings| settings.TextScaleFactor())
            .unwrap_or(1.0)
    }
    #[cfg(not(target_os = "windows"))]
    {
        1.0
    }
}

/// The operating system's region as a BCP 47 tag, such as `de-DE`, for
/// formatting dates, numbers, and sizes (ACC-6). The interface itself is in
/// English; the web view's own locale follows the display language.
#[tauri::command]
#[specta::specta]
pub fn region() -> String {
    crate::locale::region()
}

/// Developer switches, from environment variables.
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DevOptions {
    /// `SEURA_PREVIEW=webgl2` forces the WebGL2 preview on a web view with
    /// WebGPU, to check the fallback (M1 slice 5).
    pub force_webgl2: bool,
    /// `SEURA_RENDER_CHECK=1` runs the render check instead of the app.
    pub render_check: bool,
    /// `SEURA_LOCALE=pseudo` pseudo-translates every string, to check that
    /// longer translations fit (ACC-7).
    pub pseudo_locale: bool,
    /// The exposure the render check uses, so both sides agree.
    pub check_exposure_ev: f32,
    /// Whether M1's Editor is available: in development builds only, since
    /// M2 writes no edit files (M2 work plan).
    pub editor_available: bool,
    /// Every switch that changes how the app behaves, as `NAME=value`, so
    /// the UI can say they're on: they're easy to leave set by accident.
    pub active: Vec<String>,
}

/// The developer switches that change the app's behavior.
const SWITCHES: [&str; 3] = [
    "SEURA_HEIF_DECODER",
    "SEURA_PREVIEW",
    crate::locale::LOCALE_SWITCH,
];

/// The switches that are set, as `NAME=value`.
pub fn active_switches() -> Vec<String> {
    SWITCHES
        .iter()
        .filter_map(|name| {
            let value = std::env::var(name).ok()?;
            Some(format!("{name}={value}"))
        })
        .collect()
}

#[tauri::command]
#[specta::specta]
pub fn dev_options() -> DevOptions {
    DevOptions {
        force_webgl2: std::env::var("SEURA_PREVIEW").as_deref() == Ok("webgl2"),
        render_check: render_check_mode(),
        pseudo_locale: crate::locale::pseudo(),
        check_exposure_ev: seura_render::check::CHECK_SETTINGS.exposure_ev,
        editor_available: cfg!(debug_assertions),
        active: active_switches(),
    }
}

/// The render check: one preview path's render of the check image, as 8-bit
/// RGBA, compared with wgpu's. Returns the largest difference.
#[tauri::command]
#[specta::specta]
pub async fn check_render(
    app: AppHandle,
    backend: String,
    width: u32,
    height: u32,
    pixels: Vec<u8>,
) -> Result<u8, String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<Editor>()
            .check_render(backend, width, height, pixels)
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Ends the render check: the app exits, with status 0 only if every
/// preview path that ran matched export and nothing went wrong. Paths the
/// web view doesn't have are listed, not failed. Does nothing outside the check.
#[tauri::command]
#[specta::specta]
pub fn finish_render_check(app: AppHandle, skipped: Vec<String>, problems: Vec<String>) {
    if !render_check_mode() {
        return;
    }
    for path in &skipped {
        println!("render check, {path}: not available in this web view");
    }
    for problem in &problems {
        println!("render check: {problem}");
    }
    let passed = problems.is_empty() && app.state::<Editor>().checks_passed();
    println!("render check {}", if passed { "passed" } else { "FAILED" });
    app.exit(if passed { 0 } else { 1 });
}

fn render_check_mode() -> bool {
    std::env::var("SEURA_RENDER_CHECK").as_deref() == Ok("1")
}

/// The system's folder dialog (ACC-4), or `None` if the user cancels.
async fn pick_folder(app: &AppHandle) -> Result<Option<PathBuf>, String> {
    // Native dialogs must run on the main thread (macOS requires it).
    let (send, receive) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = send.send(rfd::FileDialog::new().pick_folder());
    })
    .map_err(|error| error.to_string())?;
    tauri::async_runtime::spawn_blocking(move || receive.recv().ok().flatten())
        .await
        .map_err(|error| error.to_string())
}

fn display(path: &Path) -> String {
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_text_scale_is_readable_and_sensible() {
        let scale = super::text_scale();
        assert!((1.0..=2.25).contains(&scale), "{scale}");
    }

    #[test]
    fn app_version_matches_the_package() {
        assert_eq!(super::app_version(), env!("CARGO_PKG_VERSION"));
    }
}
