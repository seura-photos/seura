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
use crate::messages::{FluentValue, t, t_with};
#[cfg(target_os = "macos")]
use seura_library::Change;

/// A menu command for the UI to carry out, so each command has one path
/// whether it starts in the menu, the toolbar, or the keyboard.
#[derive(Clone, Debug, Serialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub enum MenuCommand {
    AddFolder,
    /// For the chosen photos (BRW-8).
    ShowInFolder,
    OpenInDefaultApp,
    /// The Edit menu's Undo and Redo: the Library's last action (BRW-11),
    /// or, in a text field, the field's own.
    Undo,
    Redo,
}

const ADD_FOLDER: &str = "add-folder";
const SHOW_IN_FOLDER: &str = "show-in-folder";
const OPEN_IN_DEFAULT_APP: &str = "open-in-default-app";
const UNDO: &str = "undo";
const REDO: &str = "redo";
/// The menu bar's commands for the chosen photos, kept so they can be
/// turned on and off. (A menu's lookup by id searches only its top level,
/// not the File menu inside it, so they're held rather than looked up.)
#[cfg(target_os = "macos")]
struct PhotoCommands<R: Runtime>([MenuItem<R>; 2]);

/// The Edit menu's Undo and Redo, kept so their labels can follow the
/// Library's history.
#[cfg(target_os = "macos")]
struct HistoryCommands<R: Runtime>([MenuItem<R>; 2]);

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
    // Seura's own Undo and Redo, which name the Library's last action
    // (BRW-11) and pass to a text field's own when one has the focus. On,
    // always, so text fields keep them.
    let undo = MenuItem::with_id(app, UNDO, t("menu-undo"), true, Some("CmdOrCtrl+Z"))?;
    let redo = MenuItem::with_id(app, REDO, t("menu-redo"), true, Some("Shift+CmdOrCtrl+Z"))?;
    app.manage(HistoryCommands([undo.clone(), redo.clone()]));
    // Text fields need these on macOS, where shortcuts come from the menu.
    let edit = Submenu::with_items(
        app,
        t("menu-edit"),
        true,
        &[
            &undo,
            &redo,
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
        UNDO => MenuCommand::Undo,
        REDO => MenuCommand::Redo,
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

/// Names the Library's next undo and redo in the Edit menu, "Undo Rating (5
/// photos)", or plain "Undo" when there's nothing (BRW-11). Windows has no
/// menu bar; a notice says what was undone on both platforms.
#[cfg(target_os = "macos")]
pub fn history_changed<R: Runtime>(
    app: &AppHandle<R>,
    undo: Option<(&Change, usize)>,
    redo: Option<(&Change, usize)>,
) {
    let Some(commands) = app.try_state::<HistoryCommands<R>>() else {
        return;
    };
    let label = |id: &str, plain: &str, next: Option<(&Change, usize)>| match next {
        None => t(plain),
        Some((change, count)) => {
            let (kind, tag) = match change {
                Change::Rating(_) => ("rating", ""),
                Change::AddTag(tag) => ("addTag", tag.as_str()),
                Change::RemoveTag(tag) => ("removeTag", tag.as_str()),
            };
            t_with(
                id,
                &[
                    ("kind", FluentValue::from(kind)),
                    ("tag", FluentValue::from(tag)),
                    ("count", FluentValue::from(count)),
                ],
            )
        }
    };
    let _ = commands.0[0].set_text(label("menu-undo-action", "menu-undo", undo));
    let _ = commands.0[1].set_text(label("menu-redo-action", "menu-redo", redo));
}

#[cfg(not(target_os = "macos"))]
pub fn history_changed<R: Runtime>(
    _app: &AppHandle<R>,
    _undo: Option<(&seura_library::Change, usize)>,
    _redo: Option<(&seura_library::Change, usize)>,
) {
}
