//! Seura's settings (SET-3; decision 0015): a readable `settings.json` in the
//! `Seura Photos` folder in the user's home folder. It, not the cache, holds
//! the library folders, so clearing the cache never forgets the library.
//!
//! Written atomically like every file Seura writes. A file that can't be
//! read is never overwritten: the app runs on defaults and leaves it for the
//! user. Fields this version doesn't know, from a newer one, are kept.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The settings file's name in Seura's folder.
pub const SETTINGS_FILE: &str = "settings.json";

/// Thumbnail sizes, in logical pixels per grid cell (BRW-3).
pub const THUMBNAIL_SIZES: (u32, u32) = (96, 256);
pub const DEFAULT_THUMBNAIL_SIZE: u32 = 160;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// The library folders (LIB-1), in the order they were added.
    pub library_folders: Vec<PathBuf>,
    /// Where the cache is, if the user moved it; otherwise `Cache` in
    /// Seura's folder.
    pub cache_folder: Option<PathBuf>,
    /// Grid cell size in logical pixels, 96 to 256 (BRW-3).
    pub thumbnail_size: u32,
    /// One sort order for the whole app (BRW-2).
    pub sort: Sort,
    /// Whether to check for updates (SET-4); asked on first run, used from M7.
    pub check_for_updates: bool,
    /// Fields from a newer version, kept as they were.
    #[serde(flatten)]
    pub unknown: Map<String, Value>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Sort {
    pub by: SortBy,
    pub ascending: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortBy {
    /// Falling back to the file's date when there's none (BRW-2).
    #[default]
    DateTaken,
    FileName,
    DateModified,
}

impl Default for Sort {
    fn default() -> Sort {
        Sort {
            by: SortBy::DateTaken,
            ascending: true,
        }
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            library_folders: Vec::new(),
            cache_folder: None,
            thumbnail_size: DEFAULT_THUMBNAIL_SIZE,
            sort: Sort::default(),
            check_for_updates: true,
            unknown: Map::new(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("couldn't read the settings file: {0}")]
    Io(#[from] io::Error),
    #[error("the settings file isn't valid: {0}")]
    Invalid(String),
}

/// The settings and where they're kept.
#[derive(Debug)]
pub struct SettingsFile {
    path: PathBuf,
    pub settings: Settings,
    /// Why the file on disk can't be used, if it can't. Such a file is never
    /// overwritten; changes stay in memory.
    pub problem: Option<String>,
}

impl SettingsFile {
    /// Reads `settings.json` in `folder`. `Ok(None)` if there's no file yet,
    /// which is a first run.
    pub fn load(folder: &Path) -> Result<Option<SettingsFile>, SettingsError> {
        let path = folder.join(SETTINGS_FILE);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let mut settings: Settings = serde_json::from_slice(&bytes)
            .map_err(|error| SettingsError::Invalid(error.to_string()))?;
        settings.thumbnail_size = settings
            .thumbnail_size
            .clamp(THUMBNAIL_SIZES.0, THUMBNAIL_SIZES.1);
        Ok(Some(SettingsFile {
            path,
            settings,
            problem: None,
        }))
    }

    /// Settings kept in `folder`, starting from `settings`, not yet written.
    pub fn new(folder: &Path, settings: Settings) -> SettingsFile {
        SettingsFile {
            path: folder.join(SETTINGS_FILE),
            settings,
            problem: None,
        }
    }

    /// Defaults in memory, for when the file on disk can't be read. Saving
    /// does nothing, so that file is left alone.
    pub fn unreadable(folder: &Path, problem: String) -> SettingsFile {
        SettingsFile {
            path: folder.join(SETTINGS_FILE),
            settings: Settings::default(),
            problem: Some(problem),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Writes the settings: readable JSON, two-space indent, atomically
    /// (CLAUDE.md rule 3). Does nothing if the file on disk couldn't be read.
    pub fn save(&self) -> io::Result<()> {
        if self.problem.is_some() {
            return Ok(());
        }
        let mut bytes =
            serde_json::to_vec_pretty(&self.settings).expect("settings always serialize");
        bytes.push(b'\n');
        fs::create_dir_all(self.path.parent().expect("the file is in a folder"))?;
        seura_edit::write_atomically(&self.path, &bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_file_is_a_first_run() {
        let folder = tempfile::tempdir().unwrap();
        assert!(SettingsFile::load(folder.path()).unwrap().is_none());
    }

    #[test]
    fn settings_round_trip_as_readable_json() {
        let folder = tempfile::tempdir().unwrap();
        let mut file = SettingsFile::new(folder.path(), Settings::default());
        file.settings.library_folders = vec![PathBuf::from("/photos/2026")];
        file.settings.sort.by = SortBy::FileName;
        file.save().unwrap();

        let text = fs::read_to_string(folder.path().join(SETTINGS_FILE)).unwrap();
        assert!(text.contains("\"libraryFolders\""), "{text}");
        assert!(text.contains("\"fileName\""), "{text}");
        let loaded = SettingsFile::load(folder.path()).unwrap().unwrap();
        assert_eq!(loaded.settings, file.settings);
    }

    #[test]
    fn missing_fields_take_defaults_and_unknown_ones_are_kept() {
        let folder = tempfile::tempdir().unwrap();
        let json = r#"{ "thumbnailSize": 999, "fromTheFuture": { "a": 1 } }"#;
        fs::write(folder.path().join(SETTINGS_FILE), json).unwrap();

        let file = SettingsFile::load(folder.path()).unwrap().unwrap();
        assert_eq!(file.settings.thumbnail_size, THUMBNAIL_SIZES.1);
        assert!(file.settings.check_for_updates);
        file.save().unwrap();
        let text = fs::read_to_string(folder.path().join(SETTINGS_FILE)).unwrap();
        assert!(text.contains("\"fromTheFuture\""), "{text}");
    }

    #[test]
    fn an_unreadable_file_is_never_overwritten() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join(SETTINGS_FILE);
        fs::write(&path, b"{ not json").unwrap();

        let error = SettingsFile::load(folder.path()).unwrap_err();
        let mut file = SettingsFile::unreadable(folder.path(), error.to_string());
        file.settings.library_folders = vec![PathBuf::from("/photos")];
        file.save().unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"{ not json");
    }
}
