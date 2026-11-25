//! Moving and clearing the cache, from Settings (SET-2). The cache is
//! Seura's own and rebuilt automatically, so clearing it, and removing the
//! old copy after a move, delete it directly (decision 0018). The folder can
//! be one the user chose, holding their own files too, so only Seura's own
//! entries in it, by name, are ever touched; a new place gets a folder of its
//! own (decision 0019).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

use crate::library::{AppSettings, Library, LibraryChanged};
use crate::locations;

/// The folder Seura makes for the cache in a place the user picks
/// (decision 0019).
pub const OWN_FOLDER: &str = "Seura Photos Cache";

/// A mark Seura leaves in a web view folder it couldn't remove while the
/// web view ran, so the next launch removes it: only a folder carrying it,
/// so a folder named `WebView` that isn't Seura's is never touched.
#[cfg(any(target_os = "windows", test))]
const REMOVE_MARK: &str = "Seura - remove at next launch";

/// The cache's place in a folder the user picked: a folder of its own
/// there, unless the picked one is already that folder (decision 0019).
pub fn own_folder(picked: &Path) -> PathBuf {
    if picked.file_name().is_some_and(|name| name == OWN_FOLDER) {
        picked.to_path_buf()
    } else {
        picked.join(OWN_FOLDER)
    }
}

/// How far a move has got: thumbnails copied, out of all of them.
#[derive(Clone, Debug, Serialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct CacheMoveProgress {
    pub done: u32,
    pub total: u32,
}

/// What Move… did.
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum CacheMoved {
    Moved {
        folder: String,
    },
    /// The cache is already there.
    AlreadyThere,
}

/// Deletes one of Seura's own entries in the cache folder, by name, and
/// nothing else: the one place moving and clearing delete (CLAUDE.md rule
/// 5; decision 0018). A name that isn't one of them is refused.
fn delete_entry(cache: &Path, name: &str) -> io::Result<()> {
    if !locations::cache_entries().any(|entry| entry == name) {
        return Err(io::Error::other(format!("{name} isn't part of the cache")));
    }
    let entry = cache.join(name);
    match fs::symlink_metadata(&entry) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
        // A link is removed as itself, never followed.
        Ok(data) if data.is_dir() => fs::remove_dir_all(&entry),
        Ok(_) => fs::remove_file(&entry),
    }
}

/// Removes the cache's own folder if it's empty and is one Seura made for
/// it: `Cache` in Seura's folder, or a folder of its own (decision 0019). A
/// folder the user picked before then is left, even when empty.
fn remove_if_empty(seura: &Path, cache: &Path) {
    let own =
        cache == seura.join("Cache") || cache.file_name().is_some_and(|name| name == OWN_FOLDER);
    if own {
        // Fails, as it should, when anything is still in it.
        let _ = fs::remove_dir(cache);
    }
}

/// Clears the library's entries from the cache: thumbnails, the index
/// snapshot and its README. The web view's folder is left: on Windows it's
/// in use while the app runs ([`mark_web_view`]).
fn clear_library_entries(cache: &Path) -> io::Result<()> {
    for name in seura_library::CACHE_ENTRIES {
        // A worker may be writing a thumbnail as the folder goes; once more
        // catches it.
        if delete_entry(cache, name).is_err() {
            delete_entry(cache, name)?;
        }
    }
    Ok(())
}

/// Marks the web view's folder in `cache` to be removed at the next
/// launch, before the web view starts; on Windows it's in use until then
/// (decision 0013). The Mac's web view keeps nothing on disk.
fn mark_web_view(cache: &Path) {
    #[cfg(target_os = "windows")]
    {
        let folder = locations::web_view_folder(cache);
        if folder.is_dir() {
            let _ = fs::write(folder.join(REMOVE_MARK), "");
        }
    }
    #[cfg(not(target_os = "windows"))]
    let _ = cache;
}

/// Clears the cache (SET-2, decision 0018): the thumbnail workers stop,
/// Seura's entries are deleted, and the workers start again, remaking every
/// thumbnail. The library's folders and settings aren't touched (SET-3).
pub fn clear<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let library = app.state::<Library>();
    let cache = library.cache();
    let mut result = Ok(());
    library.restart_cache(app, &cache, || {
        result = clear_library_entries(&cache).map_err(|error| error.to_string());
    });
    mark_web_view(&cache);
    library.save();
    library.queue_all(app);
    let _ = LibraryChanged.emit(app);
    result
}

/// Every file under `folder`, for counting and copying.
fn files_under(folder: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        match entry.file_type() {
            Ok(kind) if kind.is_dir() => files_under(&entry.path(), files),
            Ok(kind) if kind.is_file() => files.push(entry.path()),
            _ => {}
        }
    }
}

