//! Exporting photos from the Export dialog (EXP-1, EXP-6, LIB-9): one photo at
//! a time, so a batch of large photos holds one in memory, downloading those in
//! the cloud as it comes to them. Cancel stops between photos and stops waiting
//! for a download.
//!
//! The UI never names a folder: it asks for the system's dialog, and the core
//! keeps what the user chose.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use seura_image::DecodeError;
use seura_library::{ExportFileFormat, ExportSettings, PhotoId};
use seura_render::{ExportError, ExportFormat, ExportOptions, ExportSize, MIN_LONG_EDGE, export};
use specta::Type;
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

use crate::downloads::{Downloaded, Downloads};
use crate::library::{AppSettings, Library};

/// The batch in progress, and what the last one wrote.
#[derive(Default)]
pub struct Exporting {
    /// Apart from Enhance's, so stopping one doesn't stop the other.
    downloads: Downloads,
    /// The folder the dialog shows.
    destination: Mutex<Option<PathBuf>>,
    /// The files the last batch wrote, for Show in Explorer or Finder.
    written: Mutex<Vec<PathBuf>>,
}

/// What the dialog asks for, remembered for next time (EXP-2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ExportChoices {
    pub png: bool,
    /// 1 to 100, for JPEG.
    pub quality: u8,
    /// The long edge's limit, at least 16; none is the original size.
    pub long_edge: Option<u32>,
    pub remove_location: bool,
    pub include_keywords: bool,
}

impl From<&ExportSettings> for ExportChoices {
    fn from(settings: &ExportSettings) -> Self {
        ExportChoices {
            png: settings.format == ExportFileFormat::Png,
            quality: settings.quality.clamp(1, 100),
            long_edge: settings.long_edge.filter(|edge| *edge >= MIN_LONG_EDGE),
            remove_location: settings.remove_location,
            include_keywords: settings.include_keywords,
        }
    }
}

impl ExportChoices {
    fn options(&self) -> ExportOptions {
        ExportOptions {
            format: if self.png {
                ExportFormat::Png
            } else {
                ExportFormat::Jpeg {
                    quality: self.quality,
                }
            },
            size: self
                .long_edge
                .map_or(ExportSize::Original, ExportSize::LongEdge),
            remove_location: self.remove_location,
        }
    }

    fn valid(&self) -> bool {
        (1..=100).contains(&self.quality) && self.long_edge.is_none_or(|edge| edge >= MIN_LONG_EDGE)
    }
}

/// What the dialog opens with.
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ExportStart {
    /// The folder, or none if the last one is gone: Export waits for one.
    pub folder: Option<String>,
    pub choices: ExportChoices,
    /// How many of the photos are only in the cloud, and their total size in
    /// bytes (LIB-9).
    pub cloud_only: u32,
    pub cloud_size: f64,
}

/// The photo the export has come to.
#[derive(Clone, Debug, Serialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ExportProgress {
    pub id: u32,
    pub name: String,
    /// Counting from 1.
    pub at: u32,
    pub total: u32,
    /// Waiting for it to arrive from the cloud (EXP-6), with its size in bytes.
    pub downloading: bool,
    pub size: f64,
}

/// Why one photo wasn't exported; the others go on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PhotoProblem {
    /// In the cloud, and the download failed or took too long.
    NotDownloaded,
    /// Gone, damaged, or not something Seura can read.
    Unreadable,
    /// HEIC on Windows without Microsoft's extensions (decision 0014).
    NeedsCodec,
    /// Its edit file can't be read, so its edits aren't known (FIL-4).
    EditsUnreadable,
    /// Too many tags for a JPEG, or the render failed.
    Failed,
}

