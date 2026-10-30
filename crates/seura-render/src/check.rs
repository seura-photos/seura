//! The reference image for checking that the preview renders exactly like
//! export (M1 slice 5): the app renders it in its web view with each preview
//! path, and compares the result with wgpu's.

use half::f16;
use seura_image::{LinearImage, Rgba8};

use crate::Settings;

/// The settings the check renders with: every operation built so far, at
/// values that aren't round, so a difference in any would show. Contrast is
/// moderate, so the ramps keep some pixels on each side of middle gray. The
/// photo is turned on its side, flipped, straightened and cropped, so a
/// preview that placed it any other way, or sampled it any other way, would
/// fail. Black and white and sepia are partial, so the colors before them
/// still count, and the vignette is measured on the cropped frame.
pub const CHECK_SETTINGS: Settings = Settings {
    rotate: 90,
    flip_h: true,
    straighten: 3.7,
    crop_x: 0.08,
    crop_y: 0.11,
    crop_width: 0.83,
    crop_height: 0.77,
    exposure_ev: 0.7,
    temperature: 23.0,
    tint: -17.0,
    highlights: -43.0,
    shadows: 31.0,
    contrast: 37.0,
    saturation: 29.0,
    black_and_white: 23.0,
    sepia: 41.0,
    vignette_amount: -47.0,
    vignette_size: 63.0,
    ..Settings::NEUTRAL
};

/// How far a preview pixel may be from export's, in 8-bit levels: GPUs
/// round a little differently, but never visibly.
pub const CHECK_TOLERANCE: u8 = 2;

/// A `width` × `height` image whose red ramps left to right, green top to
/// bottom, and blue across both, in linear light up to 1. Asymmetric, so a
/// flipped or mirrored render can't pass.
pub fn ramps(width: u32, height: u32) -> LinearImage {
    let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
    for y in 0..height {
        for x in 0..width {
            let r = x as f32 / (width - 1).max(1) as f32;
            let g = y as f32 / (height - 1).max(1) as f32;
            let b = ((x + y) % 97) as f32 / 96.0;
            pixels.extend([r, g, b, 1.0].map(f16::from_f32));
        }
    }
    LinearImage {
        width,
        height,
        pixels,
    }
}

/// The check's image.
pub fn check_image() -> LinearImage {
    ramps(97, 64)
}

/// The largest difference between two renders' color channels, and the
/// pixel where it is; `None` if their sizes differ.
pub fn difference(a: &Rgba8, b: &Rgba8) -> Option<(u8, usize)> {
    if (a.width, a.height) != (b.width, b.height) || a.pixels.len() != b.pixels.len() {
        return None;
    }
    let mut worst = (0, 0);
    for (i, (p, q)) in a.pixels.chunks(4).zip(b.pixels.chunks(4)).enumerate() {
        let off = p[..3]
            .iter()
            .zip(&q[..3])
            .map(|(x, y)| x.abs_diff(*y))
            .max()
            .unwrap_or(0);
        if off > worst.0 {
            worst = (off, i);
        }
    }
    Some(worst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mirrored_render_is_far_off() {
        let image = Rgba8::new(2, 1, vec![0, 0, 0, 255, 255, 255, 255, 255]);
        let mirrored = Rgba8::new(2, 1, vec![255, 255, 255, 255, 0, 0, 0, 255]);
        assert_eq!(difference(&image, &mirrored), Some((255, 0)));
        assert_eq!(difference(&image, &image), Some((0, 0)));
        assert_eq!(difference(&image, &Rgba8::new(1, 1, vec![0; 4])), None);
    }
}
