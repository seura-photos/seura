//! Commands the UI can call. Each one is also listed in `build.rs`, which
//! generates the permission the UI's capability grants for it.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use seura_library::{
    AddFolder, Change, EditFileState, Filter, FolderId, PhotoId, Problem, Scope, Sort, SortBy,
    THUMBNAIL_SIZES, is_placeholder,
};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use crate::editor::Editor;
use crate::library::{AppSettings, Library, LibraryChanged};
use crate::viewer::{Viewer, Wanted};
use crate::welcome;

/// The app's version, from `Cargo.toml`.
#[tauri::command]
#[specta::specta]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// A folder with photos directly in it.
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FolderInfo {
    /// The folder, for showing its photos; `None` for a library folder that
    /// has no photos yet, or whose drive has never been scanned.
    pub id: Option<u32>,
    /// For library folders: which one, for removing it (LIB-5).
    pub root: Option<u32>,
    /// For subfolders: the folder above.
    pub parent: Option<u32>,
    pub path: String,
    pub name: String,
    /// Photos directly in the folder, the ones the grid shows (BRW-1).
    pub photos: u32,
    /// Photos in the folder and everything below it.
    pub total: u32,
    /// False when its drive isn't connected (LIB-6).
    pub available: bool,
}

/// A photo, as the grid needs it.
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PhotoInfo {
    pub id: u32,
    pub name: String,
    /// Upright dimensions, once the photo has been decoded.
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// Only in the cloud: never downloaded for browsing (LIB-9).
    pub cloud_only: bool,
    /// When it was taken, in seconds since 1970 on the camera's own clock,
    /// if known (raw values: the UI formats them, ACC-6).
    pub taken: Option<f64>,
    /// The file's modification time, in seconds since 1970.
    pub modified: f64,
    /// 0 to 5 stars, from its edit file.
    pub rating: u8,
    /// Its edit file holds edits.
    pub edited: bool,
    /// Its edit file exists but can't be read (FIL-4).
    pub unreadable: bool,
}

/// The grid's order and size (BRW-2, BRW-3), from the settings.
#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ViewSettings {
    /// `dateTaken`, `fileName`, or `dateModified`.
    pub sort_by: String,
    pub ascending: bool,
    /// Grid cell size in logical pixels, 96 to 256.
    pub thumbnail_size: u32,
}

fn sort_name(by: SortBy) -> &'static str {
    match by {
        SortBy::DateTaken => "dateTaken",
        SortBy::FileName => "fileName",
        SortBy::DateModified => "dateModified",
    }
}

#[tauri::command]
#[specta::specta]
pub fn view_settings(settings: State<'_, AppSettings>) -> ViewSettings {
    let settings = &settings.0.lock().expect("settings lock").settings;
    ViewSettings {
        sort_by: sort_name(settings.sort.by).into(),
        ascending: settings.sort.ascending,
        thumbnail_size: settings.thumbnail_size,
    }
}

/// Sets the grid's order for the whole app, and remembers it (BRW-2).
#[tauri::command]
#[specta::specta]
pub fn set_sort(
    settings: State<'_, AppSettings>,
    by: String,
    ascending: bool,
) -> Result<(), String> {
    let by = match by.as_str() {
        "dateTaken" => SortBy::DateTaken,
        "fileName" => SortBy::FileName,
        "dateModified" => SortBy::DateModified,
        other => return Err(format!("no such order: {other}")),
    };
    let mut settings = settings.0.lock().expect("settings lock");
    settings.settings.sort = Sort { by, ascending };
    settings.save().map_err(|error| error.to_string())
}

/// Sets the grid's cell size, and remembers it (BRW-3).
#[tauri::command]
#[specta::specta]
pub fn set_thumbnail_size(settings: State<'_, AppSettings>, size: u32) -> Result<(), String> {
    let mut settings = settings.0.lock().expect("settings lock");
    settings.settings.thumbnail_size = size.clamp(THUMBNAIL_SIZES.0, THUMBNAIL_SIZES.1);
    settings.save().map_err(|error| error.to_string())
}

/// What adding a folder did (LIB-1).
#[derive(Clone, Debug, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AddFolderResult {
    /// The user closed the folder dialog.
    Cancelled,
    Added,
    /// Already in the library, itself or inside this library folder.
    AlreadyIncluded {
        folder: String,
    },
    /// Added, replacing these library folders inside it.
    Absorbed {
        folders: Vec<String>,
    },
}

/// Asks for a folder with the system's dialog (ACC-4), adds it to the
/// library, and scans it. The UI never sees a path it could open itself
/// (decision 0009).
#[tauri::command]
#[specta::specta]
pub async fn add_folder(app: AppHandle) -> Result<AddFolderResult, String> {
    let Some(path) = pick_folder(&app).await? else {
        return Ok(AddFolderResult::Cancelled);
    };

    tauri::async_runtime::spawn_blocking(move || {
        let library = app.state::<Library>();
        let added = library
            .index
            .write()
            .expect("index lock")
            .add_root(&path)
            .map_err(|error| error.to_string())?;
        let result = match added {
            AddFolder::AlreadyIncluded(folder) => {
                return Ok(AddFolderResult::AlreadyIncluded {
                    folder: display(&folder),
                });
            }
            AddFolder::Added => AddFolderResult::Added,
            AddFolder::Absorbed(folders) => AddFolderResult::Absorbed {
                folders: folders.iter().map(|folder| display(folder)).collect(),
            },
        };
        // The settings file holds the library folders (SET-3).
        let roots = library.index.read().expect("index lock").roots().to_vec();
        let settings = app.state::<AppSettings>();
        let mut settings = settings.0.lock().expect("settings lock");
        settings.settings.library_folders = roots;
        settings.save().map_err(|error| error.to_string())?;
        drop(settings);
        library.rewatch();
        library.rescan(&app);
        let _ = LibraryChanged.emit(&app);
        Ok(result)
    })
    .await
    .map_err(|error| error.to_string())?
}

