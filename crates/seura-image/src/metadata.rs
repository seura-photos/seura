//! What an export carries besides its pixels (EXP-4): the original's EXIF,
//! rebuilt and corrected, and optionally the photo's tags and rating as XMP.
//!
//! The EXIF block is written new rather than copied, and only the tags in
//! [`KEPT`] and, unless asked not to, the location go in. So the thumbnail, a
//! maker note, and anything else that could hold a preview or a location
//! never arrive. The original's XMP is never read.
//!
//! File Explorer doesn't read a PNG's EXIF, though it does read its XMP, so a
//! PNG also says its date, camera, and lens there.

use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;

use exif::experimental::Writer;
use exif::{Context, Field, In, Reader, Tag, Value};

use crate::exif::parse_date;

/// What [`crate::encode_jpeg`] and [`crate::encode_png`] embed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Metadata {
    /// An EXIF block, starting at its TIFF header.
    pub exif: Option<Vec<u8>>,
    /// An XMP packet.
    pub xmp: Option<String>,
}

/// A photo's tags and rating, as the edit format has them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Keywords {
    pub tags: Vec<String>,
    /// 0 to 5; 0 means unrated.
    pub rating: u8,
}

/// The original's tags that still describe an export: how and when the photo
/// was taken, by what, and by whom. Left out are the ones about the original
/// file's pixels and how they were stored, including the sizes, color
/// descriptions, and positions in the photo (`SubjectArea`), which an edit
/// would make wrong; and anything not named here, such as maker notes, which
/// can hold previews.
const KEPT: &[Tag] = &[
    Tag::ImageDescription,
    Tag::Make,
    Tag::Model,
    Tag::XResolution,
    Tag::YResolution,
    Tag::ResolutionUnit,
    Tag::Software,
    Tag::DateTime,
    Tag::Artist,
    Tag::Copyright,
    Tag::ExposureTime,
    Tag::FNumber,
    Tag::ExposureProgram,
    Tag::SpectralSensitivity,
    Tag::PhotographicSensitivity,
    Tag::OECF,
    Tag::SensitivityType,
    Tag::StandardOutputSensitivity,
    Tag::RecommendedExposureIndex,
    Tag::ISOSpeed,
    Tag::ISOSpeedLatitudeyyy,
    Tag::ISOSpeedLatitudezzz,
    Tag::ExifVersion,
    Tag::DateTimeOriginal,
    Tag::DateTimeDigitized,
    Tag::OffsetTime,
    Tag::OffsetTimeOriginal,
    Tag::OffsetTimeDigitized,
    Tag::ShutterSpeedValue,
    Tag::ApertureValue,
    Tag::BrightnessValue,
    Tag::ExposureBiasValue,
    Tag::MaxApertureValue,
    Tag::SubjectDistance,
    Tag::MeteringMode,
    Tag::LightSource,
    Tag::Flash,
    Tag::FocalLength,
    Tag::UserComment,
    Tag::SubSecTime,
    Tag::SubSecTimeOriginal,
    Tag::SubSecTimeDigitized,
    Tag::Temperature,
    Tag::Humidity,
    Tag::Pressure,
    Tag::WaterDepth,
    Tag::Acceleration,
    Tag::CameraElevationAngle,
    Tag::FlashpixVersion,
    Tag::FlashEnergy,
    Tag::ExposureIndex,
    Tag::SensingMethod,
    Tag::FileSource,
    Tag::SceneType,
    Tag::CustomRendered,
    Tag::ExposureMode,
    Tag::WhiteBalance,
    Tag::DigitalZoomRatio,
    Tag::FocalLengthIn35mmFilm,
    Tag::SceneCaptureType,
    Tag::GainControl,
    Tag::Contrast,
    Tag::Saturation,
    Tag::Sharpness,
    Tag::SubjectDistanceRange,
    Tag::ImageUniqueID,
    Tag::CameraOwnerName,
    Tag::BodySerialNumber,
    Tag::LensSpecification,
    Tag::LensMake,
    Tag::LensModel,
    Tag::LensSerialNumber,
];

