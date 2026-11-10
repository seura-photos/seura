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
    /// Moves the chosen photos to the Trash, once confirmed (BRW-9).
    Delete,
    /// Opens the Export dialog for the chosen photos (EXP-1).
    Export,
    /// Copies the focused photo's edits (PRE-1).
    CopyEdits,
    /// Asks what to paste onto the chosen photos (PRE-1).
    PasteEdits,
    /// Applies the preset with this name to the chosen photos, or the photo
    /// in the Editor (PRE-4).
    ApplyPreset(String),
    /// The Edit menu's Undo and Redo: the Library's last action (BRW-11),
    /// or, in a text field, the field's own.
    Undo,
    Redo,
    /// The Edit menu's Select All: the Library's photos, or, in a text
    /// field, the field's text (#227).
    SelectAll,
}

const ADD_FOLDER: &str = "add-folder";
const QUIT: &str = "quit";
const SHOW_IN_FOLDER: &str = "show-in-folder";
const OPEN_IN_DEFAULT_APP: &str = "open-in-default-app";
const DELETE: &str = "delete";
const EXPORT: &str = "export";
const COPY_EDITS: &str = "copy-edits";
const PASTE_EDITS: &str = "paste-edits";
const UNDO: &str = "undo";
const REDO: &str = "redo";
const SELECT_ALL: &str = "select-all";
/// Each preset's item in the Edit menu: this, then its name.
const APPLY_PRESET: &str = "apply-preset:";
/// The menu bar's commands for the chosen photos, kept so they can be
/// turned on and off. (A menu's lookup by id searches only its top level,
/// not the File menu inside it, so they're held rather than looked up.)
#[cfg(target_os = "macos")]
struct PhotoCommands<R: Runtime>([MenuItem<R>; 6]);

/// The Edit menu's Undo and Redo, kept so their labels can follow the
/// Library's history.
#[cfg(target_os = "macos")]
struct HistoryCommands<R: Runtime>([MenuItem<R>; 2]);

/// The Edit menu's Presets submenu, kept so its items can follow the
/// Presets folder (PRE-4).
#[cfg(target_os = "macos")]
struct PresetsMenu<R: Runtime>(Submenu<R>);