/// Copies the thumbnails from one cache to another, saying how far it has
/// got. The index snapshot and README are written afresh at the new place.
fn copy_thumbnails(from: &Path, to: &Path, progress: impl Fn(u32, u32)) -> io::Result<()> {
    let source = from.join(seura_library::THUMBS_FOLDER);
    let mut files = Vec::new();
    files_under(&source, &mut files);
    let total = files.len() as u32;
    progress(0, total);
    for (done, file) in files.iter().enumerate() {
        let relative = file.strip_prefix(&source).map_err(io::Error::other)?;
        let target = to.join(seura_library::THUMBS_FOLDER).join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(file, &target)?;
        let done = done as u32 + 1;
        if done.is_multiple_of(100) || done == total {
            progress(done, total);
        }
    }
    Ok(())
}

/// Moves the cache to a folder of its own in `picked` (SET-2; decision
/// 0019): copies the thumbnails, saying how far it has got, switches to the
/// new place, then deletes the old copy directly (decision 0018). If the
/// copy fails, what it copied is deleted and the cache stays where it was.
pub fn move_to<R: Runtime>(app: &AppHandle<R>, picked: &Path) -> Result<CacheMoved, String> {
    let library = app.state::<Library>();
    let seura = locations::seura_folder(app).map_err(|error| error.to_string())?;
    let old = library.cache();
    let new = own_folder(picked);
    if new == old {
        return Ok(CacheMoved::AlreadyThere);
    }
    if new.starts_with(&old) {
        return Err("the cache can't move into itself".into());
    }
    let made = !new.exists();
    fs::create_dir_all(&new).map_err(|error| error.to_string())?;
    let copied = copy_thumbnails(&old, &new, |done, total| {
        let _ = CacheMoveProgress { done, total }.emit(app);
    })
    .and_then(|()| {
        let settings = app.state::<AppSettings>();
        let mut file = settings.0.lock().expect("settings lock");
        file.settings.cache_folder = Some(new.clone());
        file.save().map_err(io::Error::other)
    });
    if let Err(problem) = copied {
        // Only what this move copied: the new place's thumbnails.
        let _ = delete_entry(&new, seura_library::THUMBS_FOLDER);
        if made {
            remove_if_empty(&seura, &new);
        }
        return Err(problem.to_string());
    }
    library.move_cache(app, &new);
    library.save();
    // The old copy goes. Not being able to remove it doesn't undo the move.
    let _ = clear_library_entries(&old);
    mark_web_view(&old);
    #[cfg(target_os = "windows")]
    remember_leftover(app, &old);
    remove_if_empty(&seura, &old);
    let _ = LibraryChanged.emit(app);
    Ok(CacheMoved::Moved {
        folder: new.display().to_string(),
    })
}

/// Remembers an old cache whose web view folder is still in use, so the
/// next launch removes it there (Windows).
#[cfg(target_os = "windows")]
fn remember_leftover<R: Runtime>(app: &AppHandle<R>, old: &Path) {
    if !locations::web_view_folder(old).is_dir() {
        return;
    }
    let settings = app.state::<AppSettings>();
    let mut file = settings.0.lock().expect("settings lock");
    if !file
        .settings
        .leftover_caches
        .iter()
        .any(|cache| cache == old)
    {
        file.settings.leftover_caches.push(old.to_path_buf());
        let _ = file.save();
    }
}

