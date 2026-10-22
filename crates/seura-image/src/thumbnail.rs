//! Thumbnails: large enough for the biggest grid size on a 2× screen
//! (DSP-4), WebP for size (technical design, section 4).

use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{PixelType, ResizeAlg, ResizeOptions, Resizer};

use crate::raster::Rgba8;

/// The thumbnail's long edge, in pixels.
pub const THUMBNAIL_SIZE: u32 = 512;

/// WebP quality, 0 to 100. 80 keeps photos clean at grid sizes.
const QUALITY: f32 = 80.0;

/// Scales an image to fit within `size` × `size`, keeping its proportions.
/// Images already that small are left alone.
pub fn fit(image: &Rgba8, size: u32) -> Rgba8 {
    let long = image.width.max(image.height);
    if long <= size {
        return image.clone();
    }
    let scale = f64::from(size) / f64::from(long);
    let width = ((f64::from(image.width) * scale).round() as u32).max(1);
    let height = ((f64::from(image.height) * scale).round() as u32).max(1);
    // Borrowed, not copied: a 12-megapixel photo is about 48 MB.
    let source = ImageRef::new(image.width, image.height, &image.pixels, PixelType::U8x4)
        .expect("the buffer matches the image size");
    let mut target = Image::new(width, height, PixelType::U8x4);
    let options = ResizeOptions::new().resize_alg(ResizeAlg::Convolution(
        fast_image_resize::FilterType::Lanczos3,
    ));
    Resizer::new()
        .resize(&source, &mut target, &options)
        .expect("same pixel type on both sides");
    Rgba8::new(width, height, target.into_vec())
}

/// The thumbnail for an upright sRGB image, as WebP bytes.
pub fn thumbnail(image: &Rgba8) -> Vec<u8> {
    let small = fit(image, THUMBNAIL_SIZE);
    webp::Encoder::from_rgba(&small.pixels, small.width, small.height)
        .encode(QUALITY)
        .to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_long_edge_becomes_the_thumbnail_size() {
        let image = Rgba8::new(4032, 3024, vec![128; 4032 * 3024 * 4]);
        let small = fit(&image, THUMBNAIL_SIZE);
        assert_eq!((small.width, small.height), (512, 384));
        let portrait = Rgba8::new(300, 1200, vec![128; 300 * 1200 * 4]);
        assert_eq!(fit(&portrait, THUMBNAIL_SIZE).height, 512);
        let tiny = Rgba8::new(100, 50, vec![128; 100 * 50 * 4]);
        assert_eq!(fit(&tiny, THUMBNAIL_SIZE), tiny);
    }

    #[test]
    fn thumbnails_are_webp() {
        let image = Rgba8::new(640, 480, vec![200; 640 * 480 * 4]);
        let bytes = thumbnail(&image);
        assert_eq!(&bytes[..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WEBP");
    }
}
