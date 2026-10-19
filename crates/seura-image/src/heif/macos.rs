//! HEIC on macOS: Apple's ImageIO, which has a licensed HEVC decoder
//! (decision 0011). ImageIO applies the container's rotation and mirroring,
//! and drawing into an sRGB bitmap lets Core Graphics convert the color.

#![allow(unsafe_code, reason = "ImageIO and Core Graphics are C APIs")]

use std::path::Path;

use objc2_core_foundation::{
    CFBoolean, CFDictionary, CFNumber, CFString, CFType, CFURL, CGFloat, CGPoint, CGRect, CGSize,
};
use objc2_core_graphics::{
    CGBitmapContextCreate, CGColorSpace, CGContext, CGImage, CGImageAlphaInfo, kCGColorSpaceSRGB,
};
use objc2_image_io::{
    CGImageSource, kCGImageSourceCreateThumbnailFromImageAlways,
    kCGImageSourceCreateThumbnailWithTransform, kCGImageSourceThumbnailMaxPixelSize,
};

use super::Pixels;
use crate::decode::{DecodeError, MAX_SIDE};
use crate::raster::Rgba8;

pub(super) fn decode(path: &Path) -> Result<Pixels, DecodeError> {
    let damaged = |what: &str| DecodeError::Damaged(format!("ImageIO: {what}"));
    let url = CFURL::from_file_path(path).ok_or_else(|| damaged("not a file path"))?;
    // Opens the file for reading only (CLAUDE.md rule 1).
    let source = unsafe { CGImageSource::with_url(&url, None) }
        .ok_or_else(|| damaged("couldn't open it"))?;

    // The full image with its orientation applied, at most MAX_SIDE on its
    // long edge, so a huge file can't exhaust memory.
    let keys: [&CFString; 3] = unsafe {
        [
            kCGImageSourceCreateThumbnailFromImageAlways,
            kCGImageSourceCreateThumbnailWithTransform,
            kCGImageSourceThumbnailMaxPixelSize,
        ]
    };
    let max_side = CFNumber::new_i32(MAX_SIDE as i32);
    let values: [&CFType; 3] = [CFBoolean::new(true), CFBoolean::new(true), &max_side];
    let options = CFDictionary::from_slices(&keys, &values);
    let image = unsafe { source.thumbnail_at_index(0, Some(options.as_opaque())) }
        .ok_or_else(|| damaged("couldn't decode it"))?;

    let width = CGImage::width(Some(&image));
    let height = CGImage::height(Some(&image));
    if width == 0 || height == 0 {
        return Err(damaged("empty image"));
    }
    let srgb = CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB }))
        .ok_or_else(|| damaged("no sRGB color space"))?;
    let mut pixels = vec![0u8; width * height * 4];
    {
        let context: Option<_> = unsafe {
            CGBitmapContextCreate(
                pixels.as_mut_ptr().cast(),
                width,
                height,
                8,
                width * 4,
                Some(&srgb),
                CGImageAlphaInfo::PremultipliedLast.0,
            )
        };
        let context = context.ok_or_else(|| damaged("couldn't make a bitmap"))?;
        let bounds = CGRect {
            origin: CGPoint { x: 0.0, y: 0.0 },
            size: CGSize {
                width: width as CGFloat,
                height: height as CGFloat,
            },
        };
        CGContext::draw_image(Some(&context), bounds, Some(&image));
        // The context writes into `pixels`; it's dropped here, before they're used.
    }
    Ok(Pixels {
        image: Rgba8::new(width as u32, height as u32, pixels),
        transformed: true,
        color_managed: true,
    })
}