/// The folders that have photos directly in them, by path.
///
/// This and the other quick commands that read the index run off the main
/// thread (`async`): the main thread also delivers events to the UI, and a
/// command waiting there waits behind them.
#[tauri::command(async)]
#[specta::specta]
pub fn folders(library: State<'_, Library>) -> Vec<FolderInfo> {
    let index = library.index.read().expect("index lock");
    let available: Vec<(PathBuf, bool)> = index
        .roots()
        .iter()
        .map(|root| (root.clone(), root.is_dir()))
        .collect();
    index
        .folder_tree()
        .into_iter()
        .map(|entry| FolderInfo {
            id: entry.id.map(|id| id.value()),
            root: entry.root.map(|position| position as u32),
            parent: entry.parent.map(|id| id.value()),
            name: entry.path.file_name().map_or_else(
                || display(&entry.path),
                |name| name.to_string_lossy().into_owned(),
            ),
            path: display(&entry.path),
            photos: entry.photos as u32,
            total: entry.total as u32,
            // A subfolder is available when its library folder is.
            available: available
                .iter()
                .find(|(root, _)| entry.path.starts_with(root))
                .is_none_or(|(_, available)| *available),
        })
        .collect()
}

/// Removes a library folder from the library (LIB-5), by its position among
/// the library folders: nothing on disk is touched, its edit files stay,
/// and its thumbnails leave the cache.
#[tauri::command]
#[specta::specta]
pub async fn remove_folder(app: AppHandle, root: u32) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let library = app.state::<Library>();
        let (keys, roots) = {
            let mut index = library.index.write().expect("index lock");
            let path = index
                .roots()
                .get(root as usize)
                .cloned()
                .ok_or("that folder isn't in the library")?;
            let keys = index.remove_root(&path);
            (keys, index.roots().to_vec())
        };
        library.drop_thumbnails(&keys);
        library.rewatch();
        let settings = app.state::<AppSettings>();
        let mut settings = settings.0.lock().expect("settings lock");
        settings.settings.library_folders = roots;
        settings.save().map_err(|error| error.to_string())?;
        drop(settings);
        library.save();
        let _ = LibraryChanged.emit(&app);
        Ok(())
    })
    .await
    .map_err(|error| error.to_string())?
}

/// A folder's own photos (BRW-1), in the app's order (BRW-2). Their
/// thumbnails are queued behind the visible ones.
#[tauri::command(async)]
#[specta::specta]
pub fn photos(
    library: State<'_, Library>,
    settings: State<'_, AppSettings>,
    folder: u32,
) -> Vec<PhotoInfo> {
    let sort = settings.0.lock().expect("settings lock").settings.sort;
    let folder = FolderId::new(folder);
    let photos: Vec<PhotoInfo> = {
        let index = library.index.read().expect("index lock");
        let ids = index.query(Scope::Folder(folder), &Filter::default());
        index
            .sorted(ids, sort)
            .into_iter()
            .filter_map(|id| {
                let photo = index.photo(id)?;
                Some(PhotoInfo {
                    id: id.value(),
                    name: photo.name.to_string(),
                    width: photo.dimensions.map(|(width, _)| width),
                    height: photo.dimensions.map(|(_, height)| height),
                    cloud_only: photo.placeholder,
                    taken: photo.taken.map(|seconds| seconds as f64),
                    modified: photo.modified.secs as f64,
                    rating: photo.rating,
                    edited: photo.edit_file == EditFileState::Read { edited: true },
                    unreadable: photo.edit_file == EditFileState::Unreadable,
                })
            })
            .collect()
    };
    library.queue_folder(folder);
    photos
}

