//! HEIC/HEIF decoding with the operating system's decoder (decision 0014):
//! ImageIO on macOS, and on Windows WIC with Microsoft's HEIF and HEVC
//! extensions. Without them, Windows can't show HEIC photos, and says so.
//!
//! Orientation and color come from the container, read by Seura itself, so
//! both decoders give the same result.

mod container;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::LazyLock;

use container::{Color, Container, Transform};

use crate::decode::{DecodeError, Source};
use crate::raster::Rgba8;

/// The developer switch: `SEURA_HEIF_DECODER=none` behaves as if the
/// system had no HEIC decoder, to see what users without Microsoft's
/// extensions see.
const DECODER_SWITCH: &str = "SEURA_HEIF_DECODER";

/// Pixels from a decoder: whether it already applied the container's
/// rotation and mirroring, and whether it already converted to Display P3.
pub(crate) struct Pixels {
    pub image: Rgba8,
    pub transformed: bool,
    pub display_p3: bool,
}

pub(crate) fn decode(path: &Path) -> Result<Source, DecodeError> {
    if std::env::var(DECODER_SWITCH).as_deref() == Ok("none") {
        return Err(DecodeError::HeifUnavailable(format!(
            "{DECODER_SWITCH}=none"
        )));
    }
    let mut head = Vec::new();
    File::open(path)?
        .take(container::HEAD_LIMIT as u64)
        .read_to_end(&mut head)?;
    let container = container::read(&head);
    let pixels = platform_decode(path)?;
    let image = if pixels.transformed {
        pixels.image
    } else {
        apply(pixels.image, &container)
    };
    let icc = if pixels.display_p3 {
        Some(DISPLAY_P3.clone())
    } else {
        match container.color {
            Some(Color::Icc(icc)) => Some(icc),
            Some(Color::Primaries(12)) => Some(DISPLAY_P3.clone()),
            _ => None,
        }
    };
    // Date taken from HEIF's EXIF item arrives with the metadata work; until
    // then HEIC photos sort by file date (BRW-2).
    Ok(Source {
        image,
        icc,
        taken: None,
    })
}

static DISPLAY_P3: LazyLock<Vec<u8>> = LazyLock::new(|| {
    moxcms::ColorProfile::new_display_p3()
        .encode()
        .expect("the built-in Display P3 profile encodes")
});

/// Applies the container's rotations and mirrors, in the order listed.
fn apply(image: Rgba8, container: &Container) -> Rgba8 {
    container.transforms.iter().fold(image, |image, transform| {
        // Expressed as the equivalent EXIF orientation (see raster.rs).
        image.orient(match transform {
            Transform::Rotate(90) => 8,
            Transform::Rotate(180) => 3,
            Transform::Rotate(270) => 6,
            Transform::Rotate(_) => 1,
            Transform::MirrorHorizontal => 2,
            Transform::MirrorVertical => 4,
        })
    })
}

#[cfg(target_os = "windows")]
fn platform_decode(path: &Path) -> Result<Pixels, DecodeError> {
    windows::decode(path)
}

#[cfg(target_os = "macos")]
fn platform_decode(path: &Path) -> Result<Pixels, DecodeError> {
    macos::decode(path)
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn platform_decode(_path: &Path) -> Result<Pixels, DecodeError> {
    Err(DecodeError::HeifUnavailable(
        "not implemented yet on this platform".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_rotation_is_counter_clockwise() {
        // 1 2 3        3 6
        // 4 5 6   →    2 5
        //              1 4
        let pixels = (1..=6u8).flat_map(|n| [n, n, n, 255]).collect();
        let image = Rgba8::new(3, 2, pixels);
        let container = Container {
            transforms: vec![Transform::Rotate(90)],
            color: None,
        };
        let rotated = apply(image, &container);
        let numbers: Vec<u8> = rotated.pixels.chunks(4).map(|p| p[0]).collect();
        assert_eq!((rotated.width, rotated.height), (2, 3));
        assert_eq!(numbers, [3, 6, 2, 5, 1, 4]);
    }
}
