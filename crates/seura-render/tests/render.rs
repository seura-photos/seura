//! The shaders, run with wgpu, against the edit format's definitions
//! computed here on the CPU, as its formulas appendix gives them: the photo
//! turned, flipped, straightened and cropped, sampled with Catmull-Rom, each
//! tone and color operation in the canonical order, then the result encoded
//! for Display P3 or sRGB (EXP-5), after the effects. The definitions are checked against the
//! appendix's own test vectors, and the shaders are run on those too.
//!
//! The images are generated: ramps through every channel, including Display
//! P3 colors outside sRGB, which sRGB output must clip.

use std::path::PathBuf;

use half::f16;
use seura_image::LinearImage;
use seura_render::check::ramps;
use seura_render::{Output, Renderer, Settings};

/// Linear Display P3 to linear sRGB, rows, from the two spaces' primaries.
const P3_TO_SRGB: [[f64; 3]; 3] = [
    [1.2249401, -0.2249404, 0.0],
    [-0.0420569, 1.0420571, 0.0],
    [-0.0196376, -0.0786361, 1.0982735],
];

/// The luminance of linear Display P3 (D65).
const P3_LUMINANCE: [f64; 3] = [0.2289746, 0.6917385, 0.0792869];

fn luminance(c: [f64; 3]) -> f64 {
    (0..3).map(|i| P3_LUMINANCE[i] * c[i]).sum()
}

/// `seura.whiteBalance`: per-channel gains, scaled so gray keeps its
/// luminance.
fn white_balance(c: [f64; 3], temperature: f64, tint: f64) -> [f64; 3] {
    let (t, n) = (temperature / 100.0, tint / 100.0);
    let gains = [t / 2.0, -n / 2.0, -t / 2.0].map(|stops| 2f64.powf(stops));
    let gray = luminance(gains);
    [0, 1, 2].map(|i| c[i] * gains[i] / gray)
}

/// `seura.contrast`: an S-curve on luminance raised to 1/2.2, around middle
/// gray, with slope 2^(amount/100) there.
fn contrast(c: [f64; 3], amount: f64) -> [f64; 3] {
    let y = luminance(c);
    if y <= 0.0 {
        return c;
    }
    let x = y.powf(1.0 / 2.2);
    if x >= 1.0 {
        return c;
    }
    let g = 2f64.powf(amount / 100.0);
    let p = 0.18f64.powf(1.0 / 2.2);
    let curved = if x < p {
        p * (x / p).powf(g)
    } else {
        1.0 - (1.0 - p) * ((1.0 - x) / (1.0 - p)).powf(g)
    };
    let scale = curved.powf(2.2) / y;
    c.map(|v| v * scale)
}

/// `seura.saturation`: away from luminance by 1 + amount/100. Components
/// below 0 are kept, and can be made.
fn saturation(c: [f64; 3], amount: f64) -> [f64; 3] {
    let y = luminance(c);
    let s = 1.0 + amount / 100.0;
    c.map(|v| y + (v - y) * s)
}

/// `seura.blackAndWhite`: toward luminance by strength/100.
fn black_and_white(c: [f64; 3], strength: f64) -> [f64; 3] {
    let y = luminance(c);
    c.map(|v| v + (y - v) * strength / 100.0)
}

/// `seura.sepia`: toward luminance in sepia's tone, (1.25, 0.97, 0.56)
/// scaled to luminance 1, by strength/100.
fn sepia(c: [f64; 3], strength: f64) -> [f64; 3] {
    let t = [1.25, 0.97, 0.56];
    let tone = t.map(|v| v / luminance(t));
    let y = luminance(c);
    [0, 1, 2].map(|i| c[i] + (y * tone[i] - c[i]) * strength / 100.0)
}

/// `seura.vignette` at `at`, fractions of the cropped frame: from
/// r = 1 - size/100 of an ellipse fitted to the frame out to its corners,
/// toward black or white.
fn vignette(c: [f64; 3], amount: f64, size: f64, at: [f64; 2]) -> [f64; 3] {
    let r = (2.0 * at[0] - 1.0).hypot(2.0 * at[1] - 1.0);
    let start = 1.0 - size / 100.0;
    let t = ((r - start) / (2f64.sqrt() - start)).clamp(0.0, 1.0);
    let m = amount / 100.0 * t * t * (3.0 - 2.0 * t);
    if m < 0.0 {
        c.map(|v| v * (1.0 + m))
    } else {
        c.map(|v| v + (1.0 - v) * m)
    }
}

