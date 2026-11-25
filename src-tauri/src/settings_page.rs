//! The Settings page (SET-2; wireframes, "Settings"): what it shows beside
//! the library folders, which come from the Library's own commands, and on
//! macOS its window. Changes apply at once and are saved in `settings.json`
//! (SET-3).

use std::path::Path;

use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager, Runtime};

use crate::exporting::ExportChoices;
use crate::library::AppSettings;
use crate::locations;

/// What the page shows, apart from the library folders and the cache's
/// size, which takes a while to add up.
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPage {
    /// The `Seura Photos` folder (decision 0015).
    pub seura_folder: String,
    pub cache_folder: String,
    /// The Export dialog's choices, which it opens with (EXP-2).
    pub export: ExportChoices,
    pub check_for_updates: bool,
}

pub fn page<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<SettingsPage> {
    let seura = locations::seura_folder(app)?;
    let settings = app.state::<AppSettings>();
    let file = settings.0.lock().expect("settings lock");
    let cache = locations::cache_folder(&seura, &file.settings);
    Ok(SettingsPage {
        seura_folder: seura.display().to_string(),
        cache_folder: cache.display().to_string(),
        export: ExportChoices::from(&file.settings.export),
        check_for_updates: file.settings.check_for_updates,
    })
}

/// The cache's size in bytes: Seura's own entries in the cache folder, added
/// up, and nothing else the folder holds. A file that can't be read counts
/// as nothing.
pub fn cache_size<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<u64> {
    let seura = locations::seura_folder(app)?;
    let settings = app.state::<AppSettings>();
    let cache = {
        let file = settings.0.lock().expect("settings lock");
        locations::cache_folder(&seura, &file.settings)
    };
    Ok(cache_entries_size(&cache))
}

fn cache_entries_size(cache: &Path) -> u64 {
    locations::cache_entries()
        .map(|name| {
            let entry = cache.join(name);
            match std::fs::symlink_metadata(&entry) {
                Ok(data) if data.is_dir() => folder_size(&entry),
                Ok(data) if data.is_file() => data.len(),
                _ => 0,
            }
        })
        .sum()
}

fn folder_size(folder: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| match entry.file_type() {
            // A link is counted as itself, never followed out of the cache.
            Ok(kind) if kind.is_dir() => folder_size(&entry.path()),
            Ok(kind) if kind.is_file() => entry.metadata().map_or(0, |data| data.len()),
            _ => 0,
        })
        .sum()
}

/// Sets what the Export dialog opens with (EXP-2), keeping its folder.
pub fn set_export<R: Runtime>(app: &AppHandle<R>, choices: &ExportChoices) -> Result<(), String> {
    if !choices.valid() {
        return Err("those export choices aren't valid".into());
    }
    let settings = app.state::<AppSettings>();
    let mut file = settings.0.lock().expect("settings lock");
    let folder = file.settings.export.folder.take();
    file.settings.export = choices.to_settings(folder);
    file.save().map_err(|error| error.to_string())
}

/// Turns update checks on or off (SET-4).
pub fn set_check_for_updates<R: Runtime>(app: &AppHandle<R>, on: bool) -> Result<(), String> {
    let settings = app.state::<AppSettings>();
    let mut file = settings.0.lock().expect("settings lock");
    file.settings.check_for_updates = on;
    file.save().map_err(|error| error.to_string())
}

/// The label of the Settings window on macOS.
#[cfg(target_os = "macos")]
pub const WINDOW: &str = "settings";

/// Opens the Settings window from the app menu (Cmd+,), or brings it to
/// the front if it's open. It's a window of its own, as Mac settings are,
/// with no buttons: it closes like any other window (SET-2). Like the main
/// window, it keeps nothing of its own (decision 0013).
#[cfg(target_os = "macos")]
pub fn open_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(WINDOW) {
        return window.set_focus();
    }
    tauri::WebviewWindowBuilder::new(
        app,
        WINDOW,
        tauri::WebviewUrl::App("index.html#settings".into()),
    )
    .title(crate::messages::t("settings-title"))
    .inner_size(640.0, 720.0)
    .min_inner_size(560.0, 480.0)
    .minimizable(false)
    .incognito(true)
    .build()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cache_s_size_is_seura_s_entries_only() {
        // A folder the user chose, holding their own files too.
        let cache = tempfile::tempdir().expect("temporary folder");
        std::fs::write(cache.path().join("index.bin"), [0; 10]).expect("file");
        std::fs::create_dir_all(cache.path().join("thumbs/ab")).expect("folders");
        std::fs::write(cache.path().join("thumbs/ab/one.webp"), [0; 32]).expect("file");
        std::fs::create_dir_all(cache.path().join("WebView")).expect("folder");
        std::fs::write(cache.path().join("WebView/Local State"), [0; 5]).expect("file");
        std::fs::write(cache.path().join("holiday.mov"), [0; 1000]).expect("file");
        std::fs::create_dir_all(cache.path().join("Work")).expect("folder");
        std::fs::write(cache.path().join("Work/notes.txt"), [0; 100]).expect("file");
        assert_eq!(cache_entries_size(cache.path()), 47);
        assert_eq!(cache_entries_size(&cache.path().join("not there")), 0);
    }
}
