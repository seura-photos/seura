//! HEIC decoding with real files (BRW-7, decision 0014). The files in
//! `fixtures/` were made for Seura with Apple's ImageIO on Nevada's Mac
//! (see `fixtures/README.md`): a 64×48 image, red top left, green top
//! right, blue bottom left, white bottom right, stored upright, turned
//! (EXIF orientation 6), mirrored (2), and in Display P3.
//!
//! In HEIC, orientation lives in the container (`irot`, `imir`); each file
//! also carries an EXIF orientation, which must be ignored, or the photo
//! turns twice. These files test exactly that.
//!
//! On Windows the decoder is WIC with Microsoft's extensions, which CI's
//! runners may not have; there the tests are skipped. Every Mac has ImageIO.

use std::path::PathBuf;

use seura_image::{DecodeError, Decoded, Rgba8, decode};

const RED: [u8; 3] = [255, 0, 0];
const GREEN: [u8; 3] = [0, 255, 0];
const BLUE: [u8; 3] = [0, 0, 255];
const WHITE: [u8; 3] = [255, 255, 255];

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Decodes a fixture; `None` on Windows without Microsoft's extensions.
fn decode_fixture(name: &str) -> Option<Decoded> {
    match decode(&fixture(name)) {
        Ok(decoded) => Some(decoded),
        Err(DecodeError::HeifUnavailable(reason)) if cfg!(windows) => {
            eprintln!("skipping: {reason}");
            None
        }
        Err(error) => panic!("{name}: {error}"),
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

/// Colors aren't compared closely here (see
/// `wic_colors_drift_on_unspecified_color`).
#[test]
fn heic_is_shown_the_right_way_up() {
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
    for (name, size, expected) in cases {
        let Some(decoded) = decode_fixture(name) else {
            continue;
        };
        let image = &decoded.image;
        assert_eq!((image.width, image.height), size, "{name}");
        assert_corners(image, expected, 60, name);
    }
}

#[test]
fn a_display_p3_profile_is_applied() {
    let Some(decoded) = decode_fixture("quadrants-p3.heic") else {
        return;
    };
    assert_corners(&decoded.image, [RED, GREEN, BLUE, WHITE], 4, "P3");
}

#[cfg(target_os = "macos")]
#[test]
fn imageio_reproduces_unspecified_color_as_srgb() {
    let decoded = decode_fixture("quadrants.heic").expect("every Mac has ImageIO");
    assert_corners(&decoded.image, [RED, GREEN, BLUE, WHITE], 4, "sRGB");
}

/// A known difference, recorded so a change is noticed: for files whose
/// `colr` box is `nclx` with unspecified primaries and transfer (as
/// ImageIO writes for sRGB), WIC's colors drift. Green comes out around
/// (0, 215, 0) instead of (0, 255, 0); ImageIO and the P3 file are exact.
/// Real iPhone photos aren't affected: they carry a profile (checked with
/// Nevada's photos, 2026-10-19).
#[cfg(windows)]
#[test]
fn wic_colors_drift_on_unspecified_color() {
    let Some(decoded) = decode_fixture("quadrants.heic") else {
        return;
    };
    let [_, green, _, _] = corners(&decoded.image);
    assert!(
        green[1] < 240,
        "WIC's green is now {green:?}; the drift may be fixed"
    );
}