fn encode(linear: f64) -> f64 {
    let v = linear.clamp(0.0, 1.0);
    if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// Luminance is floored at 2^−12 before its log.
const GUIDE_FLOOR: f64 = 1.0 / 4096.0;

fn log_luminance(c: [f64; 3]) -> f64 {
    luminance(c).max(GUIDE_FLOOR).log2()
}

/// The luminance guide (`shaders/guide.wgsl`): a guided filter on the guide
/// image's log luminance, after white balance, guided by itself. Holds each
/// guide pixel's a and b.
struct Guide {
    width: usize,
    height: usize,
    ab: Vec<[f64; 2]>,
}

impl Guide {
    fn new(image: &LinearImage, settings: &Settings) -> Guide {
        let (width, height) = (image.width as usize, image.height as usize);
        let r = ((width.max(height) as f64 * 0.02).round() as isize).max(1);
        // The mean of each pixel's 2r + 1 neighbours along one axis, the
        // edge's repeated past it.
        let mean = |values: &[[f64; 2]], across: bool| -> Vec<[f64; 2]> {
            let mut out = vec![[0.0; 2]; values.len()];
            for y in 0..height {
                for x in 0..width {
                    let mut sum = [0.0; 2];
                    for i in -r..=r {
                        let (fx, fy) = if across {
                            ((x as isize + i).clamp(0, width as isize - 1) as usize, y)
                        } else {
                            (x, (y as isize + i).clamp(0, height as isize - 1) as usize)
                        };
                        let v = values[fy * width + fx];
                        sum[0] += v[0];
                        sum[1] += v[1];
                    }
                    out[y * width + x] = sum.map(|s| s / (2 * r + 1) as f64);
                }
            }
            out
        };
        let l: Vec<[f64; 2]> = image
            .pixels
            .chunks(4)
            .map(|p| {
                let mut c = [0, 1, 2].map(|i| f64::from(p[i].to_f32()));
                if settings.temperature != 0.0 || settings.tint != 0.0 {
                    c = white_balance(c, settings.temperature.into(), settings.tint.into());
                }
                let l = log_luminance(c);
                [l, l * l]
            })
            .collect();
        let means = mean(&mean(&l, true), false);
        let coefficients: Vec<[f64; 2]> = means
            .iter()
            .map(|[m, sq]| {
                let variance = (sq - m * m).max(0.0);
                let a = variance / (variance + 0.1);
                [a, m - a * m]
            })
            .collect();
        Guide {
            width,
            height,
            ab: mean(&mean(&coefficients, true), false),
        }
    }

    /// a and b, sampled bilinearly at `u`, `v`, fractions of the photo,
    /// the edge's repeated past it.
    fn sample(&self, u: f64, v: f64) -> [f64; 2] {
        let x = u * self.width as f64 - 0.5;
        let y = v * self.height as f64 - 0.5;
        let (x0, y0) = (x.floor(), y.floor());
        let (tx, ty) = (x - x0, y - y0);
        let at = |x: f64, y: f64| {
            let x = (x as isize).clamp(0, self.width as isize - 1) as usize;
            let y = (y as isize).clamp(0, self.height as isize - 1) as usize;
            self.ab[y * self.width + x]
        };
        let lerp = |a: [f64; 2], b: [f64; 2], t: f64| [0, 1].map(|i| a[i] + (b[i] - a[i]) * t);
        lerp(
            lerp(at(x0, y0), at(x0 + 1.0, y0), tx),
            lerp(at(x0, y0 + 1.0), at(x0 + 1.0, y0 + 1.0), tx),
            ty,
        )
    }
}

/// x, a base as linear light up to 1, raised to 1/2.2.
fn tone_position(base: f64) -> f64 {
    base.exp2().clamp(0.0, 1.0).powf(1.0 / 2.2)
}

/// `seura.highlights`' gain in stops: up to 2 at 100, by x³.
fn highlights_gain(base: f64, amount: f64) -> f64 {
    amount / 100.0 * 2.0 * tone_position(base).powi(3)
}

/// `seura.shadows`' gain in stops: up to 2 at 100, by (1 − x)³.
fn shadows_gain(base: f64, amount: f64) -> f64 {
    amount / 100.0 * 2.0 * (1.0 - tone_position(base)).powi(3)
}

/// The tone and color operations, then the effects, on a linear Display P3
/// color, in the canonical order; `ab` is the guide where the pixel is, for
/// highlights and shadows, and `at` where it is in the cropped frame, as
/// fractions of it, for the vignette.
fn defined(color: [f64; 3], settings: &Settings, ab: [f64; 2], at: [f64; 2]) -> [f64; 3] {
    let ev = f64::from(settings.exposure_ev);
    let scale = ev.exp2();
    let mut p3 = color.map(|v| v * scale);
    if settings.temperature != 0.0 || settings.tint != 0.0 {
        p3 = white_balance(p3, settings.temperature.into(), settings.tint.into());
    }
    if settings.highlights != 0.0 || settings.shadows != 0.0 {
        let mut base = ab[0] * log_luminance(p3.map(|v| v / scale)) + ab[1] + ev;
        if settings.highlights != 0.0 {
            let gain = highlights_gain(base, settings.highlights.into());
            p3 = p3.map(|v| v * gain.exp2());
            base += gain;
        }
        if settings.shadows != 0.0 {
            let gain = shadows_gain(base, settings.shadows.into()).exp2();
            p3 = p3.map(|v| v * gain);
        }
    }
    if settings.contrast != 0.0 {
        p3 = contrast(p3, settings.contrast.into());
    }
    if settings.saturation != 0.0 {
        p3 = saturation(p3, settings.saturation.into());
    }
    if settings.black_and_white != 0.0 {
        p3 = black_and_white(p3, settings.black_and_white.into());
    }
    if settings.sepia != 0.0 {
        p3 = sepia(p3, settings.sepia.into());
    }
    if settings.vignette_amount != 0.0 {
        p3 = vignette(
            p3,
            settings.vignette_amount.into(),
            settings.vignette_size.into(),
            at,
        );
    }
    p3
}

/// What a pixel must come out as, given the photo's color, the guide
/// where it is, and where it is in the cropped frame.
fn expected(
    color: [f64; 3],
    settings: &Settings,
    output: Output,
    ab: [f64; 2],
    at: [f64; 2],
) -> [u8; 3] {
    let p3 = defined(color, settings, ab, at);
    let linear = match output {
        Output::DisplayP3 => p3,
        Output::Srgb => P3_TO_SRGB.map(|row| row.iter().zip(p3).map(|(m, v)| m * v).sum()),
    };
    linear.map(|v| (encode(v) * 255.0).round() as u8)
}

fn renderer() -> Renderer {
    let renderer = Renderer::new().expect("a graphics adapter, or the software one");
    eprintln!("rendering with {}", renderer.adapter());
    renderer
}

/// round() in the geometry appendix: halves round up.
fn round_half_up(value: f64) -> f64 {
    (value + 0.5).floor()
}

/// The geometry stage as the appendix defines it (`geometry.md`), for an
/// upright photo of `width` × `height`: the result's size, and where the
/// center of each of its pixels is in the upright photo.
fn placed(settings: &Settings, width: usize, height: usize) -> (usize, usize, Vec<(f64, f64)>) {
    let sideways = settings.rotate % 180 == 90;
    let (across, down) = if sideways {
        (height as f64, width as f64)
    } else {
        (width as f64, height as f64)
    };
    let left = round_half_up(settings.crop_x * across);
    let top = round_half_up(settings.crop_y * down);
    let result_width = round_half_up(settings.crop_width * across).max(1.0) as usize;
    let result_height = round_half_up(settings.crop_height * down).max(1.0) as usize;
    let theta = f64::from(settings.straighten).to_radians();
    let mut positions = Vec::with_capacity(result_width * result_height);
    for j in 0..result_height {
        for i in 0..result_width {
            let p = (left + i as f64 + 0.5, top + j as f64 + 0.5);
            let d = (p.0 - across / 2.0, p.1 - down / 2.0);
            let mut q = (
                across / 2.0 + d.0 * theta.cos() + d.1 * theta.sin(),
                down / 2.0 - d.0 * theta.sin() + d.1 * theta.cos(),
            );
            if settings.flip_h {
                q.0 = across - q.0;
            }
            if settings.flip_v {
                q.1 = down - q.1;
            }
            positions.push(match settings.rotate {
                90 => (q.1, across - q.0),
                180 => (across - q.0, down - q.1),
                270 => (down - q.1, q.0),
                _ => q,
            });
        }
    }
    (result_width, result_height, positions)
}

/// Catmull-Rom at position x, y of the photo, in its pixels, as the
/// geometry appendix defines it.
fn catmull_rom(image: &LinearImage, x: f64, y: f64) -> [f64; 3] {
    let (width, height) = (image.width as f64, image.height as f64);
    let x = x.clamp(0.5, width - 0.5) - 0.5;
    let y = y.clamp(0.5, height - 0.5) - 0.5;
    let weights = |t: f64| {
        [
            (-t.powi(3) + 2.0 * t * t - t) / 2.0,
            (3.0 * t.powi(3) - 5.0 * t * t + 2.0) / 2.0,
            (-3.0 * t.powi(3) + 4.0 * t * t + t) / 2.0,
            (t.powi(3) - t * t) / 2.0,
        ]
    };
    let (wx, wy) = (weights(x - x.floor()), weights(y - y.floor()));
    let mut color = [0.0; 3];
    for (n, wy) in wy.iter().enumerate() {
        let row = (y.floor() as i64 - 1 + n as i64).clamp(0, image.height as i64 - 1) as usize;
        for (m, wx) in wx.iter().enumerate() {
            let column =
                (x.floor() as i64 - 1 + m as i64).clamp(0, image.width as i64 - 1) as usize;
            let at = (row * image.width as usize + column) * 4;
            for (k, channel) in color.iter_mut().enumerate() {
                *channel += wx * wy * f64::from(image.pixels[at + k].to_f32());
            }
        }
    }
    color
}

/// Every pixel within one level of the definition: GPU arithmetic isn't
/// bit-exact, but it's far closer than the 8-bit output can show.
fn assert_matches(image: &LinearImage, settings: &Settings, output: Output, renderer: &Renderer) {
    let rendered = renderer
        .render(image, &image.guide_image(), settings, output)
        .unwrap();
    let (width, height) = (image.width as usize, image.height as usize);
    let (across, down, positions) = placed(settings, width, height);
    assert_eq!(
        (rendered.width as usize, rendered.height as usize),
        (across, down)
    );
    let guide = Guide::new(&image.guide_image(), settings);
    // The guide is sampled where the pixel is in the upright photo.
    let want = |i: usize| {
        let (x, y) = positions[i];
        let ab = guide.sample(x / width as f64, y / height as f64);
        let at = [
            ((i % across) as f64 + 0.5) / across as f64,
            ((i / across) as f64 + 0.5) / down as f64,
        ];
        expected(catmull_rom(image, x, y), settings, output, ab, at)
    };
    let mut worst = (0, 0);
    for (i, found) in rendered.pixels.chunks(4).enumerate() {
        let off = found[..3]
            .iter()
            .zip(want(i))
            .map(|(a, b)| a.abs_diff(b))
            .max()
            .unwrap();
        if off > worst.0 {
            worst = (off, i);
        }
    }
    let (off, i) = worst;
    assert!(
        off <= 1,
        "{settings:?}, {output:?}: pixel {i} is off by {off}: found {:?}, expected {:?}",
        &rendered.pixels[i * 4..i * 4 + 3],
        want(i),
    );
}

/// The geometry vectors' file, from the edit format.
fn geometry_vectors(name: &str) -> serde_json::Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../third_party/edit-format/tests/formulas/geometry")
        .join(format!("{name}.json"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn the_geometry_definitions_match_the_appendix_vectors() {
    let close = |found: f64, want: &serde_json::Value| {
        let want = want.as_f64().unwrap();
        (found - want).abs() <= 0.0001 + 0.001 * want.abs()
    };
    for case in geometry_vectors("positions")["cases"].as_array().unwrap() {
        let number = |value: &serde_json::Value| value.as_f64().unwrap();
        let settings = Settings {
            rotate: case["orient"]["rotate"].as_u64().unwrap() as u16,
            flip_h: case["orient"]["flipH"].as_bool().unwrap(),
            flip_v: case["orient"]["flipV"].as_bool().unwrap(),
            straighten: number(&case["angle"]) as f32,
            crop_x: number(&case["crop"]["x"]),
            crop_y: number(&case["crop"]["y"]),
            crop_width: number(&case["crop"]["width"]),
            crop_height: number(&case["crop"]["height"]),
            ..Settings::NEUTRAL
        };
        let upright = &case["upright"];
        let (across, down, positions) = placed(
            &settings,
            upright[0].as_u64().unwrap() as usize,
            upright[1].as_u64().unwrap() as usize,
        );
        assert_eq!(
            [across, down],
            [0, 1].map(|k| case["outputSize"][k].as_u64().unwrap() as usize)
        );
        for (pixel, want) in case["pixels"]
            .as_array()
            .unwrap()
            .iter()
            .zip(case["positions"].as_array().unwrap())
        {
            let i =
                pixel[1].as_u64().unwrap() as usize * across + pixel[0].as_u64().unwrap() as usize;
            let found = positions[i];
            assert!(
                close(found.0, &want[0]) && close(found.1, &want[1]),
                "{case}: {pixel} at {found:?}"
            );
        }
    }
    let file = geometry_vectors("sampling");
    let rows = file["image"].as_array().unwrap();
    let image = LinearImage {
        width: rows[0].as_array().unwrap().len() as u32,
        height: rows.len() as u32,
        pixels: rows
            .iter()
            .flat_map(|row| row.as_array().unwrap().iter())
            .flat_map(|color| {
                let c: Vec<f32> = color
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64().unwrap() as f32)
                    .collect();
                [c[0], c[1], c[2], 1.0].map(f16::from_f32)
            })
            .collect(),
    };
    for case in file["cases"].as_array().unwrap() {
        let at = &case["position"];
        let found = catmull_rom(&image, at[0].as_f64().unwrap(), at[1].as_f64().unwrap());
        for (found, want) in found.iter().zip(case["output"].as_array().unwrap()) {
            assert!(close(*found, want), "{case}: found {found}");
        }
    }
}

/// Straightened and cropped (EDT-2, EDT-3), with every turn, and with
/// highlights and shadows, whose guide covers the upright photo.
#[test]
fn straightened_and_cropped_photos_match_the_definition() {
    let renderer = renderer();
    let image = ramps(61, 41);
    for (rotate, flip_h) in [(0, false), (90, true), (270, false)] {
        for straighten in [7.3, -31.0] {
            let settings = Settings {
                rotate,
                flip_h,
                straighten,
                crop_x: 0.17,
                crop_y: 0.2,
                crop_width: 0.61,
                crop_height: 0.5,
                highlights: -40.0,
                shadows: 35.0,
                ..Settings::NEUTRAL
            };
            assert_matches(&image, &settings, Output::DisplayP3, &renderer);
        }
    }
    // A crop past the turned photo shows its corners as the nearest edge
    // pixel, as a file written elsewhere may have them.
    let corners = Settings {
        straighten: 20.0,
        ..Settings::NEUTRAL
    };
    assert_matches(&image, &corners, Output::Srgb, &renderer);
}

/// Both flips are a half turn. Seura stores a half turn, but readers must
/// accept both flips (edit format: `seura.orient`), so a file that has them
/// draws exactly as the half turn does.
#[test]
fn both_flips_draw_as_a_half_turn() {
    let renderer = renderer();
    let image = ramps(37, 23);
    let document = seura_edit::EditDocument::parse(
        br#"{"format": "seura-edit", "formatVersion": "1.0",
            "photo": {"file": "a.jpg", "size": 1},
            "operations": [{"id": "seura.orient", "version": 1,
                "params": {"rotate": 90, "flipH": true, "flipV": true}}]}"#,
    )
    .unwrap();
    let flipped = Settings::from_document(&document);
    assert_eq!(
        (flipped.rotate, flipped.flip_h, flipped.flip_v),
        (90, true, true)
    );
    let half_turn = Settings {
        rotate: 270,
        ..Settings::NEUTRAL
    };
    let draw = |settings: &Settings| {
        renderer
            .render(&image, &image.guide_image(), settings, Output::Srgb)
            .unwrap()
    };
    assert_eq!(draw(&flipped), draw(&half_turn));
}

/// Every turn and flip (EDT-4), on a photo that isn't square, with
/// highlights and shadows, whose guide covers the upright photo.
#[test]
fn turned_and_flipped_photos_match_the_definition() {
    let renderer = renderer();
    let image = ramps(37, 23);
    for rotate in [0, 90, 180, 270] {
        for (flip_h, flip_v) in [(false, false), (true, false), (false, true), (true, true)] {
            let settings = Settings {
                rotate,
                flip_h,
                flip_v,
                highlights: -40.0,
                shadows: 35.0,
                ..Settings::NEUTRAL
            };
            assert_matches(&image, &settings, Output::DisplayP3, &renderer);
        }
    }
}

#[test]
fn exposure_matches_the_definition() {
    let renderer = renderer();
    let image = ramps(97, 64);
    for ev in [0.0, 1.0, -1.5, 0.3, 4.0, -4.0] {
        let settings = Settings {
            exposure_ev: ev,
            ..Settings::NEUTRAL
        };
        for output in [Output::Srgb, Output::DisplayP3] {
            assert_matches(&image, &settings, output, &renderer);
        }
    }
}

/// Each light and color operation alone at its ends and between, then all
/// together, as the appendix defines them (EDT-5).
#[test]
fn light_and_color_match_the_definitions() {
    let renderer = renderer();
    let image = ramps(97, 64);
    let neutral = Settings::NEUTRAL;
    let mut cases = Vec::new();
    for value in [-100.0, -40.0, 25.0, 100.0] {
        cases.push(Settings {
            temperature: value,
            ..neutral
        });
        cases.push(Settings {
            tint: value,
            ..neutral
        });
        cases.push(Settings {
            contrast: value,
            ..neutral
        });
        cases.push(Settings {
            saturation: value,
            ..neutral
        });
        cases.push(Settings {
            highlights: value,
            ..neutral
        });
        cases.push(Settings {
            shadows: value,
            ..neutral
        });
    }
    cases.push(Settings {
        exposure_ev: 0.6,
        temperature: 30.0,
        tint: -15.0,
        highlights: -55.0,
        shadows: 70.0,
        contrast: 45.0,
        saturation: -20.0,
        ..neutral
    });
    for settings in &cases {
        for output in [Output::Srgb, Output::DisplayP3] {
            assert_matches(&image, settings, output, &renderer);
        }
    }
}

/// Each effect alone, then together with light and color, and the
/// vignette on a cropped, straightened, turned photo, where it's measured
/// on the cropped frame, and in tiles (EDT-6).
#[test]
fn effects_match_the_definitions() {
    let renderer = renderer();
    let image = ramps(97, 64);
    let neutral = Settings::NEUTRAL;
    let mut cases = Vec::new();
    for strength in [30.0, 100.0] {
        cases.push(Settings {
            black_and_white: strength,
            ..neutral
        });
        cases.push(Settings {
            sepia: strength,
            ..neutral
        });
    }
    for (amount, size) in [(-100.0, 0.0), (-35.0, 60.0), (45.0, 25.0), (100.0, 100.0)] {
        cases.push(Settings {
            vignette_amount: amount,
            vignette_size: size,
            ..neutral
        });
    }
    let together = Settings {
        exposure_ev: 0.4,
        contrast: 30.0,
        saturation: 25.0,
        black_and_white: 40.0,
        sepia: 70.0,
        vignette_amount: -50.0,
        vignette_size: 45.0,
        ..neutral
    };
    cases.push(together);
    cases.push(Settings {
        rotate: 90,
        straighten: 6.0,
        crop_x: 0.2,
        crop_y: 0.15,
        crop_width: 0.55,
        crop_height: 0.6,
        ..together
    });
    for settings in &cases {
        for output in [Output::Srgb, Output::DisplayP3] {
            assert_matches(&image, settings, output, &renderer);
        }
    }
    // In tiles, each tile places the vignette by where it is in the result.
    let large = ramps(2300, 1700);
    assert_matches(
        &large,
        &Settings {
            vignette_amount: -80.0,
            vignette_size: 70.0,
            crop_x: 0.1,
            crop_width: 0.85,
            ..neutral
        },
        Output::Srgb,
        &renderer,
    );
}

/// What the effects promise: black and white and sepia keep luminance,
/// sepia at 100 depends only on it, and the vignette leaves the center as
/// it is and reaches black or white at the very corners.
#[test]
fn effects_keep_what_they_promise() {
    let color = [0.6, 0.3, 0.1];
    let y = luminance(color);
    assert!((luminance(black_and_white(color, 70.0)) - y).abs() < 1e-12);
    assert!((luminance(sepia(color, 45.0)) - y).abs() < 1e-12);
    let gray = [y; 3];
    let toned = sepia(gray, 100.0);
    assert_eq!(sepia(color, 100.0), toned);
    assert!(
        toned[0] > toned[1] && toned[1] > toned[2],
        "warm: {toned:?}"
    );
    for size in [0.0, 50.0, 100.0] {
        assert_eq!(vignette(color, -100.0, size, [0.5, 0.5]), color);
        assert!(
            vignette(color, -100.0, size, [0.0, 0.0])
                .iter()
                .all(|v| v.abs() < 1e-12)
        );
        assert!(
            vignette(color, 100.0, size, [1.0, 1.0])
                .iter()
                .all(|v| (v - 1.0).abs() < 1e-12)
        );
    }
    // A larger size reaches further in.
    let edge = [0.9, 0.5];
    assert_eq!(vignette(color, -60.0, 0.0, edge), color);
    assert!(vignette(color, -60.0, 80.0, edge)[0] < color[0]);
}

/// What the formulas promise, on single pixels: gray keeps its luminance
/// through white balance, contrast keeps black, middle gray and white where
/// they are, and saturation −100 leaves gray.
#[test]
fn colors_outside_display_p3_go_through_every_operation() {
    let renderer = renderer();
    // Wider-gamut colors have components below 0 in Display P3.
    let pixels = [[-0.05, 0.4, 0.2], [0.6, -0.02, 0.3], [0.1, 0.5, -0.1]]
        .iter()
        .flat_map(|[r, g, b]| [*r, *g, *b, 1.0].map(f16::from_f32))
        .collect();
    let image = LinearImage {
        width: 3,
        height: 1,
        pixels,
    };
    let settings = Settings {
        temperature: -30.0,
        contrast: 40.0,
        saturation: 60.0,
        ..Settings::NEUTRAL
    };
    for output in [Output::Srgb, Output::DisplayP3] {
        assert_matches(&image, &settings, output, &renderer);
    }
    // A component below 0 stays below 0 through saturation.
    assert!(saturation([-0.05, 0.4, 0.2], 60.0)[0] < -0.05);
}

#[test]
fn light_and_color_keep_what_they_promise() {
    let gray = [0.18, 0.18, 0.18];
    let balanced = white_balance(gray, 80.0, -30.0);
    assert!((luminance(balanced) - 0.18).abs() < 1e-12);
    assert!(balanced[0] > balanced[2], "warmer: more red than blue");
    for amount in [-100.0, 50.0, 100.0] {
        assert_eq!(contrast([0.0; 3], amount), [0.0; 3]);
        assert!((luminance(contrast(gray, amount)) - 0.18).abs() < 1e-12);
        assert!((luminance(contrast([1.0; 3], amount)) - 1.0).abs() < 1e-12);
    }
    let darker = luminance(contrast([0.05; 3], 60.0));
    assert!(darker < 0.05, "more contrast darkens the shadows");
    let color = [0.6, 0.3, 0.1];
    let gray_out = saturation(color, -100.0);
    assert!(
        gray_out
            .iter()
            .all(|v| (v - luminance(color)).abs() < 1e-12)
    );
}

/// One of the formulas appendix's test vectors: an operation's params, an
/// input color, and what it must come out as; for a neighborhood operation,
/// also the pixel's base, from the luminance guide, and for the vignette,
/// where the pixel is.
struct Vector {
    settings: Settings,
    base: Option<f64>,
    /// For an operation measured on the cropped frame, where the pixel is
    /// in it, as fractions.
    position: Option<[f64; 2]>,
    input: [f64; 3],
    output: [f64; 3],
}

/// Every vector in the edit format's `tests/formulas/`, with the settings
/// that give each operation its params alone.
fn appendix_vectors() -> Vec<(String, Vec<Vector>)> {
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../third_party/edit-format/tests/formulas");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&folder)
        .expect("the edit format's test vectors; run `git submodule update --init`")
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    files.sort();
    files
        .iter()
        .map(|path| {
            let file: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
            let operation = file["operation"].as_str().unwrap().to_string();
            let color = |value: &serde_json::Value| -> [f64; 3] {
                let values: Vec<f64> = value
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64().unwrap())
                    .collect();
                values.try_into().unwrap()
            };
            let vectors = file["cases"]
                .as_array()
                .unwrap()
                .iter()
                .map(|case| {
                    let param = |name: &str| case["params"][name].as_f64().unwrap() as f32;
                    let neutral = Settings::NEUTRAL;
                    let settings = match operation.as_str() {
                        "seura.exposure" => Settings {
                            exposure_ev: param("ev"),
                            ..neutral
                        },
                        "seura.whiteBalance" => Settings {
                            temperature: param("temperature"),
                            tint: param("tint"),
                            ..neutral
                        },
                        "seura.contrast" => Settings {
                            contrast: param("amount"),
                            ..neutral
                        },
                        "seura.saturation" => Settings {
                            saturation: param("amount"),
                            ..neutral
                        },
                        "seura.highlights" => Settings {
                            highlights: param("amount"),
                            ..neutral
                        },
                        "seura.shadows" => Settings {
                            shadows: param("amount"),
                            ..neutral
                        },
                        "seura.blackAndWhite" => Settings {
                            black_and_white: param("strength"),
                            ..neutral
                        },
                        "seura.sepia" => Settings {
                            sepia: param("strength"),
                            ..neutral
                        },
                        "seura.vignette" => Settings {
                            vignette_amount: param("amount"),
                            vignette_size: param("size"),
                            ..neutral
                        },
                        other => panic!("{other} has test vectors, but no test here yet"),
                    };
                    Vector {
                        settings,
                        base: case["base"].as_f64(),
                        position: case["position"]
                            .as_array()
                            .map(|at| [at[0].as_f64().unwrap(), at[1].as_f64().unwrap()]),
                        input: color(&case["input"]),
                        output: color(&case["output"]),
                    }
                })
                .collect();
            (operation, vectors)
        })
        .collect()
}

