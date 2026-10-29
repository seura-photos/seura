//! The shaders, run with wgpu, against the edit format's definitions
//! computed here on the CPU, as its formulas appendix gives them: each tone
//! and color operation in the canonical order, then the result encoded for
//! Display P3 or sRGB (EXP-5).
//!
//! The images are generated: ramps through every channel, including Display
//! P3 colors outside sRGB, which sRGB output must clip.

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

/// What a pixel must come out as.
fn expected(pixel: [f16; 4], settings: &Settings, output: Output) -> [u8; 3] {
    let scale = 2f64.powf(settings.exposure_ev.into());
    let mut p3 = [0, 1, 2].map(|i| f64::from(pixel[i].to_f32()) * scale);
    if settings.temperature != 0.0 || settings.tint != 0.0 {
        p3 = white_balance(p3, settings.temperature.into(), settings.tint.into());
    }
    if settings.contrast != 0.0 {
        p3 = contrast(p3, settings.contrast.into());
    }
    if settings.saturation != 0.0 {
        p3 = saturation(p3, settings.saturation.into());
    }
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
fn assert_matches(image: &LinearImage, settings: &Settings, output: Output, renderer: &Renderer) {
    let rendered = renderer.render(image, settings, output).unwrap();
    assert_eq!(
        (rendered.width, rendered.height),
        (image.width, image.height)
    );
    let mut worst = (0, 0);
    for (i, (found, source)) in rendered
        .pixels
        .chunks(4)
        .zip(image.pixels.chunks(4))
        .enumerate()
    {
        let want = expected(source.try_into().unwrap(), settings, output);
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
        expected(
            image.pixels[i * 4..i * 4 + 4].try_into().unwrap(),
            settings,
            output
        ),
    );
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
    }
    cases.push(Settings {
        exposure_ev: 0.6,
        temperature: 30.0,
        tint: -15.0,
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

/// Larger than one tile both ways, so tiles are placed and joined right.
#[test]
fn large_photos_are_rendered_in_tiles_seamlessly() {
    let renderer = renderer();
    let image = ramps(2300, 2100);
    let settings = Settings {
        exposure_ev: 0.5,
        contrast: 30.0,
        ..Settings::NEUTRAL
    };
    assert_matches(&image, &settings, Output::Srgb, &renderer);
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
        .render(&image, &Settings::default(), Output::Srgb)
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
        .render(&image, &Settings::default(), Output::DisplayP3)
        .unwrap();
    let srgb = renderer
        .render(&image, &Settings::default(), Output::Srgb)
        .unwrap();
    assert_ne!(
        &p3.pixels[..3],
        &srgb.pixels[..3],
        "a saturated color differs between the two encodings"
    );
}
