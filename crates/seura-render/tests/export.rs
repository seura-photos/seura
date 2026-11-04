//! Whole exports, from a file on disk to a file on disk (EXP-2 to EXP-5),
//! read back as another app would: formats and sizes, the colors worked out
//! here from the color spaces' definitions, the metadata, and the original
//! untouched.

use std::fs;
use std::path::{Path, PathBuf};

use exif::{Field, In, Reader, Tag, Value};
use seura_image::{Keywords, Metadata, Rgba8, decode, encode_jpeg, encode_png};
use seura_render::{
    ExportError, ExportFormat, ExportOptions, ExportSize, Renderer, Settings, export,
};

const JPEG: ExportFormat = ExportFormat::Jpeg { quality: 90 };

fn renderer() -> Renderer {
    Renderer::new().expect("a graphics adapter, or the software one")
}

/// A smooth image: scaling and JPEG both keep it close, so a result can be
/// compared with the same formula at its own size.
fn smooth(x: f64, y: f64) -> [u8; 3] {
    [
        (40.0 + 180.0 * x).round() as u8,
        (220.0 - 150.0 * y).round() as u8,
        (60.0 + 90.0 * (x + y) / 2.0).round() as u8,
    ]
}

fn smooth_image(width: u32, height: u32) -> Rgba8 {
    let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
    for y in 0..height {
        for x in 0..width {
            let [r, g, b] = smooth(
                (f64::from(x) + 0.5) / f64::from(width),
                (f64::from(y) + 0.5) / f64::from(height),
            );
            pixels.extend([r, g, b, 255]);
        }
    }
    Rgba8::new(width, height, pixels)
}

/// A PNG photo of `image`, with no profile, so sRGB.
fn write_photo(folder: &Path, name: &str, image: &Rgba8) -> PathBuf {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, image.width, image.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&image.pixels).unwrap();
    writer.finish().unwrap();
    let path = folder.join(name);
    fs::write(&path, bytes).unwrap();
    path
}

/// How far an image is from [`smooth`] at its own size.
fn off_from_smooth(image: &Rgba8) -> u8 {
    let mut off = 0;
    for y in 0..image.height {
        for x in 0..image.width {
            let want = smooth(
                (f64::from(x) + 0.5) / f64::from(image.width),
                (f64::from(y) + 0.5) / f64::from(image.height),
            );
            let at = ((y * image.width + x) * 4) as usize;
            for (found, want) in image.pixels[at..at + 3].iter().zip(want) {
                off = off.max(found.abs_diff(want));
            }
        }
    }
    off
}

/// The ICC profile in a PNG or JPEG file, read without Seura's decoders.
fn embedded_profile(file: &[u8]) -> Vec<u8> {
    if file.starts_with(b"\x89PNG") {
        let reader = png::Decoder::new(std::io::Cursor::new(file))
            .read_info()
            .unwrap();
        return reader.info().icc_profile.as_ref().unwrap().to_vec();
    }
    // One APP2 segment: the marker, a length of two bytes that counts
    // itself, the name, its sequence number and count, then the profile.
    const NAME: &[u8] = b"ICC_PROFILE\0";
    let at = file
        .windows(NAME.len())
        .position(|window| window == NAME)
        .expect("a profile in the JPEG");
    assert_eq!(file[at - 4..at - 2], [0xFF, 0xE2]);
    assert_eq!(file[at + NAME.len()..at + NAME.len() + 2], [1, 1]);
    let length = usize::from(u16::from_be_bytes([file[at - 2], file[at - 1]]));
    file[at + NAME.len() + 2..at - 2 + length].to_vec()
}

fn srgb_encode(linear: f64) -> u8 {
    let v = linear.clamp(0.0, 1.0);
    let encoded = if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round() as u8
}