/// The definitions above are the appendix's: every vector, within its
/// files' tolerance (absolute 0.0001, relative 0.001).
#[test]
fn the_definitions_match_the_appendix_vectors() {
    let all = appendix_vectors();
    assert!(
        all.len() >= 9,
        "exposure, white balance, highlights, shadows, contrast, saturation, \
         black and white, sepia and the vignette"
    );
    for (operation, vectors) in &all {
        for vector in vectors {
            // A guide with a = 0 and b = the base gives every pixel that base.
            let found = defined(
                vector.input,
                &vector.settings,
                [0.0, vector.base.unwrap_or(0.0)],
                vector.position.unwrap_or([0.5; 2]),
            );
            for (found, want) in found.iter().zip(vector.output) {
                assert!(
                    (found - want).abs() <= 0.0001 + 0.001 * want.abs(),
                    "{operation}, {:?} on {:?}: found {found:?}, expected {:?}",
                    vector.settings,
                    vector.input,
                    vector.output,
                );
            }
        }
    }
}

/// The shaders give the appendix's outputs, within one level once encoded:
/// the vectors' inputs are half floats, so they go in unchanged.
#[test]
fn the_shaders_match_the_appendix_vectors() {
    let renderer = renderer();
    for (operation, vectors) in appendix_vectors() {
        // A neighborhood operation's base comes from the photo, and the
        // vignette's position from the pixel's place, so a vector's can't
        // be given to the shaders; the tests above compare them with the
        // definitions instead.
        for vector in vectors
            .iter()
            .filter(|vector| vector.base.is_none() && vector.position.is_none())
        {
            let pixel = [vector.input[0], vector.input[1], vector.input[2], 1.0].map(f16::from_f64);
            assert_eq!(pixel.map(|v| v.to_f64())[..3], vector.input, "exact in f16");
            let image = LinearImage {
                width: 1,
                height: 1,
                pixels: pixel.to_vec(),
            };
            let rendered = renderer
                .render(
                    &image,
                    &image.guide_image(),
                    &vector.settings,
                    Output::DisplayP3,
                )
                .unwrap();
            let want = vector.output.map(|v| (encode(v) * 255.0).round() as u8);
            let off = rendered.pixels[..3]
                .iter()
                .zip(want)
                .map(|(a, b)| a.abs_diff(b))
                .max()
                .unwrap();
            assert!(
                off <= 1,
                "{operation}, {:?} on {:?}: found {:?}, expected {want:?}",
                vector.settings,
                vector.input,
                &rendered.pixels[..3],
            );
        }
    }
}