/// One photo's details for the Info panel (BRW-6).
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PhotoDetails {
    pub id: u32,
    pub name: String,
    /// The folder, from its library folder down: `["Pictures", "2026", "Summer"]`.
    pub folder: Vec<String>,
    /// In bytes.
    pub size: f64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// As in `PhotoInfo`.
    pub taken: Option<f64>,
    pub rating: u8,
    pub tags: Vec<String>,
    pub edited: bool,
    pub unreadable: bool,
    /// In a read-only folder: it can't be tagged or rated (TAG-7).
    pub read_only: bool,
    /// Only in the cloud: its metadata isn't read, which would download it
    /// (LIB-9), so the camera, lens, and location are unknown.
    pub cloud_only: bool,
    pub camera: Option<String>,
    pub lens: Option<String>,
    /// In millimeters.
    pub focal_length: Option<f64>,
    /// The 35 mm equivalent, where the photo records it.
    pub focal_length_35mm: Option<f64>,
    pub f_number: Option<f64>,
    /// Shutter speed in seconds, ISO, and exposure compensation in stops.
    pub exposure_time: Option<f64>,
    pub iso: Option<u32>,
    pub exposure_bias: Option<f64>,
    /// The file's modification time, in seconds since 1970: shown when
    /// there's no date taken.
    pub modified: f64,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

/// A photo's details for the Info panel (BRW-6): what the index knows, and
/// the camera, lens, and location from its EXIF, read now (decision 0012).
#[tauri::command]
#[specta::specta]
pub async fn photo_details(app: AppHandle, id: u32) -> Result<PhotoDetails, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (mut details, path) = {
            let library = app.state::<Library>();
            let index = library.index.read().expect("index lock");
            let photo = index
                .photo(PhotoId::new(id))
                .ok_or("that photo isn't in the library")?;
            let path = index
                .path(PhotoId::new(id))
                .ok_or("that photo isn't in the library")?;
            let root = index.roots().iter().find(|root| path.starts_with(root));
            let folder = path
                .parent()
                .map(|parent| folder_names(root.map(PathBuf::as_path), parent))
                .unwrap_or_default();
            let details = PhotoDetails {
                id,
                name: photo.name.to_string(),
                folder,
                size: photo.size as f64,
                width: photo.dimensions.map(|(width, _)| width),
                height: photo.dimensions.map(|(_, height)| height),
                taken: photo.taken.map(|seconds| seconds as f64),
                rating: photo.rating,
                tags: photo
                    .tags
                    .iter()
                    .filter_map(|tag| index.tag(*tag).map(str::to_string))
                    .collect(),
                edited: photo.edit_file == EditFileState::Read { edited: true },
                unreadable: photo.edit_file == EditFileState::Unreadable,
                read_only: index
                    .folder(photo.folder)
                    .is_some_and(|folder| folder.read_only),
                cloud_only: std::fs::metadata(&path)
                    .map_or(photo.placeholder, |metadata| is_placeholder(&metadata)),
                camera: None,
                lens: None,
                focal_length: None,
                focal_length_35mm: None,
                f_number: None,
                exposure_time: None,
                iso: None,
                exposure_bias: None,
                modified: photo.modified.secs as f64,
                latitude: None,
                longitude: None,
            };
            (details, path)
        };
        if !details.cloud_only {
            let exif = seura_image::details(&path);
            details.taken = details.taken.or(exif.taken.map(|seconds| seconds as f64));
            details.camera = exif.camera;
            details.lens = exif.lens;
            details.focal_length = exif.focal_length;
            details.focal_length_35mm = exif.focal_length_35mm;
            details.f_number = exif.f_number;
            details.exposure_time = exif.exposure_time;
            details.iso = exif.iso;
            details.exposure_bias = exif.exposure_bias;
            details.latitude = exif.latitude;
            details.longitude = exif.longitude;
        }
        Ok(details)
    })
    .await
    .map_err(|error| error.to_string())?
}

/// A folder's names from its library folder down, or just its own name
/// outside one.
fn folder_names(root: Option<&Path>, folder: &Path) -> Vec<String> {
    let name = |path: &Path| {
        path.file_name().map_or_else(
            || path.display().to_string(),
            |name| name.to_string_lossy().into_owned(),
        )
    };
    match root.and_then(|root| Some((root, folder.strip_prefix(root).ok()?))) {
        Some((root, below)) => std::iter::once(name(root))
            .chain(
                below
                    .components()
                    .map(|part| part.as_os_str().to_string_lossy().into_owned()),
            )
            .collect(),
        None => vec![name(folder)],
    }
}

/// What several selected photos share, for the Info panel (BRW-6). From the
/// index alone: nothing is read.
#[derive(Clone, Debug, Default, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SelectionSummary {
    pub count: u32,
    /// Their sizes added up, in bytes.
    pub size: f64,
    /// The earliest and latest date taken among those that have one.
    pub first_taken: Option<f64>,
    pub last_taken: Option<f64>,
    /// Their rating, if they all have the same one.
    pub rating: Option<u8>,
    /// Tags every one of them has.
    pub tags: Vec<String>,
    /// Tags only some of them have, with how many (TAG-8).
    pub partial_tags: Vec<TagCount>,
    /// How many are in read-only folders (TAG-7).
    pub read_only: u32,
}

/// A tag and a number of photos.
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TagCount {
    pub tag: String,
    pub count: u32,
}

/// How many of `photos` have each of their tags, in natural order.
fn tag_counts(index: &seura_library::Index, photos: &[&seura_library::Photo]) -> Vec<TagCount> {
    let mut counts: Vec<(seura_library::TagId, u32)> = Vec::new();
    for photo in photos {
        for tag in &photo.tags {
            match counts.iter_mut().find(|(id, _)| id == tag) {
                Some((_, count)) => *count += 1,
                None => counts.push((*tag, 1)),
            }
        }
    }
    let mut counts: Vec<TagCount> = counts
        .into_iter()
        .filter_map(|(id, count)| {
            Some(TagCount {
                tag: index.tag(id)?.to_string(),
                count,
            })
        })
        .collect();
    counts.sort_by(|a, b| seura_library::natural(&a.tag, &b.tag));
    counts
}

