//! Where Seura keeps its own files (CLAUDE.md rule 2; decisions 0013 and
//! 0015), and the settings it starts with.

use std::{
    fs,
    path::{Path, PathBuf},
};

use seura_library::{Index, Settings, SettingsFile};
use tauri::{AppHandle, Manager, Runtime};

/// The developer switch that puts Seura's folder elsewhere, for measuring
/// and testing without touching the real one (M2 slice 10).
pub const DATA_SWITCH: &str = "SEURA_DATA";

/// `Seura Photos` in the user's home folder: settings, presets, and by
/// default the cache (decision 0015). Not Documents, which is often synced.
/// On Windows the home folder comes from `FOLDERID_Profile`. With
/// `SEURA_DATA`, the folder it names instead.
pub fn seura_folder<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<PathBuf> {
    if let Some(folder) = data_override() {
        return Ok(folder);
    }
    Ok(app.path().home_dir()?.join("Seura Photos"))
}

/// The folder `SEURA_DATA` names, if it's set.
fn data_override() -> Option<PathBuf> {
    std::env::var_os(DATA_SWITCH)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// Where M1 builds kept everything, read once to carry the library over.
fn m1_cache_folder<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    // A folder of its own carries nothing over.
    if data_override().is_some() {
        return None;
    }
    Some(
        app.path()
            .document_dir()
            .ok()?
            .join("Seura Photos")
            .join("Cache"),
    )
}

/// The cache folder: the user's choice, or the default in Seura's folder.
pub fn cache_folder(seura: &Path, settings: &Settings) -> PathBuf {
    settings
        .cache_folder
        .clone()
        .unwrap_or_else(|| default_cache(seura))
}

/// The cache's default place: `Seura Photos Cache` in Seura's folder, the
/// same name as a cache anywhere else (decision 0019).
pub fn default_cache(seura: &Path) -> PathBuf {
    seura.join(crate::cache::OWN_FOLDER)
}

/// The default cache's name before decision 0019's amendment.
pub const OLD_DEFAULT_CACHE: &str = "Cache";

/// Renames a default cache from before decision 0019's amendment, `Cache`
/// in Seura's folder, to the new name, once. Before the web view starts, so
/// nothing in it is in use. Nothing is copied or deleted. If the rename
/// fails, the old folder stays the cache, recorded as the user's choice so
/// it isn't rebuilt elsewhere. Returns whether the settings changed.
pub fn rename_old_default_cache(seura: &Path, settings: &mut Settings) -> bool {
    let old = seura.join(OLD_DEFAULT_CACHE);
    let new = default_cache(seura);
    if settings.cache_folder.is_some() || !old.is_dir() || new.exists() {
        return false;
    }
    match fs::rename(&old, &new) {
        Ok(()) => false,
        Err(error) => {
            eprintln!(
                "The cache couldn't be renamed to {}, so it stays at {}: {error}",
                new.display(),
                old.display()
            );
            settings.cache_folder = Some(old);
            true
        }
    }
}

/// The web view's own data folder, inside the cache so it's visible, movable
/// with the cache, and safe to delete. Only WebView2 (Windows) honors it;
/// WKWebView (macOS) runs with a non-persistent store instead (decision 0013).
#[cfg(any(target_os = "windows", test))]
pub fn web_view_folder(cache: &Path) -> PathBuf {
    cache.join(WEB_VIEW_FOLDER)
}

/// The web view's folder's name, in the cache.
pub const WEB_VIEW_FOLDER: &str = "WebView";

/// What Seura keeps in the cache folder, by name: the library's entries and
/// the web view's folder. Only these are the cache, since the folder may be
/// one the user chose that holds other things too (decision 0018).
pub fn cache_entries() -> impl Iterator<Item = &'static str> {
    seura_library::CACHE_ENTRIES
        .iter()
        .copied()
        .chain([WEB_VIEW_FOLDER])
}

/// How the settings were found at launch.
#[derive(Debug, PartialEq, Eq)]
pub enum Origin {
    Loaded,
    /// The file exists but can't be read; it's left alone.
    Unreadable,
    /// No settings yet, and M1's library was carried over from Documents.
    CarriedOver,
    /// No settings and nothing to carry over: the Welcome screen's case.
    FirstRun,
}

pub fn load_settings<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<(SettingsFile, Origin)> {
    Ok(settings_at(
        &seura_folder(app)?,
        m1_cache_folder(app).as_deref(),
    ))
}

