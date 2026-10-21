//! The first run's Welcome screen (SET-1, SET-4): which folders hold the
//! user's photos, where Seura keeps its own files, and whether to check for
//! updates.
//!
//! The UI only ever names folders the core offered it, by id: the Pictures
//! folder, or ones the user picked in the system's folder dialog. It never
//! sends a path of its own (decision 0009).

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

use crate::library::{AppSettings, Library, LibraryChanged};
use crate::locations;
use crate::synced::sync_service;

/// What the Welcome screen knows, and the folders it has offered.
#[derive(Default)]
pub struct Welcome {
    needed: AtomicBool,
    folders: Mutex<Vec<PathBuf>>,
    caches: Mutex<Vec<PathBuf>>,
}

/// A folder the user can choose, by id.
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Choice {
    pub id: u32,
    pub name: String,
    pub path: String,
    /// The service that syncs it, if any (for a cache location).
    pub synced_to: Option<String>,
}

#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WelcomeInfo {
    /// Whether this is a first run, so the Welcome screen is shown.
    pub needed: bool,
    /// Where Seura keeps its settings and cache (decision 0015).
    pub seura_folder: String,
    /// Folders suggested to start with: the Pictures folder, if there is one.
    pub suggested: Vec<Choice>,
    /// The service the default cache location is synced to, if any.
    pub cache_synced_to: Option<String>,
}

/// The developer switch: `SEURA_WELCOME=1` shows the Welcome screen even
/// when it isn't a first run, to check it. Nothing is saved unless the user
/// gets started.
pub const WELCOME_SWITCH: &str = "SEURA_WELCOME";

pub fn forced() -> bool {
    std::env::var(WELCOME_SWITCH).as_deref() == Ok("1")
}

impl Welcome {
    pub fn new(needed: bool) -> Welcome {
        Welcome {
            needed: AtomicBool::new(needed),
            ..Welcome::default()
        }
    }

    fn offer(list: &Mutex<Vec<PathBuf>>, path: PathBuf) -> Choice {
        let mut list = list.lock().expect("welcome lock");
        let id = match list.iter().position(|known| *known == path) {
            Some(id) => id,
            None => {
                list.push(path.clone());
                list.len() - 1
            }
        };
        Choice {
            id: id as u32,
            name: name(&path),
            path: path.display().to_string(),
            synced_to: sync_service(&path),
        }
    }

    fn offered(list: &Mutex<Vec<PathBuf>>, id: u32) -> Result<PathBuf, String> {
        list.lock()
            .expect("welcome lock")
            .get(id as usize)
            .cloned()
            .ok_or_else(|| "that folder wasn't offered".to_string())
    }
}

pub fn info<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<WelcomeInfo> {
    let welcome = app.state::<Welcome>();
    let seura = locations::seura_folder(app)?;
    let suggested = app
        .path()
        .picture_dir()
        .ok()
        .filter(|pictures| pictures.is_dir())
        .map(|pictures| Welcome::offer(&welcome.folders, pictures))
        .into_iter()
        .collect();
    let settings = app.state::<AppSettings>();
    let cache =
        locations::cache_folder(&seura, &settings.0.lock().expect("settings lock").settings);
    Ok(WelcomeInfo {
        needed: welcome.needed.load(Ordering::Relaxed),
        seura_folder: seura.display().to_string(),
        suggested,
        cache_synced_to: sync_service(&cache),
    })
}

/// Offers a folder the user picked, to add or to hold the cache.
pub fn offer_folder<R: Runtime>(app: &AppHandle<R>, path: PathBuf, for_cache: bool) -> Choice {
    let welcome = app.state::<Welcome>();
    Welcome::offer(
        if for_cache {
            &welcome.caches
        } else {
            &welcome.folders
        },
        path,
    )
}

/// Get started: adds the chosen folders, saves the settings (which ends the
/// first run), and starts scanning (LIB-3).
pub fn finish<R: Runtime>(
    app: &AppHandle<R>,
    folders: &[u32],
    check_for_updates: bool,
    cache: Option<u32>,
) -> Result<(), String> {
    let welcome = app.state::<Welcome>();
    let library = app.state::<Library>();
    let folders = folders
        .iter()
        .map(|id| Welcome::offered(&welcome.folders, *id))
        .collect::<Result<Vec<_>, _>>()?;
    let cache = cache
        .map(|id| Welcome::offered(&welcome.caches, id))
        .transpose()?;

    let settings = app.state::<AppSettings>();
    let mut settings = settings.0.lock().expect("settings lock");
    if let Some(cache) = &cache {
        settings.settings.cache_folder = Some(cache.clone());
        library.move_cache(app, cache);
    }
    {
        let mut index = library.index.write().expect("index lock");
        for folder in &folders {
            index.add_root(folder).map_err(|error| error.to_string())?;
        }
        settings.settings.library_folders = index.roots().to_vec();
    }
    settings.settings.check_for_updates = check_for_updates;
    settings.save().map_err(|error| error.to_string())?;
    drop(settings);
    welcome.needed.store(false, Ordering::Relaxed);

    let handle = app.clone();
    std::thread::spawn(move || {
        handle.state::<Library>().rescan(&handle);
        let _ = LibraryChanged.emit(&handle);
    });
    Ok(())
}

/// A folder's display name: its last component.
fn name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offered_folders_keep_their_ids() {
        let list = Mutex::new(Vec::new());
        let a = Welcome::offer(&list, PathBuf::from("C:/Users/Ann/Pictures"));
        let b = Welcome::offer(&list, PathBuf::from("D:/Photos"));
        let again = Welcome::offer(&list, PathBuf::from("C:/Users/Ann/Pictures"));
        assert_eq!((a.id, b.id, again.id), (0, 1, 0));
        assert_eq!(a.name, "Pictures");
        assert_eq!(name(Path::new("D:/Photos")), "Photos");
        assert_eq!(
            Welcome::offered(&list, 1).unwrap(),
            PathBuf::from("D:/Photos")
        );
    }

    #[test]
    fn a_folder_that_wasn_t_offered_is_refused() {
        let list = Mutex::new(vec![PathBuf::from("D:/Photos")]);
        assert!(Welcome::offered(&list, 7).is_err());
    }
}