/// Why the export stopped: every photo after this would fail the same way.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FolderProblem {
    Full,
    NotWritable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Failure {
    Photo(PhotoProblem),
    Folder(FolderProblem),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ExportFailure {
    pub name: String,
    pub problem: PhotoProblem,
}

/// What an export did, for the summary or message (EXP-6).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ExportReport {
    pub exported: u32,
    /// Of those, how many got a number because their name was taken (EXP-3),
    /// which is worth saying: exporting again makes copies (Nevada, 2026-11-04).
    pub renamed: u32,
    /// The first file's name, which is all one photo's message needs.
    pub file: Option<String>,
    pub failures: Vec<ExportFailure>,
    /// Not exported because the user canceled.
    pub stopped: u32,
    /// The user canceled during a download. Seura stopped waiting, but its
    /// read goes on, so the photo may arrive anyway, not exported.
    pub still_downloading: bool,
    pub folder_problem: Option<FolderProblem>,
    /// Not tried because of that problem.
    pub left: u32,
}

/// A photo to export.
#[derive(Clone, Debug)]
struct Item {
    id: PhotoId,
    path: PathBuf,
    name: String,
    cloud: bool,
    size: u64,
}

/// Goes through the photos in order, one at a time.
fn run_batch(
    items: &[Item],
    stopped: impl Fn() -> bool,
    mut progress: impl FnMut(u32, &Item, bool),
    mut download: impl FnMut(&Item) -> Downloaded,
    mut write: impl FnMut(&Item) -> Result<PathBuf, Failure>,
) -> (ExportReport, Vec<PathBuf>) {
    let mut report = ExportReport::default();
    let mut written = Vec::new();
    for (done, item) in items.iter().enumerate() {
        let remaining = (items.len() - done) as u32;
        if stopped() {
            report.stopped = remaining;
            break;
        }
        let at = done as u32 + 1;
        if item.cloud {
            progress(at, item, true);
            match download(item) {
                Downloaded::Yes => {}
                Downloaded::Failed => {
                    report.failures.push(ExportFailure {
                        name: item.name.clone(),
                        problem: PhotoProblem::NotDownloaded,
                    });
                    continue;
                }
                Downloaded::Stopped => {
                    report.stopped = remaining;
                    report.still_downloading = true;
                    break;
                }
            }
        }
        progress(at, item, false);
        match write(item) {
            Ok(path) => {
                report.exported += 1;
                if path.file_stem() != item.path.file_stem() {
                    report.renamed += 1;
                }
                if report.file.is_none() {
                    report.file = path
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned());
                }
                written.push(path);
            }
            Err(Failure::Photo(problem)) => report.failures.push(ExportFailure {
                name: item.name.clone(),
                problem,
            }),
            Err(Failure::Folder(problem)) => {
                report.folder_problem = Some(problem);
                report.left = remaining;
                break;
            }
        }
    }
    (report, written)
}

/// Exports one photo with its saved edits.
fn write_photo(
    photo: &Path,
    folder: &Path,
    options: &ExportOptions,
    include_keywords: bool,
) -> Result<PathBuf, Failure> {
    let (settings, keywords) = crate::editor::export_edits(photo)
        .map_err(|_| Failure::Photo(PhotoProblem::EditsUnreadable))?;
    let keywords = include_keywords.then_some(&keywords);
    crate::gpu::with_renderer(|renderer| {
        export(renderer, photo, &settings, folder, options, keywords)
    })
    .map_err(|_| Failure::Photo(PhotoProblem::Failed))?
    .map_err(|error| failure(&error))
}

fn failure(error: &ExportError) -> Failure {
    match error {
        ExportError::Write(error) => Failure::Folder(match error.kind() {
            std::io::ErrorKind::StorageFull | std::io::ErrorKind::QuotaExceeded => {
                FolderProblem::Full
            }
            _ => FolderProblem::NotWritable,
        }),
        ExportError::Decode(DecodeError::HeifUnavailable(_)) => {
            Failure::Photo(PhotoProblem::NeedsCodec)
        }
        ExportError::Decode(_) => Failure::Photo(PhotoProblem::Unreadable),
        _ => Failure::Photo(PhotoProblem::Failed),
    }
}

