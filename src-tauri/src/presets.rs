//! Presets as the app holds them (PRE-3, PRE-5): the Presets folder in
//! Seura's folder (decision 0015), watched while the app runs so files
//! copied in appear without restarting.

use std::path::Path;
use std::sync::Mutex;

use serde::Serialize;
use seura_edit::{PresetError, PresetsFolder, Stage};
use seura_library::Watcher;
use specta::Type;
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

/// The Presets folder changed: a preset was saved, renamed, removed, or
/// copied in.
#[derive(Clone, Debug, Serialize, Type, Event)]
pub struct PresetsChanged;

pub struct Presets {
    pub folder: PresetsFolder,
    /// Watches the folder once it exists, which may be only after the first
    /// preset is saved.
    watcher: Mutex<Option<Watcher>>,
}

impl Presets {
    pub fn new(seura: &Path) -> Presets {
        Presets {
            folder: PresetsFolder::new(seura.join("Presets")),
            watcher: Mutex::new(None),
        }
    }

    /// Starts watching the folder, if it exists and isn't watched yet.
    pub fn watch<R: Runtime>(&self, app: &AppHandle<R>) {
        let mut watcher = self.watcher.lock().expect("presets watcher lock");
        if watcher
            .as_ref()
            .is_some_and(|watcher| !watcher.watched().is_empty())
            || !self.folder.path().is_dir()
        {
            return;
        }
        let handle = app.clone();
        let Ok(mut started) = Watcher::start(Vec::new(), move |_| changed(&handle)) else {
            return;
        };
        started.watch(&[self.folder.path().to_path_buf()]);
        *watcher = Some(started);
    }

    /// What applying the preset `name` pastes, and the groups it changes
    /// (PRE-4), read from its file now.
    pub fn load(
        &self,
        name: &str,
    ) -> Result<(seura_edit::CopiedEdits, seura_edit::PasteGroups), PresetProblem> {
        let preset = self
            .folder
            .list()
            .map_err(|_| PresetProblem::Gone)?
            .into_iter()
            .find(|preset| preset.name == name)
            .ok_or(PresetProblem::Gone)?;
        let document = preset.document.map_err(|_| PresetProblem::Unreadable)?;
        seura_edit::CopiedEdits::from_preset(&document).ok_or(PresetProblem::Empty)
    }
}

/// Why a preset can't be applied.
#[derive(Clone, Copy, Debug, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PresetProblem {
    /// It's no longer in the Presets folder.
    Gone,
    /// Its file can't be read (rule 4).
    Unreadable,
    /// It holds nothing Seura can apply, such as only another app's operations.
    Empty,
}

/// Tells the UI, and the Mac's Edit menu, that the Presets folder changed.
pub fn changed<R: Runtime>(app: &AppHandle<R>) {
    let _ = PresetsChanged.emit(app);
    if let Some(presets) = app.try_state::<Presets>()
        && let Ok(list) = summaries(&presets.folder)
    {
        crate::menu::presets_changed(app, &list);
    }
}

/// One preset in the list (PRE-5).
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PresetSummary {
    /// Its file's name, without the suffix.
    pub name: String,
    pub adjustments: bool,
    pub effects: bool,
    pub geometry: bool,
    /// Why it can't be read, if it can't (rule 4).
    pub problem: Option<String>,
}

/// The presets, by name.
pub fn summaries(folder: &PresetsFolder) -> std::io::Result<Vec<PresetSummary>> {
    Ok(folder
        .list()?
        .into_iter()
        .map(|preset| {
            let holds = |stage: Stage| {
                preset.document.as_ref().is_ok_and(|document| {
                    document.operations().iter().any(|operation| {
                        operation
                            .core
                            .as_ref()
                            .is_some_and(|core| core.id().stage() == stage)
                    })
                })
            };
            PresetSummary {
                adjustments: holds(Stage::Tone),
                effects: holds(Stage::Effects),
                geometry: holds(Stage::Geometry),
                problem: preset.document.as_ref().err().cloned(),
                name: preset.name,
            }
        })
        .collect())
}

/// What saving, renaming, or removing a preset did.
#[derive(Clone, Debug, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PresetOutcome {
    /// Done, under this name.
    Done { name: String },
    /// Another preset has the name, ignoring letter case; `readable` is
    /// whether it could be replaced.
    NameInUse { name: String, readable: bool },
    /// Nothing a file name can hold is left of the name.
    InvalidName,
    /// Nothing in the chosen groups to save.
    NothingToSave,
    /// The photo's edit file, or the preset, can't be read (rule 4).
    Unreadable,
    /// The photo or the preset is no longer there.
    Gone,
    /// The drive has no Recycle Bin or Trash, so the preset was left as it is.
    NoTrash,
}

/// The outcome of a preset error, or the error's own words when it's a
/// failure to report as one.
pub fn outcome(error: PresetError) -> Result<PresetOutcome, String> {
    Ok(match error {
        PresetError::InvalidName => PresetOutcome::InvalidName,
        PresetError::NameInUse { name, readable } => PresetOutcome::NameInUse { name, readable },
        PresetError::NotFound => PresetOutcome::Gone,
        PresetError::Unreadable(_) => PresetOutcome::Unreadable,
        PresetError::Io(error) if seura_edit::NoTrash::is(&error) => PresetOutcome::NoTrash,
        other => return Err(other.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_says_what_each_preset_holds_and_which_cant_be_read() {
        let dir = tempfile::tempdir().unwrap();
        let folder = PresetsFolder::new(dir.path().to_path_buf());
        std::fs::write(
            dir.path().join("Warm Vintage.seura-preset.json"),
            include_bytes!("../../third_party/edit-format/examples/Warm Vintage.seura-preset.json"),
        )
        .unwrap();
        std::fs::write(dir.path().join("Broken.seura-preset.json"), b"oops").unwrap();

        let list = summaries(&folder).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "Broken");
        assert!(list[0].problem.is_some());
        assert!(!list[0].adjustments);
        let warm = &list[1];
        assert_eq!(warm.name, "Warm Vintage");
        assert!(warm.problem.is_none());
        assert!(warm.adjustments && warm.effects && !warm.geometry);
    }
}