/// Larger than one tile both ways, so tiles are placed and joined right.
#[test]
fn large_photos_are_rendered_in_tiles_seamlessly() {
    let renderer = renderer();
    let image = ramps(2300, 2100);
    let settings = Settings {
        exposure_ev: 0.5,
        highlights: -60.0,
        shadows: 50.0,
        contrast: 30.0,
        ..Settings::NEUTRAL
    };
    assert_matches(&image, &settings, Output::Srgb, &renderer);
    // Turned, each tile of the drawn photo comes from a different part of
    // the upright one; straightened, from a part wider than the tile.
    let turned = Settings {
        rotate: 90,
        flip_v: true,
        ..settings
    };
    assert_matches(&image, &turned, Output::Srgb, &renderer);
    let straightened = Settings {
        straighten: -3.5,
        crop_x: 0.05,
        crop_y: 0.04,
        crop_width: 0.9,
        crop_height: 0.92,
        ..settings
    };
    assert_matches(&image, &straightened, Output::Srgb, &renderer);
}

/// A dark half beside a bright one, each with fine texture: shadows lifts
/// the dark half and highlights darkens the bright one, each evenly up to
/// the edge between them, where a plain blur would leave a halo, and the
/// texture stays (journal, 2026-10-29).
#[test]
fn highlights_and_shadows_keep_edges_without_halos() {
    let (width, height) = (400usize, 60usize);
    let mut pixels = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let texture = 1.0 + 0.25 * (x as f32 * 0.8).sin() * (y as f32 * 0.8).sin();
            let v = if x < width / 2 { 0.02 } else { 0.7 } * texture;
            pixels.extend([v, v, v, 1.0].map(f16::from_f32));
        }
    }
    let image = LinearImage {
        width: width as u32,
        height: height as u32,
        pixels,
    };
    let settings = Settings {
        highlights: -100.0,
        shadows: 100.0,
        ..Settings::NEUTRAL
    };
    let guide = Guide::new(&image.guide_image(), &settings);
    // The gain in stops, averaged down each column.
    let gain = |x: usize| -> f64 {
        (0..height)
            .map(|y| {
                let i = y * width + x;
                let c = [0, 1, 2].map(|k| f64::from(image.pixels[i * 4 + k].to_f32()));
                let ab = guide.sample(
                    (x as f64 + 0.5) / width as f64,
                    (y as f64 + 0.5) / height as f64,
                );
                log_luminance(defined(c, &settings, ab, [0.5; 2])) - log_luminance(c)
            })
            .sum::<f64>()
            / height as f64
    };
    let (dark, bright) = (gain(width / 8), gain(width * 7 / 8));
    assert!(dark > 1.0, "the dark half is lifted: {dark}");
    assert!(bright < -0.5, "the bright half is darkened: {bright}");
    for x in 0..width {
        let far = if x < width / 2 { dark } else { bright };
        let off = (gain(x) - far).abs();
        assert!(
            off < 0.1,
            "column {x}: {off} stops from the rest of its half"
        );
    }
    // Across one period of the texture, the base hardly moves, so the
    // texture is kept rather than evened out.
    let bases: Vec<f64> = (40..48)
        .map(|x| {
            let i = 30 * width + x;
            let c = [0, 1, 2].map(|k| f64::from(image.pixels[i * 4 + k].to_f32()));
            let [a, b] = guide.sample((x as f64 + 0.5) / width as f64, 30.5 / height as f64);
            a * log_luminance(c) + b
        })
        .collect();
    let spread = bases.iter().cloned().fold(f64::MIN, f64::max)
        - bases.iter().cloned().fold(f64::MAX, f64::min);
    assert!(
        spread < 0.25,
        "the base follows the texture by {spread} stops"
    );
    let renderer = renderer();
    for output in [Output::Srgb, Output::DisplayP3] {
        assert_matches(&image, &settings, output, &renderer);
    }
}