impl Exporting {
    /// What the dialog opens with: the last folder if it's still there, the
    /// first time the Pictures folder (EXP-1), and the last choices (EXP-2).
    pub fn start<R: Runtime>(&self, app: &AppHandle<R>, ids: &[PhotoId]) -> ExportStart {
        let settings = app.state::<AppSettings>();
        let remembered = settings
            .0
            .lock()
            .expect("settings lock")
            .settings
            .export
            .clone();
        let folder = starting_folder(remembered.folder.as_deref(), app.path().picture_dir().ok());
        let library = app.state::<Library>();
        for &id in ids {
            library.downloaded(app, id);
        }
        let (mut cloud_only, mut cloud_size) = (0, 0.0);
        let index = library.index.read().expect("index lock");
        for &id in ids {
            if let Some(photo) = index.photo(id).filter(|photo| photo.placeholder) {
                cloud_only += 1;
                cloud_size += photo.size as f64;
            }
        }
        let shown = folder.as_ref().map(|folder| folder.display().to_string());
        *self.destination.lock().expect("destination lock") = folder;
        ExportStart {
            folder: shown,
            choices: ExportChoices::from(&remembered),
            cloud_only,
            cloud_size,
        }
    }

    pub fn destination(&self) -> Option<PathBuf> {
        self.destination.lock().expect("destination lock").clone()
    }

    pub fn choose(&self, folder: PathBuf) {
        *self.destination.lock().expect("destination lock") = Some(folder);
    }

    /// Exports photos to the folder chosen, remembering the folder and the
    /// choices. Photos on the computer go first. One batch at a time.
    pub fn run<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        ids: &[PhotoId],
        choices: &ExportChoices,
    ) -> Result<ExportReport, String> {
        if !choices.valid() {
            return Err("those export choices aren't valid".into());
        }
        let folder = self.destination().ok_or("no folder is chosen")?;
        remember(app, &folder, choices);
        let ticket = self.downloads.ticket();
        let _turn = self.downloads.turn();
        let library = app.state::<Library>();
        let (cloud, here): (Vec<Item>, Vec<Item>) = {
            for &id in ids {
                library.downloaded(app, id);
            }
            let index = library.index.read().expect("index lock");
            ids.iter()
                .filter_map(|&id| {
                    let path = index.path(id)?;
                    let photo = index.photo(id)?;
                    Some(Item {
                        id,
                        name: path
                            .file_name()
                            .map(|name| name.to_string_lossy().into_owned())
                            .unwrap_or_default(),
                        path,
                        cloud: photo.placeholder,
                        size: photo.size,
                    })
                })
                .partition(|item| item.cloud)
        };
        let items: Vec<Item> = here.into_iter().chain(cloud).collect();
        let total = items.len() as u32;
        let options = choices.options();
        let (report, written) = run_batch(
            &items,
            || self.downloads.stopped(ticket),
            |at, item, downloading| {
                let _ = ExportProgress {
                    id: item.id.value(),
                    name: item.name.clone(),
                    at,
                    total,
                    downloading,
                    size: item.size as f64,
                }
                .emit(app);
            },
            |item| {
                let outcome = crate::downloads::download(item.path.clone(), || {
                    self.downloads.stopped(ticket)
                });
                if outcome == Downloaded::Yes {
                    library.downloaded(app, item.id);
                }
                outcome
            },
            |item| write_photo(&item.path, &folder, &options, choices.include_keywords),
        );
        *self.written.lock().expect("written lock") = written;
        Ok(report)
    }

    /// Stops the batch: see [`Downloads::stop`]. Files already written stay.
    pub fn stop(&self) {
        self.downloads.stop();
    }

    pub fn written(&self) -> Vec<PathBuf> {
        self.written.lock().expect("written lock").clone()
    }
}

/// The last folder, or none if it's gone; without one, the Pictures folder.
fn starting_folder(remembered: Option<&Path>, pictures: Option<PathBuf>) -> Option<PathBuf> {
    match remembered {
        Some(folder) => Some(folder.to_path_buf()).filter(|folder| folder.is_dir()),
        None => pictures.filter(|folder| folder.is_dir()),
    }
}

