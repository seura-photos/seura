//! Encoding exports (EXP-2, EXP-5): JPEG or PNG in sRGB with the sRGB profile
//! embedded, so every viewer shows the colors as intended, and with whatever
//! other metadata the export carries (EXP-4).

use std::sync::LazyLock;

use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{PixelType, ResizeAlg, ResizeOptions, Resizer, create_srgb_mapper};
use jpeg_encoder::{ColorType, Encoder, EncodingError};

use crate::metadata::Metadata;
use crate::raster::Rgba8;
use crate::thumbnail::fitted;

/// The default JPEG quality (EXP-2).
pub const JPEG_QUALITY: u8 = 90;

/// What starts an XMP packet's JPEG segment.
const JPEG_XMP_HEADER: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";

static SRGB_PROFILE: LazyLock<Vec<u8>> = LazyLock::new(|| {
    moxcms::ColorProfile::new_srgb()
        .encode()
        .expect("the built-in sRGB profile encodes")
});

/// An upright sRGB image as JPEG bytes. Alpha is dropped. Metadata too large
/// for a JPEG segment, about 64 KB, is an error rather than quietly lost.
pub fn encode_jpeg(
    image: &Rgba8,
    quality: u8,
    metadata: &Metadata,
) -> Result<Vec<u8>, EncodingError> {
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes, quality);
    encoder.set_optimized_huffman_tables(true);
    if let Some(exif) = &metadata.exif {
        encoder.add_exif_metadata(exif)?;
    }
    if let Some(xmp) = &metadata.xmp {
        encoder.add_app_segment(1, [JPEG_XMP_HEADER, xmp.as_bytes()].concat())?;
    }
    encoder.add_icc_profile(&SRGB_PROFILE)?;
    // Width and height are at most MAX_SIDE, well within JPEG's limit.
    encoder.encode(
        &image.pixels,
        image.width as u16,
        image.height as u16,
        ColorType::Rgba,
    )?;
    Ok(bytes)
}

/// An upright sRGB image as PNG bytes. Alpha is dropped: a render is opaque.
pub fn encode_png(image: &Rgba8, metadata: &Metadata) -> Result<Vec<u8>, png::EncodingError> {
    let mut info = png::Info::with_size(image.width, image.height);
    info.color_type = png::ColorType::Rgb;
    info.bit_depth = png::BitDepth::Eight;
    info.icc_profile = Some(SRGB_PROFILE.as_slice().into());
    info.exif_metadata = metadata.exif.as_deref().map(Into::into);
    if let Some(xmp) = &metadata.xmp {
        info.utf8_text
            .push(png::text_metadata::ITXtChunk::new("XML:com.adobe.xmp", xmp));
    }
    let mut bytes = Vec::new();
    let mut writer = png::Encoder::with_info(&mut bytes, info)?.write_header()?;
    let rgb: Vec<u8> = image
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|&[r, g, b, _]| [r, g, b])
        .collect();
    writer.write_image_data(&rgb)?;
    writer.finish()?;
    Ok(bytes)
}

