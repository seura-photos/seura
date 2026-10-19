//! The photo open in the Editor (M1: exposure only): its preview in the
//! working space, its edit file and autosave (EDT-11), and export. Also the
//! render check that compares the preview with export (M1 slice 5).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use seura_edit::{
    Action, CoreOperation, Edit, EditDocument, EditFile, Exposure, PhotoIdentity, Writer,
    edit_file_path,
};
use seura_image::{LinearImage, Rgba8};
use seura_library::{fingerprint_file, to_hex};
use seura_render::check::{CHECK_SETTINGS, CHECK_TOLERANCE, check_image, difference};
use seura_render::{Output, Renderer, Settings, export_jpeg};
use time::OffsetDateTime;

const WRITER: Writer = Writer {
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
    /// Made on first use; it takes a moment.
    renderer: OnceLock<Result<Renderer, String>>,
    /// The render check's results so far: backend, and whether it passed.
    checks: Mutex<Vec<(String, bool)>>,
}

struct Session {
    id: u32,
    photo: PathBuf,
    /// Upright pixel dimensions, if the index knows them.
    dimensions: Option<(u32, u32)>,
    /// Made on the first change, so opening a photo writes nothing.
    file: Option<EditFile>,
    /// Why the edit file can't be changed: it exists but can't be read, so
    /// it's never overwritten (CLAUDE.md rule 4).
    problem: Option<String>,
    /// The preview pixels, as served by the `seura://` protocol.
    preview: Arc<Vec<u8>>,
}

/// What the UI needs to show an opened photo.
pub struct Opened {
    pub width: u32,
    pub height: u32,
    pub exposure_ev: f64,
    pub problem: Option<String>,
}

impl Editor {
    /// Opens photo `id` of the index, at `photo`: decodes its preview at
    /// `long_edge` and reads its edit file, if any. Only reads (CLAUDE.md
    /// rule 1).
    pub fn open(
        &self,
        id: u32,
        photo: PathBuf,
        dimensions: Option<(u32, u32)>,
        long_edge: u32,
    ) -> Result<Opened, String> {
        let preview = seura_image::decode_linear(&photo, Some(long_edge.clamp(1, MAX_PREVIEW)))
            .map_err(|error| error.to_string())?;
        let (file, problem) = match EditFile::load(&edit_file_path(&photo)) {
            Ok(file) => (file, None),
            Err(error) => (None, Some(error.to_string())),
        };
        let settings = file
            .as_ref()
            .map(|file| Settings::from_document(&file.document))
            .unwrap_or_default();
        let opened = Opened {
            width: preview.width,
            height: preview.height,
            exposure_ev: f64::from(settings.exposure_ev),
            problem: problem.clone(),
        };
        *self.session.lock().expect("editor lock") = Some(Session {
            id,
            photo,
            dimensions,
            file,
            problem,
            preview: Arc::new(encode_linear(&preview)),
        });
        Ok(opened)
    }

    pub fn close(&self) {
        *self.session.lock().expect("editor lock") = None;
    }

    /// The open photo's preview pixels, for the protocol.
    pub fn preview(&self) -> Option<Arc<Vec<u8>>> {
        let session = self.session.lock().expect("editor lock");
        session.as_ref().map(|session| Arc::clone(&session.preview))
    }

