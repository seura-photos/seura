//! The shaders, run with wgpu, against the edit format's definitions
//! computed here on the CPU, as its formulas appendix gives them: the photo
//! turned and flipped, each tone and color operation in the canonical
//! order, then the result encoded for Display P3 or sRGB (EXP-5). The definitions are checked against the
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

/// The tone and color operations on a linear Display P3 color, in the
/// canonical order; `ab` is the guide where the pixel is, for highlights
/// and shadows.
fn defined(color: [f64; 3], settings: &Settings, ab: [f64; 2]) -> [f64; 3] {
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
    p3
}

/// What a pixel must come out as, given the guide where it is.
fn expected(pixel: [f16; 4], settings: &Settings, output: Output, ab: [f64; 2]) -> [u8; 3] {
    let p3 = defined(
        [0, 1, 2].map(|i| f64::from(pixel[i].to_f32())),
        settings,
        ab,
    );
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

/// Every pixel within one level of the definition: GPU arithmetic isn't
/// bit-exact, but it's far closer than the 8-bit output can show.
/// `seura.orient` as the edit format defines it: where each pixel of the
/// upright photo lands, by index, turned clockwise and then flipped as it's
/// then seen. Returns, for each drawn pixel, the upright pixel drawn there,
/// and the drawn size.
fn oriented(settings: &Settings, width: usize, height: usize) -> (Vec<usize>, usize, usize) {
    let sideways = settings.rotate % 180 == 90;
    let (across, down) = if sideways {
        (height, width)
    } else {
        (width, height)
    };
    let mut from = vec![usize::MAX; width * height];
    for y in 0..height {
        for x in 0..width {
            let (u, v) = match settings.rotate {
                90 => (height - 1 - y, x),
                180 => (width - 1 - x, height - 1 - y),
                270 => (y, width - 1 - x),
                _ => (x, y),
            };
            let u = if settings.flip_h { across - 1 - u } else { u };
            let v = if settings.flip_v { down - 1 - v } else { v };
            from[v * across + u] = y * width + x;
        }
    }
    (from, across, down)
}

fn assert_matches(image: &LinearImage, settings: &Settings, output: Output, renderer: &Renderer) {
    let rendered = renderer
        .render(image, &image.guide_image(), settings, output)
        .unwrap();
    let (width, height) = (image.width as usize, image.height as usize);
    let (from, across, down) = oriented(settings, width, height);
    assert_eq!(
        (rendered.width as usize, rendered.height as usize),
        (across, down)
    );
    let guide = Guide::new(&image.guide_image(), settings);
    // The guide is sampled where the pixel is in the upright photo.
    let ab = |i: usize| {
        let j = from[i];
        guide.sample(
            ((j % width) as f64 + 0.5) / width as f64,
            ((j / width) as f64 + 0.5) / height as f64,
        )
    };
    let source = |i: usize| -> [f16; 4] {
        image.pixels[from[i] * 4..from[i] * 4 + 4]
            .try_into()
            .unwrap()
    };
    let mut worst = (0, 0);
    for (i, found) in rendered.pixels.chunks(4).enumerate() {
        let want = expected(source(i), settings, output, ab(i));
        let off = found[..3]
            .iter()
            .zip(want)
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
        expected(source(i), settings, output, ab(i)),
    );
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
/// also the pixel's base, from the luminance guide.
struct Vector {
    settings: Settings,
    base: Option<f64>,
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
                        other => panic!("{other} has test vectors, but no test here yet"),
                    };
                    Vector {
                        settings,
                        base: case["base"].as_f64(),
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
        all.len() >= 6,
        "exposure, white balance, highlights, shadows, contrast and saturation"
    );
    for (operation, vectors) in &all {
        for vector in vectors {
            // A guide with a = 0 and b = the base gives every pixel that base.
            let found = defined(
                vector.input,
                &vector.settings,
                [0.0, vector.base.unwrap_or(0.0)],
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
        // A neighborhood operation's base comes from the photo, so a vector's
        // can't be given to the shaders; the tests above compare them with
        // the definitions instead.
        for vector in vectors.iter().filter(|vector| vector.base.is_none()) {
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
    // the upright one.
    let turned = Settings {
        rotate: 90,
        flip_v: true,
        ..settings
    };
    assert_matches(&image, &turned, Output::Srgb, &renderer);
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
                log_luminance(defined(c, &settings, ab)) - log_luminance(c)
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