/// An opaque sRGB image scaled to fit `long_edge`, in linear light, so fine
/// detail keeps its brightness. One already that small is returned as it is:
/// a limit never enlarges (EXP-2).
pub fn fit_for_export(image: Rgba8, long_edge: u32) -> Rgba8 {
    let (width, height) = fitted(image.width, image.height, long_edge);
    if (width, height) == (image.width, image.height) {
        return image;
    }
    let mapper = create_srgb_mapper();
    let source = ImageRef::new(image.width, image.height, &image.pixels, PixelType::U8x4)
        .expect("the buffer matches the image size");
    let mut linear = Image::new(image.width, image.height, PixelType::U16x4);
    mapper
        .forward_map(&source, &mut linear)
        .expect("the same size on both sides");
    let mut scaled = Image::new(width, height, PixelType::U16x4);
    let options = ResizeOptions::new()
        .resize_alg(ResizeAlg::Convolution(
            fast_image_resize::FilterType::Lanczos3,
        ))
        .use_alpha(false);
    Resizer::new()
        .resize(&linear, &mut scaled, &options)
        .expect("same pixel type on both sides");
    let mut target = Image::new(width, height, PixelType::U8x4);
    mapper
        .backward_map(&scaled, &mut target)
        .expect("the same size on both sides");
    Rgba8::new(width, height, target.into_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_decode_back_with_their_srgb_profile() {
        let image = Rgba8::new(16, 8, [200, 100, 50, 255].repeat(16 * 8));
        let bytes = encode_jpeg(&image, JPEG_QUALITY, &Metadata::default()).unwrap();
        let decoded = crate::decode::decode_jpeg(&bytes).unwrap();
        let profile = moxcms::ColorProfile::new_from_slice(decoded.icc.as_deref().unwrap());
        assert!(profile.is_ok(), "the embedded profile reads back");
        assert_eq!((decoded.image.width, decoded.image.height), (16, 8));
        let pixel = &decoded.image.pixels[..3];
        assert!(
            pixel
                .iter()
                .zip([200, 100, 50])
                .all(|(a, b)| a.abs_diff(b) <= 3),
            "{pixel:?}"
        );
    }

    #[test]
    fn png_exports_keep_every_value_and_carry_the_srgb_profile() {
        let pixels: Vec<u8> = (0..=255).flat_map(|v| [v, 255 - v, v / 2, 255]).collect();
        let image = Rgba8::new(16, 16, pixels);
        let decoded =
            crate::decode::decode_png(&encode_png(&image, &Metadata::default()).unwrap()).unwrap();
        assert_eq!((decoded.image.width, decoded.image.height), (16, 16));
        assert_eq!(decoded.image.pixels, image.pixels);
        assert_eq!(decoded.icc.as_deref(), Some(SRGB_PROFILE.as_slice()));
    }

    fn found(bytes: &[u8], text: &[u8]) -> bool {
        bytes.windows(text.len()).any(|window| window == text)
    }

    #[test]
    fn metadata_goes_in_where_other_apps_look_for_it() {
        let image = Rgba8::new(8, 8, [9, 8, 7, 255].repeat(64));
        let metadata = Metadata {
            exif: Some(b"MM\0*\0\0\0\x08\0\0\0\0\0\0".to_vec()),
            xmp: Some("<packet/>".into()),
        };
        let jpeg = encode_jpeg(&image, JPEG_QUALITY, &metadata).unwrap();
        assert!(found(&jpeg, b"Exif\0\0MM\0*"));
        assert!(found(&jpeg, b"http://ns.adobe.com/xap/1.0/\0<packet/>"));
        let png = encode_png(&image, &metadata).unwrap();
        assert!(found(&png, b"eXIfMM\0*"));
        assert!(found(&png, b"iTXtXML:com.adobe.xmp\0\0\0\0\0<packet/>"));
        // Before the pixels in both.
        let pixels = png.windows(4).position(|window| window == b"IDAT").unwrap();
        let text = png.windows(4).position(|window| window == b"iTXt").unwrap();
        assert!(text < pixels);
    }

    #[test]
    fn no_metadata_means_no_segments() {
        let image = Rgba8::new(8, 8, [9, 8, 7, 255].repeat(64));
        let jpeg = encode_jpeg(&image, JPEG_QUALITY, &Metadata::default()).unwrap();
        assert!(!found(&jpeg, b"Exif\0\0") && !found(&jpeg, b"ns.adobe.com"));
        let png = encode_png(&image, &Metadata::default()).unwrap();
        assert!(!found(&png, b"eXIf") && !found(&png, b"iTXt"));
    }

    #[test]
    fn metadata_too_large_for_a_jpeg_is_an_error_and_fine_in_a_png() {
        let image = Rgba8::new(8, 8, [9, 8, 7, 255].repeat(64));
        let metadata = Metadata {
            exif: None,
            xmp: Some("x".repeat(70_000)),
        };
        assert!(encode_jpeg(&image, JPEG_QUALITY, &metadata).is_err());
        assert!(encode_png(&image, &metadata).is_ok());
    }

    #[test]
    fn a_limit_never_enlarges() {
        let image = Rgba8::new(40, 20, [9, 8, 7, 255].repeat(40 * 20));
        let same = fit_for_export(image.clone(), 40);
        assert_eq!(
            (same.width, same.height, same.pixels),
            (40, 20, image.pixels.clone())
        );
        let same = fit_for_export(image.clone(), 2048);
        assert_eq!(
            (same.width, same.height, same.pixels),
            (40, 20, image.pixels)
        );
    }

    #[test]
    fn the_long_edge_is_limited_and_the_shape_kept() {
        let wide = Rgba8::new(4000, 3000, [9, 8, 7, 255].repeat(4000 * 3000));
        let fitted = fit_for_export(wide, 1024);
        assert_eq!((fitted.width, fitted.height), (1024, 768));
        let tall = Rgba8::new(30, 60, [9, 8, 7, 255].repeat(30 * 60));
        let fitted = fit_for_export(tall, 16);
        assert_eq!((fitted.width, fitted.height), (8, 16));
        // A very thin photo still has a row of pixels.
        let thin = Rgba8::new(1000, 2, [9, 8, 7, 255].repeat(2000));
        let fitted = fit_for_export(thin, 16);
        assert_eq!((fitted.width, fitted.height), (16, 1));
    }

    #[test]
    fn scaling_is_done_in_linear_light() {
        // Black and white stripes average to half the light, which sRGB
        // encodes as 188. Averaging the encoded values would give 128.
        let pixels: Vec<u8> = (0..64 * 64)
            .flat_map(|i| {
                let v = if i % 2 == 0 { 0 } else { 255 };
                [v, v, v, 255]
            })
            .collect();
        let scaled = fit_for_export(Rgba8::new(64, 64, pixels), 16);
        // Away from the edges, where the filter has only one side to use.
        for y in 4..12 {
            for x in 4..12 {
                let pixel = &scaled.pixels[(y * 16 + x) * 4..][..4];
                assert!(pixel[0].abs_diff(188) <= 1, "{pixel:?}");
                assert_eq!(pixel[3], 255);
            }
        }
    }

    #[test]
    fn a_flat_color_stays_the_same_when_scaled() {
        let image = Rgba8::new(64, 64, [200, 100, 50, 255].repeat(64 * 64));
        let scaled = fit_for_export(image, 16);
        for pixel in scaled.pixels.chunks(4) {
            assert_eq!(pixel, [200, 100, 50, 255]);
        }
    }
}
