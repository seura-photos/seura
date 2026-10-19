//! Decoding real files, generated here so the repository holds no photos it
//! can't redistribute: JPEG with EXIF orientation, date, and a Display P3
//! profile; PNG in color and gray, with and without a profile (BRW-7).

use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;

use seura_image::{DecodeError, Rgba8, decode};

const RED: [u8; 3] = [255, 0, 0];
const GREEN: [u8; 3] = [0, 255, 0];
const BLUE: [u8; 3] = [0, 0, 255];
const WHITE: [u8; 3] = [255, 255, 255];

/// A 64×48 RGB image: red top left, green top right, blue bottom left,
/// white bottom right.
fn quadrants() -> Vec<u8> {
    let (w, h) = (64, 48);
    (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .flat_map(|(x, y)| match (x < w / 2, y < h / 2) {
            (true, true) => RED,
            (false, true) => GREEN,
            (true, false) => BLUE,
            (false, false) => WHITE,
        })
        .collect()
}

/// The color near each corner of a decoded image: top left, top right,
/// bottom left, bottom right.
fn corners(image: &Rgba8) -> [[u8; 3]; 4] {
    let at = |x: u32, y: u32| {
        let i = ((y * image.width + x) * 4) as usize;
        [image.pixels[i], image.pixels[i + 1], image.pixels[i + 2]]
    };
    let (r, b) = (image.width - 4, image.height - 4);
    [at(3, 3), at(r, 3), at(3, b), at(r, b)]
}

/// Whether two colors match within JPEG's loss.
fn close(a: [u8; 3], b: [u8; 3]) -> bool {
    a.iter().zip(b).all(|(x, y)| x.abs_diff(y) <= 24)
}

fn assert_corners(image: &Rgba8, expected: [[u8; 3]; 4]) {
    let found = corners(image);
    for (corner, (found, expected)) in ["top left", "top right", "bottom left", "bottom right"]
        .iter()
        .zip(found.iter().zip(expected))
    {
        assert!(
            close(*found, expected),
            "{corner}: found {found:?}, expected {expected:?}"
        );
    }
}

/// A little-endian TIFF block with Orientation and DateTime in IFD0.
fn exif(orientation: u16, date: &str) -> Vec<u8> {
    let mut tiff = b"II*\0".to_vec();
    tiff.extend(8u32.to_le_bytes()); // IFD0 right after the header
    tiff.extend(2u16.to_le_bytes()); // two entries
    // Orientation: SHORT, one value, stored inline.
    tiff.extend(0x0112u16.to_le_bytes());
    tiff.extend(3u16.to_le_bytes());
    tiff.extend(1u32.to_le_bytes());
    tiff.extend(u32::from(orientation).to_le_bytes());
    // DateTime: ASCII, 20 bytes, stored after the IFD.
    let date_offset = 8 + 2 + 2 * 12 + 4;
    tiff.extend(0x0132u16.to_le_bytes());
    tiff.extend(2u16.to_le_bytes());
    tiff.extend(20u32.to_le_bytes());
    tiff.extend((date_offset as u32).to_le_bytes());
    tiff.extend(0u32.to_le_bytes()); // no next IFD
    tiff.extend(date.as_bytes());
    tiff.push(0);
    tiff
}

fn jpeg(orientation: Option<u16>, icc: Option<&[u8]>) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = jpeg_encoder::Encoder::new(&mut bytes, 95);
    if let Some(orientation) = orientation {
        let mut app1 = b"Exif\0\0".to_vec();
        app1.extend(exif(orientation, "2024:07:14 18:30:05"));
        encoder.add_app_segment(1, app1).unwrap();
    }
    if let Some(icc) = icc {
        encoder.add_icc_profile(icc).unwrap();
    }
    encoder
        .encode(&quadrants(), 64, 48, jpeg_encoder::ColorType::Rgb)
        .unwrap();
    bytes
}

fn png(color: png::ColorType, data: &[u8], icc: Option<&[u8]>) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut info = png::Info::with_size(64, 48);
    info.color_type = color;
    info.bit_depth = png::BitDepth::Eight;
    info.icc_profile = icc.map(|icc| Cow::Owned(icc.to_vec()));
    let encoder = png::Encoder::with_info(&mut bytes, info).unwrap();
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(data).unwrap();
    writer.finish().unwrap();
    bytes
}

fn display_p3() -> Vec<u8> {
    moxcms::ColorProfile::new_display_p3().encode().unwrap()
}

