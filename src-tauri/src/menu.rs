//! The native menu (ACC-4). macOS always shows an app menu, so Seura gives
//! it the standard items plus its own commands. Windows has no menu bar:
//! everything in the menu is also in the window and on the keyboard, and
//! Nevada decided there's no need for one unless something would live only
//! there (2026-10-21).

use serde::Serialize;
use specta::Type;
#[cfg(target_os = "macos")]
use tauri::Manager;
#[cfg(target_os = "macos")]
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Runtime};
use tauri_specta::Event;

#[cfg(target_os = "macos")]
use crate::messages::t;

/// A menu command for the UI to carry out, so each command has one path
/// whether it starts in the menu, the toolbar, or the keyboard.
#[derive(Clone, Debug, Serialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub enum MenuCommand {
    AddFolder,
    /// For the chosen photos (BRW-8).
    ShowInFolder,
    OpenInDefaultApp,
}

const ADD_FOLDER: &str = "add-folder";
const SHOW_IN_FOLDER: &str = "show-in-folder";
const OPEN_IN_DEFAULT_APP: &str = "open-in-default-app";
/// The menu bar's commands for the chosen photos, kept so they can be
/// turned on and off. (A menu's lookup by id searches only its top level,
/// not the File menu inside it, so they're held rather than looked up.)
#[cfg(target_os = "macos")]
struct PhotoCommands<R: Runtime>([MenuItem<R>; 2]);

#[cfg(target_os = "macos")]
pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let separator = || PredefinedMenuItem::separator(app);
    let app_menu = Submenu::with_items(
        app,
        t("app-name"),
        true,
        &[
            &PredefinedMenuItem::about(app, Some(t("menu-about").as_str()), None)?,
            &separator()?,
            &PredefinedMenuItem::hide(app, Some(t("menu-hide").as_str()))?,
            &PredefinedMenuItem::hide_others(app, Some(t("menu-hide-others").as_str()))?,
            &PredefinedMenuItem::show_all(app, Some(t("menu-show-all").as_str()))?,
            &separator()?,
            &PredefinedMenuItem::quit(app, Some(t("menu-quit").as_str()))?,
        ],
    )?;
    // The photo menu's commands, also in the File menu: a context menu is
    // hidden by default, so Apple's guidelines put its items in the menu bar
    // too (BRW-8). Off until a photo is chosen.
    let show_in_folder = MenuItem::with_id(
        app,
        SHOW_IN_FOLDER,
        t("menu-show-in-finder"),
        false,
        Some("Alt+CmdOrCtrl+R"),
    )?;
    let open_in_default_app = MenuItem::with_id(
        app,
        OPEN_IN_DEFAULT_APP,
        t("menu-open-in-default-app"),
        false,
        Some("Alt+CmdOrCtrl+O"),
    )?;
    app.manage(PhotoCommands([
        show_in_folder.clone(),
        open_in_default_app.clone(),
    ]));
    let file = Submenu::with_items(
        app,
        t("menu-file"),
        true,
        &[
            &MenuItem::with_id(app, ADD_FOLDER, t("menu-add-folder"), true, None::<&str>)?,
            &separator()?,
            &show_in_folder,
            &open_in_default_app,
            &separator()?,
            &PredefinedMenuItem::close_window(app, Some(t("menu-close-window").as_str()))?,
        ],
    )?;
    // Text fields need these on macOS, where shortcuts come from the menu.
    let edit = Submenu::with_items(
        app,
        t("menu-edit"),
        true,
        &[
            &PredefinedMenuItem::undo(app, Some(t("menu-undo").as_str()))?,
            &PredefinedMenuItem::redo(app, Some(t("menu-redo").as_str()))?,
            &separator()?,
            &PredefinedMenuItem::cut(app, Some(t("menu-cut").as_str()))?,
            &PredefinedMenuItem::copy(app, Some(t("menu-copy").as_str()))?,
            &PredefinedMenuItem::paste(app, Some(t("menu-paste").as_str()))?,
            &PredefinedMenuItem::select_all(app, Some(t("menu-select-all").as_str()))?,
        ],
    )?;
    let window = Submenu::with_items(
        app,
        t("menu-window"),
        true,
        &[
            &PredefinedMenuItem::minimize(app, Some(t("menu-minimize").as_str()))?,
            &PredefinedMenuItem::fullscreen(app, Some(t("menu-fullscreen").as_str()))?,
        ],
    )?;
    Menu::with_items(app, &[&app_menu, &file, &edit, &window])
}

/// Passes Seura's own menu commands to the UI.
pub fn handle<R: Runtime>(app: &AppHandle<R>, id: &str) {
    let command = match id {
        ADD_FOLDER => MenuCommand::AddFolder,
        SHOW_IN_FOLDER => MenuCommand::ShowInFolder,
        OPEN_IN_DEFAULT_APP => MenuCommand::OpenInDefaultApp,
        _ => return,
    };
    let _ = command.emit(app);
}

/// Turns the photo commands on while a photo is chosen. Windows has no
/// menu bar.
#[cfg(target_os = "macos")]
pub fn photos_chosen<R: Runtime>(app: &AppHandle<R>, chosen: bool) {
    if let Some(commands) = app.try_state::<PhotoCommands<R>>() {
        for item in &commands.0 {
            let _ = item.set_enabled(chosen);
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn photos_chosen<R: Runtime>(_app: &AppHandle<R>, _chosen: bool) {}
