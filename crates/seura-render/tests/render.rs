//! The shaders, run with wgpu, against the edit format's definitions
//! computed here on the CPU: `seura.exposure` multiplies linear light by
//! 2^ev, then the result is encoded for Display P3 or sRGB (EXP-5).
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

fn encode(linear: f64) -> f64 {
    let v = linear.clamp(0.0, 1.0);
    if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// What a pixel must come out as.
fn expected(pixel: [f16; 4], ev: f64, output: Output) -> [u8; 3] {
    let scale = 2f64.powf(ev);
    let p3 = [0, 1, 2].map(|i| f64::from(pixel[i].to_f32()) * scale);
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
fn assert_matches(image: &LinearImage, ev: f64, output: Output, renderer: &Renderer) {
    let settings = Settings {
        exposure_ev: ev as f32,
        ..Settings::NEUTRAL
    };
    let rendered = renderer.render(image, &settings, output).unwrap();
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
        let want = expected(source.try_into().unwrap(), ev, output);
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
        "ev {ev}, {output:?}: pixel {i} is off by {off}: found {:?}, expected {:?}",
        &rendered.pixels[i * 4..i * 4 + 3],
        expected(
            image.pixels[i * 4..i * 4 + 4].try_into().unwrap(),
            ev,
            output
        ),
    );
}

#[test]
fn exposure_matches_the_definition() {
    let renderer = renderer();
    let image = ramps(97, 64);
    for ev in [0.0, 1.0, -1.5, 0.3, 4.0, -4.0] {
        for output in [Output::Srgb, Output::DisplayP3] {
            assert_matches(&image, ev, output, &renderer);
        }
    }
}

/// Larger than one tile both ways, so tiles are placed and joined right.
#[test]
fn large_photos_are_rendered_in_tiles_seamlessly() {
    let renderer = renderer();
    let image = ramps(2300, 2100);
    assert_matches(&image, 0.5, Output::Srgb, &renderer);
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