fn srgb_decode(value: u8) -> f64 {
    let v = f64::from(value) / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn apply(matrix: [[f64; 3]; 3], c: [f64; 3]) -> [f64; 3] {
    matrix.map(|row| row[0] * c[0] + row[1] * c[1] + row[2] * c[2])
}

#[test]
fn every_format_and_size_exports_the_photo() {
    let folder = tempfile::tempdir().unwrap();
    let image = smooth_image(2400, 1200);
    let photo = write_photo(folder.path(), "photo.png", &image);
    let renderer = renderer();
    let sizes = [
        (ExportSize::Original, (2400, 1200)),
        (ExportSize::LongEdge(2048), (2048, 1024)),
        (ExportSize::LongEdge(1600), (1600, 800)),
        (ExportSize::LongEdge(1024), (1024, 512)),
        (ExportSize::LongEdge(301), (301, 151)),
        (ExportSize::LongEdge(16), (16, 8)),
        // A limit never enlarges.
        (ExportSize::LongEdge(5000), (2400, 1200)),
    ];
    for (format, extension, tolerance) in [(JPEG, "jpg", 5), (ExportFormat::Png, "png", 2)] {
        for (size, expected) in sizes {
            let out = tempfile::tempdir().unwrap();
            let options = ExportOptions {
                format,
                size,
                ..Default::default()
            };
            let path = export(
                &renderer,
                &photo,
                &Settings::default(),
                out.path(),
                &options,
                None,
            )
            .unwrap();
            assert_eq!(path, out.path().join(format!("photo.{extension}")));
            let decoded = decode(&path).unwrap();
            assert_eq!(
                (decoded.image.width, decoded.image.height),
                expected,
                "{options:?}"
            );
            let off = off_from_smooth(&decoded.image);
            eprintln!("{options:?}: off by at most {off}");
            assert!(off <= tolerance, "{options:?}: off by {off}");
            let bytes = fs::read(&path).unwrap();
            let profile = moxcms::ColorProfile::new_from_slice(&embedded_profile(&bytes)).unwrap();
            let srgb = moxcms::ColorProfile::new_srgb();
            assert_eq!(profile.red_colorant, srgb.red_colorant, "{options:?}");
            assert_eq!(profile.green_colorant, srgb.green_colorant, "{options:?}");
            assert_eq!(profile.blue_colorant, srgb.blue_colorant, "{options:?}");
        }
    }
}

#[test]
fn a_size_applies_to_the_photo_as_cropped() {
    let folder = tempfile::tempdir().unwrap();
    let image = smooth_image(2000, 1000);
    let photo = write_photo(folder.path(), "photo.png", &image);
    let renderer = renderer();
    // The left half, 1000 × 1000.
    let settings = Settings {
        crop_width: 0.5,
        ..Settings::default()
    };
    let exported = |size| {
        let out = tempfile::tempdir().unwrap();
        let options = ExportOptions {
            format: ExportFormat::Png,
            size,
            ..Default::default()
        };
        let path = export(&renderer, &photo, &settings, out.path(), &options, None).unwrap();
        let decoded = decode(&path).unwrap();
        (decoded.image.width, decoded.image.height)
    };
    assert_eq!(exported(ExportSize::Original), (1000, 1000));
    assert_eq!(
        exported(ExportSize::LongEdge(1500)),
        (1000, 1000),
        "a limit larger than the crop doesn't enlarge it, though the photo is larger"
    );
    assert_eq!(exported(ExportSize::LongEdge(500)), (500, 500));
}

#[test]
fn the_edits_are_in_the_export() {
    let folder = tempfile::tempdir().unwrap();
    let image = Rgba8::new(32, 32, [128, 128, 128, 255].repeat(32 * 32));
    let photo = write_photo(folder.path(), "photo.png", &image);
    let settings = Settings {
        exposure_ev: 1.0,
        ..Settings::default()
    };
    let options = ExportOptions {
        format: ExportFormat::Png,
        size: ExportSize::Original,
        ..Default::default()
    };
    let path = export(
        &renderer(),
        &photo,
        &settings,
        folder.path(),
        &options,
        None,
    )
    .unwrap();
    let want = srgb_encode(srgb_decode(128) * 2.0);
    let found = decode(&path).unwrap().image.pixels[0];
    assert!(found.abs_diff(want) <= 1, "{found}, expected {want}");
}

/// A photo with an ICC profile: its colors arrive in sRGB, and those
/// outside it are clipped, in both formats.
#[test]
fn wide_color_photos_are_converted_to_srgb() {
    // Linear Display P3 to linear sRGB, from the two spaces' primaries.
    const P3_TO_SRGB: [[f64; 3]; 3] = [
        [1.2249401, -0.2249404, 0.0],
        [-0.0420569, 1.0420571, 0.0],
        [-0.0196376, -0.0786361, 1.0982735],
    ];
    // Linear Adobe RGB to linear sRGB, likewise.
    const ADOBE_TO_SRGB: [[f64; 3]; 3] = [
        [1.3983557, -0.3983557, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -0.0429289, 1.0429289],
    ];
    let adobe_decode = |value: u8| (f64::from(value) / 255.0).powf(563.0 / 256.0);
    let cases: [(moxcms::ColorProfile, [u8; 3], [f64; 3]); 4] = [
        // Within sRGB.
        (
            moxcms::ColorProfile::new_display_p3(),
            [150, 120, 100],
            apply(P3_TO_SRGB, [150, 120, 100].map(srgb_decode)),
        ),
        // Display P3's green, outside sRGB.
        (
            moxcms::ColorProfile::new_display_p3(),
            [0, 255, 0],
            apply(P3_TO_SRGB, [0, 255, 0].map(srgb_decode)),
        ),
        (
            moxcms::ColorProfile::new_adobe_rgb(),
            [150, 120, 100],
            apply(ADOBE_TO_SRGB, [150, 120, 100].map(adobe_decode)),
        ),
        // Adobe RGB's green, outside Display P3 as well.
        (
            moxcms::ColorProfile::new_adobe_rgb(),
            [0, 255, 0],
            apply(ADOBE_TO_SRGB, [0, 255, 0].map(adobe_decode)),
        ),
    ];
    let renderer = renderer();
    for (profile, color, linear) in cases {
        let folder = tempfile::tempdir().unwrap();
        let mut info = png::Info::with_size(8, 8);
        info.color_type = png::ColorType::Rgb;
        info.bit_depth = png::BitDepth::Eight;
        info.icc_profile = Some(profile.encode().unwrap().into());
        let mut bytes = Vec::new();
        let mut writer = png::Encoder::with_info(&mut bytes, info)
            .unwrap()
            .write_header()
            .unwrap();
        writer.write_image_data(&color.repeat(64)).unwrap();
        writer.finish().unwrap();
        let photo = folder.path().join("photo.png");
        fs::write(&photo, bytes).unwrap();
        let want = linear.map(srgb_encode);
        for (format, tolerance) in [(JPEG, 3), (ExportFormat::Png, 1)] {
            let options = ExportOptions {
                format,
                size: ExportSize::Original,
                ..Default::default()
            };
            let path = export(
                &renderer,
                &photo,
                &Settings::default(),
                folder.path(),
                &options,
                None,
            )
            .unwrap();
            let found = decode(&path).unwrap().image;
            for k in 0..3 {
                assert!(
                    found.pixels[k].abs_diff(want[k]) <= tolerance,
                    "{format:?} of {color:?}: found {:?}, expected {want:?}",
                    &found.pixels[..3],
                );
            }
        }
    }
}

#[test]
fn the_original_is_never_touched_even_in_its_own_folder() {
    let folder = tempfile::tempdir().unwrap();
    let image = smooth_image(64, 64);
    // A JPEG original, exported as a JPEG beside it: the same name.
    let bytes = encode_jpeg(&image, 90, &Default::default()).unwrap();
    let jpeg = folder.path().join("IMG_1.jpg");
    fs::write(&jpeg, &bytes).unwrap();
    let modified = fs::metadata(&jpeg).unwrap().modified().unwrap();
    // A PNG original, likewise.
    let png_bytes = encode_png(&image, &Default::default()).unwrap();
    let png = folder.path().join("IMG_2.png");
    fs::write(&png, &png_bytes).unwrap();
    let renderer = renderer();
    let exported = |photo: &Path, format| {
        export(
            &renderer,
            photo,
            &Settings::default(),
            folder.path(),
            &ExportOptions {
                format,
                size: ExportSize::Original,
                ..Default::default()
            },
            None,
        )
        .unwrap()
    };
    assert_eq!(
        exported(&jpeg, JPEG),
        folder.path().join("IMG_1 (1).jpg"),
        "the original's name is taken"
    );
    assert_eq!(
        exported(&jpeg, ExportFormat::Png),
        folder.path().join("IMG_1.png"),
        "the new extension is free"
    );
    assert_eq!(
        exported(&png, ExportFormat::Png),
        folder.path().join("IMG_2 (1).png")
    );
    assert_eq!(fs::read(&jpeg).unwrap(), bytes);
    assert_eq!(fs::metadata(&jpeg).unwrap().modified().unwrap(), modified);
    assert_eq!(fs::read(&png).unwrap(), png_bytes);
    // No temporary file is left behind.
    assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 5);
}

