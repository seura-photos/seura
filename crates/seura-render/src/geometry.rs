//! The geometry stage as one coordinate transform (edit format, formulas
//! appendix, `geometry.md`): where each pixel of the result comes from in
//! the upright photo, through the crop, the straighten turn, and
//! `seura.orient`. The shader does the same per pixel (`main.wgsl`'s
//! `upright_position`); this side sizes the result and finds which part of
//! the photo a tile needs.

use crate::Settings;

/// How far past the positions a tile samples its part of the photo
/// reaches, in pixels: Catmull-Rom reads two pixels on each side.
const MARGIN: f64 = 3.0;

/// A rectangle of whole pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
}

/// The geometry of one photo's render: the upright photo's size and the
/// settings' orient, turn, and crop.
#[derive(Clone, Copy, Debug)]
pub struct Geometry {
    /// The upright photo, w × h.
    upright: (u32, u32),
    /// The oriented photo, W × H: the frame the turn and crop are in.
    frame: (u32, u32),
    /// The crop in whole pixels of the frame.
    crop: Rect,
    /// The turn, in radians, clockwise as seen.
    angle: f64,
    rotate: u16,
    flip_h: bool,
    flip_v: bool,
}

/// round() in the appendix: halves round up.
fn round(value: f64) -> f64 {
    (value + 0.5).floor()
}

impl Geometry {
    pub fn new(settings: &Settings, width: u32, height: u32) -> Geometry {
        let frame = settings.oriented_size(width, height);
        let (across, down) = (f64::from(frame.0), f64::from(frame.1));
        let left = round(settings.crop_x * across);
        let top = round(settings.crop_y * down);
        let crop = Rect {
            left: left.max(0.0) as u32,
            top: top.max(0.0) as u32,
            width: round(settings.crop_width * across).max(1.0) as u32,
            height: round(settings.crop_height * down).max(1.0) as u32,
        };
        Geometry {
            upright: (width, height),
            frame,
            crop,
            angle: f64::from(settings.straighten).to_radians(),
            rotate: settings.rotate,
            flip_h: settings.flip_h,
            flip_v: settings.flip_v,
        }
    }

    /// The result's size: the crop's, in whole pixels.
    pub fn output_size(&self) -> (u32, u32) {
        (self.crop.width, self.crop.height)
    }

    /// The crop in whole pixels of the oriented photo.
    pub fn crop(&self) -> Rect {
        self.crop
    }

    /// Where position `x`, `y` of the result (pixels from its top left; a
    /// pixel's center is at i + 0.5) is in the upright photo.
    pub fn upright_position(&self, x: f64, y: f64) -> (f64, f64) {
        let (across, down) = (f64::from(self.frame.0), f64::from(self.frame.1));
        // The crop.
        let p = (f64::from(self.crop.left) + x, f64::from(self.crop.top) + y);
        // The turn back, about the frame's center.
        let (cx, cy) = (across / 2.0, down / 2.0);
        let (dx, dy) = (p.0 - cx, p.1 - cy);
        let (sin, cos) = self.angle.sin_cos();
        let (mut qx, mut qy) = (cx + dx * cos + dy * sin, cy - dx * sin + dy * cos);
        // seura.orient: its flips undone, then its turn.
        if self.flip_h {
            qx = across - qx;
        }
        if self.flip_v {
            qy = down - qy;
        }
        match self.rotate {
            90 => (qy, across - qx),
            180 => (across - qx, down - qy),
            270 => (down - qy, qx),
            _ => (qx, qy),
        }
    }

