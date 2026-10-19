//! Encoding exports (EXP-2, EXP-5): JPEG in sRGB with the sRGB profile
//! embedded, so every viewer shows the colors as intended.

use std::sync::LazyLock;

use jpeg_encoder::{ColorType, Encoder, EncodingError};

use crate::raster::Rgba8;

/// The default JPEG quality (EXP-2).
pub const JPEG_QUALITY: u8 = 90;

static SRGB_PROFILE: LazyLock<Vec<u8>> = LazyLock::new(|| {
    moxcms::ColorProfile::new_srgb()
        .encode()
        .expect("the built-in sRGB profile encodes")
});

/// An upright sRGB image as JPEG bytes. Alpha is dropped.
pub fn encode_jpeg(image: &Rgba8, quality: u8) -> Result<Vec<u8>, EncodingError> {
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes, quality);
    encoder.set_optimized_huffman_tables(true);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_decode_back_with_their_srgb_profile() {
        let image = Rgba8::new(16, 8, [200, 100, 50, 255].repeat(16 * 8));
        let bytes = encode_jpeg(&image, JPEG_QUALITY).unwrap();
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
}