#[test]
fn options_out_of_range_export_nothing() {
    let folder = tempfile::tempdir().unwrap();
    let photo = write_photo(folder.path(), "photo.png", &smooth_image(32, 32));
    let renderer = renderer();
    let exported = |format, size| {
        export(
            &renderer,
            &photo,
            &Settings::default(),
            folder.path(),
            &ExportOptions {
                format,
                size,
                ..Default::default()
            },
            None,
        )
    };
    for quality in [0, 101] {
        assert!(matches!(
            exported(ExportFormat::Jpeg { quality }, ExportSize::Original),
            Err(ExportError::Quality(found)) if found == quality
        ));
    }
    assert!(matches!(
        exported(ExportFormat::Png, ExportSize::LongEdge(15)),
        Err(ExportError::LongEdge(15))
    ));
    assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 1);
    assert!(exported(ExportFormat::Png, ExportSize::LongEdge(16)).is_ok());
    assert!(exported(ExportFormat::Jpeg { quality: 1 }, ExportSize::Original).is_ok());
    assert!(exported(ExportFormat::Jpeg { quality: 100 }, ExportSize::Original).is_ok());
}

/// A JPEG photo with everything an export must keep and everything it must
/// not: EXIF with a location, a thumbnail and a maker note, its own XMP, and a
/// preview after the image with the MPF segment that points to it. Each thing
/// that must go says so in its bytes, so the tests can look for it without a
/// metadata reader. Turned on its side by its orientation tag.
fn photo_with_metadata(folder: &Path) -> PathBuf {
    let field = |tag, value| Field {
        tag,
        ifd_num: In::PRIMARY,
        value,
    };
    let ascii = |text: &str| Value::Ascii(vec![text.as_bytes().to_vec()]);
    let fields = [
        field(Tag::Make, ascii("Apple")),
        field(Tag::Model, ascii("iPhone 15 Pro")),
        field(Tag::DateTimeOriginal, ascii("2026:10:18 19:40:12")),
        field(Tag::LensModel, ascii("iPhone 15 Pro back camera")),
        field(Tag::Orientation, Value::Short(vec![6])),
        field(Tag::ColorSpace, Value::Short(vec![65535])),
        field(Tag::PixelXDimension, Value::Long(vec![48])),
        field(Tag::PixelYDimension, Value::Long(vec![32])),
        field(Tag::GPSMapDatum, ascii("GPS-MUST-GO")),
        field(
            Tag::MakerNote,
            Value::Undefined(b"NOTE-MUST-GO".to_vec(), 0),
        ),
    ];
    let mut writer = exif::experimental::Writer::new();
    for field in &fields {
        writer.push_field(field);
    }
    writer.set_jpeg(b"THUMBNAIL-MUST-GO", In::THUMBNAIL);
    let mut tiff = std::io::Cursor::new(Vec::new());
    writer.write(&mut tiff, true).unwrap();
    let metadata = Metadata {
        exif: Some(tiff.into_inner()),
        xmp: Some("<x:xmpmeta>XMP-MUST-GO</x:xmpmeta>".into()),
    };
    let mut jpeg = encode_jpeg(&smooth_image(48, 32), 95, &metadata).unwrap();
    let mpf = b"MPF\0MPF-MUST-GO";
    let mut segment = vec![0xFF, 0xE2];
    segment.extend(((mpf.len() + 2) as u16).to_be_bytes());
    segment.extend(mpf);
    jpeg.splice(2..2, segment);
    jpeg.extend(b"\xFF\xD8PREVIEW-MUST-GO\xFF\xD9");
    let path = folder.join("IMG_1.jpg");
    fs::write(&path, jpeg).unwrap();
    path
}