/// What an export of `photo` at this size embeds (EXP-4), with the date,
/// camera, and lens in the XMP too if `details_in_xmp`. The photo is only read
/// (CLAUDE.md rule 1).
pub fn metadata_for_export(
    photo: &Path,
    size: (u32, u32),
    remove_location: bool,
    keywords: Option<&Keywords>,
    details_in_xmp: bool,
) -> Metadata {
    // A photo with no EXIF gets none, and so does one whose EXIF can't be
    // read: a photo with damaged metadata still exports.
    let exif = read_exif(photo);
    let details = exif
        .as_ref()
        .filter(|_| details_in_xmp)
        .map(XmpDetails::from)
        .unwrap_or_default();
    Metadata {
        exif: exif
            .as_ref()
            .and_then(|exif| rebuilt_exif(exif, size, remove_location)),
        xmp: xmp_packet(keywords.unwrap_or(&Keywords::default()), &details),
    }
}

fn read_exif(photo: &Path) -> Option<exif::Exif> {
    let file = File::open(photo).ok()?;
    Reader::new()
        .continue_on_error(true)
        .read_from_container(&mut BufReader::new(file))
        .or_else(|error| error.distill_partial_result(|_| {}))
        .ok()
}

/// The photo's EXIF, rebuilt and corrected, or none if it can't be written
/// again.
fn rebuilt_exif(
    exif: &exif::Exif,
    (width, height): (u32, u32),
    remove_location: bool,
) -> Option<Vec<u8>> {
    let field = |tag, value| Field {
        tag,
        ifd_num: In::PRIMARY,
        value,
    };
    // The pixels are upright and in sRGB, at this size.
    let corrected = [
        field(Tag::Orientation, Value::Short(vec![1])),
        field(Tag::ColorSpace, Value::Short(vec![1])),
        field(Tag::PixelXDimension, Value::Long(vec![width])),
        field(Tag::PixelYDimension, Value::Long(vec![height])),
    ];
    let mut writer = Writer::new();
    for field in exif
        .fields()
        .filter(|field| {
            field.ifd_num == In::PRIMARY
                && !matches!(field.value, Value::Unknown(..))
                && (KEPT.contains(&field.tag)
                    || (field.tag.context() == Context::Gps && !remove_location))
        })
        .chain(&corrected)
    {
        writer.push_field(field);
    }
    let mut bytes = Cursor::new(Vec::new());
    writer.write(&mut bytes, exif.little_endian()).ok()?;
    Some(bytes.into_inner())
}

/// The date, camera, and lens, as XML text, for a packet.
#[derive(Default)]
struct XmpDetails {
    make: Option<String>,
    model: Option<String>,
    lens: Option<String>,
    /// As XMP writes dates: `2026-10-18T19:40:12`, with the time zone if the
    /// photo says.
    taken: Option<String>,
}

impl From<&exif::Exif> for XmpDetails {
    fn from(exif: &exif::Exif) -> Self {
        let text = |tag: Tag| -> Option<String> {
            match &exif.get_field(tag, In::PRIMARY)?.value {
                Value::Ascii(parts) => {
                    let text = String::from_utf8_lossy(parts.first()?);
                    Some(text.into_owned()).filter(|text| !text.trim().is_empty())
                }
                _ => None,
            }
        };
        let taken = text(Tag::DateTimeOriginal)
            .filter(|date| parse_date(date.as_bytes()).is_some())
            .map(|date| {
                let date = date.replacen(':', "-", 2).replacen(' ', "T", 1);
                let zone = text(Tag::OffsetTimeOriginal).unwrap_or_default();
                format!("{date}{zone}")
            });
        XmpDetails {
            make: text(Tag::Make).map(|text| xml_text(&text)),
            model: text(Tag::Model).map(|text| xml_text(&text)),
            lens: text(Tag::LensModel).map(|text| xml_text(&text)),
            taken: taken.map(|text| xml_text(&text)),
        }
    }
}

