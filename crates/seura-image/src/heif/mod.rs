//! HEIC/HEIF decoding (decision 0011): the operating system's decoder where
//! there is one (ImageIO on macOS, WIC on Windows with Microsoft's
//! extensions), libheif on Windows otherwise.
//!
//! Orientation and color come from the container, read by Seura itself, so
//! every decoder gives the same result.

mod container;
#[cfg(target_os = "windows")]
mod windows;

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::LazyLock;

use container::{Color, Container, Transform};

use crate::decode::{DecodeError, Decoded};
use crate::raster::Rgba8;

/// The developer switch for the HEIC decoder: `SEURA_HEIF_DECODER=libheif`
/// forces the libheif fallback on a Windows PC that has Microsoft's
/// extensions, so both paths can be tested on one machine; `=system`
/// forces the operating system's decoder.
pub const DECODER_SWITCH: &str = "SEURA_HEIF_DECODER";

/// Which HEIC decoder to use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeifDecoder {
    /// The system's decoder if it works, libheif otherwise (Windows).
    Automatic,
    System,
    Libheif,
}

impl HeifDecoder {
    /// The choice made by the developer switch, if any.
    pub fn from_switch() -> HeifDecoder {
        match std::env::var(DECODER_SWITCH).as_deref() {
            Ok("libheif") => HeifDecoder::Libheif,
            Ok("system") => HeifDecoder::System,
            _ => HeifDecoder::Automatic,
        }
    }
}

/// Pixels from a decoder, and whether it already applied the container's
/// rotation and mirroring.
pub(crate) struct Pixels {
    pub image: Rgba8,
    pub transformed: bool,
}

pub fn decode(path: &Path) -> Result<Decoded, DecodeError> {
    decode_with(path, HeifDecoder::from_switch())
}

/// Decodes with a particular decoder; for tests that compare them.
pub fn decode_with(path: &Path, decoder: HeifDecoder) -> Result<Decoded, DecodeError> {
    let mut head = Vec::new();
    File::open(path)?
        .take(container::HEAD_LIMIT as u64)
        .read_to_end(&mut head)?;
    let container = container::read(&head);
    let pixels = platform_decode(path, decoder)?;
    let image = if pixels.transformed {
        pixels.image
    } else {
        apply(pixels.image, &container)
    };
    let icc = match &container.color {
        Some(Color::Icc(icc)) => Some(icc.as_slice()),
        Some(Color::Primaries(12)) => Some(DISPLAY_P3.as_slice()),
        _ => None,
    };
    // Date taken from HEIF's EXIF item arrives with the metadata work; until
    // then HEIC photos sort by file date (BRW-2).
    Ok(Decoded {
        image: image.from_profile(icc),
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
fn platform_decode(path: &Path, decoder: HeifDecoder) -> Result<Pixels, DecodeError> {
    windows::decode(path, decoder)
}

#[cfg(not(target_os = "windows"))]
fn platform_decode(_path: &Path, _decoder: HeifDecoder) -> Result<Pixels, DecodeError> {
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
