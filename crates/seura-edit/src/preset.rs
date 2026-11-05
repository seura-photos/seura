//! The Presets folder (PRE-3, PRE-5; edit format, "Presets"): one file per
//! preset, `<name>.seura-preset.json`. The file name is the preset's name as
//! shown, so a file renamed by hand shows its new name; Seura keeps the
//! `name` field the same when it saves or renames, and never rewrites a file
//! just to match. A file that can't be read is listed, never rewritten, and
//! only ever moved to the Trash (CLAUDE.md rules 4 and 5).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use time::OffsetDateTime;
use unicode_normalization::UnicodeNormalization;

use crate::document::{EditDocument, Kind, tag_key};
use crate::file::{EditFile, LoadError, SaveError, Writer};

/// What every preset's file name ends with.
pub const PRESET_SUFFIX: &str = ".seura-preset.json";

/// The longest name kept, in characters: well inside both systems' limit
/// for a file name, with the suffix.
const LONGEST_NAME: usize = 100;

/// Characters Windows doesn't allow in a file name; `/` and `:` are the Mac's.
const NOT_IN_NAMES: [char; 9] = ['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

/// Names Windows keeps for devices, even with an extension after them.
const DEVICE_NAMES: [&str; 4] = ["CON", "PRN", "AUX", "NUL"];

/// A name made safe for a file name on both systems, or `None` if nothing
/// is left of it. Characters a file name can't hold become `-`; leading and
/// trailing spaces and dots go (a leading dot hides a file on the Mac, and
/// Windows drops trailing ones); a device name such as `CON` gets a `_`.
pub fn clean_preset_name(name: &str) -> Option<String> {
    let replaced: String = name
        .nfc()
        .map(|c| {
            if c.is_control() || NOT_IN_NAMES.contains(&c) {
                '-'
            } else {
                c
            }
        })
        .collect();
    let trimmed = trim(&replaced);
    let mut cleaned = trim(&trimmed.chars().take(LONGEST_NAME).collect::<String>());
    if cleaned.is_empty() {
        return None;
    }
    let first = cleaned.split('.').next().unwrap_or_default();
    if is_device_name(first.trim_end()) {
        cleaned.insert(first.len(), '_');
    }
    Some(cleaned)
}

fn trim(name: &str) -> String {
    name.trim_matches(|c: char| c.is_whitespace() || c == '.')
        .to_string()
}

fn is_device_name(name: &str) -> bool {
    let upper = name.to_uppercase();
    if DEVICE_NAMES.contains(&upper.as_str()) {
        return true;
    }
    let numbered = |prefix: &str| {
        upper.strip_prefix(prefix).is_some_and(|rest| {
            let mut chars = rest.chars();
            matches!(
                (chars.next(), chars.next()),
                (Some('0'..='9' | '¹' | '²' | '³'), None)
            )
        })
    };
    numbered("COM") || numbered("LPT")
}

/// What names are compared by: as tags are, without letter case, since both
/// systems' usual drives ignore it.
fn key(name: &str) -> String {
    tag_key(name)
}

/// One file in the Presets folder.
#[derive(Debug)]
pub struct Preset {
    /// Its file name without the suffix, as shown.
    pub name: String,
    pub path: PathBuf,
    /// The preset, or why it can't be read.
    pub document: Result<EditDocument, String>,
}

/// Why a preset wasn't saved, renamed, or removed. In every case the
/// folder is as it was.
#[derive(Debug, thiserror::Error)]
pub enum PresetError {
    #[error("the name has nothing a file name can hold")]
    InvalidName,
    /// Another preset has this name, ignoring letter case.
    #[error("a preset named {name} already exists")]
    NameInUse { name: String, readable: bool },
    #[error("the preset is no longer there")]
    NotFound,
    /// The file can't be read, so it's never rewritten (CLAUDE.md rule 4).
    #[error("the preset can't be read: {0}")]
    Unreadable(String),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Load(#[from] LoadError),
    #[error(transparent)]
    Save(#[from] SaveError),
}

/// The Presets folder, which may not exist until the first preset is saved.
#[derive(Clone, Debug)]
pub struct PresetsFolder {
    path: PathBuf,
}

impl PresetsFolder {
    pub fn new(path: PathBuf) -> PresetsFolder {
        PresetsFolder { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Every preset file, by name without letter case. Other files are left
    /// out, and a missing folder has none.
    pub fn list(&self) -> io::Result<Vec<Preset>> {
        let entries = match fs::read_dir(&self.path) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        let mut presets = Vec::new();
        for entry in entries {
            let entry = entry?;
            let Some(name) = entry.file_name().to_str().and_then(preset_name) else {
                continue;
            };
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let document = read(&path);
            presets.push(Preset {
                name,
                path,
                document,
            });
        }
        presets.sort_by(|a, b| key(&a.name).cmp(&key(&b.name)).then(a.name.cmp(&b.name)));
        Ok(presets)
    }

    /// Saves `preset` as `name`, cleaned for a file name, and returns the
    /// name it was saved as. A preset already named so, ignoring letter
    /// case, is replaced only with `replace`, keeping its name, and never if
    /// it can't be read. Atomic, as every save is (CLAUDE.md rule 3).
    pub fn save(
        &self,
        name: &str,
        mut preset: EditDocument,
        replace: bool,
        writer: Writer,
        at: OffsetDateTime,
    ) -> Result<String, PresetError> {
        let name = clean_preset_name(name).ok_or(PresetError::InvalidName)?;
        let existing = self
            .list()?
            .into_iter()
            .find(|other| key(&other.name) == key(&name));
        let (name, mut file) = match existing {
            Some(other) if !replace => {
                return Err(PresetError::NameInUse {
                    name: other.name,
                    readable: other.document.is_ok(),
                });
            }
            Some(Preset {
                document: Err(problem),
                ..
            }) => return Err(PresetError::Unreadable(problem)),
            Some(other) => {
                // Loaded, so the save refuses if it changed since.
                let file = EditFile::load(&other.path)?.ok_or(PresetError::NotFound)?;
                (other.name, file)
            }
            None => {
                fs::create_dir_all(&self.path)?;
                let path = self.path.join(format!("{name}{PRESET_SUFFIX}"));
                // A new file: the save refuses if one appears there first.
                let file = EditFile::create(&path, EditDocument::new_preset(&name));
                (name, file)
            }
        };
        preset
            .set_preset_name(&name)
            .map_err(|error| PresetError::Save(error.into()))?;
        file.document = preset;
        file.save(writer, at)?;
        Ok(name)
    }

    /// Renames the preset `from` to `to`, cleaned for a file name, and
    /// returns its new name. Another preset's name is refused, so nothing
    /// is replaced; changing only the letter case is a rename.
    pub fn rename(
        &self,
        from: &str,
        to: &str,
        writer: Writer,
        at: OffsetDateTime,
    ) -> Result<String, PresetError> {
        let to = clean_preset_name(to).ok_or(PresetError::InvalidName)?;
        let presets = self.list()?;
        let preset = presets
            .iter()
            .find(|preset| preset.name == from)
            .ok_or(PresetError::NotFound)?;
        if let Err(problem) = &preset.document {
            return Err(PresetError::Unreadable(problem.clone()));
        }
        if let Some(other) = presets
            .iter()
            .find(|other| other.path != preset.path && key(&other.name) == key(&to))
        {
            return Err(PresetError::NameInUse {
                name: other.name.clone(),
                readable: other.document.is_ok(),
            });
        }
        if preset.name == to {
            return Ok(to);
        }
        // The file is moved, not copied and removed, so nothing is deleted.
        // No other preset has the name, as checked just now; on Windows and
        // the Mac, a rename that changes only the letter case is the same file.
        let path = self.path.join(format!("{to}{PRESET_SUFFIX}"));
        fs::rename(&preset.path, &path)?;
        // The name shown is the file's, so the rename has happened. Its
        // `name` field follows when it can; a failure here leaves only the
        // field behind, which no reader relies on.
        if let Ok(Some(mut file)) = EditFile::load(&path)
            && file.document.set_preset_name(&to).is_ok()
        {
            let _ = file.save(writer, at);
        }
        Ok(to)
    }

    /// Moves the preset `name` to the Trash or Recycle Bin with `trash`,
    /// readable or not; it's never deleted (CLAUDE.md rule 5).
    pub fn remove(
        &self,
        name: &str,
        trash: impl FnOnce(&Path) -> io::Result<()>,
    ) -> Result<(), PresetError> {
        let preset = self
            .list()?
            .into_iter()
            .find(|preset| preset.name == name)
            .ok_or(PresetError::NotFound)?;
        trash(&preset.path)?;
        Ok(())
    }
}

/// The preset's name from its file name, if it is a preset's: NFC, since the
/// Mac may give names decomposed.
fn preset_name(file_name: &str) -> Option<String> {
    let split = file_name.len().checked_sub(PRESET_SUFFIX.len())?;
    let suffix = file_name.get(split..)?;
    if split == 0 || !suffix.eq_ignore_ascii_case(PRESET_SUFFIX) {
        return None;
    }
    Some(file_name[..split].nfc().collect())
}

fn read(path: &Path) -> Result<EditDocument, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    let document = EditDocument::parse(&bytes).map_err(|error| error.to_string())?;
    match document.kind() {
        Kind::Preset => Ok(document),
        Kind::Photo => Err("it's a photo's edit file, not a preset".into()),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::{Action, Amount, CoreOperation, Edit};

    const WRITER: Writer = Writer {
        app: "Seura Photos",
        version: "test",
    };

    fn contrast(amount: f64) -> EditDocument {
        let mut preset = EditDocument::new_preset("Test");
        let action = Action {
            label: String::new(),
            edits: vec![Edit::Set(CoreOperation::Contrast(Amount { amount }))],
        };
        preset.apply(&action, OffsetDateTime::UNIX_EPOCH).unwrap();
        preset
    }

    fn names(folder: &PresetsFolder) -> Vec<String> {
        folder
            .list()
            .unwrap()
            .into_iter()
            .map(|preset| preset.name)
            .collect()
    }

    fn files(folder: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        names
    }

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }

    #[test]
    fn names_are_cleaned_for_both_systems() {
        assert_eq!(
            clean_preset_name("Warm Vintage").as_deref(),
            Some("Warm Vintage")
        );
        assert_eq!(clean_preset_name("  Warm  ").as_deref(), Some("Warm"));
        assert_eq!(
            clean_preset_name("1/2 stop: brighter?").as_deref(),
            Some("1-2 stop- brighter-")
        );
        assert_eq!(clean_preset_name("Tab\there").as_deref(), Some("Tab-here"));
        // Hidden on the Mac, or dropped by Windows.
        assert_eq!(clean_preset_name(".hidden").as_deref(), Some("hidden"));
        assert_eq!(clean_preset_name("Warm...").as_deref(), Some("Warm"));
        assert_eq!(clean_preset_name("  . . "), None);
        assert_eq!(clean_preset_name(""), None);
        // Windows' device names, even before a dot.
        assert_eq!(clean_preset_name("con").as_deref(), Some("con_"));
        assert_eq!(clean_preset_name("COM1").as_deref(), Some("COM1_"));
        assert_eq!(clean_preset_name("nul.warm").as_deref(), Some("nul_.warm"));
        assert_eq!(clean_preset_name("Console").as_deref(), Some("Console"));
        assert_eq!(clean_preset_name("COM10").as_deref(), Some("COM10"));
        // Long names are cut, by character, not byte.
        let long = "é".repeat(150);
        assert_eq!(
            clean_preset_name(&long).unwrap().chars().count(),
            LONGEST_NAME
        );
        // Stored composed, as the Mac may give it decomposed.
        assert_eq!(clean_preset_name("Cafe\u{301}").as_deref(), Some("Café"));
    }

    #[test]
    fn a_saved_preset_is_listed_and_reads_back_as_one() {
        let dir = tempfile::tempdir().unwrap();
        let folder = PresetsFolder::new(dir.path().join("Presets"));
        assert!(
            folder.list().unwrap().is_empty(),
            "a missing folder has none"
        );

        let name = folder
            .save("Warm: soft", contrast(20.0), false, WRITER, now())
            .unwrap();
        assert_eq!(name, "Warm- soft");
        assert_eq!(files(folder.path()), ["Warm- soft.seura-preset.json"]);

        let presets = folder.list().unwrap();
        let document = presets[0].document.as_ref().unwrap();
        assert_eq!(document.kind(), Kind::Preset);
        assert_eq!(document.preset_name(), Some("Warm- soft"));
        assert!(document.json().contains_key("writer"));
        assert!(!document.json().contains_key("history"));
    }

    #[test]
    fn a_name_in_use_asks_first_and_replacing_keeps_its_name() {
        let dir = tempfile::tempdir().unwrap();
        let folder = PresetsFolder::new(dir.path().to_path_buf());
        folder
            .save("Warm", contrast(20.0), false, WRITER, now())
            .unwrap();
        let before = fs::read(dir.path().join("Warm.seura-preset.json")).unwrap();

        let refused = folder.save("warm", contrast(40.0), false, WRITER, now());
        assert!(matches!(
            refused,
            Err(PresetError::NameInUse { ref name, readable: true }) if name == "Warm"
        ));
        assert_eq!(
            fs::read(dir.path().join("Warm.seura-preset.json")).unwrap(),
            before
        );

        let name = folder
            .save("warm", contrast(40.0), true, WRITER, now())
            .unwrap();
        assert_eq!(name, "Warm");
        assert_eq!(names(&folder), ["Warm"]);
        let document = folder.list().unwrap().remove(0).document.unwrap();
        assert_eq!(
            document.core(crate::CoreOpId::Contrast),
            Some(CoreOperation::Contrast(Amount { amount: 40.0 }))
        );
    }

    #[test]
    fn a_file_that_cant_be_read_is_listed_and_never_rewritten() {
        let dir = tempfile::tempdir().unwrap();
        let folder = PresetsFolder::new(dir.path().to_path_buf());
        let broken = dir.path().join("Broken.seura-preset.json");
        fs::write(&broken, b"{ not json").unwrap();
        // Not presets: another kind of file, and a temporary one.
        fs::write(dir.path().join("notes.txt"), b"hi").unwrap();
        fs::write(dir.path().join("Warm.seura-preset.json.1-0.tmp"), b"{}").unwrap();

        let presets = folder.list().unwrap();
        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0].name, "Broken");
        assert!(presets[0].document.is_err());

        for replace in [false, true] {
            let result = folder.save("broken", contrast(20.0), replace, WRITER, now());
            assert!(result.is_err());
        }
        assert!(matches!(
            folder.rename("Broken", "Fixed", WRITER, now()),
            Err(PresetError::Unreadable(_))
        ));
        assert_eq!(fs::read(&broken).unwrap(), b"{ not json");
    }

    #[test]
    fn a_photo_s_edit_file_named_as_a_preset_isnt_one() {
        let dir = tempfile::tempdir().unwrap();
        let folder = PresetsFolder::new(dir.path().to_path_buf());
        let photo = EditDocument::new_for_photo(&crate::PhotoIdentity {
            file: "IMG_1.jpg".into(),
            size: 1,
            sha256: None,
            width: None,
            height: None,
        });
        fs::write(
            dir.path().join("Photo.seura-preset.json"),
            photo.to_bytes().unwrap(),
        )
        .unwrap();
        assert!(folder.list().unwrap()[0].document.is_err());
    }

    #[test]
    fn renaming_moves_the_file_and_its_name_follows() {
        let dir = tempfile::tempdir().unwrap();
        let folder = PresetsFolder::new(dir.path().to_path_buf());
        folder
            .save("Warm", contrast(20.0), false, WRITER, now())
            .unwrap();
        folder
            .save("Cool", contrast(-20.0), false, WRITER, now())
            .unwrap();

        let name = folder.rename("Warm", "Sunny?", WRITER, now()).unwrap();
        assert_eq!(name, "Sunny-");
        assert_eq!(
            files(dir.path()),
            ["Cool.seura-preset.json", "Sunny-.seura-preset.json"]
        );
        let sunny = folder.list().unwrap().remove(1).document.unwrap();
        assert_eq!(sunny.preset_name(), Some("Sunny-"));

        // Only the letter case.
        assert_eq!(
            folder.rename("Sunny-", "sunny-", WRITER, now()).unwrap(),
            "sunny-"
        );
        assert_eq!(names(&folder), ["Cool", "sunny-"]);

        // Another preset's name: refused, and both stay.
        assert!(matches!(
            folder.rename("sunny-", "COOL", WRITER, now()),
            Err(PresetError::NameInUse { ref name, .. }) if name == "Cool"
        ));
        assert!(matches!(
            folder.rename("Gone", "Other", WRITER, now()),
            Err(PresetError::NotFound)
        ));
        assert!(matches!(
            folder.rename("Cool", " .. ", WRITER, now()),
            Err(PresetError::InvalidName)
        ));
        assert_eq!(names(&folder), ["Cool", "sunny-"]);
    }

    #[test]
    fn a_file_renamed_by_hand_shows_its_new_name_and_isnt_rewritten() {
        let dir = tempfile::tempdir().unwrap();
        let folder = PresetsFolder::new(dir.path().to_path_buf());
        folder
            .save("Warm", contrast(20.0), false, WRITER, now())
            .unwrap();
        let moved = dir.path().join("Golden.seura-preset.json");
        fs::rename(dir.path().join("Warm.seura-preset.json"), &moved).unwrap();
        let before = fs::read(&moved).unwrap();

        assert_eq!(names(&folder), ["Golden"]);
        assert_eq!(fs::read(&moved).unwrap(), before);
    }

    #[test]
    fn removing_goes_through_the_trash_readable_or_not() {
        let dir = tempfile::tempdir().unwrap();
        let folder = PresetsFolder::new(dir.path().to_path_buf());
        folder
            .save("Warm", contrast(20.0), false, WRITER, now())
            .unwrap();
        fs::write(dir.path().join("Broken.seura-preset.json"), b"oops").unwrap();
        let trashed = RefCell::new(Vec::new());
        let trash = |path: &Path| {
            trashed.borrow_mut().push(path.to_path_buf());
            Ok(())
        };

        folder.remove("Warm", trash).unwrap();
        folder.remove("Broken", trash).unwrap();
        assert!(matches!(
            folder.remove("Gone", trash),
            Err(PresetError::NotFound)
        ));
        assert_eq!(
            *trashed.borrow(),
            [
                dir.path().join("Warm.seura-preset.json"),
                dir.path().join("Broken.seura-preset.json")
            ]
        );
        // The trash function moved nothing, so both are still there:
        // removing never deletes.
        assert_eq!(names(&folder), ["Broken", "Warm"]);
    }
}