#[tauri::command(async)]
#[specta::specta]
pub fn selection_summary(library: State<'_, Library>, ids: Vec<u32>) -> SelectionSummary {
    let index = library.index.read().expect("index lock");
    let photos: Vec<_> = ids
        .iter()
        .filter_map(|id| index.photo(PhotoId::new(*id)))
        .collect();
    let Some(first) = photos.first() else {
        return SelectionSummary::default();
    };
    let taken = photos.iter().filter_map(|photo| photo.taken);
    let rating = first.rating;
    SelectionSummary {
        count: photos.len() as u32,
        size: photos.iter().map(|photo| photo.size as f64).sum(),
        first_taken: taken.clone().min().map(|seconds| seconds as f64),
        last_taken: taken.max().map(|seconds| seconds as f64),
        rating: photos
            .iter()
            .all(|photo| photo.rating == rating)
            .then_some(rating),
        tags: first
            .tags
            .iter()
            .filter(|tag| photos.iter().all(|photo| photo.tags.contains(tag)))
            .filter_map(|tag| index.tag(*tag).map(str::to_string))
            .collect(),
        partial_tags: tag_counts(&index, &photos)
            .into_iter()
            .filter(|tag| (tag.count as usize) < photos.len())
            .collect(),
        read_only: photos
            .iter()
            .filter(|photo| {
                index
                    .folder(photo.folder)
                    .is_some_and(|folder| folder.read_only)
            })
            .count() as u32,
    }
}

/// A change to photos' tags or rating (TAG-3, TAG-4, TAG-6).
#[derive(Clone, Debug, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PhotoChange {
    /// 0 to 5 stars; 0 clears the rating.
    Rating {
        stars: u8,
    },
    AddTag {
        tag: String,
    },
    RemoveTag {
        tag: String,
    },
}

/// What a change did, in numbers of photos.
#[derive(Clone, Debug, Default, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChangeResult {
    /// Their edit files were written, created, or moved to the Trash.
    pub changed: u32,
    /// In a read-only folder (TAG-7).
    pub read_only: u32,
    /// Their edit file can't be read, so it's left alone (FIL-4).
    pub unreadable: u32,
    /// Couldn't be changed for another reason, such as a full drive.
    pub failed: u32,
}

/// Changes the tags or rating of photos in the library, writing their edit
/// files beside them (FIL-1, FIL-2). Photos already as asked are left alone.
#[tauri::command]
#[specta::specta]
pub async fn change_photos(
    app: AppHandle,
    ids: Vec<u32>,
    change: PhotoChange,
) -> Result<ChangeResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let ids: Vec<PhotoId> = ids.into_iter().map(PhotoId::new).collect();
        let change = match change {
            PhotoChange::Rating { stars } => Change::Rating(stars.min(5)),
            PhotoChange::AddTag { tag } => Change::AddTag(tag),
            PhotoChange::RemoveTag { tag } => Change::RemoveTag(tag),
        };
        let report = app.state::<Library>().change(&app, &ids, &change);
        let mut result = ChangeResult {
            changed: report.changed.len() as u32,
            ..ChangeResult::default()
        };
        for (_, problem) in &report.problems {
            match problem {
                Problem::ReadOnly => result.read_only += 1,
                Problem::Unreadable => result.unreadable += 1,
                _ => result.failed += 1,
            }
        }
        result
    })
    .await
    .map_err(|error| error.to_string())
}

/// What the tag picker offers for photos, filtered by what's typed (TAG-3,
/// TAG-8). Matching ignores case and Unicode form (TAG-2).
#[derive(Clone, Debug, Default, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TagChoices {
    /// Tags all of them have: shown as chips, each with ×.
    pub on_all: Vec<String>,
    /// Tags only some of them have, with how many: listed first, so Enter
    /// adds one to the rest (TAG-8).
    pub on_some: Vec<TagCount>,
    /// The library's other tags that contain the text, with how many photos
    /// have each (TAG-3, TAG-5).
    pub others: Vec<TagCount>,
    /// The text, cleaned up, is already a tag in the library, so there's
    /// nothing to create.
    pub exists: bool,
}

#[tauri::command(async)]
#[specta::specta]
pub fn tag_choices(library: State<'_, Library>, ids: Vec<u32>, text: String) -> TagChoices {
    choose_tags(&library.index.read().expect("index lock"), &ids, text)
}

fn choose_tags(index: &seura_library::Index, ids: &[u32], text: String) -> TagChoices {
    let photos: Vec<_> = ids
        .iter()
        .filter_map(|id| index.photo(PhotoId::new(*id)))
        .collect();
    let typed = seura_edit::clean_tags(&[text]).pop().unwrap_or_default();
    let key = seura_edit::tag_key(&typed);
    let wanted = |tag: &str| seura_edit::tag_key(tag).contains(&key);
    let (on_all, on_some): (Vec<_>, Vec<_>) = tag_counts(index, &photos)
        .into_iter()
        .partition(|tag| tag.count as usize == photos.len());
    let mut others: Vec<TagCount> = index
        .tags()
        .filter(|(_, tag, _)| {
            wanted(tag)
                && !on_all
                    .iter()
                    .chain(&on_some)
                    .any(|chosen| chosen.tag == *tag)
        })
        .map(|(_, tag, count)| TagCount {
            tag: tag.to_string(),
            count: count as u32,
        })
        .collect();
    others.sort_by(|a, b| seura_library::natural(&a.tag, &b.tag));
    TagChoices {
        exists: !typed.is_empty() && index.matching_tag(&typed).is_some(),
        on_all: on_all.into_iter().map(|tag| tag.tag).collect(),
        on_some: on_some.into_iter().filter(|tag| wanted(&tag.tag)).collect(),
        others,
    }
}