const MUST_GO: &str = "MUST-GO";

fn holds(bytes: &[u8], text: &str) -> bool {
    bytes
        .windows(text.len())
        .any(|window| window == text.as_bytes())
}

fn read_exif(path: &Path) -> exif::Exif {
    Reader::new()
        .read_from_container(&mut std::io::BufReader::new(fs::File::open(path).unwrap()))
        .unwrap()
}

fn text(exif: &exif::Exif, tag: Tag) -> String {
    match &exif.get_field(tag, In::PRIMARY).unwrap().value {
        Value::Ascii(parts) => String::from_utf8_lossy(&parts[0]).into_owned(),
        other => panic!("{tag}: {other:?}"),
    }
}

fn number(exif: &exif::Exif, tag: Tag) -> u32 {
    exif.get_field(tag, In::PRIMARY)
        .unwrap()
        .value
        .get_uint(0)
        .unwrap()
}

#[test]
fn every_export_keeps_the_details_and_corrects_them() {
    let folder = tempfile::tempdir().unwrap();
    let photo = photo_with_metadata(folder.path());
    let renderer = renderer();
    for format in [JPEG, ExportFormat::Png] {
        for (size, expected) in [
            (ExportSize::Original, (32, 48)),
            (ExportSize::LongEdge(24), (16, 24)),
        ] {
            let out = tempfile::tempdir().unwrap();
            let options = ExportOptions {
                format,
                size,
                ..Default::default()
            };
            let path = export(
                &renderer,
                &photo,
                &Settings::default(),
                out.path(),
                &options,
                None,
            )
            .unwrap();
            let exif = read_exif(&path);
            assert_eq!(text(&exif, Tag::Make), "Apple");
            assert_eq!(text(&exif, Tag::Model), "iPhone 15 Pro");
            assert_eq!(text(&exif, Tag::DateTimeOriginal), "2026:10:18 19:40:12");
            assert_eq!(text(&exif, Tag::LensModel), "iPhone 15 Pro back camera");
            assert_eq!(number(&exif, Tag::Orientation), 1);
            assert_eq!(number(&exif, Tag::ColorSpace), 1);
            assert_eq!(number(&exif, Tag::PixelXDimension), expected.0);
            assert_eq!(number(&exif, Tag::PixelYDimension), expected.1);
            // The tag and the pixels agree, in Seura's decoder too.
            let image = decode(&path).unwrap().image;
            assert_eq!((image.width, image.height), expected);
        }
    }
}

