//! Exporting a photo with its edits (EXP-1 to EXP-5): decode at full size,
//! render with wgpu in sRGB, scale once to the size asked for, encode JPEG or
//! PNG, and write it to the destination under a name no file has, never
//! replacing one (EXP-3).

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use seura_image::{
    DecodeError, JPEG_QUALITY, decode_linear, encode_jpeg, encode_png, fit_for_export,
};

use crate::{Output, RenderError, Renderer, Settings};

/// The smallest long edge a custom size can ask for (EXP-2).
pub const MIN_LONG_EDGE: u32 = 16;

/// The file an export makes (EXP-2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    /// `quality` is 1 to 100.
    Jpeg {
        quality: u8,
    },
    Png,
}

/// The size an export is, of the photo as cropped (EXP-2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportSize {
    Original,
    /// The long edge limited to this many pixels, at least [`MIN_LONG_EDGE`].
    /// A smaller photo keeps its own size.
    LongEdge(u32),
}

/// What the Export dialog asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub size: ExportSize,
}

impl Default for ExportOptions {
    fn default() -> Self {
        ExportOptions {
            format: ExportFormat::Jpeg {
                quality: JPEG_QUALITY,
            },
            size: ExportSize::Original,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("the JPEG quality must be from 1 to 100, not {0}")]
    Quality(u8),
    #[error("the long edge must be at least {MIN_LONG_EDGE} pixels, not {0}")]
    LongEdge(u32),
    #[error(transparent)]
    Decode(#[from] DecodeError),
    #[error(transparent)]
    Render(#[from] RenderError),
    #[error("couldn't encode the export: {0}")]
    Encode(String),
    #[error("couldn't write the export: {0}")]
    Write(#[from] io::Error),
}

/// Exports `photo` into `folder` as `options` ask, and returns where it went.
/// The photo is only read (CLAUDE.md rule 1).
pub fn export(
    renderer: &Renderer,
    photo: &Path,
    settings: &Settings,
    folder: &Path,
    options: &ExportOptions,
) -> Result<PathBuf, ExportError> {
    if let ExportFormat::Jpeg { quality } = options.format
        && !(1..=100).contains(&quality)
    {
        return Err(ExportError::Quality(quality));
    }
    if let ExportSize::LongEdge(long_edge) = options.size
        && long_edge < MIN_LONG_EDGE
    {
        return Err(ExportError::LongEdge(long_edge));
    }
    // The decoded photo goes before scaling and encoding, which need memory
    // of their own.
    let rendered = {
        let decoded = decode_linear(photo, None)?;
        renderer.render(&decoded.image, &decoded.guide, settings, Output::Srgb)?
    };
    let rendered = match options.size {
        ExportSize::Original => rendered,
        ExportSize::LongEdge(long_edge) => fit_for_export(rendered, long_edge),
    };
    let (encoded, extension) = match options.format {
        ExportFormat::Jpeg { quality } => (
            encode_jpeg(&rendered, quality).map_err(|error| error.to_string()),
            "jpg",
        ),
        ExportFormat::Png => (
            encode_png(&rendered).map_err(|error| error.to_string()),
            "png",
        ),
    };
    let bytes = encoded.map_err(ExportError::Encode)?;
    let stem = photo
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Photo".into());
    write_new(folder, &stem, extension, &bytes)
}

/// Writes `bytes` to `folder/stem.extension`, or `stem (1).extension` and
/// so on if that's taken: a temporary file in the same folder, renamed to
/// the first free name without ever replacing a file, even one that
/// appears meanwhile. A failed write deletes its own temporary file
/// (CLAUDE.md rules 3 and 5).
pub(crate) fn write_new(
    folder: &Path,
    stem: &str,
    extension: &str,
    bytes: &[u8],
) -> Result<PathBuf, ExportError> {
    write_new_after(folder, stem, extension, bytes, |_| {})
}

/// [`write_new`], with `before_rename` run on each name just after it was
/// found free, so a test can take it meanwhile.
fn write_new_after(
    folder: &Path,
    stem: &str,
    extension: &str,
    bytes: &[u8],
    mut before_rename: impl FnMut(&Path),
) -> Result<PathBuf, ExportError> {
    let mut temporary = tempfile::Builder::new()
        .prefix(".seura-export-")
        .suffix(".tmp")
        .tempfile_in(folder)?;
    temporary.write_all(bytes)?;
    temporary.as_file().sync_all()?;
    for n in 0.. {
        let name = if n == 0 {
            format!("{stem}.{extension}")
        } else {
            format!("{stem} ({n}).{extension}")
        };
        let path = folder.join(name);
        if fs::symlink_metadata(&path).is_ok() {
            continue;
        }
        before_rename(&path);
        match temporary.persist_noclobber(&path) {
            Ok(_) => return Ok(path),
            // Taken since the check: try the next name.
            Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists => {
                temporary = error.file;
            }
            Err(error) => return Err(error.error.into()),
        }
    }
    unreachable!("some name is free")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_taken_name_gets_a_number_and_nothing_is_replaced() {
        let folder = tempfile::tempdir().unwrap();
        fs::write(folder.path().join("IMG_1.jpg"), b"original").unwrap();
        fs::write(folder.path().join("IMG_1 (1).jpg"), b"earlier export").unwrap();

        let path = write_new(folder.path(), "IMG_1", "jpg", b"new").unwrap();

        assert_eq!(path, folder.path().join("IMG_1 (2).jpg"));
        assert_eq!(fs::read(&path).unwrap(), b"new");
        assert_eq!(
            fs::read(folder.path().join("IMG_1.jpg")).unwrap(),
            b"original"
        );
        assert_eq!(
            fs::read(folder.path().join("IMG_1 (1).jpg")).unwrap(),
            b"earlier export"
        );
        // No temporary file is left behind.
        assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 3);
    }

    #[test]
    fn a_free_name_is_used_as_is() {
        let folder = tempfile::tempdir().unwrap();
        let path = write_new(folder.path(), "IMG_1", "jpg", b"new").unwrap();
        assert_eq!(path, folder.path().join("IMG_1.jpg"));
    }

    #[test]
    fn another_extension_is_another_name() {
        let folder = tempfile::tempdir().unwrap();
        fs::write(folder.path().join("IMG_1.jpg"), b"original").unwrap();
        let path = write_new(folder.path(), "IMG_1", "png", b"new").unwrap();
        assert_eq!(path, folder.path().join("IMG_1.png"));
    }

    #[test]
    fn a_folder_with_the_name_counts_as_taken() {
        let folder = tempfile::tempdir().unwrap();
        fs::create_dir(folder.path().join("IMG_1.jpg")).unwrap();
        let path = write_new(folder.path(), "IMG_1", "jpg", b"new").unwrap();
        assert_eq!(path, folder.path().join("IMG_1 (1).jpg"));
    }

    #[test]
    fn a_name_taken_meanwhile_is_not_replaced() {
        let folder = tempfile::tempdir().unwrap();
        let taken = folder.path().join("IMG_1.jpg");
        let path = write_new_after(folder.path(), "IMG_1", "jpg", b"new", |path| {
            if path == taken {
                fs::write(path, b"arrived meanwhile").unwrap();
            }
        })
        .unwrap();
        assert_eq!(path, folder.path().join("IMG_1 (1).jpg"));
        assert_eq!(fs::read(&path).unwrap(), b"new");
        assert_eq!(fs::read(&taken).unwrap(), b"arrived meanwhile");
        assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 2);
    }

    #[test]
    fn a_folder_that_cannot_be_written_leaves_nothing() {
        let folder = tempfile::tempdir().unwrap();
        let missing = folder.path().join("missing");
        assert!(write_new(&missing, "IMG_1", "jpg", b"new").is_err());
        assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 0);
    }
}