/// The files of photos in the index, never any other path.
fn photo_paths(library: &Library, ids: &[u32]) -> Vec<PathBuf> {
    let index = library.index.read().expect("index lock");
    ids.iter()
        .filter_map(|id| index.path(PhotoId::new(*id)))
        .collect()
}

/// Whether any photo is chosen (selected, or shown in the Viewer), so the
/// macOS menu bar's photo commands can be turned on or off (BRW-8).
#[tauri::command]
#[specta::specta]
pub fn photos_chosen(app: AppHandle, chosen: bool) {
    crate::menu::photos_chosen(&app, chosen);
}

/// Shows photos in Explorer or Finder, selected there (BRW-8). Only their
/// attributes are read, so a cloud-only photo isn't downloaded (LIB-9).
#[tauri::command]
#[specta::specta]
pub async fn show_in_folder(app: AppHandle, ids: Vec<u32>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let paths = photo_paths(&app.state::<Library>(), &ids);
        if paths.is_empty() {
            return Err("those photos aren't in the library".to_string());
        }
        tauri_plugin_opener::reveal_items_in_dir(&paths).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

/// What opening photos in the default app would involve, for the UI to
/// confirm first when it's many or some are in the cloud (BRW-8, LIB-9).
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Opening {
    pub count: u32,
    /// How many are cloud-only, asked of the files' attributes: opening
    /// them downloads them.
    pub cloud_only: u32,
}

#[tauri::command(async)]
#[specta::specta]
pub fn opening(library: State<'_, Library>, ids: Vec<u32>) -> Opening {
    let paths = photo_paths(&library, &ids);
    let cloud_only = paths
        .iter()
        .filter(|path| std::fs::metadata(path).is_ok_and(|metadata| is_placeholder(&metadata)))
        .count();
    Opening {
        count: paths.len() as u32,
        cloud_only: cloud_only as u32,
    }
}

/// Opens photos in the system's default app for them (BRW-8). That app
/// reads them, so it downloads a cloud-only one: the user asked for it, and
/// the UI confirms first when it's several (LIB-9).
#[tauri::command]
#[specta::specta]
pub async fn open_in_default_app(app: AppHandle, ids: Vec<u32>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let paths = photo_paths(&app.state::<Library>(), &ids);
        if paths.is_empty() {
            return Err("those photos aren't in the library".to_string());
        }
        for path in paths {
            tauri_plugin_opener::open_path(&path, None::<&str>)
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    })
    .await
    .map_err(|error| error.to_string())?
}

/// The photos now on screen, in order: their thumbnails are made first (LIB-3).
#[tauri::command(async)]
#[specta::specta]
pub fn show_thumbnails(library: State<'_, Library>, ids: Vec<u32>) {
    let jobs = {
        let index = library.index.read().expect("index lock");
        ids.into_iter()
            .filter_map(|id| index.thumbnail_job(PhotoId::new(id)))
            .collect()
    };
    library.show(jobs);
}

/// A photo opened in the Editor.
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenedPhoto {
    /// The preview's size, fetched from `seura://…/preview`.
    pub width: u32,
    pub height: u32,
    pub exposure_ev: f64,
    /// Why the photo can't be edited, if it can't: its edit file can't be
    /// read, and is never overwritten (FIL-4).
    pub problem: Option<String>,
}

/// What the Viewer can show of a photo (BRW-5).
#[derive(Clone, Debug, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ViewedPhoto {
    /// Decoded: its pixels are at `seura://…/view-<id>`.
    #[serde(rename_all = "camelCase")]
    Shown {
        width: u32,
        height: u32,
        /// The photo's own upright size, for zooming.
        full_width: u32,
        full_height: u32,
        exposure_ev: f64,
    },
    /// Only in the cloud, and not read: ask again with `download` to read
    /// it, which downloads it (LIB-9).
    #[serde(rename_all = "camelCase")]
    CloudOnly {
        /// In bytes, from the placeholder: what the download will be.
        size: f64,
    },
}

/// How long the Viewer waits for a cloud-only photo before saying it
/// couldn't be downloaded. Neither system reports progress (LIB-9), so this
/// is generous: about 1 Mbit/s for a 20 MB photo.
const DOWNLOAD_WAIT: Duration = Duration::from_secs(180);

