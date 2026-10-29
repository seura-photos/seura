//! The photo open in the Editor: its preview in the working space, the
//! changes the Editor makes (saved through the Library's one write path,
//! EDT-11), and M1's development-only export. Also the render check that
//! compares the preview with export (M1 slice 5).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use seura_edit::{
    Action, CoreOpId, CoreOperation, Edit, EditDocument, EditFile, Exposure, Step, Writer,
    edit_file_path,
};
use seura_image::{LinearImage, Rgba8};
use seura_library::{PhotoId, Problem};
use seura_render::check::{CHECK_SETTINGS, CHECK_TOLERANCE, check_image, difference};
use seura_render::{Output, Settings, export_jpeg};
use specta::Type;
use tauri::{AppHandle, Runtime};
use tauri_specta::Event;

/// Recorded in every edit file Seura writes.
pub(crate) const WRITER: Writer = Writer {
    app: "Seura Photos",
    version: env!("CARGO_PKG_VERSION"),
};

/// The preview's long edge is kept within this, whatever the window.
const MAX_PREVIEW: u32 = 8192;

/// Exposure's range in the edit format.
const EXPOSURE_RANGE: (f64, f64) = (-4.0, 4.0);

#[derive(Default)]
pub struct Editor {
    session: Mutex<Option<Session>>,
    /// The render check's results so far: backend, and whether it passed.
    checks: Mutex<Vec<(String, bool)>>,
}

struct Session {
    id: u32,
    photo: PathBuf,
    /// The preview pixels, as served by the `seura://` protocol.
    preview: Arc<Vec<u8>>,
    /// The document of its edit file, while the file is in the Trash
    /// because undo left nothing in it, for redo to write back.
    kept: Option<EditDocument>,
    /// Seura's operations as the Editor shows them.
    mine: Mine,
    /// Whether its edit file changed outside Seura to something else, and
    /// the user hasn't yet chosen to keep theirs or load the other (FIL-3).
    outside: bool,
}

/// Seura's operations as the Editor last saved or loaded them, in canonical
/// order: what "Keep yours" writes back when the edit file changes outside
/// Seura (FIL-3).
#[derive(Clone, Debug, Default, PartialEq)]
struct Mine(Vec<Option<CoreOperation>>);

impl Mine {
    fn of(document: Option<&EditDocument>) -> Mine {
        Mine(
            CoreOpId::ALL
                .iter()
                .map(|id| document.and_then(|document| document.core(*id)))
                .collect(),
        )
    }

    fn get(&self, id: CoreOpId) -> Option<&CoreOperation> {
        let at = CoreOpId::ALL.iter().position(|each| *each == id)?;
        self.0.get(at)?.as_ref()
    }

    /// The Editor's change, made to these as the edit format makes it to
    /// a file: a neutral setting removes the operation.
    fn apply(&mut self, action: &Action) {
        if self.0.is_empty() {
            *self = Mine::of(None);
        }
        for edit in &action.edits {
            let (id, operation) = match edit {
                Edit::Set(operation) => (
                    operation.id(),
                    (!operation.is_neutral()).then(|| operation.clone()),
                ),
                Edit::Remove(id) => (*id, None),
                Edit::Tags(_) | Edit::Rating(_) => continue,
            };
            if let Some(at) = CoreOpId::ALL.iter().position(|each| *each == id) {
                self.0[at] = operation;
            }
        }
    }

    /// The one change that makes `theirs` show these again. Tags, rating,
    /// and other apps' operations are theirs, and stay.
    fn keep(&self, theirs: Option<&EditDocument>) -> Action {
        let theirs = Mine::of(theirs);
        Action {
            label: KEEP.into(),
            edits: CoreOpId::ALL
                .iter()
                .filter(|id| self.get(**id) != theirs.get(**id))
                .map(|id| match self.get(*id) {
                    Some(operation) => Edit::Set(operation.clone()),
                    None => Edit::Remove(*id),
                })
                .collect(),
        }
    }
}