/// The settings in `seura`, or, on the first run of a build with them, the
/// library folders from an M1 build's index snapshot in `old_cache`. The old
/// folder is only read (decision 0015).
fn settings_at(seura: &Path, old_cache: Option<&Path>) -> (SettingsFile, Origin) {
    match SettingsFile::load(seura) {
        Ok(Some(file)) => (file, Origin::Loaded),
        Err(error) => (
            SettingsFile::unreadable(seura, error.to_string()),
            Origin::Unreadable,
        ),
        Ok(None) => {
            let folders = old_cache
                .and_then(Index::load_snapshot)
                .map(|index| index.roots().to_vec())
                .unwrap_or_default();
            if folders.is_empty() {
                return (
                    SettingsFile::new(seura, Settings::default()),
                    Origin::FirstRun,
                );
            }
            let file = SettingsFile::new(
                seura,
                Settings {
                    library_folders: folders,
                    ..Settings::default()
                },
            );
            // Written now, so the carry-over happens once.
            match file.save() {
                Ok(()) => (file, Origin::CarriedOver),
                Err(error) => (
                    SettingsFile::unreadable(seura, error.to_string()),
                    Origin::Unreadable,
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use seura_library::SETTINGS_FILE;

    use super::*;

    #[test]
    fn web_view_data_lives_inside_the_cache() {
        let cache = Path::new("cache");
        assert!(web_view_folder(cache).starts_with(cache));
    }

    #[test]
    fn the_cache_is_in_seura_s_folder_unless_moved() {
        let seura = Path::new("home/Seura Photos");
        let mut settings = Settings::default();
        assert_eq!(
            cache_folder(seura, &settings),
            seura.join("Seura Photos Cache")
        );
        settings.cache_folder = Some(PathBuf::from("D:/Cache"));
        assert_eq!(cache_folder(seura, &settings), PathBuf::from("D:/Cache"));
    }

    #[test]
    fn an_old_default_cache_is_renamed_once_with_everything_in_it() {
        let seura = tempfile::tempdir().unwrap();
        let old = seura.path().join("Cache");
        fs::create_dir_all(old.join("thumbs/ab")).unwrap();
        fs::write(old.join("thumbs/ab/one.webp"), b"thumb").unwrap();
        fs::write(old.join("index.bin"), b"index").unwrap();
        let mut settings = Settings::default();

        assert!(!rename_old_default_cache(seura.path(), &mut settings));
        let new = cache_folder(seura.path(), &settings);
        assert_eq!(new, seura.path().join("Seura Photos Cache"));
        assert_eq!(fs::read(new.join("thumbs/ab/one.webp")).unwrap(), b"thumb");
        assert_eq!(fs::read(new.join("index.bin")).unwrap(), b"index");
        assert!(!old.exists());
        assert_eq!(settings.cache_folder, None);

        // Never over a cache already at the new name.
        fs::create_dir_all(&old).unwrap();
        fs::write(old.join("index.bin"), b"older").unwrap();
        assert!(!rename_old_default_cache(seura.path(), &mut settings));
        assert_eq!(fs::read(new.join("index.bin")).unwrap(), b"index");
        assert!(old.join("index.bin").is_file());
    }

    #[test]
    fn a_cache_the_user_chose_is_never_renamed() {
        let seura = tempfile::tempdir().unwrap();
        let old = seura.path().join("Cache");
        fs::create_dir_all(&old).unwrap();
        let mut settings = Settings {
            cache_folder: Some(PathBuf::from("D:/Elsewhere")),
            ..Settings::default()
        };
        assert!(!rename_old_default_cache(seura.path(), &mut settings));
        assert!(old.is_dir());
        assert!(!seura.path().join("Seura Photos Cache").exists());
    }

    #[test]
    fn nothing_to_carry_over_is_a_first_run() {
        let home = tempfile::tempdir().unwrap();
        let seura = home.path().join("Seura Photos");
        let (file, origin) = settings_at(&seura, Some(&home.path().join("old")));
        assert_eq!(origin, Origin::FirstRun);
        assert!(file.settings.library_folders.is_empty());
        // Nothing is written until the user does something.
        assert!(!seura.exists());
    }

    #[test]
    fn m1_s_library_is_carried_over_once_and_the_old_folder_left_alone() {
        let home = tempfile::tempdir().unwrap();
        let old = home.path().join("Documents/Seura Photos/Cache");
        let photos = home.path().join("Pictures");
        fs::create_dir_all(&photos).unwrap();
        let mut index = Index::new();
        index.add_root(&photos).unwrap();
        index.save_snapshot(&old).unwrap();
        let old_snapshot = fs::read(old.join(seura_library::SNAPSHOT_FILE)).unwrap();
        let seura = home.path().join("Seura Photos");

        let (file, origin) = settings_at(&seura, Some(&old));
        assert_eq!(origin, Origin::CarriedOver);
        assert_eq!(file.settings.library_folders, index.roots());
        assert!(seura.join(SETTINGS_FILE).is_file());
        assert_eq!(
            fs::read(old.join(seura_library::SNAPSHOT_FILE)).unwrap(),
            old_snapshot
        );

        // The next launch reads the new settings.
        let (_, origin) = settings_at(&seura, Some(&old));
        assert_eq!(origin, Origin::Loaded);
    }

    #[test]
    fn unreadable_settings_are_left_alone() {
        let home = tempfile::tempdir().unwrap();
        let seura = home.path().join("Seura Photos");
        fs::create_dir_all(&seura).unwrap();
        fs::write(seura.join(SETTINGS_FILE), b"oops").unwrap();
        let (file, origin) = settings_at(&seura, None);
        assert_eq!(origin, Origin::Unreadable);
        assert!(file.problem.is_some());
        file.save().unwrap();
        assert_eq!(fs::read(seura.join(SETTINGS_FILE)).unwrap(), b"oops");
    }
}