/// Writes bytes to a temporary file with the given name and decodes it.
fn decode_bytes(name: &str, bytes: &[u8]) -> Result<seura_image::Decoded, DecodeError> {
    let dir = tempfile::tempdir().unwrap();
    let path: PathBuf = dir.path().join(name);
    fs::write(&path, bytes).unwrap();
    decode(&path)
}

#[test]
fn a_plain_jpeg_decodes_as_stored() {
    let decoded = decode_bytes("plain.jpg", &jpeg(None, None)).unwrap();
    assert_eq!((decoded.image.width, decoded.image.height), (64, 48));
    assert_corners(&decoded.image, [RED, GREEN, BLUE, WHITE]);
    assert_eq!(decoded.taken, None);
}

#[test]
fn exif_orientation_is_applied() {
    // Orientation 6: the camera was turned; the photo must turn 90° clockwise.
    let decoded = decode_bytes("turned.jpg", &jpeg(Some(6), None)).unwrap();
    assert_eq!((decoded.image.width, decoded.image.height), (48, 64));
    assert_corners(&decoded.image, [BLUE, RED, WHITE, GREEN]);
    // And the date taken comes along: 2024-07-14 18:30:05.
    assert_eq!(decoded.taken, Some(1_720_981_805));

    let decoded = decode_bytes("upside-down.JPEG", &jpeg(Some(3), None)).unwrap();
    assert_corners(&decoded.image, [WHITE, BLUE, GREEN, RED]);
}

#[test]
fn display_p3_colors_are_converted_to_srgb() {
    // A PNG (lossless) of two mid-tone colors in Display P3. Converted to
    // sRGB they change by a known amount, computed independently from the
    // standard P3-to-sRGB matrix: (200, 100, 50) → (215, 93, 31) and
    // (120, 160, 90) → (108, 161, 80). Unconverted, they'd stay the same.
    let pixels: Vec<u8> = (0..64 * 48)
        .flat_map(|i| {
            if i % 64 < 32 {
                [200, 100, 50]
            } else {
                [120, 160, 90]
            }
        })
        .collect();
    let file = png(png::ColorType::Rgb, &pixels, Some(&display_p3()));
    let decoded = decode_bytes("p3.png", &file).unwrap();
    let [left, right, _, _] = corners(&decoded.image);
    let near = |a: [u8; 3], b: [u8; 3]| a.iter().zip(b).all(|(x, y)| x.abs_diff(y) <= 2);
    assert!(near(left, [215, 93, 31]), "{left:?}");
    assert!(near(right, [108, 161, 80]), "{right:?}");

    // The same in a JPEG: its profile is read too, so the colors differ
    // from the same pixels with no profile.
    let mut bytes = Vec::new();
    let mut encoder = jpeg_encoder::Encoder::new(&mut bytes, 100);
    encoder.add_icc_profile(&display_p3()).unwrap();
    encoder
        .encode(&pixels, 64, 48, jpeg_encoder::ColorType::Rgb)
        .unwrap();
    let [left, _, _, _] = corners(&decode_bytes("p3.jpg", &bytes).unwrap().image);
    assert!(left[0] >= 210 && left[2] <= 38, "{left:?}");
}

#[test]
fn pngs_decode_in_color_and_gray() {
    let decoded = decode_bytes(
        "quadrants.png",
        &png(png::ColorType::Rgb, &quadrants(), None),
    )
    .unwrap();
    assert_corners(&decoded.image, [RED, GREEN, BLUE, WHITE]);

    let gray: Vec<u8> = (0..64 * 48).map(|i| (i % 256) as u8).collect();
    let decoded = decode_bytes("gray.png", &png(png::ColorType::Grayscale, &gray, None)).unwrap();
    assert_eq!(&decoded.image.pixels[4..8], &[1, 1, 1, 255]);
}

#[test]
fn damaged_and_unknown_files_are_errors_not_panics() {
    let jpeg = jpeg(None, None);
    assert!(matches!(
        decode_bytes("cut.jpg", &jpeg[..jpeg.len() / 3]),
        Err(DecodeError::Damaged(_))
    ));
    assert!(matches!(
        decode_bytes("noise.png", b"\x89PNG\r\n\x1a\nnoise"),
        Err(DecodeError::Damaged(_))
    ));
    assert!(matches!(
        decode_bytes("notes.txt", b"hello"),
        Err(DecodeError::Unsupported)
    ));
}