/// "Keep yours", in the history.
const KEEP: &str = "Keep my edits";
/// "Load theirs", which isn't in the history: nothing is written.
const LOAD: &str = "Load theirs";

/// The open photo's edit file changed outside Seura (FIL-3): `conflict`
/// when Seura's operations in it are no longer what the Editor shows, so
/// the user chooses to keep theirs or load the other. Otherwise only its
/// tags, rating, or history changed, and undo and redo are as `steps`.
#[derive(Clone, Debug, Serialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct EditFileChanged {
    pub id: u32,
    pub conflict: bool,
    pub steps: EditSteps,
}

/// What the Editor's Undo and Redo would undo or redo next, by the labels
/// of the changes in the photo's own history (EDT-9). `None` when there's
/// nothing.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditSteps {
    pub undo: Option<String>,
    pub redo: Option<String>,
}

impl EditSteps {
    fn of(document: Option<&EditDocument>) -> EditSteps {
        let label = |record: Option<serde_json::Value>| {
            record.and_then(|record| record.get("label")?.as_str().map(str::to_string))
        };
        EditSteps {
            undo: document.and_then(|document| label(document.last_change())),
            redo: document.and_then(|document| label(document.next_redo())),
        }
    }
}

/// A change to a photo's rating or tags, which the Editor doesn't show, so
/// the message after undoing or redoing it says what it was.
#[derive(Clone, Debug, PartialEq, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ChangeDetail {
    /// The rating was set to `stars`; 0 is removing it.
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

