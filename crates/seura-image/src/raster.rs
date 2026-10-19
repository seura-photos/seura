//! Decoded pixels, and the corrections applied to every photo: orientation
//! and color profile (BRW-7).

use moxcms::{ColorProfile, Layout, TransformOptions};

/// An image as 8-bit RGBA in sRGB, upright.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rgba8 {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Rgba8 {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Rgba8 {
        debug_assert_eq!(pixels.len(), width as usize * height as usize * 4);
        Rgba8 {
            width,
            height,
            pixels,
        }
    }

    /// Applies an EXIF orientation (1 to 8), so the result is upright.
    pub fn orient(self, orientation: u8) -> Rgba8 {
        if !(2..=8).contains(&orientation) {
            return self;
        }
        let (w, h) = (self.width as usize, self.height as usize);
        // Orientations 5 to 8 swap width and height.
        let (out_w, out_h) = if orientation >= 5 { (h, w) } else { (w, h) };
        let mut out = vec![0; self.pixels.len()];
        for y in 0..h {
            for x in 0..w {
                // Where source pixel (x, y) lands in the upright image.
                let (ox, oy) = match orientation {
                    2 => (w - 1 - x, y),
                    3 => (w - 1 - x, h - 1 - y),
                    4 => (x, h - 1 - y),
                    5 => (y, x),
                    6 => (h - 1 - y, x),
                    7 => (h - 1 - y, w - 1 - x),
                    _ => (y, w - 1 - x),
                };
                let from = (y * w + x) * 4;
                let to = (oy * out_w + ox) * 4;
                out[to..to + 4].copy_from_slice(&self.pixels[from..from + 4]);
            }
        }
        Rgba8::new(out_w as u32, out_h as u32, out)
    }

    /// Converts from the photo's embedded color profile to sRGB. Photos
    /// without one, or with one that can't be read, are taken as sRGB.
    pub fn from_profile(self, icc: Option<&[u8]>) -> Rgba8 {
        let Some(source) = icc.and_then(|icc| ColorProfile::new_from_slice(icc).ok()) else {
            return self;
        };
        let srgb = ColorProfile::new_srgb();
        let Ok(transform) = source.create_transform_8bit(
            Layout::Rgba,
            &srgb,
            Layout::Rgba,
            TransformOptions::default(),
        ) else {
            return self;
        };
        let mut converted = vec![0; self.pixels.len()];
        match transform.transform(&self.pixels, &mut converted) {
            Ok(()) => Rgba8 {
                pixels: converted,
                ..self
            },
            Err(_) => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 3×2 image whose pixels are numbered 1 to 6, left to right, top to bottom.
    fn numbered() -> Rgba8 {
        let pixels = (1..=6u8).flat_map(|n| [n, n, n, 255]).collect();
        Rgba8::new(3, 2, pixels)
    }

    fn numbers(image: &Rgba8) -> Vec<u8> {
        image.pixels.chunks(4).map(|pixel| pixel[0]).collect()
    }

    #[test]
    fn every_orientation_comes_out_upright() {
        // The stored grid is
        //   1 2 3
        //   4 5 6
        // and each orientation says how the camera stored it; the result is
        // what a viewer should show.
        let cases: [(u8, (u32, u32), [u8; 6]); 8] = [
            (1, (3, 2), [1, 2, 3, 4, 5, 6]),
            (2, (3, 2), [3, 2, 1, 6, 5, 4]),
            (3, (3, 2), [6, 5, 4, 3, 2, 1]),
            (4, (3, 2), [4, 5, 6, 1, 2, 3]),
            (5, (2, 3), [1, 4, 2, 5, 3, 6]),
            (6, (2, 3), [4, 1, 5, 2, 6, 3]),
            (7, (2, 3), [6, 3, 5, 2, 4, 1]),
            (8, (2, 3), [3, 6, 2, 5, 1, 4]),
        ];
        for (orientation, size, expected) in cases {
            let image = numbered().orient(orientation);
            assert_eq!(
                (image.width, image.height),
                size,
                "orientation {orientation}"
            );
            assert_eq!(numbers(&image), expected, "orientation {orientation}");
        }
    }

    #[test]
    fn display_p3_red_becomes_the_nearest_srgb_red() {
        let p3 = ColorProfile::new_display_p3().encode().unwrap();
        let image = Rgba8::new(1, 1, vec![255, 0, 0, 255]).from_profile(Some(&p3));
        // P3's red is outside sRGB, so it maps to full red, not to itself
        // re-labelled; and a mid-gray stays gray.
        assert_eq!(image.pixels[0], 255);
        assert!(
            image.pixels[1] < 10 && image.pixels[2] < 10,
            "{:?}",
            image.pixels
        );
        let gray = Rgba8::new(1, 1, vec![128, 128, 128, 255]).from_profile(Some(&p3));
        assert!(
            gray.pixels[..3].iter().all(|v| v.abs_diff(128) <= 1),
            "{:?}",
            gray.pixels
        );
    }

    #[test]
    fn a_damaged_profile_is_ignored() {
        let image = Rgba8::new(1, 1, vec![10, 20, 30, 255]);
        assert_eq!(image.clone().from_profile(Some(b"not a profile")), image);
    }
}
