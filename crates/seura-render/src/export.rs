//! Exporting a photo with its edits (EXP-1 to EXP-5): decode at full size,
//! render with wgpu in sRGB, encode JPEG, and write it to the destination
//! under a name no file has, never replacing one (EXP-3).

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use seura_image::{DecodeError, JPEG_QUALITY, decode_linear, encode_jpeg};

use crate::{Output, RenderError, Renderer, Settings};

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error(transparent)]
    Decode(#[from] DecodeError),
    #[error(transparent)]
    Render(#[from] RenderError),
    #[error("couldn't encode the JPEG: {0}")]
    Encode(String),
    #[error("couldn't write the export: {0}")]
    Write(#[from] io::Error),
}

/// Exports `photo` as a JPEG in `folder`, and returns where it went. The
/// photo is only read (CLAUDE.md rule 1).
pub fn export_jpeg(
    renderer: &Renderer,
    photo: &Path,
    settings: &Settings,
    folder: &Path,
) -> Result<PathBuf, ExportError> {
    let decoded = decode_linear(photo, None)?;
    let rendered = renderer.render(&decoded.image, &decoded.guide, settings, Output::Srgb)?;
    let bytes = encode_jpeg(&rendered, JPEG_QUALITY)
        .map_err(|error| ExportError::Encode(error.to_string()))?;
    let stem = photo
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Photo".into());
    write_new(folder, &stem, "jpg", &bytes)
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
}