/// Tags, rating, and details as a small XMP packet, or none if there are none.
/// Each level of a tag is its own standard keyword, without repeats, and the
/// whole tag goes in Lightroom's hierarchical keywords: `family/kids` gives
/// `family`, `kids`, and `family|kids`.
fn xmp_packet(keywords: &Keywords, details: &XmpDetails) -> Option<String> {
    let mut subjects: Vec<String> = Vec::new();
    let mut hierarchical: Vec<String> = Vec::new();
    for tag in &keywords.tags {
        let levels: Vec<String> = tag
            .split('/')
            .map(xml_text)
            .filter(|level| !level.is_empty())
            .collect();
        for level in &levels {
            // Letter case, as the edit format treats tags.
            if !subjects.iter().any(|s| s.eq_ignore_ascii_case(level)) {
                subjects.push(level.clone());
            }
        }
        if !levels.is_empty() {
            hierarchical.push(levels.join("|"));
        }
    }
    let element = |name: &str, value: &Option<String>| {
        value
            .as_ref()
            .map_or(String::new(), |value| format!("<{name}>{value}</{name}>"))
    };
    let details = [
        element("tiff:Make", &details.make),
        element("tiff:Model", &details.model),
        element("exifEX:LensModel", &details.lens),
        element("exif:DateTimeOriginal", &details.taken),
    ]
    .concat();
    if subjects.is_empty() && keywords.rating == 0 && details.is_empty() {
        return None;
    }
    let bag = |name: &str, items: &[String]| {
        if items.is_empty() {
            return String::new();
        }
        let items: String = items
            .iter()
            .map(|item| format!("<rdf:li>{item}</rdf:li>"))
            .collect();
        format!("<{name}><rdf:Bag>{items}</rdf:Bag></{name}>")
    };
    let rating = match keywords.rating {
        0 => String::new(),
        rating => format!("<xmp:Rating>{rating}</xmp:Rating>"),
    };
    Some(format!(
        concat!(
            "<?xpacket begin=\"\u{FEFF}\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>",
            "<x:xmpmeta xmlns:x=\"adobe:ns:meta/\">",
            "<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">",
            "<rdf:Description rdf:about=\"\"",
            " xmlns:dc=\"http://purl.org/dc/elements/1.1/\"",
            " xmlns:exif=\"http://ns.adobe.com/exif/1.0/\"",
            " xmlns:exifEX=\"http://cipa.jp/exif/1.0/\"",
            " xmlns:lr=\"http://ns.adobe.com/lightroom/1.0/\"",
            " xmlns:tiff=\"http://ns.adobe.com/tiff/1.0/\"",
            " xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\">",
            "{details}{rating}{subject}{hierarchical}",
            "</rdf:Description></rdf:RDF></x:xmpmeta>",
            "<?xpacket end=\"r\"?>",
        ),
        details = details,
        rating = rating,
        subject = bag("dc:subject", &subjects),
        hierarchical = bag("lr:hierarchicalSubject", &hierarchical),
    ))
}