#[cfg(target_os = "macos")]
pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let separator = || PredefinedMenuItem::separator(app);
    let app_menu = Submenu::with_items(
        app,
        t("app-name"),
        true,
        &[
            &PredefinedMenuItem::about(
                app,
                Some(t("menu-about").as_str()),
                Some(crate::about::metadata()),
            )?,
            &separator()?,
            &PredefinedMenuItem::hide(app, Some(t("menu-hide").as_str()))?,
            &PredefinedMenuItem::hide_others(app, Some(t("menu-hide-others").as_str()))?,
            &PredefinedMenuItem::show_all(app, Some(t("menu-show-all").as_str()))?,
            &separator()?,
            // Not the predefined Quit, which exits without waiting for a save (#174).
            &MenuItem::with_id(app, QUIT, t("menu-quit"), true, Some("CmdOrCtrl+Q"))?,
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
    // Cmd+Delete, as in Finder and Photos.
    let delete = MenuItem::with_id(
        app,
        DELETE,
        t("menu-move-to-trash"),
        false,
        Some("CmdOrCtrl+Backspace"),
    )?;
    let export = MenuItem::with_id(app, EXPORT, t("menu-export"), false, Some("CmdOrCtrl+E"))?;
    let copy_edits = MenuItem::with_id(
        app,
        COPY_EDITS,
        t("menu-copy-edits"),
        false,
        Some("Shift+CmdOrCtrl+C"),
    )?;
    let paste_edits = MenuItem::with_id(
        app,
        PASTE_EDITS,
        t("menu-paste-edits"),
        false,
        Some("Shift+CmdOrCtrl+V"),
    )?;
    app.manage(PhotoCommands([
        show_in_folder.clone(),
        open_in_default_app.clone(),
        delete.clone(),
        export.clone(),
        copy_edits.clone(),
        paste_edits.clone(),
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
            &export,
            &separator()?,
            &delete,
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
    // Presets for the chosen photos (PRE-4), filled in from the folder by
    // `presets_changed`; off until a photo is chosen.
    let presets = Submenu::new(app, t("menu-presets"), false)?;
    app.manage(PresetsMenu(presets.clone()));
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
            // Seura's own, since the system's only reaches a text field (#227).
            &MenuItem::with_id(
                app,
                SELECT_ALL,
                t("menu-select-all"),
                true,
                Some("CmdOrCtrl+A"),
            )?,
            &separator()?,
            &copy_edits,
            &paste_edits,
            &presets,
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
    if id == QUIT {
        crate::quitting::request(app);
        return;
    }
    let command = match id {
        ADD_FOLDER => MenuCommand::AddFolder,
        SHOW_IN_FOLDER => MenuCommand::ShowInFolder,
        OPEN_IN_DEFAULT_APP => MenuCommand::OpenInDefaultApp,
        DELETE => MenuCommand::Delete,
        EXPORT => MenuCommand::Export,
        COPY_EDITS => MenuCommand::CopyEdits,
        PASTE_EDITS => MenuCommand::PasteEdits,
        UNDO => MenuCommand::Undo,
        REDO => MenuCommand::Redo,
        SELECT_ALL => MenuCommand::SelectAll,
        _ => match id.strip_prefix(APPLY_PRESET) {
            Some(name) => MenuCommand::ApplyPreset(name.into()),
            None => return,
        },
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
    if let Some(presets) = app.try_state::<PresetsMenu<R>>() {
        let _ = presets.0.set_enabled(chosen);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn photos_chosen<R: Runtime>(_app: &AppHandle<R>, _chosen: bool) {}

/// Fills the Edit menu's Presets submenu from the Presets folder: one item
/// each, by name, off for one that can't be read, or a note that there are
/// none. Windows has no menu bar.
#[cfg(target_os = "macos")]
pub fn presets_changed<R: Runtime>(app: &AppHandle<R>, presets: &[crate::presets::PresetSummary]) {
    let Some(menu) = app.try_state::<PresetsMenu<R>>() else {
        return;
    };
    if let Ok(items) = menu.0.items() {
        for item in items {
            let _ = menu.0.remove(&item);
        }
    }
    if presets.is_empty() {
        if let Ok(none) = MenuItem::new(app, t("menu-no-presets"), false, None::<&str>) {
            let _ = menu.0.append(&none);
        }
        return;
    }
    for preset in presets {
        let id = format!("{APPLY_PRESET}{}", preset.name);
        let readable = preset.problem.is_none();
        if let Ok(item) = MenuItem::with_id(app, id, &preset.name, readable, None::<&str>) {
            let _ = menu.0.append(&item);
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn presets_changed<R: Runtime>(
    _app: &AppHandle<R>,
    _presets: &[crate::presets::PresetSummary],
) {
}

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
                Change::Rotate { clockwise: true } => ("rotateRight", ""),
                Change::Rotate { clockwise: false } => ("rotateLeft", ""),
                Change::Enhance => ("enhance", ""),
                Change::Paste => ("paste", ""),
                Change::Preset(name) => ("preset", name.as_str()),
                Change::Unmark => ("unmark", ""),
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

/// Names the Editor's next undo and redo in the Edit menu, "Undo
/// Brightness", from the photo's own history (EDT-9), while it's open.
#[cfg(target_os = "macos")]
pub fn edit_steps_changed<R: Runtime>(app: &AppHandle<R>, steps: &crate::editor::EditSteps) {
    let Some(commands) = app.try_state::<HistoryCommands<R>>() else {
        return;
    };
    let label = |id: &str, plain: &str, next: &Option<String>| match next {
        None => t(plain),
        Some(change) => t_with(id, &[("change", FluentValue::from(change.as_str()))]),
    };
    let _ = commands.0[0].set_text(label("menu-undo-edit", "menu-undo", &steps.undo));
    let _ = commands.0[1].set_text(label("menu-redo-edit", "menu-redo", &steps.redo));
}

#[cfg(not(target_os = "macos"))]
pub fn edit_steps_changed<R: Runtime>(_app: &AppHandle<R>, _steps: &crate::editor::EditSteps) {}

#[cfg(not(target_os = "macos"))]
pub fn history_changed<R: Runtime>(
    _app: &AppHandle<R>,
    _undo: Option<(&seura_library::Change, usize)>,
    _redo: Option<(&seura_library::Change, usize)>,
) {
}