    /// The part of the upright photo that `tile` of the result samples:
    /// the box around where its corners land, with room for the filter,
    /// within the photo.
    pub fn source(&self, tile: Rect) -> Rect {
        let corners = [
            (tile.left, tile.top),
            (tile.left + tile.width, tile.top),
            (tile.left, tile.top + tile.height),
            (tile.left + tile.width, tile.top + tile.height),
        ]
        .map(|(x, y)| self.upright_position(f64::from(x), f64::from(y)));
        let (w, h) = (f64::from(self.upright.0), f64::from(self.upright.1));
        let low = |values: [f64; 4], limit: f64| {
            (values.into_iter().fold(f64::INFINITY, f64::min) - MARGIN)
                .floor()
                .clamp(0.0, limit - 1.0)
        };
        let high = |values: [f64; 4], limit: f64| {
            (values.into_iter().fold(f64::NEG_INFINITY, f64::max) + MARGIN)
                .ceil()
                .clamp(1.0, limit)
        };
        let xs = corners.map(|corner| corner.0);
        let ys = corners.map(|corner| corner.1);
        let (left, top) = (low(xs, w), low(ys, h));
        let (right, bottom) = (high(xs, w).max(left + 1.0), high(ys, h).max(top + 1.0));
        Rect {
            left: left as u32,
            top: top as u32,
            width: (right - left) as u32,
            height: (bottom - top) as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn vectors(name: &str) -> serde_json::Value {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../third_party/edit-format/tests/formulas/geometry")
            .join(format!("{name}.json"));
        serde_json::from_str(
            &std::fs::read_to_string(path)
                .expect("the edit format's geometry vectors; run `git submodule update --init`"),
        )
        .unwrap()
    }

    /// The settings a case's params make.
    fn settings(case: &serde_json::Value) -> Settings {
        let number = |value: &serde_json::Value| value.as_f64().unwrap() as f32;
        Settings {
            rotate: case["orient"]["rotate"].as_u64().unwrap() as u16,
            flip_h: case["orient"]["flipH"].as_bool().unwrap(),
            flip_v: case["orient"]["flipV"].as_bool().unwrap(),
            straighten: number(&case["angle"]),
            crop_x: case["crop"]["x"].as_f64().unwrap(),
            crop_y: case["crop"]["y"].as_f64().unwrap(),
            crop_width: case["crop"]["width"].as_f64().unwrap(),
            crop_height: case["crop"]["height"].as_f64().unwrap(),
            ..Settings::NEUTRAL
        }
    }

    #[test]
    fn positions_match_the_appendix_vectors() {
        let file = vectors("positions");
        for case in file["cases"].as_array().unwrap() {
            let upright = &case["upright"];
            let geometry = Geometry::new(
                &settings(case),
                upright[0].as_u64().unwrap() as u32,
                upright[1].as_u64().unwrap() as u32,
            );
            let size = &case["outputSize"];
            assert_eq!(
                geometry.output_size(),
                (
                    size[0].as_u64().unwrap() as u32,
                    size[1].as_u64().unwrap() as u32
                ),
                "{case}"
            );
            let pixels = case["pixels"].as_array().unwrap();
            let positions = case["positions"].as_array().unwrap();
            for (pixel, expected) in pixels.iter().zip(positions) {
                let (i, j) = (pixel[0].as_f64().unwrap(), pixel[1].as_f64().unwrap());
                let found = geometry.upright_position(i + 0.5, j + 0.5);
                for (found, expected) in [found.0, found.1].iter().zip(expected.as_array().unwrap())
                {
                    let expected = expected.as_f64().unwrap();
                    // f32 settings: the angle and crop are as the shader has them.
                    assert!(
                        (found - expected).abs() <= 0.0001 + 0.001 * expected.abs(),
                        "{case}: pixel {pixel} at {found}, expected {expected}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_tile_s_source_holds_every_position_it_samples_with_room_for_the_filter() {
        let settings = Settings {
            rotate: 90,
            flip_h: true,
            straighten: -12.5,
            crop_x: 0.2,
            crop_y: 0.15,
            crop_width: 0.6,
            crop_height: 0.55,
            ..Settings::NEUTRAL
        };
        let geometry = Geometry::new(&settings, 700, 500);
        let tile = Rect {
            left: 40,
            top: 30,
            width: 100,
            height: 80,
        };
        let source = geometry.source(tile);
        for y in tile.top..tile.top + tile.height {
            for x in tile.left..tile.left + tile.width {
                let (u, v) = geometry.upright_position(f64::from(x) + 0.5, f64::from(y) + 0.5);
                assert!(
                    u - 2.0 >= f64::from(source.left)
                        && u + 2.0 <= f64::from(source.left + source.width)
                );
                assert!(
                    v - 2.0 >= f64::from(source.top)
                        && v + 2.0 <= f64::from(source.top + source.height)
                );
            }
        }
        // Within the photo.
        assert!(source.left + source.width <= 700 && source.top + source.height <= 500);
    }

    #[test]
    fn an_unstraightened_tile_s_source_is_its_pixels_and_the_margin() {
        let geometry = Geometry::new(&Settings::NEUTRAL, 100, 80);
        let source = geometry.source(Rect {
            left: 10,
            top: 20,
            width: 30,
            height: 40,
        });
        assert_eq!(
            source,
            Rect {
                left: 7,
                top: 17,
                width: 36,
                height: 46
            }
        );
    }
}