#[test]
fn nothing_of_the_original_but_its_details_arrives() {
    let folder = tempfile::tempdir().unwrap();
    let photo = photo_with_metadata(folder.path());
    assert!(holds(&fs::read(&photo).unwrap(), "THUMBNAIL-MUST-GO"));
    assert!(holds(&fs::read(&photo).unwrap(), "PREVIEW-MUST-GO"));
    assert!(holds(&fs::read(&photo).unwrap(), "MPF-MUST-GO"));
    assert!(holds(&fs::read(&photo).unwrap(), "XMP-MUST-GO"));
    assert!(holds(&fs::read(&photo).unwrap(), "NOTE-MUST-GO"));
    let renderer = renderer();
    for format in [JPEG, ExportFormat::Png] {
        let out = tempfile::tempdir().unwrap();
        let options = ExportOptions {
            format,
            remove_location: true,
            ..Default::default()
        };
        let path = export(
            &renderer,
            &photo,
            &Settings::default(),
            out.path(),
            &options,
            None,
        )
        .unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(!holds(&bytes, MUST_GO), "{format:?}");
        assert!(!holds(&bytes, "MPF\0"), "{format:?}");
        assert!(!holds(&bytes, "GPS"), "{format:?}");
        assert_eq!(
            read_exif(&path)
                .fields()
                .filter(|field| field.tag.context() == exif::Context::Gps)
                .count(),
            0
        );
    }
}

#[test]
fn the_location_stays_unless_asked_to_go() {
    let folder = tempfile::tempdir().unwrap();
    let photo = photo_with_metadata(folder.path());
    let renderer = renderer();
    for format in [JPEG, ExportFormat::Png] {
        let out = tempfile::tempdir().unwrap();
        let options = ExportOptions {
            format,
            ..Default::default()
        };
        let path = export(
            &renderer,
            &photo,
            &Settings::default(),
            out.path(),
            &options,
            None,
        )
        .unwrap();
        assert_eq!(text(&read_exif(&path), Tag::GPSMapDatum), "GPS-MUST-GO");
    }
}

