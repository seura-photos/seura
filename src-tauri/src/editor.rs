//! The photo open in the Editor: its preview in the working space, the
//! changes the Editor makes (saved through the Library's one write path,
//! EDT-11), and M1's development-only export. Also the render check that
//! compares the preview with export (M1 slice 5).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use seura_edit::{Action, CoreOperation, Edit, EditFile, Exposure, Writer, edit_file_path};
use seura_image::{LinearImage, Rgba8};
use seura_render::check::{CHECK_SETTINGS, CHECK_TOLERANCE, check_image, difference};
use seura_render::{Output, Settings, export_jpeg};

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
}

/// What the UI needs to show an opened photo.
pub struct Opened {
    pub width: u32,
    pub height: u32,
    pub settings: Settings,
    pub problem: Option<String>,
}

impl Editor {
    /// Opens photo `id` of the index, at `photo`: decodes its preview at
    /// `long_edge` and reads its edit file, if any. Only reads (CLAUDE.md
    /// rule 1).
    pub fn open(&self, id: u32, photo: PathBuf, long_edge: u32) -> Result<Opened, String> {
        let preview = seura_image::decode_linear(&photo, Some(long_edge.clamp(1, MAX_PREVIEW)))
            .map_err(|error| error.to_string())?;
        let (settings, problem) = match saved_settings(&photo) {
            Ok(settings) => (settings, None),
            Err(problem) => (Settings::default(), Some(problem)),
        };
        *self.session.lock().expect("editor lock") = Some(Session {
            id,
            photo,
            preview: Arc::new(encode_linear(&preview)),
        });
        Ok(Opened {
            width: preview.width,
            height: preview.height,
            settings,
            problem,
        })
    }

    /// Closes photo `id`, unless another has been opened since: moving
    /// through the filmstrip opens the next photo before the last one's
    /// close arrives.
    pub fn close(&self, id: u32) {
        let mut session = self.session.lock().expect("editor lock");
        if session.as_ref().is_some_and(|session| session.id == id) {
            *session = None;
        }
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
            (session.photo.clone(), saved_settings(&session.photo)?)
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

/// The settings in a photo's edit file, neutral without one. An error if
/// the file can't be read: the photo can't be edited then, and its file is
/// never overwritten (FIL-4; CLAUDE.md rule 4).
fn saved_settings(photo: &Path) -> Result<Settings, String> {
    let file = EditFile::load(&edit_file_path(photo)).map_err(|error| error.to_string())?;
    Ok(file
        .map(|file| Settings::from_document(&file.document))
        .unwrap_or_default())
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