impl ChangeDetail {
    /// The detail of a history record that changed only the rating, or
    /// added or removed one tag; `None` for anything else, which its label
    /// names well enough.
    fn of(record: &serde_json::Value) -> Option<ChangeDetail> {
        let [change] = record.get("changes")?.as_array()?.as_slice() else {
            return None;
        };
        let tags = |side: &str| -> Option<Vec<&str>> {
            change
                .get(side)?
                .as_array()?
                .iter()
                .map(serde_json::Value::as_str)
                .collect()
        };
        match change.get("field")?.as_str()? {
            "rating" => Some(ChangeDetail::Rating {
                stars: u8::try_from(change.get("to")?.as_u64()?).ok()?,
            }),
            "tags" => {
                let (from, to) = (tags("from")?, tags("to")?);
                let added: Vec<_> = to.iter().filter(|tag| !from.contains(tag)).collect();
                let removed: Vec<_> = from.iter().filter(|tag| !to.contains(tag)).collect();
                match (added.as_slice(), removed.as_slice()) {
                    ([tag], []) => Some(ChangeDetail::AddTag {
                        tag: tag.to_string(),
                    }),
                    ([], [tag]) => Some(ChangeDetail::RemoveTag {
                        tag: tag.to_string(),
                    }),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

/// What undoing or redoing a change in the Editor did, and the photo now.
pub struct Stepped {
    pub step: Step,
    /// The change undone or redone, when it was to the rating or tags.
    pub detail: Option<ChangeDetail>,
    pub settings: Settings,
    pub steps: EditSteps,
}

/// What the UI needs to show an opened photo.
pub struct Opened {
    pub width: u32,
    pub height: u32,
    pub settings: Settings,
    pub steps: EditSteps,
    pub problem: Option<String>,
}

impl Editor {
    /// Opens photo `id` of the index, at `photo`: decodes its preview at
    /// `long_edge` and reads its edit file, if any. Only reads (CLAUDE.md
    /// rule 1).
    pub fn open(&self, id: u32, photo: PathBuf, long_edge: u32) -> Result<Opened, String> {
        let preview = seura_image::decode_linear(&photo, Some(long_edge.clamp(1, MAX_PREVIEW)))
            .map_err(|error| error.to_string())?;
        let photo_path = photo.clone();
        let ((settings, steps), problem) = match saved(&photo, None) {
            Ok(saved) => (saved, None),
            Err(problem) => ((Settings::default(), EditSteps::default()), Some(problem)),
        };
        *self.session.lock().expect("editor lock") = Some(Session {
            id,
            photo,
            preview: Arc::new(encode_linear(&preview)),
            kept: None,
            mine: Mine::of(current(&photo_path, None).ok().flatten().as_ref()),
            outside: false,
        });
        Ok(Opened {
            width: preview.width,
            height: preview.height,
            settings,
            steps,
            problem,
        })
    }

    /// Closes photo `id`, unless another has been opened since: moving
    /// through the filmstrip opens the next photo before the last one's
    /// close arrives.
    /// Returns whether it closed.
    pub fn close(&self, id: u32) -> bool {
        let mut session = self.session.lock().expect("editor lock");
        let open = session.as_ref().is_some_and(|session| session.id == id);
        if open {
            *session = None;
        }
        open
    }

    /// Saves a change to photo `id` (EDT-11) through the Library's one
    /// write path, going on from the document kept for undo if its file
    /// went to the Trash. Returns what undo and redo would do now.
    pub fn edit<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        library: &crate::library::Library,
        id: u32,
        action: &Action,
    ) -> Result<EditSteps, Problem> {
        let mut session = self.session.lock().expect("editor lock");
        let session = session
            .as_mut()
            .filter(|session| session.id == id)
            .ok_or_else(|| Problem::Failed("that photo isn't open".into()))?;
        library.edit_with(app, PhotoId::new(id), &mut session.kept, |_| action.clone())?;
        let document = current(&session.photo, session.kept.as_ref()).map_err(Problem::Failed)?;
        // While the user hasn't chosen, theirs is the change on top of the
        // other edits; their own stay as the Editor shows them.
        if session.outside {
            session.mine.apply(action);
        } else {
            session.mine = Mine::of(document.as_ref());
        }
        Ok(EditSteps::of(document.as_ref()))
    }

    /// The watcher read these photos' edit files again (FIL-3): if one is
    /// open here, says how it changed. Seura's own writes aren't reported,
    /// and the Editor's lock is held while it writes, so it's never taken
    /// for another app's.
    pub fn changed_outside<R: Runtime>(&self, app: &AppHandle<R>, ids: &[PhotoId]) {
        let mut session = self.session.lock().expect("editor lock");
        let Some(session) = session
            .as_mut()
            .filter(|session| ids.contains(&PhotoId::new(session.id)))
        else {
            return;
        };
        let document = current(&session.photo, None);
        let theirs = document.as_ref().ok().and_then(Option::as_ref);
        // A file that can't be read isn't the Editor's to show, either.
        if document.is_err() || Mine::of(theirs) != session.mine {
            session.outside = true;
        }
        if document.as_ref().is_ok_and(Option::is_some) {
            session.kept = None;
        }
        let _ = EditFileChanged {
            id: session.id,
            conflict: session.outside,
            steps: EditSteps::of(theirs),
        }
        .emit(app);
    }

    /// After the edit file changed outside Seura (FIL-3): with `keep`,
    /// writes the Editor's edits back over the other, as one change that
    /// can be undone; otherwise shows the other.
    pub fn resolve<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        library: &crate::library::Library,
        id: u32,
        keep: bool,
    ) -> Result<Stepped, Problem> {
        let mut session = self.session.lock().expect("editor lock");
        let session = session
            .as_mut()
            .filter(|session| session.id == id)
            .ok_or_else(|| Problem::Failed("that photo isn't open".into()))?;
        let mine = session.mine.clone();
        let changed = keep
            && library.edit_with(app, PhotoId::new(id), &mut session.kept, |loaded| {
                mine.keep(loaded.map(|file| &file.document))
            })?;
        let document = current(&session.photo, session.kept.as_ref()).map_err(Problem::Failed)?;
        let document = document.as_ref();
        session.mine = Mine::of(document);
        session.outside = false;
        // Loading the other changes nothing on disk, but what's shown.
        Ok(Stepped {
            step: Step::Done(if changed { KEEP } else { LOAD }.into()),
            detail: None,
            settings: document.map(Settings::from_document).unwrap_or_default(),
            steps: EditSteps::of(document),
        })
    }

    /// The open photo, unless its edit file changed outside Seura and the
    /// user hasn't chosen yet: undo, redo, and revert work on the file's
    /// history, which is the other's now.
    fn settled(session: &mut Option<Session>, id: u32) -> Result<&mut Session, Problem> {
        let session = session
            .as_mut()
            .filter(|session| session.id == id)
            .ok_or_else(|| Problem::Failed("that photo isn't open".into()))?;
        if session.outside {
            return Err(Problem::ChangedOnDisk);
        }
        Ok(session)
    }

    /// Reverts photo `id` to the original (EDT-10), through the Library's
    /// one write path: one change in its history, so undo brings the edits
    /// back. Tags and rating stay.
    pub fn revert<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        library: &crate::library::Library,
        id: u32,
    ) -> Result<Stepped, Problem> {
        let mut session = self.session.lock().expect("editor lock");
        let session = Editor::settled(&mut session, id)?;
        let changed = library.edit_with(app, PhotoId::new(id), &mut session.kept, |loaded| {
            seura_library::revert_action(loaded.map(|file| &file.document))
        })?;
        let document = current(&session.photo, session.kept.as_ref()).map_err(Problem::Failed)?;
        let document = document.as_ref();
        session.mine = Mine::of(document);
        Ok(Stepped {
            step: if changed {
                Step::Done(seura_library::REVERT.into())
            } else {
                Step::Nothing
            },
            detail: None,
            settings: document.map(Settings::from_document).unwrap_or_default(),
            steps: EditSteps::of(document),
        })
    }

    /// Undoes or redoes the last change in photo `id`'s history (EDT-9),
    /// through the Library's one write path.
    pub fn step<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        library: &crate::library::Library,
        id: u32,
        undo: bool,
    ) -> Result<Stepped, Problem> {
        let mut session = self.session.lock().expect("editor lock");
        let session = Editor::settled(&mut session, id)?;
        let step = library.step_edit(app, PhotoId::new(id), undo, &mut session.kept)?;
        let document = current(&session.photo, session.kept.as_ref()).map_err(Problem::Failed)?;
        let document = document.as_ref();
        session.mine = Mine::of(document);
        // The record just stepped: after undo the first undone, after redo
        // the last that isn't.
        let record = document.and_then(|document| {
            if undo {
                document.next_redo()
            } else {
                document.last_change()
            }
        });
        Ok(Stepped {
            detail: match step {
                Step::Done(_) => record.as_ref().and_then(ChangeDetail::of),
                _ => None,
            },
            step,
            settings: document.map(Settings::from_document).unwrap_or_default(),
            steps: EditSteps::of(document),
        })
    }

    /// The open photo's preview pixels, for the protocol.
    pub fn preview(&self) -> Option<Arc<Vec<u8>>> {
        let session = self.session.lock().expect("editor lock");
        session.as_ref().map(|session| Arc::clone(&session.preview))
    }

    /// Exports the open photo with its edits into `folder` (EXP-1).
    pub fn export(&self, id: u32, folder: &Path) -> Result<PathBuf, String> {
        let (photo, settings) = {
            let session = self.session.lock().expect("editor lock");
            let session = session
                .as_ref()
                .filter(|session| session.id == id)
                .ok_or("that photo isn't open")?;
            (session.photo.clone(), saved(&session.photo, None)?.0)
        };
        crate::gpu::with_renderer(|renderer| export_jpeg(renderer, &photo, &settings, folder))?
            .map_err(|error| error.to_string())
    }

    /// Compares a preview path's render of the check image with wgpu's, and
    /// records the result. Returns the largest difference.
    pub fn check_render(
        &self,
        backend: String,
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    ) -> Result<u8, String> {
        let expected = crate::gpu::with_renderer(|renderer| {
            renderer.render(&check_image(), &CHECK_SETTINGS, Output::Srgb)
        })?
        .map_err(|error| error.to_string())?;
        if pixels.len() != width as usize * height as usize * 4 {
            return Err("the pixels don't match the size".into());
        }
        let found = Rgba8::new(width, height, pixels);
        let (off, at) = difference(&found, &expected).ok_or_else(|| {
            format!(
                "rendered {width}×{height}, expected {}×{}",
                expected.width, expected.height
            )
        })?;
        let passed = off <= CHECK_TOLERANCE;
        println!(
            "render check, {backend}: largest difference {off} (pixel {at}): {}",
            if passed { "passed" } else { "FAILED" }
        );
        self.checks
            .lock()
            .expect("check lock")
            .push((backend, passed));
        Ok(off)
    }

    /// Whether at least one preview path was checked and all passed.
    pub fn checks_passed(&self) -> bool {
        let checks = self.checks.lock().expect("check lock");
        !checks.is_empty() && checks.iter().all(|(_, passed)| *passed)
    }
}

/// The settings in a photo's edit file, neutral without one, and what undo
/// and redo would do, from its history or, without a file, from `kept`. An
/// error if the file can't be read: the photo can't be edited then, and its
/// file is never overwritten (FIL-4; CLAUDE.md rule 4).
fn saved(photo: &Path, kept: Option<&EditDocument>) -> Result<(Settings, EditSteps), String> {
    let document = current(photo, kept)?;
    Ok((
        document
            .as_ref()
            .map(Settings::from_document)
            .unwrap_or_default(),
        EditSteps::of(document.as_ref()),
    ))
}

/// The photo's edit document: its file's, or without a file, `kept`.
fn current(photo: &Path, kept: Option<&EditDocument>) -> Result<Option<EditDocument>, String> {
    let file = EditFile::load(&edit_file_path(photo)).map_err(|error| error.to_string())?;
    Ok(file.map(|file| file.document).or_else(|| kept.cloned()))
}

/// The action for a brightness setting (EDT-5): one record in the photo's
/// history; a neutral setting removes the operation.
pub fn exposure_action(ev: f64) -> Action {
    Action {
        label: "Brightness".into(),
        edits: vec![Edit::Set(CoreOperation::Exposure(Exposure {
            ev: ev.clamp(EXPOSURE_RANGE.0, EXPOSURE_RANGE.1),
        }))],
    }
}

/// Working-space pixels as the preview fetches them: width and height as
/// little-endian `u32`s, then RGBA half floats, little-endian.
pub fn encode_linear(image: &LinearImage) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8 + image.pixels.len() * 2);
    bytes.extend(image.width.to_le_bytes());
    bytes.extend(image.height.to_le_bytes());
    for value in &image.pixels {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

/// The check image, encoded for the protocol.
pub fn check_image_bytes() -> Vec<u8> {
    encode_linear(&check_image())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use half::f16;
    use seura_edit::EditDocument;

    use super::*;

    /// A gray JPEG in a new folder, and the folder.
    fn photo() -> (tempfile::TempDir, PathBuf) {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("IMG_1.jpg");
        let image = Rgba8::new(32, 24, [90, 90, 90, 255].repeat(32 * 24));
        fs::write(&path, seura_image::encode_jpeg(&image, 95).unwrap()).unwrap();
        (folder, path)
    }

    /// Writes an edit file with this exposure beside the photo.
    fn save_exposure(photo: &Path, ev: f64) {
        let identity = seura_edit::PhotoIdentity {
            file: "IMG_1.jpg".into(),
            size: fs::metadata(photo).unwrap().len(),
            sha256: None,
            width: None,
            height: None,
        };
        let mut file = EditFile::create(
            &edit_file_path(photo),
            EditDocument::new_for_photo(&identity),
        );
        let now = time::OffsetDateTime::now_utc();
        file.document.apply(&exposure_action(ev), now).unwrap();
        file.save(WRITER, now).unwrap();
    }

    /// A document with these operations, as another app might leave it.
    fn document(operations: serde_json::Value) -> EditDocument {
        let root = serde_json::json!({
            "format": "seura-edit",
            "formatVersion": "1.0",
            "writer": { "app": "Another app", "version": "1" },
            "modified": "2026-10-29T12:00:00Z",
            "photo": { "file": "IMG_1.jpg", "size": 1 },
            "operations": operations,
            "rating": 4
        });
        EditDocument::parse(&serde_json::to_vec(&root).unwrap()).unwrap()
    }

    fn exposure(ev: f64) -> CoreOperation {
        CoreOperation::Exposure(Exposure { ev })
    }

    #[test]
    fn keeping_yours_writes_back_only_seura_s_operations_that_differ_fil_3() {
        let saved = document(serde_json::json!([
            { "id": "seura.exposure", "version": 1, "params": { "ev": 0.5 } }
        ]));
        let mut mine = Mine::of(Some(&saved));
        // A change made while the user hadn't chosen yet.
        mine.apply(&exposure_action(1.0));
        assert_eq!(mine.get(CoreOpId::Exposure), Some(&exposure(1.0)));

        // The other: brightness changed, contrast added, another app's
        // operation added.
        let theirs = document(serde_json::json!([
            { "id": "seura.exposure", "version": 1, "params": { "ev": 2.0 } },
            { "id": "seura.contrast", "version": 1, "params": { "amount": 30 } },
            { "id": "com.example.grain", "version": 1, "params": {}, "group": "effects" }
        ]));
        assert_ne!(Mine::of(Some(&theirs)), mine);
        let keep = mine.keep(Some(&theirs));
        assert_eq!(keep.label, KEEP);
        assert_eq!(
            keep.edits,
            [Edit::Set(exposure(1.0)), Edit::Remove(CoreOpId::Contrast)]
        );
        // A neutral setting removes it, as in the file.
        mine.apply(&exposure_action(0.0));
        assert_eq!(mine.get(CoreOpId::Exposure), None);
        assert_eq!(mine, Mine::of(None));
    }

    #[test]
    fn only_tags_or_rating_changed_isn_t_a_conflict_fil_3() {
        let ours = document(serde_json::json!([
            { "id": "seura.exposure", "version": 1, "params": { "ev": 0.5 } }
        ]));
        let mut theirs = ours.clone();
        theirs
            .apply(
                &Action {
                    label: "Rating".into(),
                    edits: vec![Edit::Rating(1)],
                },
                time::OffsetDateTime::now_utc(),
            )
            .unwrap();
        assert_eq!(Mine::of(Some(&ours)), Mine::of(Some(&theirs)));
        assert!(mine_keeps_nothing(&Mine::of(Some(&ours)), &theirs));
    }

    fn mine_keeps_nothing(mine: &Mine, theirs: &EditDocument) -> bool {
        mine.keep(Some(theirs)).edits.is_empty()
    }

    #[test]
    fn opening_a_photo_writes_nothing() {
        let (folder, photo) = photo();
        let editor = Editor::default();
        let opened = editor.open(1, photo, 16).unwrap();
        assert_eq!((opened.width, opened.height), (16, 12));
        assert_eq!(opened.settings, Settings::NEUTRAL);
        assert_eq!(opened.problem, None);
        assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 1);
    }

    #[test]
    fn reopened_a_photo_shows_what_was_saved_edt_8() {
        let (_folder, photo) = photo();
        save_exposure(&photo, 1.25);
        let opened = Editor::default().open(1, photo, 16).unwrap();
        assert_eq!(opened.settings.exposure_ev, 1.25);
    }

    #[test]
    fn a_rating_or_tag_change_is_described_for_the_message() {
        let record = |changes: serde_json::Value| serde_json::json!({ "changes": changes });
        assert_eq!(
            ChangeDetail::of(&record(serde_json::json!([
                { "field": "rating", "from": 0, "to": 3 }
            ]))),
            Some(ChangeDetail::Rating { stars: 3 })
        );
        assert_eq!(
            ChangeDetail::of(&record(serde_json::json!([
                { "field": "tags", "from": ["sea"], "to": ["sea", "beach"] }
            ]))),
            Some(ChangeDetail::AddTag {
                tag: "beach".into()
            })
        );
        assert_eq!(
            ChangeDetail::of(&record(serde_json::json!([
                { "field": "tags", "from": ["sea", "beach"], "to": ["beach"] }
            ]))),
            Some(ChangeDetail::RemoveTag { tag: "sea".into() })
        );
        // An edit, or several changes at once, is named by its label.
        assert_eq!(
            ChangeDetail::of(&record(serde_json::json!([
                { "op": "seura.exposure", "from": null, "to": { "ev": 1.0 } }
            ]))),
            None
        );
        assert_eq!(
            ChangeDetail::of(&record(serde_json::json!([
                { "field": "rating", "from": 0, "to": 3 },
                { "field": "tags", "from": [], "to": ["sea"] }
            ]))),
            None
        );
    }

    #[test]
    fn brightness_is_clamped_to_the_format_s_range() {
        assert_eq!(
            exposure_action(9.0).edits,
            [Edit::Set(CoreOperation::Exposure(Exposure { ev: 4.0 }))]
        );
    }

    #[test]
    fn an_unreadable_edit_file_is_reported_and_left_alone() {
        let (_folder, photo) = photo();
        let edit_file = edit_file_path(&photo);
        fs::write(&edit_file, b"{ not json").unwrap();

        let opened = Editor::default().open(1, photo, 16).unwrap();
        assert!(opened.problem.is_some());
        assert_eq!(fs::read(&edit_file).unwrap(), b"{ not json");
    }

    #[test]
    fn closing_an_earlier_photo_keeps_the_one_open_now() {
        let (_folder, photo) = photo();
        let editor = Editor::default();
        editor.open(1, photo.clone(), 16).unwrap();
        editor.open(2, photo, 16).unwrap();
        editor.close(1);
        assert!(editor.preview().is_some());
        editor.close(2);
        assert!(editor.preview().is_none());
    }

    #[test]
    fn export_uses_the_saved_exposure() {
        let (folder, photo) = photo();
        save_exposure(&photo, 1.0);
        let editor = Editor::default();
        editor.open(1, photo, 16).unwrap();
        let out = tempfile::tempdir().unwrap();

        let exported = editor.export(1, out.path()).unwrap();

        assert_eq!(exported, out.path().join("IMG_1.jpg"));
        let pixel = seura_image::decode(&exported).unwrap().image.pixels[..3].to_vec();
        // sRGB 90, one stop brighter, is about 125 (JPEG adds a little error).
        assert!(pixel.iter().all(|v| v.abs_diff(125) <= 3), "{pixel:?}");
        // Nothing was written beside the photo but its edit file.
        assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 2);
    }

    #[test]
    fn preview_bytes_start_with_the_size() {
        let image = LinearImage {
            width: 2,
            height: 1,
            pixels: [1.0, 0.5, 0.0, 1.0, 0.25, 0.0, 0.0, 1.0]
                .map(f16::from_f32)
                .to_vec(),
        };
        let bytes = encode_linear(&image);
        assert_eq!(&bytes[..8], &[2, 0, 0, 0, 1, 0, 0, 0]);
        assert_eq!(bytes.len(), 8 + 8 * 2);
        // 1.0 as a half float is 0x3c00.
        assert_eq!(&bytes[8..10], &[0x00, 0x3c]);
    }
}