/// Shows photo `id` in the Viewer, decoded to fit an area of `width` ×
/// `height` device pixels (DSP-1), and decodes `neighbors` ahead (PRF-5).
/// A cloud-only photo is read, downloading it, only with `download`, and
/// then waited for at most [`DOWNLOAD_WAIT`]; neighbors in the cloud are
/// never decoded ahead (LIB-9).
#[tauri::command]
#[specta::specta]
pub async fn view_photo(
    app: AppHandle,
    id: u32,
    neighbors: Vec<u32>,
    width: u32,
    height: u32,
    download: bool,
) -> Result<ViewedPhoto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let wanted = |id: u32| viewable(&app, id);
        let (photo, cloud_only, size) = wanted(id).ok_or("that photo isn't in the library")?;
        let neighbors: Vec<Wanted> = neighbors
            .into_iter()
            .filter_map(wanted)
            .filter(|(_, cloud_only, _)| !cloud_only)
            .map(|(wanted, _, _)| wanted)
            .collect();
        if cloud_only && !download {
            return Ok(ViewedPhoto::CloudOnly { size: size as f64 });
        }
        let shown = if cloud_only {
            // Read on a thread of its own, so a download that never ends
            // doesn't hold the Viewer: past the wait, its result is dropped.
            let (sender, receiver) = std::sync::mpsc::channel();
            let handle = app.clone();
            std::thread::spawn(move || {
                let shown = handle
                    .state::<Viewer>()
                    .show(&photo, &neighbors, (width, height));
                let _ = sender.send(shown);
            });
            receiver
                .recv_timeout(DOWNLOAD_WAIT)
                .map_err(|_| "the download took too long".to_string())??
        } else {
            app.state::<Viewer>()
                .show(&photo, &neighbors, (width, height))?
        };
        Ok(ViewedPhoto::Shown {
            width: shown.width,
            height: shown.height,
            full_width: shown.full_width,
            full_height: shown.full_height,
            exposure_ev: shown.exposure_ev,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

/// A photo the Viewer may read, whether it's only in the cloud, and its
/// size. Whether it's in the cloud is asked of the file itself, which only
/// reads its attributes: one downloaded since the last scan opens at once.
fn viewable(app: &AppHandle, id: u32) -> Option<(Wanted, bool, u64)> {
    let (path, indexed, size) = {
        let library = app.state::<Library>();
        let index = library.index.read().expect("index lock");
        let photo = index.photo(PhotoId::new(id))?;
        (index.path(PhotoId::new(id))?, photo.placeholder, photo.size)
    };
    let cloud_only = std::fs::metadata(&path).map_or(indexed, |metadata| is_placeholder(&metadata));
    Some((Wanted { id, path }, cloud_only, size))
}

/// A part of a photo, cut for the Viewer.
#[derive(Clone, Debug, Serialize, Type)]
pub struct RegionSize {
    pub width: u32,
    pub height: u32,
}

/// Zoomed in (BRW-5): cuts the part of photo `id` at `x`, `y`, `width` ×
/// `height` (in its own upright pixels) at full resolution, scaled to
/// `outWidth` × `outHeight` but never enlarged, for `seura://…/view-region`.
/// Never for a cloud-only photo (LIB-9).
#[tauri::command]
#[specta::specta]
#[allow(clippy::too_many_arguments)]
pub async fn view_region(
    app: AppHandle,
    id: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    out_width: u32,
    out_height: u32,
) -> Result<RegionSize, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (photo, cloud_only, _) = viewable(&app, id).ok_or("that photo isn't in the library")?;
        if cloud_only {
            return Err("that photo is only in the cloud".into());
        }
        let region = seura_image::Region {
            x,
            y,
            width,
            height,
        };
        let (width, height) = app
            .state::<Viewer>()
            .region(&photo, region, out_width, out_height)?;
        Ok(RegionSize { width, height })
    })
    .await
    .map_err(|error| error.to_string())?
}

/// The Viewer closed: its decoded photos are let go.
#[tauri::command]
#[specta::specta]
pub fn close_viewer(viewer: State<'_, Viewer>) {
    viewer.close();
}

/// Opens a photo in the Editor, with a preview `longEdge` pixels on its long
/// side (the canvas size × the device pixel ratio, DSP-1).
#[tauri::command]
#[specta::specta]
pub async fn open_photo(app: AppHandle, id: u32, long_edge: u32) -> Result<OpenedPhoto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (photo, dimensions) = {
            let library = app.state::<Library>();
            let index = library.index.read().expect("index lock");
            let photo = index
                .path(PhotoId::new(id))
                .ok_or("that photo isn't in the library")?;
            let dimensions = index
                .photo(PhotoId::new(id))
                .and_then(|photo| photo.dimensions);
            (photo, dimensions)
        };
        let opened = app
            .state::<Editor>()
            .open(id, photo, dimensions, long_edge)?;
        Ok(OpenedPhoto {
            width: opened.width,
            height: opened.height,
            exposure_ev: opened.exposure_ev,
            problem: opened.problem,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
#[specta::specta]
pub fn close_photo(editor: State<'_, Editor>) {
    editor.close();
}

/// Sets the open photo's exposure (EDT-5) and saves its edit file (EDT-11).
#[tauri::command]
#[specta::specta]
pub async fn set_exposure(app: AppHandle, id: u32, ev: f64) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || app.state::<Editor>().set_exposure(id, ev))
        .await
        .map_err(|error| error.to_string())?
}

/// What exporting did.
#[derive(Clone, Debug, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ExportResult {
    /// The user closed the folder dialog.
    Cancelled,
    Exported {
        file: String,
        folder: String,
    },
}

