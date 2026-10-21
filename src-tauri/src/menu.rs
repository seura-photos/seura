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
        t("app.name"),
        true,
        &[
            &PredefinedMenuItem::about(app, Some(t("menu.about")), None)?,
            &separator()?,
            &PredefinedMenuItem::hide(app, Some(t("menu.hide")))?,
            &PredefinedMenuItem::hide_others(app, Some(t("menu.hideOthers")))?,
            &PredefinedMenuItem::show_all(app, Some(t("menu.showAll")))?,
            &separator()?,
            &PredefinedMenuItem::quit(app, Some(t("menu.quit")))?,
        ],
    )?;
    let file = Submenu::with_items(
        app,
        t("menu.file"),
        true,
        &[
            &MenuItem::with_id(app, ADD_FOLDER, t("menu.addFolder"), true, None::<&str>)?,
            &separator()?,
            &PredefinedMenuItem::close_window(app, Some(t("menu.closeWindow")))?,
        ],
    )?;
    // Text fields need these on macOS, where shortcuts come from the menu.
    let edit = Submenu::with_items(
        app,
        t("menu.edit"),
        true,
        &[
            &PredefinedMenuItem::undo(app, Some(t("menu.undo")))?,
            &PredefinedMenuItem::redo(app, Some(t("menu.redo")))?,
            &separator()?,
            &PredefinedMenuItem::cut(app, Some(t("menu.cut")))?,
            &PredefinedMenuItem::copy(app, Some(t("menu.copy")))?,
            &PredefinedMenuItem::paste(app, Some(t("menu.paste")))?,
            &PredefinedMenuItem::select_all(app, Some(t("menu.selectAll")))?,
        ],
    )?;
    let window = Submenu::with_items(
        app,
        t("menu.window"),
        true,
        &[
            &PredefinedMenuItem::minimize(app, Some(t("menu.minimize")))?,
            &PredefinedMenuItem::fullscreen(app, Some(t("menu.fullscreen")))?,
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
