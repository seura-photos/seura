//! HEIC decoding with real files (BRW-7, decision 0011). The files in
//! `fixtures/` were made for Seura with Apple's ImageIO on Nevada's Mac
//! (see `fixtures/README.md`): a 64×48 image, red top left, green top
//! right, blue bottom left, white bottom right, stored upright, turned
//! (EXIF orientation 6), mirrored (2), and in Display P3.
//!
//! In HEIC, orientation lives in the container (`irot`, `imir`); each file
//! also carries an EXIF orientation, which must be ignored, or the photo
//! turns twice. These files test exactly that.

#![cfg(target_os = "windows")]

use std::path::PathBuf;

use seura_image::{DecodeError, Decoded, HeifDecoder, Rgba8, decode_heif_with};

const RED: [u8; 3] = [255, 0, 0];
const GREEN: [u8; 3] = [0, 255, 0];
const BLUE: [u8; 3] = [0, 0, 255];
const WHITE: [u8; 3] = [255, 255, 255];

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Decodes with one decoder; `None` if it isn't available on this machine
/// (WIC without Microsoft's extensions, as on CI's Windows runners).
fn decode(name: &str, decoder: HeifDecoder) -> Option<Decoded> {
    match decode_heif_with(&fixture(name), decoder) {
        Ok(decoded) => Some(decoded),
        Err(DecodeError::HeifUnavailable(reason)) if decoder == HeifDecoder::System => {
            eprintln!("skipping WIC: {reason}");
            None
        }
        Err(error) => panic!("{name} with {decoder:?}: {error}"),
    }
}

/// Top left, top right, bottom left, bottom right, sampled away from the
/// quadrant edges (the encoding is lossy and its color is subsampled).
fn corners(image: &Rgba8) -> [[u8; 3]; 4] {
    let at = |x: u32, y: u32| {
        let i = ((y * image.width + x) * 4) as usize;
        [image.pixels[i], image.pixels[i + 1], image.pixels[i + 2]]
    };
    let (r, b) = (image.width - 6, image.height - 6);
    [at(5, 5), at(r, 5), at(5, b), at(r, b)]
}

fn assert_corners(image: &Rgba8, expected: [[u8; 3]; 4], tolerance: u8, what: &str) {
    for (found, expected) in corners(image).iter().zip(expected) {
        let close = found
            .iter()
            .zip(expected)
            .all(|(a, b)| a.abs_diff(b) <= tolerance);
        assert!(
            close,
            "{what}: corners {:?}, expected {expected:?}",
            corners(image)
        );
    }
}

/// Orientation is right with every decoder. Colors aren't compared
/// closely here (see `wic_colors_drift_on_unspecified_color`).
#[test]
fn every_decoder_shows_heic_the_right_way_up() {
    let cases = [
        ("quadrants.heic", (64, 48), [RED, GREEN, BLUE, WHITE]),
        // Turned 90° clockwise: what was on the left is now on top.
        (
            "quadrants-orientation-6.heic",
            (48, 64),
            [BLUE, RED, WHITE, GREEN],
        ),
        // Mirrored left to right.
        (
            "quadrants-orientation-2.heic",
            (64, 48),
            [GREEN, RED, WHITE, BLUE],
        ),
        ("quadrants-p3.heic", (64, 48), [RED, GREEN, BLUE, WHITE]),
    ];
    for decoder in [HeifDecoder::Libheif, HeifDecoder::System] {
        for (name, size, expected) in cases {
            let Some(decoded) = decode(name, decoder) else {
                continue;
            };
            let image = &decoded.image;
            assert_eq!((image.width, image.height), size, "{name} with {decoder:?}");
            assert_corners(image, expected, 60, &format!("{name} with {decoder:?}"));
        }
    }
}

#[test]
fn libheif_reproduces_the_colors() {
    for name in ["quadrants.heic", "quadrants-p3.heic"] {
        let decoded = decode(name, HeifDecoder::Libheif).unwrap();
        assert_corners(&decoded.image, [RED, GREEN, BLUE, WHITE], 4, name);
    }
}

#[test]
fn a_display_p3_profile_is_applied_with_every_decoder() {
    for decoder in [HeifDecoder::Libheif, HeifDecoder::System] {
        let Some(decoded) = decode("quadrants-p3.heic", decoder) else {
            continue;
        };
        assert_corners(
            &decoded.image,
            [RED, GREEN, BLUE, WHITE],
            4,
            &format!("{decoder:?}"),
        );
    }
}

/// A known difference, recorded so a change is noticed: for files whose
/// `colr` box is `nclx` with unspecified primaries and transfer (as
/// ImageIO writes for sRGB), WIC's colors drift. Green comes out around
/// (0, 215, 0) instead of (0, 255, 0); libheif and the P3 file are exact.
/// Whether real iPhone photos are affected is checked with Nevada's photos.
#[test]
fn wic_colors_drift_on_unspecified_color() {
    let Some(decoded) = decode("quadrants.heic", HeifDecoder::System) else {
        return;
    };
    let [_, green, _, _] = corners(&decoded.image);
    assert!(
        green[1] < 240,
        "WIC's green is now {green:?}; the drift may be fixed"
    );
}