/// A level of a tag as XML text: trimmed, escaped, and without the few
/// characters XML can't hold at all, which would spoil the whole packet.
fn xml_text(level: &str) -> String {
    let mut text = String::with_capacity(level.len());
    for c in level.trim().chars() {
        match c {
            '&' => text.push_str("&amp;"),
            '<' => text.push_str("&lt;"),
            '>' => text.push_str("&gt;"),
            '\t' | '\n' | '\r' => text.push(' '),
            c if c < ' ' || matches!(c, '\u{FFFE}' | '\u{FFFF}') => {}
            c => text.push(c),
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(tag: Tag, value: Value) -> Field {
        Field {
            tag,
            ifd_num: In::PRIMARY,
            value,
        }
    }

    fn ascii(text: &str) -> Value {
        Value::Ascii(vec![text.as_bytes().to_vec()])
    }

    /// An EXIF block with what a camera writes, what a phone says about
    /// color, and each thing that must not reach an export.
    fn camera_exif() -> Vec<u8> {
        let fields = [
            field(Tag::Make, ascii("Canon")),
            field(Tag::Model, ascii("Canon EOS R6")),
            field(Tag::DateTimeOriginal, ascii("2026:10:18 19:40:12")),
            field(Tag::LensModel, ascii("RF24-105mm")),
            field(Tag::Orientation, Value::Short(vec![6])),
            field(Tag::ColorSpace, Value::Short(vec![65535])),
            field(Tag::PixelXDimension, Value::Long(vec![4000])),
            field(Tag::PixelYDimension, Value::Long(vec![3000])),
            field(Tag::SubjectArea, Value::Short(vec![10, 20, 30, 40])),
            field(Tag::GPSLatitudeRef, ascii("N")),
            field(Tag::GPSMapDatum, ascii("WGS-84")),
            field(Tag::InteroperabilityIndex, ascii("R03")),
            field(Tag::MakerNote, Value::Undefined(b"maker note".to_vec(), 0)),
            // Windows' own rating, which isn't one of Seura's.
            field(Tag(Context::Tiff, 0x4746), Value::Short(vec![2])),
        ];
        let mut writer = Writer::new();
        for field in &fields {
            writer.push_field(field);
        }
        writer.set_jpeg(b"embedded thumbnail", In::THUMBNAIL);
        let thumbnail_field = field(Tag::Make, ascii("thumbnail make"));
        let thumbnail_field = Field {
            ifd_num: In::THUMBNAIL,
            ..thumbnail_field
        };
        writer.push_field(&thumbnail_field);
        let mut bytes = Cursor::new(Vec::new());
        writer.write(&mut bytes, false).unwrap();
        bytes.into_inner()
    }

    fn jpeg_with(tiff: &[u8]) -> Vec<u8> {
        let mut app1 = b"Exif\0\0".to_vec();
        app1.extend(tiff);
        let mut jpeg = vec![0xFF, 0xD8, 0xFF, 0xE1];
        jpeg.extend(((app1.len() + 2) as u16).to_be_bytes());
        jpeg.extend(app1);
        jpeg.extend([0xFF, 0xD9]);
        jpeg
    }

    /// A minimal HEIF file whose only item is the EXIF block.
    fn heif_with(tiff: &[u8]) -> Vec<u8> {
        let boxed = |kind: &[u8; 4], body: &[u8]| {
            let mut bytes = ((body.len() + 8) as u32).to_be_bytes().to_vec();
            bytes.extend(kind);
            bytes.extend(body);
            bytes
        };
        let ftyp = boxed(b"ftyp", b"heic\0\0\0\0mif1heic");
        let infe = boxed(b"infe", b"\x02\0\0\0\0\x01\0\0Exif\0");
        let iinf = boxed(b"iinf", &[b"\0\0\0\0\0\x01".as_slice(), &infe].concat());
        let iloc_for = |offset: u32, length: u32| {
            let mut body = b"\0\0\0\0\x44\0\0\x01\0\x01\0\0\0\x01".to_vec();
            body.extend(offset.to_be_bytes());
            body.extend(length.to_be_bytes());
            boxed(b"iloc", &body)
        };
        let meta_for = |offset: u32, length: u32| {
            let body = [b"\0\0\0\0".as_slice(), &iinf, &iloc_for(offset, length)].concat();
            boxed(b"meta", &body)
        };
        let item = [&[0u8; 4], tiff].concat();
        let offset = ftyp.len() + meta_for(0, 0).len() + 8;
        let meta = meta_for(offset as u32, item.len() as u32);
        [ftyp, meta, boxed(b"mdat", &item)].concat()
    }

    fn png_with(tiff: &[u8]) -> Vec<u8> {
        let mut info = png::Info::with_size(1, 1);
        info.color_type = png::ColorType::Grayscale;
        info.exif_metadata = Some(tiff.into());
        let mut bytes = Vec::new();
        let mut writer = png::Encoder::with_info(&mut bytes, info)
            .unwrap()
            .write_header()
            .unwrap();
        writer.write_image_data(&[0]).unwrap();
        writer.finish().unwrap();
        bytes
    }

    fn exported(file: &[u8], remove_location: bool) -> exif::Exif {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("photo");
        std::fs::write(&path, file).unwrap();
        let exif = read_exif(&path).unwrap();
        let bytes = rebuilt_exif(&exif, (800, 600), remove_location).unwrap();
        Reader::new().read_raw(bytes).unwrap()
    }

    #[test]
    fn exif_is_read_from_every_kind_of_original() {
        let tiff = camera_exif();
        for file in [jpeg_with(&tiff), heif_with(&tiff), png_with(&tiff)] {
            let exif = exported(&file, false);
            let make = exif.get_field(Tag::Make, In::PRIMARY).unwrap();
            assert_eq!(make.display_value().to_string(), "\"Canon\"");
        }
    }

    #[test]
    fn the_camera_and_date_are_kept() {
        let exif = exported(&jpeg_with(&camera_exif()), false);
        for (tag, value) in [
            (Tag::Make, "Canon"),
            (Tag::Model, "Canon EOS R6"),
            (Tag::DateTimeOriginal, "2026:10:18 19:40:12"),
            (Tag::LensModel, "RF24-105mm"),
        ] {
            let found = exif.get_field(tag, In::PRIMARY).unwrap();
            let Value::Ascii(found) = &found.value else {
                panic!("{tag}");
            };
            assert_eq!(found[0], value.as_bytes(), "{tag}");
        }
    }

    #[test]
    fn what_the_pixels_changed_is_corrected() {
        let exif = exported(&jpeg_with(&camera_exif()), false);
        let number = |tag| {
            exif.get_field(tag, In::PRIMARY)
                .and_then(|field| field.value.get_uint(0))
        };
        assert_eq!(number(Tag::Orientation), Some(1));
        assert_eq!(number(Tag::ColorSpace), Some(1));
        assert_eq!(number(Tag::PixelXDimension), Some(800));
        assert_eq!(number(Tag::PixelYDimension), Some(600));
    }

    #[test]
    fn what_could_hold_a_picture_or_mislead_is_dropped() {
        let exif = exported(&jpeg_with(&camera_exif()), false);
        for tag in [
            Tag::MakerNote,
            Tag::SubjectArea,
            Tag::InteroperabilityIndex,
            Tag(Context::Tiff, 0x4746),
        ] {
            assert!(exif.get_field(tag, In::PRIMARY).is_none(), "{tag}");
        }
        assert!(exif.fields().all(|field| field.ifd_num == In::PRIMARY));
        assert!(
            exif.get_field(Tag::JPEGInterchangeFormat, In::THUMBNAIL)
                .is_none()
        );
        let bytes = exif.buf();
        for text in [&b"embedded thumbnail"[..], b"maker note", b"thumbnail make"] {
            assert!(!bytes.windows(text.len()).any(|window| window == text));
        }
    }

    #[test]
    fn the_location_goes_only_when_asked() {
        let file = jpeg_with(&camera_exif());
        let gps = |exif: &exif::Exif| {
            exif.fields()
                .filter(|field| field.tag.context() == Context::Gps)
                .count()
        };
        assert_eq!(gps(&exported(&file, false)), 2);
        let removed = exported(&file, true);
        assert_eq!(gps(&removed), 0);
        assert!(
            removed
                .get_field(Tag::GPSInfoIFDPointer, In::PRIMARY)
                .is_none()
        );
        // Only the location goes.
        assert!(removed.get_field(Tag::Make, In::PRIMARY).is_some());
    }

    #[test]
    fn a_photo_without_exif_gets_none() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("plain.jpg");
        std::fs::write(&path, [0xFF, 0xD8, 0xFF, 0xD9]).unwrap();
        let metadata = metadata_for_export(&path, (8, 8), false, None, true);
        assert_eq!(metadata, Metadata::default());
        let missing = folder.path().join("missing.jpg");
        let metadata = metadata_for_export(&missing, (8, 8), false, None, true);
        assert_eq!(metadata, Metadata::default());
    }

    #[test]
    fn damaged_exif_exports_without_it() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("damaged.jpg");
        std::fs::write(&path, jpeg_with(b"not exif at all")).unwrap();
        let metadata = metadata_for_export(&path, (8, 8), false, None, true);
        assert_eq!(metadata, Metadata::default());
    }

    fn keywords(tags: &[&str], rating: u8) -> Option<String> {
        let keywords = Keywords {
            tags: tags.iter().map(|tag| tag.to_string()).collect(),
            rating,
        };
        xmp_packet(&keywords, &XmpDetails::default())
    }

    #[test]
    fn a_tag_s_levels_are_keywords_and_the_whole_tag_is_hierarchical() {
        let packet = keywords(&["family/kids", "vacation/2026", "family"], 4).unwrap();
        assert!(packet.contains(
            "<dc:subject><rdf:Bag><rdf:li>family</rdf:li><rdf:li>kids</rdf:li>\
             <rdf:li>vacation</rdf:li><rdf:li>2026</rdf:li></rdf:Bag></dc:subject>"
        ));
        assert!(packet.contains(
            "<lr:hierarchicalSubject><rdf:Bag><rdf:li>family|kids</rdf:li>\
             <rdf:li>vacation|2026</rdf:li><rdf:li>family</rdf:li></rdf:Bag>\
             </lr:hierarchicalSubject>"
        ));
        assert!(packet.contains("<xmp:Rating>4</xmp:Rating>"));
    }

    #[test]
    fn levels_differing_only_in_case_are_one_keyword() {
        let packet = keywords(&["Family/kids", "family/dogs"], 0).unwrap();
        assert!(packet.contains(
            "<dc:subject><rdf:Bag><rdf:li>Family</rdf:li><rdf:li>kids</rdf:li>\
             <rdf:li>dogs</rdf:li></rdf:Bag></dc:subject>"
        ));
        assert!(packet.contains("<rdf:li>Family|kids</rdf:li><rdf:li>family|dogs</rdf:li>"));
    }

    #[test]
    fn details_go_in_the_packet_only_when_asked_and_never_the_location() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("photo.jpg");
        std::fs::write(&path, jpeg_with(&camera_exif())).unwrap();

        let packet = metadata_for_export(&path, (8, 8), false, None, true)
            .xmp
            .unwrap();
        assert!(packet.contains("<tiff:Make>Canon</tiff:Make>"));
        assert!(packet.contains("<tiff:Model>Canon EOS R6</tiff:Model>"));
        assert!(packet.contains("<exifEX:LensModel>RF24-105mm</exifEX:LensModel>"));
        assert!(packet.contains("<exif:DateTimeOriginal>2026-10-18T19:40:12</"));
        assert!(!packet.contains("GPS"));

        let without = metadata_for_export(&path, (8, 8), false, None, false);
        assert_eq!(without.xmp, None);
    }

    #[test]
    fn a_time_zone_is_kept_and_an_unset_clock_is_left_out() {
        let packet_for = |date: &str| {
            let fields = [
                field(Tag::DateTimeOriginal, ascii(date)),
                field(Tag::OffsetTimeOriginal, ascii("-07:00")),
            ];
            let mut writer = Writer::new();
            for field in &fields {
                writer.push_field(field);
            }
            let mut bytes = Cursor::new(Vec::new());
            writer.write(&mut bytes, false).unwrap();
            let exif = Reader::new().read_raw(bytes.into_inner()).unwrap();
            xmp_packet(&Keywords::default(), &XmpDetails::from(&exif))
        };
        assert!(
            packet_for("2026:10:18 19:40:12")
                .unwrap()
                .contains(">2026-10-18T19:40:12-07:00<")
        );
        assert_eq!(packet_for("0000:00:00 00:00:00"), None);
    }

    #[test]
    fn nothing_to_say_means_no_packet() {
        assert_eq!(keywords(&[], 0), None);
        assert_eq!(keywords(&[" / "], 0), None);
    }

    #[test]
    fn a_rating_alone_makes_a_packet_and_unrated_says_nothing() {
        let packet = keywords(&[], 5).unwrap();
        assert!(packet.contains("<xmp:Rating>5</xmp:Rating>"));
        assert!(!packet.contains("dc:subject"));
        assert!(!keywords(&["a"], 0).unwrap().contains("xmp:Rating"));
    }

    #[test]
    fn tags_are_valid_xml_whatever_they_hold() {
        let packet = keywords(&["R&D/<b>", "été/写真", "a\u{1}b"], 0).unwrap();
        assert!(packet.contains("<rdf:li>R&amp;D</rdf:li><rdf:li>&lt;b&gt;</rdf:li>"));
        assert!(packet.contains("<rdf:li>été</rdf:li><rdf:li>写真</rdf:li>"));
        assert!(packet.contains("<rdf:li>ab</rdf:li>"));
    }
}
