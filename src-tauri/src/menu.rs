//! The native menu (ACC-4). macOS always shows an app menu, so Seura gives
//! it the standard items plus its own commands. Windows has no menu bar for
//! now: modern Windows apps rarely show one, and the toolbar and keyboard
//! cover the same commands (a question for a design session).

use serde::Serialize;
use specta::Type;
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
}

const ADD_FOLDER: &str = "add-folder";

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
    let file = Submenu::with_items(
        app,
        t("menu-file"),
        true,
        &[
            &MenuItem::with_id(app, ADD_FOLDER, t("menu-add-folder"), true, None::<&str>)?,
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
    if id == ADD_FOLDER {
        let _ = MenuCommand::AddFolder.emit(app);
    }
}