/// At launch, before the web view starts (Windows): removes the web view
/// folders Seura marked while they were in use, in the cache cleared and in
/// caches moved away from, and those old caches' own folders once empty.
/// Returns whether the list of old caches changed, to be saved.
#[cfg(any(target_os = "windows", test))]
pub fn tidy_at_launch(seura: &Path, cache: &Path, leftovers: &mut Vec<PathBuf>) -> bool {
    let marked = |folder: &Path| {
        folder
            .join(locations::WEB_VIEW_FOLDER)
            .join(REMOVE_MARK)
            .is_file()
    };
    if marked(cache) {
        let _ = delete_entry(cache, locations::WEB_VIEW_FOLDER);
    }
    let before = leftovers.len();
    leftovers.retain(|old| {
        if old != cache && marked(old) {
            let _ = delete_entry(old, locations::WEB_VIEW_FOLDER);
            remove_if_empty(seura, old);
        }
        // Kept only while its folder is still there to remove.
        old != cache && marked(old)
    });
    leftovers.len() != before
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cache in a folder the user chose, with their own files beside it.
    fn shared_cache() -> tempfile::TempDir {
        let folder = tempfile::tempdir().expect("temporary folder");
        let path = folder.path();
        fs::create_dir_all(path.join("thumbs/ab")).unwrap();
        fs::write(path.join("thumbs/ab/one.webp"), b"thumb").unwrap();
        fs::write(path.join("index.bin"), b"index").unwrap();
        fs::write(path.join("README.txt"), b"readme").unwrap();
        fs::create_dir_all(path.join("WebView/EBWebView")).unwrap();
        fs::write(path.join("holiday.mov"), b"theirs").unwrap();
        fs::create_dir_all(path.join("Work")).unwrap();
        fs::write(path.join("Work/notes.txt"), b"theirs").unwrap();
        folder
    }

    fn theirs_untouched(path: &Path) {
        assert_eq!(fs::read(path.join("holiday.mov")).unwrap(), b"theirs");
        assert_eq!(fs::read(path.join("Work/notes.txt")).unwrap(), b"theirs");
    }

    #[test]
    fn a_picked_folder_gets_a_folder_of_its_own_decision_0019() {
        let picked = Path::new("D:/Photos");
        assert_eq!(own_folder(picked), picked.join("Seura Photos Cache"));
        let own = picked.join("Seura Photos Cache");
        assert_eq!(own_folder(&own), own);
    }

    #[test]
    fn clearing_deletes_only_seura_s_own_entries_decision_0018() {
        let cache = shared_cache();
        clear_library_entries(cache.path()).unwrap();
        assert!(!cache.path().join("thumbs").exists());
        assert!(!cache.path().join("index.bin").exists());
        assert!(!cache.path().join("README.txt").exists());
        // In use while the app runs; marked for the next launch instead.
        assert!(cache.path().join("WebView/EBWebView").is_dir());
        theirs_untouched(cache.path());
    }

    #[test]
    fn only_the_cache_s_own_names_can_be_deleted() {
        let cache = shared_cache();
        assert!(delete_entry(cache.path(), "Work").is_err());
        assert!(delete_entry(cache.path(), "holiday.mov").is_err());
        assert!(delete_entry(cache.path(), "../holiday.mov").is_err());
        theirs_untouched(cache.path());
        // Gone already is fine.
        delete_entry(cache.path(), "index.bin").unwrap();
        delete_entry(cache.path(), "index.bin").unwrap();
    }

    #[test]
    fn copying_brings_every_thumbnail_and_says_how_far() {
        let old = shared_cache();
        let new = tempfile::tempdir().unwrap();
        let seen = std::cell::RefCell::new(Vec::new());
        copy_thumbnails(old.path(), new.path(), |done, total| {
            seen.borrow_mut().push((done, total));
        })
        .unwrap();
        assert_eq!(
            fs::read(new.path().join("thumbs/ab/one.webp")).unwrap(),
            b"thumb"
        );
        assert_eq!(*seen.borrow(), [(0, 1), (1, 1)]);
        // Nothing of the user's comes along.
        assert!(!new.path().join("holiday.mov").exists());
        assert!(!new.path().join("Work").exists());
    }

    #[test]
    fn only_a_folder_seura_made_is_removed_once_empty() {
        let seura = tempfile::tempdir().unwrap();
        let own = seura.path().join("elsewhere").join(OWN_FOLDER);
        let picked = seura.path().join("picked");
        let default = seura.path().join("Cache");
        for folder in [&own, &picked, &default] {
            fs::create_dir_all(folder).unwrap();
            remove_if_empty(seura.path(), folder);
        }
        assert!(!own.exists());
        assert!(!default.exists());
        // A folder the user picked before decision 0019 stays.
        assert!(picked.is_dir());
        // Never one with something in it.
        fs::create_dir_all(&own).unwrap();
        fs::write(own.join("kept.txt"), b"").unwrap();
        remove_if_empty(seura.path(), &own);
        assert!(own.join("kept.txt").is_file());
    }

    #[test]
    fn launch_removes_only_marked_web_view_folders() {
        let seura = tempfile::tempdir().unwrap();
        let cache = shared_cache();
        let old = seura.path().join("old").join(OWN_FOLDER);
        fs::create_dir_all(old.join("WebView")).unwrap();
        fs::write(old.join("WebView").join(REMOVE_MARK), b"").unwrap();
        let unmarked = shared_cache();
        let mut leftovers = vec![old.clone(), unmarked.path().to_path_buf()];

        // The current cache isn't marked: its web view folder stays.
        assert!(tidy_at_launch(seura.path(), cache.path(), &mut leftovers));
        assert!(cache.path().join("WebView/EBWebView").is_dir());
        // The marked old one goes, with its emptied folder.
        assert!(!old.exists());
        // One that isn't marked is never touched, and is forgotten.
        assert!(unmarked.path().join("WebView/EBWebView").is_dir());
        assert!(leftovers.is_empty());

        // Once cleared, the current cache's web view folder goes too.
        fs::write(cache.path().join("WebView").join(REMOVE_MARK), b"").unwrap();
        tidy_at_launch(seura.path(), cache.path(), &mut leftovers);
        assert!(!cache.path().join("WebView").exists());
        theirs_untouched(cache.path());
    }
}
