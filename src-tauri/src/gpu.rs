//! The app's one GPU renderer (decision 0004), shared by export, the render
//! check, and edited thumbnails (BRW-1).
//!
//! A render writes the shared parameters buffer and then submits, so two
//! renders at once could mix their settings. The renderer is used by one
//! caller at a time.

use std::path::Path;
use std::sync::{Mutex, OnceLock};

use seura_edit::{EditFile, edit_file_path};
use seura_image::Rgba8;
use seura_render::{Output, Renderer, Settings};

static RENDERER: OnceLock<Result<Mutex<Renderer>, String>> = OnceLock::new();

/// Runs `use_it` with the renderer, made on first use (it takes a moment).
pub fn with_renderer<T>(use_it: impl FnOnce(&Renderer) -> T) -> Result<T, String> {
    let renderer = RENDERER
        .get_or_init(|| {
            Renderer::new()
                .map(Mutex::new)
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(Clone::clone)?;
    let renderer = renderer.lock().map_err(|error| error.to_string())?;
    Ok(use_it(&renderer))
}

/// A photo's edits, rendered onto its thumbnail-sized image (BRW-1). The
/// image is the decoded photo in sRGB; the result is too. Photos whose edit
/// file can't be read, or holds nothing Seura renders, keep the image as it
/// is, and so does a computer without a usable GPU.
pub fn apply_edits(photo: &Path, image: Rgba8) -> Rgba8 {
    let Ok(Some(file)) = EditFile::load(&edit_file_path(photo)) else {
        return image;
    };
    let settings = Settings::from_document(&file.document);
    if settings == Settings::default() {
        return image;
    }
    let linear = image.to_linear_p3(None);
    match with_renderer(|renderer| {
        renderer.render(&linear, &linear.guide_image(), &settings, Output::Srgb)
    }) {
        Ok(Ok(rendered)) => rendered,
        _ => image,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use seura_edit::{Action, CoreOperation, Edit, EditDocument, Exposure, PhotoIdentity, Writer};
    use time::OffsetDateTime;

    use super::*;

    fn gray() -> Rgba8 {
        Rgba8::new(4, 4, [90, 90, 90, 255].repeat(16))
    }

    #[test]
    fn an_unedited_photo_keeps_its_thumbnail() {
        let folder = tempfile::tempdir().unwrap();
        let photo = folder.path().join("a.jpg");
        assert_eq!(apply_edits(&photo, gray()), gray());
    }

    #[test]
    fn an_edited_photo_s_thumbnail_shows_its_exposure() {
        let folder = tempfile::tempdir().unwrap();
        let photo = folder.path().join("a.jpg");
        fs::write(&photo, b"stand-in").unwrap();
        let identity = PhotoIdentity {
            file: "a.jpg".into(),
            size: 8,
            sha256: None,
            width: None,
            height: None,
        };
        let mut file = EditFile::create(
            &edit_file_path(&photo),
            EditDocument::new_for_photo(&identity),
        );
        let action = Action {
            label: "Exposure".into(),
            edits: vec![Edit::Set(CoreOperation::Exposure(Exposure { ev: 1.0 }))],
        };
        file.document
            .apply(&action, OffsetDateTime::now_utc())
            .unwrap();
        file.save(
            Writer {
                app: "Seura Photos",
                version: "test",
            },
            OffsetDateTime::now_utc(),
        )
        .unwrap();

        let rendered = apply_edits(&photo, gray());

        // sRGB 90, one stop brighter, is 125 (as in export's test).
        let pixel = &rendered.pixels[..3];
        assert!(pixel.iter().all(|v| v.abs_diff(125) <= 1), "{pixel:?}");
    }
}
