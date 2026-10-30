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
    if image.width.max(image.height) <= size {
        return image.clone();
    }
    let (width, height) = fitted(image.width, image.height, size);
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

/// The size [`fit`] scales a `width` × `height` image to.
pub(crate) fn fitted(width: u32, height: u32, size: u32) -> (u32, u32) {
    let long = width.max(height);
    if long <= size {
        return (width, height);
    }
    let scale = f64::from(size) / f64::from(long);
    let scaled = |side: u32| ((f64::from(side) * scale).round() as u32).max(1);
    (scaled(width), scaled(height))
}

/// A part of an image, in its pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// The `region` of an image (which must lie within it), scaled to
/// `width` × `height`. Copied as it is when that's its own size.
pub(crate) fn resize_region(image: &Rgba8, region: Region, width: u32, height: u32) -> Rgba8 {
    if (width, height) == (region.width, region.height) {
        let row = region.width as usize * 4;
        let mut pixels = Vec::with_capacity(row * region.height as usize);
        for y in region.y..region.y + region.height {
            let start = (y as usize * image.width as usize + region.x as usize) * 4;
            pixels.extend_from_slice(&image.pixels[start..start + row]);
        }
        return Rgba8::new(width, height, pixels);
    }
    let source = ImageRef::new(image.width, image.height, &image.pixels, PixelType::U8x4)
        .expect("the buffer matches the image size");
    let mut target = Image::new(width.max(1), height.max(1), PixelType::U8x4);
    let options = ResizeOptions::new()
        .resize_alg(ResizeAlg::Convolution(
            fast_image_resize::FilterType::Lanczos3,
        ))
        .crop(
            f64::from(region.x),
            f64::from(region.y),
            f64::from(region.width),
            f64::from(region.height),
        );
    Resizer::new()
        .resize(&source, &mut target, &options)
        .expect("the region lies within the image");
    Rgba8::new(width.max(1), height.max(1), target.into_vec())
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
    fn a_region_is_cut_out_and_scaled() {
        // Left half red, right half blue.
        let pixels = (0..100 * 50)
            .flat_map(|i| {
                if i % 100 < 50 {
                    [255, 0, 0, 255]
                } else {
                    [0, 0, 255, 255]
                }
            })
            .collect();
        let image = Rgba8::new(100, 50, pixels);
        let right = Region {
            x: 60,
            y: 10,
            width: 40,
            height: 20,
        };
        let copied = resize_region(&image, right, 40, 20);
        assert_eq!((copied.width, copied.height), (40, 20));
        assert!(copied.pixels.chunks(4).all(|p| p == [0, 0, 255, 255]));
        let scaled = resize_region(&image, right, 10, 5);
        assert_eq!((scaled.width, scaled.height), (10, 5));
        assert!(scaled.pixels.chunks(4).all(|p| p[2] > 240 && p[0] < 15));
    }

    #[test]
    fn thumbnails_are_webp() {
        let image = Rgba8::new(640, 480, vec![200; 640 * 480 * 4]);
        let bytes = thumbnail(&image);
        assert_eq!(&bytes[..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WEBP");
    }
}