    /// Sets the open photo's exposure and saves the edit file at once, so a
    /// change is on disk well within a second (EDT-11). The UI sends a
    /// change when the slider pauses, so each pause is one history record.
    pub fn set_exposure(&self, id: u32, ev: f64) -> Result<(), String> {
        let mut session = self.session.lock().expect("editor lock");
        let session = session
            .as_mut()
            .filter(|session| session.id == id)
            .ok_or("that photo isn't open")?;
        if let Some(problem) = &session.problem {
            return Err(problem.clone());
        }
        let file = match &mut session.file {
            Some(file) => file,
            None => {
                let identity = identity(&session.photo, session.dimensions)?;
                let document = EditDocument::new_for_photo(&identity);
                session
                    .file
                    .insert(EditFile::create(&edit_file_path(&session.photo), document))
            }
        };
        let ev = ev.clamp(EXPOSURE_RANGE.0, EXPOSURE_RANGE.1);
        let action = Action {
            label: "Exposure".into(),
            // A neutral setting removes the operation.
            edits: vec![Edit::Set(CoreOperation::Exposure(Exposure { ev }))],
        };
        let now = OffsetDateTime::now_utc();
        if file
            .document
            .apply(&action, now)
            .map_err(|error| error.to_string())?
        {
            file.save(WRITER, now).map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    /// Exports the open photo with its edits into `folder` (EXP-1).
    pub fn export(&self, id: u32, folder: &Path) -> Result<PathBuf, String> {
        let (photo, settings) = {
            let session = self.session.lock().expect("editor lock");
            let session = session
                .as_ref()
                .filter(|session| session.id == id)
                .ok_or("that photo isn't open")?;
            let settings = session
                .file
                .as_ref()
                .map(|file| Settings::from_document(&file.document))
                .unwrap_or_default();
            (session.photo.clone(), settings)
        };
        export_jpeg(self.renderer()?, &photo, &settings, folder).map_err(|error| error.to_string())
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
        let expected = self
            .renderer()?
            .render(&check_image(), &CHECK_SETTINGS, Output::Srgb)
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

    fn renderer(&self) -> Result<&Renderer, String> {
        self.renderer
            .get_or_init(|| Renderer::new().map_err(|error| error.to_string()))
            .as_ref()
            .map_err(Clone::clone)
    }
}

/// The photo's identity for a new edit file: name, size, fingerprint, and
/// dimensions if the index knows them (edit format: "Photo identity").
fn identity(photo: &Path, dimensions: Option<(u32, u32)>) -> Result<PhotoIdentity, String> {
    let size = std::fs::metadata(photo)
        .map_err(|error| error.to_string())?
        .len();
    let sha256 = fingerprint_file(photo).map_err(|error| error.to_string())?;
    Ok(PhotoIdentity {
        file: photo
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        size,
        sha256: Some(to_hex(&sha256)),
        width: dimensions.map(|(width, _)| width),
        height: dimensions.map(|(_, height)| height),
    })
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
    use seura_edit::{CoreOpId, EditDocument};

    use super::*;

    /// A gray JPEG in a new folder, and the folder.
    fn photo() -> (tempfile::TempDir, PathBuf) {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("IMG_1.jpg");
        let image = Rgba8::new(32, 24, [90, 90, 90, 255].repeat(32 * 24));
        fs::write(&path, seura_image::encode_jpeg(&image, 95).unwrap()).unwrap();
        (folder, path)
    }

    fn saved(photo: &Path) -> EditDocument {
        EditDocument::parse(&fs::read(edit_file_path(photo)).unwrap()).unwrap()
    }

    #[test]
    fn opening_a_photo_writes_nothing() {
        let (folder, photo) = photo();
        let editor = Editor::default();
        let opened = editor.open(1, photo.clone(), None, 16).unwrap();
        assert_eq!((opened.width, opened.height), (16, 12));
        assert_eq!(opened.exposure_ev, 0.0);
        assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 1);
    }

    #[test]
    fn each_change_is_saved_with_its_history() {
        let (_folder, photo) = photo();
        let original = fs::read(&photo).unwrap();
        let editor = Editor::default();
        editor.open(1, photo.clone(), Some((32, 24)), 16).unwrap();

        editor.set_exposure(1, 0.5).unwrap();
        editor.set_exposure(1, 1.25).unwrap();

        let document = saved(&photo);
        assert_eq!(
            document.core(CoreOpId::Exposure),
            Some(CoreOperation::Exposure(Exposure { ev: 1.25 }))
        );
        assert_eq!(document.history_len(), 2);
        let identity = document.photo().unwrap();
        assert_eq!(identity.file, "IMG_1.jpg");
        assert_eq!(identity.size, original.len() as u64);
        assert!(identity.sha256.is_some());
        assert_eq!((identity.width, identity.height), (Some(32), Some(24)));
        // The photo itself is untouched (CLAUDE.md rule 1).
        assert_eq!(fs::read(&photo).unwrap(), original);
        // For checking with the format's own tool (M1 slice 5's "done when").
        if let Some(out) = std::env::var_os("SEURA_KEEP_EDIT_FILE") {
            fs::copy(edit_file_path(&photo), out).unwrap();
        }

        // Reopened, the photo starts from what was saved.
        let reopened = editor.open(1, photo.clone(), Some((32, 24)), 16).unwrap();
        assert_eq!(reopened.exposure_ev, 1.25);
    }

    #[test]
    fn exposure_is_clamped_to_the_format_s_range() {
        let (_folder, photo) = photo();
        let editor = Editor::default();
        editor.open(1, photo.clone(), None, 16).unwrap();
        editor.set_exposure(1, 9.0).unwrap();
        assert_eq!(
            saved(&photo).core(CoreOpId::Exposure),
            Some(CoreOperation::Exposure(Exposure { ev: 4.0 }))
        );
    }

    #[test]
    fn an_unreadable_edit_file_is_never_overwritten() {
        let (_folder, photo) = photo();
        let edit_file = edit_file_path(&photo);
        fs::write(&edit_file, b"{ not json").unwrap();
        let editor = Editor::default();

        let opened = editor.open(1, photo.clone(), None, 16).unwrap();
        assert!(opened.problem.is_some());
        assert!(editor.set_exposure(1, 0.5).is_err());
        assert_eq!(fs::read(&edit_file).unwrap(), b"{ not json");
    }

    #[test]
    fn another_photo_s_change_is_refused() {
        let (_folder, photo) = photo();
        let editor = Editor::default();
        editor.open(1, photo.clone(), None, 16).unwrap();
        assert!(editor.set_exposure(2, 0.5).is_err());
        assert!(!edit_file_path(&photo).exists());
    }

    #[test]
    fn export_uses_the_saved_exposure() {
        let (folder, photo) = photo();
        let editor = Editor::default();
        editor.open(1, photo.clone(), None, 16).unwrap();
        editor.set_exposure(1, 1.0).unwrap();
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