#[test]
fn tags_and_rating_go_in_only_when_asked() {
    let folder = tempfile::tempdir().unwrap();
    let photo = write_photo(folder.path(), "photo.png", &smooth_image(32, 32));
    let renderer = renderer();
    let keywords = Keywords {
        tags: vec!["family/kids".into()],
        rating: 4,
    };
    for format in [JPEG, ExportFormat::Png] {
        let out = tempfile::tempdir().unwrap();
        let options = ExportOptions {
            format,
            ..Default::default()
        };
        let with = export(
            &renderer,
            &photo,
            &Settings::default(),
            out.path(),
            &options,
            Some(&keywords),
        )
        .unwrap();
        let bytes = fs::read(&with).unwrap();
        assert!(holds(&bytes, "<rdf:li>kids</rdf:li>"), "{format:?}");
        assert!(holds(&bytes, "<rdf:li>family|kids</rdf:li>"), "{format:?}");
        assert!(holds(&bytes, "<xmp:Rating>4</xmp:Rating>"), "{format:?}");
        let without = export(
            &renderer,
            &photo,
            &Settings::default(),
            out.path(),
            &options,
            None,
        )
        .unwrap();
        assert!(
            !holds(&fs::read(&without).unwrap(), "xmpmeta"),
            "{format:?}"
        );
    }
}

/// File Explorer reads a PNG's XMP but not its EXIF, so the details are in
/// both. A JPEG's EXIF is enough.
#[test]
fn a_png_says_its_date_camera_and_lens_in_xmp_too() {
    let folder = tempfile::tempdir().unwrap();
    let photo = photo_with_metadata(folder.path());
    let renderer = renderer();
    let exported = |format| {
        let out = tempfile::tempdir().unwrap();
        let options = ExportOptions {
            format,
            ..Default::default()
        };
        let path = export(
            &renderer,
            &photo,
            &Settings::default(),
            out.path(),
            &options,
            None,
        )
        .unwrap();
        fs::read(path).unwrap()
    };
    let png = exported(ExportFormat::Png);
    assert!(holds(&png, "<tiff:Make>Apple</tiff:Make>"));
    assert!(holds(&png, "<tiff:Model>iPhone 15 Pro</tiff:Model>"));
    assert!(holds(&png, ">iPhone 15 Pro back camera</exifEX:LensModel>"));
    assert!(holds(&png, ">2026-10-18T19:40:12</exif:DateTimeOriginal>"));
    assert!(!holds(&exported(JPEG), "xmpmeta"));
}

#[test]
fn a_photo_with_no_exif_gets_none_invented() {
    let folder = tempfile::tempdir().unwrap();
    let photo = write_photo(folder.path(), "photo.png", &smooth_image(32, 32));
    let renderer = renderer();
    for format in [JPEG, ExportFormat::Png] {
        let out = tempfile::tempdir().unwrap();
        let options = ExportOptions {
            format,
            ..Default::default()
        };
        let path = export(
            &renderer,
            &photo,
            &Settings::default(),
            out.path(),
            &options,
            None,
        )
        .unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(!holds(&bytes, "Exif\0\0") && !holds(&bytes, "eXIf"));
    }
}

#[test]
fn tags_too_many_for_a_jpeg_export_nothing_and_fit_a_png() {
    let folder = tempfile::tempdir().unwrap();
    let photo = write_photo(folder.path(), "photo.png", &smooth_image(32, 32));
    let out = tempfile::tempdir().unwrap();
    let renderer = renderer();
    let keywords = Keywords {
        tags: (0..4000).map(|n| format!("tag number {n}")).collect(),
        rating: 0,
    };
    let exported = |format| {
        export(
            &renderer,
            &photo,
            &Settings::default(),
            out.path(),
            &ExportOptions {
                format,
                ..Default::default()
            },
            Some(&keywords),
        )
    };
    assert!(matches!(exported(JPEG), Err(ExportError::Encode(_))));
    assert_eq!(fs::read_dir(out.path()).unwrap().count(), 0);
    assert!(exported(ExportFormat::Png).is_ok());
}