/// Display P3 green is outside sRGB: it must clip to sRGB's green, not
/// come out as the same numbers re-labelled.
#[test]
fn wide_color_is_converted_for_srgb_export() {
    let renderer = renderer();
    let green = [0.0, 1.0, 0.0, 1.0].map(f16::from_f32);
    let image = LinearImage {
        width: 1,
        height: 1,
        pixels: green.to_vec(),
    };
    let srgb = renderer
        .render(
            &image,
            &image.guide_image(),
            &Settings::default(),
            Output::Srgb,
        )
        .unwrap();
    // P3 green in linear sRGB is (-0.2249, 1.0421, -0.0786): clipped, (0, 1, 0).
    assert_eq!(&srgb.pixels[..3], &[0, 255, 0]);
    let mid = [0.5, 0.2, 0.1, 1.0].map(f16::from_f32);
    let image = LinearImage {
        width: 1,
        height: 1,
        pixels: mid.to_vec(),
    };
    let p3 = renderer
        .render(
            &image,
            &image.guide_image(),
            &Settings::default(),
            Output::DisplayP3,
        )
        .unwrap();
    let srgb = renderer
        .render(
            &image,
            &image.guide_image(),
            &Settings::default(),
            Output::Srgb,
        )
        .unwrap();
    assert_ne!(
        &p3.pixels[..3],
        &srgb.pixels[..3],
        "a saturated color differs between the two encodings"
    );
}