/// Asks for a folder and exports the open photo there as JPEG (EXP-1, EXP-5).
#[tauri::command]
#[specta::specta]
pub async fn export_photo(app: AppHandle, id: u32) -> Result<ExportResult, String> {
    let Some(folder) = pick_folder(&app).await? else {
        return Ok(ExportResult::Cancelled);
    };
    tauri::async_runtime::spawn_blocking(move || {
        let path = app.state::<Editor>().export(id, &folder)?;
        Ok(ExportResult::Exported {
            file: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            folder: display(&folder),
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

/// The system's text size as a scale, 1 at the default (ACC-3). Windows
/// reads it from "Text size" in Accessibility settings. macOS has no
/// system-wide text size that apps can read, so it's always 1 there.
#[tauri::command]
#[specta::specta]
pub fn text_scale() -> f64 {
    #[cfg(target_os = "windows")]
    {
        windows::UI::ViewManagement::UISettings::new()
            .and_then(|settings| settings.TextScaleFactor())
            .unwrap_or(1.0)
    }
    #[cfg(not(target_os = "windows"))]
    {
        1.0
    }
}

/// The operating system's region as a BCP 47 tag, such as `de-DE`, for
/// formatting dates, numbers, and sizes (ACC-6). The interface itself is in
/// English; the web view's own locale follows the display language.
#[tauri::command]
#[specta::specta]
pub fn region() -> String {
    crate::locale::region()
}

/// Developer switches, from environment variables.
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DevOptions {
    /// `SEURA_PREVIEW=webgl2` forces the WebGL2 preview on a web view with
    /// WebGPU, to check the fallback (M1 slice 5).
    pub force_webgl2: bool,
    /// `SEURA_RENDER_CHECK=1` runs the render check instead of the app.
    pub render_check: bool,
    /// `SEURA_LOCALE=pseudo` pseudo-translates every string, to check that
    /// longer translations fit (ACC-7).
    pub pseudo_locale: bool,
    /// The exposure the render check uses, so both sides agree.
    pub check_exposure_ev: f32,
    /// Whether M1's Editor is available: in development builds only, since
    /// M2 writes no edit files (M2 work plan).
    pub editor_available: bool,
    /// `SEURA_MEASURE=1` runs the measurement script (M2 slice 10).
    pub measure: bool,
    /// Every switch that changes how the app behaves, as `NAME=value`, so
    /// the UI can say they're on: they're easy to leave set by accident.
    pub active: Vec<String>,
}

/// The developer switches that change the app's behavior.
const SWITCHES: [&str; 6] = [
    "SEURA_HEIF_DECODER",
    "SEURA_PREVIEW",
    crate::locale::LOCALE_SWITCH,
    welcome::WELCOME_SWITCH,
    crate::locations::DATA_SWITCH,
    crate::measure::MEASURE_SWITCH,
];

/// The switches that are set, as `NAME=value`.
pub fn active_switches() -> Vec<String> {
    SWITCHES
        .iter()
        .filter_map(|name| {
            let value = std::env::var(name).ok()?;
            Some(format!("{name}={value}"))
        })
        .collect()
}

#[tauri::command]
#[specta::specta]
pub fn dev_options() -> DevOptions {
    DevOptions {
        force_webgl2: std::env::var("SEURA_PREVIEW").as_deref() == Ok("webgl2"),
        render_check: render_check_mode(),
        pseudo_locale: crate::locale::pseudo(),
        check_exposure_ev: seura_render::check::CHECK_SETTINGS.exposure_ev,
        editor_available: cfg!(debug_assertions),
        measure: crate::measure::active(),
        active: active_switches(),
    }
}

/// How far scanning and thumbnails have got, now, for the measurement script.
#[tauri::command(async)]
#[specta::specta]
pub fn scan_progress(library: State<'_, Library>) -> crate::library::ScanProgress {
    library.progress_now()
}

/// Milliseconds since the app started, for the measurement script.
#[tauri::command]
#[specta::specta]
pub fn measure_elapsed() -> f64 {
    crate::measure::elapsed_ms()
}

/// The measurement script's results, as JSON: written with the core's
/// memory to `measurement.json` in Seura's folder, and then the app quits.
/// Only while `SEURA_MEASURE=1`.
#[tauri::command]
#[specta::specta]
pub fn measure_report(app: AppHandle, report: String) -> Result<(), String> {
    if !crate::measure::active() {
        return Err("not measuring".into());
    }
    crate::measure::report(&app, &report)
}

/// The render check: one preview path's render of the check image, as 8-bit
/// RGBA, compared with wgpu's. Returns the largest difference.
#[tauri::command]
#[specta::specta]
pub async fn check_render(
    app: AppHandle,
    backend: String,
    width: u32,
    height: u32,
    pixels: Vec<u8>,
) -> Result<u8, String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<Editor>()
            .check_render(backend, width, height, pixels)
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Ends the render check: the app exits, with status 0 only if every
/// preview path that ran matched export and nothing went wrong. Paths the
/// web view doesn't have are listed, not failed. Does nothing outside the check.
#[tauri::command]
#[specta::specta]
pub fn finish_render_check(app: AppHandle, skipped: Vec<String>, problems: Vec<String>) {
    if !render_check_mode() {
        return;
    }
    for path in &skipped {
        println!("render check, {path}: not available in this web view");
    }
    for problem in &problems {
        println!("render check: {problem}");
    }
    let passed = problems.is_empty() && app.state::<Editor>().checks_passed();
    println!("render check {}", if passed { "passed" } else { "FAILED" });
    app.exit(if passed { 0 } else { 1 });
}

fn render_check_mode() -> bool {
    std::env::var("SEURA_RENDER_CHECK").as_deref() == Ok("1")
}

/// The Welcome screen's facts: whether it's a first run, Seura's folder,
/// the suggested Pictures folder, and whether the cache would be synced
/// (SET-1).
#[tauri::command]
#[specta::specta]
pub fn welcome(app: AppHandle) -> Result<welcome::WelcomeInfo, String> {
    welcome::info(&app).map_err(|error| error.to_string())
}

/// Add another folder, on the Welcome screen: the system's folder dialog,
/// and the folder offered to the UI by id.
#[tauri::command]
#[specta::specta]
pub async fn welcome_add_folder(app: AppHandle) -> Result<Option<welcome::Choice>, String> {
    Ok(pick_folder(&app)
        .await?
        .map(|path| welcome::offer_folder(&app, path, false)))
}

/// Store thumbnails somewhere else, on the Welcome screen (SET-1).
#[tauri::command]
#[specta::specta]
pub async fn welcome_choose_cache(app: AppHandle) -> Result<Option<welcome::Choice>, String> {
    Ok(pick_folder(&app)
        .await?
        .map(|path| welcome::offer_folder(&app, path, true)))
}

/// Get started: the chosen folders (by id), the update choice (SET-4), and
/// the cache location if the user chose one.
#[tauri::command]
#[specta::specta]
pub async fn finish_welcome(
    app: AppHandle,
    folders: Vec<u32>,
    check_for_updates: bool,
    cache: Option<u32>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        welcome::finish(&app, &folders, check_for_updates, cache)
    })
    .await
    .map_err(|error| error.to_string())?
}

/// The system's folder dialog (ACC-4), or `None` if the user cancels.
async fn pick_folder(app: &AppHandle) -> Result<Option<PathBuf>, String> {
    // Native dialogs must run on the main thread (macOS requires it).
    let (send, receive) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = send.send(rfd::FileDialog::new().pick_folder());
    })
    .map_err(|error| error.to_string())?;
    tauri::async_runtime::spawn_blocking(move || receive.recv().ok().flatten())
        .await
        .map_err(|error| error.to_string())
}

fn display(path: &Path) -> String {
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_is_named_from_its_library_folder_down() {
        let root = Path::new("C:/Users/Nevada/Pictures");
        assert_eq!(
            folder_names(Some(root), &root.join("2026").join("Summer")),
            ["Pictures", "2026", "Summer"]
        );
        assert_eq!(folder_names(Some(root), root), ["Pictures"]);
        assert_eq!(folder_names(None, Path::new("D:/Scans")), ["Scans"]);
    }

    #[test]
    fn the_text_scale_is_readable_and_sensible() {
        let scale = super::text_scale();
        assert!((1.0..=2.25).contains(&scale), "{scale}");
    }

    #[test]
    fn app_version_matches_the_package() {
        assert_eq!(super::app_version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn the_tag_picker_offers_shared_partial_and_matching_tags() {
        use seura_library::{Change, Index};
        let dir = tempfile::tempdir().unwrap();
        for n in 1..=3 {
            let mut bytes = vec![0xFF, 0xD8, 0xFF, 0xE1];
            bytes.extend([n; 4096]);
            std::fs::write(dir.path().join(format!("IMG_{n}.jpg")), bytes).unwrap();
        }
        let mut index = Index::new();
        index.add_root(dir.path()).unwrap();
        let root = index.roots()[0].clone();
        index.scan(&root);
        let mut ids: Vec<_> = index.photo_ids().collect();
        ids.sort_by_key(|id| index.path(*id));
        let index = std::sync::RwLock::new(index);
        let tag = |which: &[seura_library::PhotoId], tag: &str| {
            Index::change_shared(
                &index,
                which,
                &Change::AddTag(tag.into()),
                crate::editor::WRITER,
                time::OffsetDateTime::now_utc(),
                &|_| Ok(()),
            );
        };
        tag(&ids, "vacation/2026");
        tag(&ids[..1], "family");
        tag(&ids[2..], "family/kids");
        tag(&ids[2..], "beach");
        let index = index.read().unwrap();
        let chosen = [ids[0].value(), ids[1].value()];

        let all = choose_tags(&index, &chosen, String::new());
        assert_eq!(all.on_all, ["vacation/2026"]);
        assert_eq!(all.on_some.len(), 1);
        assert_eq!(
            (all.on_some[0].tag.as_str(), all.on_some[0].count),
            ("family", 1)
        );
        let others: Vec<_> = all
            .others
            .iter()
            .map(|t| (t.tag.as_str(), t.count))
            .collect();
        assert_eq!(others, [("beach", 1), ("family/kids", 1)]);
        assert!(!all.exists);

        // Typing filters, ignoring case, and says when the tag exists.
        let fam = choose_tags(&index, &chosen, "FAM".into());
        assert_eq!(fam.on_some[0].tag, "family");
        assert_eq!(fam.others.len(), 1);
        assert_eq!(fam.others[0].tag, "family/kids");
        assert!(!fam.exists);
        assert!(choose_tags(&index, &chosen, " Family ".into()).exists);
    }
}