/// Saves the folder and the choices in `settings.json`.
fn remember<R: Runtime>(app: &AppHandle<R>, folder: &Path, choices: &ExportChoices) {
    let settings = app.state::<AppSettings>();
    let mut file = settings.0.lock().expect("settings lock");
    file.settings.export = ExportSettings {
        folder: Some(folder.to_path_buf()),
        format: if choices.png {
            ExportFileFormat::Png
        } else {
            ExportFileFormat::Jpeg
        },
        quality: choices.quality,
        long_edge: choices.long_edge,
        remove_location: choices.remove_location,
        include_keywords: choices.include_keywords,
    };
    // Not being able to remember doesn't stop the export.
    let _ = file.save();
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fs;

    use seura_image::{Metadata, Rgba8, encode_jpeg};

    use super::*;

    fn item(name: &str, cloud: bool) -> Item {
        Item {
            id: PhotoId::new(0),
            path: PathBuf::from(name),
            name: name.into(),
            cloud,
            size: 1000,
        }
    }

    fn written_as(item: &Item) -> Result<PathBuf, Failure> {
        Ok(PathBuf::from(format!("out/{}", item.name)))
    }

    fn failed(name: &str, problem: PhotoProblem) -> ExportFailure {
        ExportFailure {
            name: name.into(),
            problem,
        }
    }

    #[test]
    fn every_photo_is_exported_in_turn() {
        let items = [item("a", false), item("b", false)];
        let (report, written) = run_batch(
            &items,
            || false,
            |_, _, _| {},
            |_| panic!("nothing to download"),
            written_as,
        );
        assert_eq!(report.exported, 2);
        assert_eq!(report.file.as_deref(), Some("a"));
        assert_eq!(written, [PathBuf::from("out/a"), PathBuf::from("out/b")]);
    }

    #[test]
    fn photos_whose_names_were_taken_are_counted_exp_3() {
        let items = [item("IMG_1.heic", false), item("IMG_2.heic", false)];
        let (report, _) = run_batch(
            &items,
            || false,
            |_, _, _| {},
            |_| Downloaded::Yes,
            |item| {
                Ok(PathBuf::from(if item.name == "IMG_1.heic" {
                    "out/IMG_1 (1).jpg"
                } else {
                    "out/IMG_2.jpg"
                }))
            },
        );
        assert_eq!((report.exported, report.renamed), (2, 1));
    }

    #[test]
    fn one_unreadable_photo_is_reported_and_the_others_go_on_exp_6() {
        let items = [item("a", false), item("b", false), item("c", false)];
        let (report, written) = run_batch(
            &items,
            || false,
            |_, _, _| {},
            |_| Downloaded::Yes,
            |item| {
                if item.name == "b" {
                    Err(Failure::Photo(PhotoProblem::Unreadable))
                } else {
                    written_as(item)
                }
            },
        );
        assert_eq!(report.exported, 2);
        assert_eq!(report.failures, [failed("b", PhotoProblem::Unreadable)]);
        assert_eq!(written.len(), 2);
    }

    #[test]
    fn a_folder_problem_stops_the_batch_and_says_how_many_are_left() {
        let items = [item("a", false), item("b", false), item("c", false)];
        let tried = RefCell::new(0);
        let (report, _) = run_batch(
            &items,
            || false,
            |_, _, _| {},
            |_| Downloaded::Yes,
            |item| {
                *tried.borrow_mut() += 1;
                if item.name == "b" {
                    Err(Failure::Folder(FolderProblem::Full))
                } else {
                    written_as(item)
                }
            },
        );
        assert_eq!(*tried.borrow(), 2, "the third wasn't tried");
        assert_eq!(report.exported, 1);
        assert_eq!(report.folder_problem, Some(FolderProblem::Full));
        assert_eq!(report.left, 2);
    }

    #[test]
    fn cancel_stops_between_photos_and_keeps_what_was_written() {
        let items = [item("a", false), item("b", false), item("c", false)];
        let count = RefCell::new(0);
        let (report, written) = run_batch(
            &items,
            || *count.borrow() == 1,
            |_, _, _| {},
            |_| Downloaded::Yes,
            |item| {
                *count.borrow_mut() += 1;
                written_as(item)
            },
        );
        assert_eq!(report.exported, 1);
        assert_eq!(report.stopped, 2);
        assert!(!report.still_downloading);
        assert_eq!(written.len(), 1);
    }

    #[test]
    fn a_cloud_photo_shows_as_downloading_and_then_as_exporting_lib_9() {
        let items = [item("a", true)];
        let seen = RefCell::new(Vec::new());
        let (report, _) = run_batch(
            &items,
            || false,
            |at, _, downloading| seen.borrow_mut().push((at, downloading)),
            |_| Downloaded::Yes,
            written_as,
        );
        assert_eq!(report.exported, 1);
        assert_eq!(*seen.borrow(), [(1, true), (1, false)]);
    }

    #[test]
    fn a_failed_download_is_reported_and_the_others_go_on() {
        let items = [item("a", true), item("b", true)];
        let (report, _) = run_batch(
            &items,
            || false,
            |_, _, _| {},
            |item| {
                if item.name == "a" {
                    Downloaded::Failed
                } else {
                    Downloaded::Yes
                }
            },
            written_as,
        );
        assert_eq!(report.exported, 1);
        assert_eq!(report.failures, [failed("a", PhotoProblem::NotDownloaded)]);
    }

    #[test]
    fn cancel_during_a_download_stops_waiting_for_it_exp_6() {
        let items = [item("a", false), item("b", true), item("c", true)];
        let (report, _) = run_batch(
            &items,
            || false,
            |_, _, _| {},
            |_| Downloaded::Stopped,
            written_as,
        );
        assert_eq!(report.exported, 1);
        assert_eq!(report.stopped, 2);
        assert!(report.still_downloading);
    }

    #[test]
    fn the_first_export_starts_in_pictures_and_a_lost_folder_starts_empty_exp_1() {
        let there = tempfile::tempdir().unwrap();
        let pictures = there.path().to_path_buf();
        assert_eq!(
            starting_folder(None, Some(pictures.clone())),
            Some(pictures.clone())
        );
        assert_eq!(starting_folder(None, Some(pictures.join("gone"))), None);
        assert_eq!(starting_folder(None, None), None);
        let last = there.path().join("Exports");
        fs::create_dir(&last).unwrap();
        assert_eq!(
            starting_folder(Some(&last), Some(pictures.clone())),
            Some(last.clone())
        );
        fs::remove_dir(&last).unwrap();
        // Empty, not Pictures: Export waits for the user to choose.
        assert_eq!(starting_folder(Some(&last), Some(pictures)), None);
    }

    #[test]
    fn choices_are_remembered_as_they_were_and_tidied_if_not_usable() {
        let settings = ExportSettings {
            folder: None,
            format: ExportFileFormat::Png,
            quality: 75,
            long_edge: Some(1600),
            remove_location: true,
            include_keywords: false,
        };
        let choices = ExportChoices::from(&settings);
        assert!(choices.png && choices.remove_location && !choices.include_keywords);
        assert_eq!((choices.quality, choices.long_edge), (75, Some(1600)));
        assert!(choices.valid());

        // A settings file edited by hand.
        let odd = ExportChoices::from(&ExportSettings {
            quality: 0,
            long_edge: Some(3),
            ..settings
        });
        assert_eq!((odd.quality, odd.long_edge), (1, None));
        assert!(odd.valid());
    }

    #[test]
    fn choices_become_the_render_s_options_and_out_of_range_ones_are_refused() {
        let choices = ExportChoices {
            png: false,
            quality: 80,
            long_edge: Some(1024),
            remove_location: true,
            include_keywords: false,
        };
        let options = choices.options();
        assert_eq!(options.format, ExportFormat::Jpeg { quality: 80 });
        assert_eq!(options.size, ExportSize::LongEdge(1024));
        assert!(options.remove_location);
        for bad in [
            ExportChoices {
                quality: 0,
                ..choices.clone()
            },
            ExportChoices {
                long_edge: Some(15),
                ..choices.clone()
            },
        ] {
            assert!(!bad.valid());
        }
    }

    fn photo(folder: &Path) -> PathBuf {
        let path = folder.join("IMG_1.jpg");
        let image = Rgba8::new(32, 24, [90, 90, 90, 255].repeat(32 * 24));
        fs::write(
            &path,
            encode_jpeg(&image, 95, &Metadata::default()).unwrap(),
        )
        .unwrap();
        path
    }

    /// An edit file with this exposure and these tags beside the photo.
    fn edit_file(photo: &Path) {
        let text = r#"{
            "format": "seura-edit",
            "formatVersion": "1.0",
            "writer": { "app": "Test", "version": "1" },
            "modified": "2026-10-29T12:00:00Z",
            "photo": { "file": "IMG_1.jpg", "size": 1 },
            "operations": [
                { "id": "seura.exposure", "version": 1, "params": { "ev": 1.0 } }
            ],
            "tags": ["family/kids"],
            "rating": 4
        }"#;
        fs::write(seura_edit::edit_file_path(photo), text).unwrap();
    }

    #[test]
    fn a_photo_exports_with_its_saved_edits() {
        let folder = tempfile::tempdir().unwrap();
        let out = tempfile::tempdir().unwrap();
        let path = photo(folder.path());
        let plain = write_photo(&path, out.path(), &ExportOptions::default(), false).unwrap();
        edit_file(&path);
        let edited = write_photo(&path, out.path(), &ExportOptions::default(), false).unwrap();
        assert_eq!(plain, out.path().join("IMG_1.jpg"));
        assert_eq!(edited, out.path().join("IMG_1 (1).jpg"));
        // sRGB 90, one stop brighter, is about 125 (JPEG adds a little error).
        let value = |path: &Path| seura_image::decode(path).unwrap().image.pixels[0];
        assert!(value(&plain).abs_diff(90) <= 3);
        assert!(value(&edited).abs_diff(125) <= 3);
        // Nothing was written beside the photo but its edit file.
        assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 2);
    }

    #[test]
    fn tags_and_rating_go_in_only_when_chosen() {
        let folder = tempfile::tempdir().unwrap();
        let out = tempfile::tempdir().unwrap();
        let path = photo(folder.path());
        edit_file(&path);
        let holds = |path: &Path, text: &str| {
            fs::read(path)
                .unwrap()
                .windows(text.len())
                .any(|window| window == text.as_bytes())
        };
        let with = write_photo(&path, out.path(), &ExportOptions::default(), true).unwrap();
        assert!(holds(&with, "<xmp:Rating>4</xmp:Rating>"));
        assert!(holds(&with, "family|kids"));
        let without = write_photo(&path, out.path(), &ExportOptions::default(), false).unwrap();
        assert!(!holds(&without, "xmpmeta"));
    }

    #[test]
    fn a_photo_whose_edits_cant_be_read_isnt_exported_without_them_fil_4() {
        let folder = tempfile::tempdir().unwrap();
        let out = tempfile::tempdir().unwrap();
        let path = photo(folder.path());
        fs::write(seura_edit::edit_file_path(&path), "not an edit file").unwrap();
        assert_eq!(
            write_photo(&path, out.path(), &ExportOptions::default(), false),
            Err(Failure::Photo(PhotoProblem::EditsUnreadable))
        );
        assert_eq!(fs::read_dir(out.path()).unwrap().count(), 0);
    }

    #[test]
    fn a_missing_photo_is_a_problem_with_it_and_a_missing_folder_stops_the_batch() {
        let folder = tempfile::tempdir().unwrap();
        let out = tempfile::tempdir().unwrap();
        let options = ExportOptions::default();
        assert_eq!(
            write_photo(&folder.path().join("gone.jpg"), out.path(), &options, false),
            Err(Failure::Photo(PhotoProblem::Unreadable))
        );
        let path = photo(folder.path());
        assert_eq!(
            write_photo(&path, &out.path().join("missing"), &options, false),
            Err(Failure::Folder(FolderProblem::NotWritable))
        );
    }

    #[test]
    fn a_full_drive_is_told_apart_from_a_folder_that_cant_be_written() {
        let error = |kind| ExportError::Write(std::io::Error::from(kind));
        assert_eq!(
            failure(&error(std::io::ErrorKind::StorageFull)),
            Failure::Folder(FolderProblem::Full)
        );
        assert_eq!(
            failure(&error(std::io::ErrorKind::PermissionDenied)),
            Failure::Folder(FolderProblem::NotWritable)
        );
    }
}
