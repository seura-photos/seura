//! One-click enhance (EDT-12; technical design, "One-click enhance"): from
//! the photo's own pixels, as cropped and before any tone or color setting,
//! the seven settings that correct its brightness and color. The result is
//! ordinary slider settings, so the user can adjust or undo it, and since
//! it never looks at the tone settings it replaces, pressing it again gives
//! the same result, not a stronger one.
//!
//! It works on the guide image, the photo scaled to 1024 pixels on its long
//! side (seura-image's `guide_image`): small enough to read in a few
//! milliseconds on the CPU, and the same whichever size the photo is
//! decoded at, so the Editor and the grid get the same answer.
//!
//! Every number below was chosen by eye on generated test photos and
//! Nevada's own (M4 slice 8, journal), and may change as that tuning goes
//! on: they're not part of the edit format, which stores only the result.

use seura_image::LinearImage;

use crate::Settings;
use crate::geometry::Geometry;

/// What Enhance sets: the seven settings it owns (EDT-12), on the sliders'
/// scales. Neutral ones remove their operation, as any slider's do.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Enhanced {
    /// `seura.exposure`, in stops, in the brightness slider's 25ths.
    pub exposure_ev: f32,
    pub temperature: f32,
    pub tint: f32,
    pub highlights: f32,
    pub shadows: f32,
    pub contrast: f32,
    pub saturation: f32,
}

/// The luminance of linear Display P3 (the formulas appendix).
const LUMINANCE: [f32; 3] = [0.228_974_6, 0.691_738_5, 0.079_286_9];

/// How much of a near-neutral color cast is taken out: real scenes' grays
/// aren't quite gray on average, so a full correction overshoots.
const WHITE_BALANCE_STRENGTH: f32 = 0.8;
/// The largest white balance correction, on the sliders' −100 to 100.
const WHITE_BALANCE_LIMIT: f32 = 60.0;
/// Near-neutral colors, found in steps: how far below its highest channel
/// a color's lowest may be, as a share of it, loosely and then strictly.
const NEUTRAL_STEPS: [f32; 3] = [0.45, 0.3, 0.2];
/// Too few near-neutral pixels, and the cast can't be told from the scene.
const NEUTRAL_SHARE: f32 = 0.02;

/// Middle gray, where a typical photo's average sits.
const MIDDLE_GRAY: f32 = 0.18;
/// How far toward middle gray the brightness is moved.
const EXPOSURE_STRENGTH: f32 = 0.6;
/// How far the bright end may be pushed past white: highlights pulls back
/// the rest.
const EXPOSURE_HEADROOM: f32 = 0.5;
const EXPOSURE_LIMIT: f32 = 2.0;

/// The spread from the darkest to the brightest tones, as positions from
/// black to white, below which the photo looks flat.
const FLAT_SPREAD: f32 = 0.85;
const CONTRAST_GAIN: f32 = 150.0;
const CONTRAST_LIMIT: f32 = 40.0;

/// Very dark and very bright, as positions from black to white.
const DEEP_SHADOW: f32 = 0.15;
const BRIGHT_HIGHLIGHT: f32 = 0.9;
/// Shadows and highlights' amount for each share of the photo that's very
/// dark or very bright, and their limit.
const TONE_GAIN: f32 = 250.0;
const TONE_LIMIT: f32 = 60.0;

/// The average colorfulness below which a little saturation helps, and
/// its limit.
const DULL: f32 = 0.3;
const SATURATION_GAIN: f32 = 100.0;
const SATURATION_LIMIT: f32 = 20.0;

/// Analyzes `image`, the upright photo at the guide's size, as `settings`
/// crop and turn it, before any of their tone or color.
pub fn enhance(image: &LinearImage, settings: &Settings) -> Enhanced {
    let pixels = cropped_pixels(image, settings);
    if pixels.is_empty() {
        return Enhanced::default();
    }

    // White balance: from the average of the near-neutral pixels. A cast
    // tints the grays too, so they're found in steps: loosely at first,
    // then more strictly, each time as the cast found so far corrects them,
    // each weighed by how near gray it is.
    let midtones: Vec<[f32; 3]> = pixels
        .iter()
        .copied()
        .filter(|c| (0.02..=0.9).contains(&luminance(*c)))
        .collect();
    let mut cast = [1.0f32; 3];
    let mut found = false;
    for limit in NEUTRAL_STEPS {
        let mut sum = [0.0f32; 3];
        let mut weights = 0.0;
        for c in &midtones {
            let fixed = [0, 1, 2].map(|k| c[k] / cast[k]);
            let low = fixed[0].min(fixed[1]).min(fixed[2]);
            let high = fixed[0].max(fixed[1]).max(fixed[2]);
            if high <= 0.0 {
                continue;
            }
            let weight = 1.0 - (high - low) / high / limit;
            if weight > 0.0 {
                let y = luminance(*c).max(1e-6);
                for k in 0..3 {
                    sum[k] += weight * c[k] / y;
                }
                weights += weight;
            }
        }
        found = weights >= NEUTRAL_SHARE * pixels.len() as f32;
        if !found {
            break;
        }
        let mean = sum.map(|v| v / weights);
        let y = luminance(mean);
        cast = mean.map(|v| (v / y).max(1e-6));
    }
    let (mut temperature, mut tint) = (0.0, 0.0);
    if found {
        let [r, g, b] = cast.map(f32::log2);
        // The whiteBalance formula's gains, 2^(t/2), 2^(-n/2), 2^(-t/2),
        // make the three equal for t = b - r and n = 2g - r - b.
        let slider = |value: f32| {
            let value = (value * 100.0 * WHITE_BALANCE_STRENGTH)
                .clamp(-WHITE_BALANCE_LIMIT, WHITE_BALANCE_LIMIT)
                .round();
            if value.abs() < 3.0 { 0.0 } else { value }
        };
        temperature = slider(b - r);
        tint = slider(2.0 * g - r - b);
    }
    let gains = white_balance_gains(temperature, tint);
    let balanced: Vec<[f32; 3]> = pixels
        .iter()
        .map(|c| [c[0] * gains[0], c[1] * gains[1], c[2] * gains[2]])
        .collect();

    // Brightness: the log-average toward middle gray, keeping most of the
    // bright end below white.
    let mut lums: Vec<f32> = balanced.iter().map(|c| luminance(*c).max(0.0)).collect();
    lums.sort_by(f32::total_cmp);
    let percentile = |p: f32| lums[((lums.len() - 1) as f32 * p).round() as usize];
    let key = (lums.iter().map(|y| y.max(1e-4).ln()).sum::<f32>() / lums.len() as f32).exp();
    let mut ev = (MIDDLE_GRAY / key).log2() * EXPOSURE_STRENGTH;
    let brightest = percentile(0.995).max(1e-4);
    ev = ev.min((1.0 / brightest).log2() + EXPOSURE_HEADROOM);
    let ev = (ev.clamp(-EXPOSURE_LIMIT, EXPOSURE_LIMIT) * 25.0).round() / 25.0;
    let ev = if ev.abs() < 0.05 { 0.0 } else { ev };

    // Then, as positions from black to white, how the tones spread.
    let scale = ev.exp2();
    let position = |y: f32| (y * scale).clamp(0.0, 1.0).powf(1.0 / 2.2);
    let share = |test: &dyn Fn(f32) -> bool| {
        lums.iter().filter(|y| test(position(**y))).count() as f32 / lums.len() as f32
    };
    let shadows = (share(&|x| x < DEEP_SHADOW) * TONE_GAIN)
        .min(TONE_LIMIT)
        .round();
    let highlights = -(share(&|x| x > BRIGHT_HIGHLIGHT) * TONE_GAIN)
        .min(TONE_LIMIT)
        .round();
    let spread = position(percentile(0.995)) - position(percentile(0.005));
    let contrast = ((FLAT_SPREAD - spread) * CONTRAST_GAIN)
        .clamp(0.0, CONTRAST_LIMIT)
        .round();

    // Colorfulness: a little more when the photo is dull, never less.
    let colorful: Vec<f32> = balanced
        .iter()
        .filter(|c| luminance(**c) > 0.02)
        .map(|c| {
            let (low, high) = (c[0].min(c[1]).min(c[2]), c[0].max(c[1]).max(c[2]));
            if high > 0.0 { (high - low) / high } else { 0.0 }
        })
        .collect();
    let saturation = if colorful.is_empty() {
        0.0
    } else {
        let mean = colorful.iter().sum::<f32>() / colorful.len() as f32;
        ((DULL - mean) * SATURATION_GAIN)
            .clamp(0.0, SATURATION_LIMIT)
            .round()
    };

    Enhanced {
        exposure_ev: ev,
        temperature,
        tint,
        // −0 as 0, so nothing recovers no highlights.
        highlights: highlights + 0.0,
        shadows,
        contrast,
        saturation,
    }
}

fn luminance(c: [f32; 3]) -> f32 {
    LUMINANCE[0] * c[0] + LUMINANCE[1] * c[1] + LUMINANCE[2] * c[2]
}

/// `seura.whiteBalance`'s gains (the formulas appendix): scaled so gray
/// keeps its luminance.
fn white_balance_gains(temperature: f32, tint: f32) -> [f32; 3] {
    let (t, n) = (temperature / 100.0, tint / 100.0);
    let gains = [(t / 2.0).exp2(), (-n / 2.0).exp2(), (-t / 2.0).exp2()];
    let gray = luminance(gains);
    gains.map(|gain| gain / gray)
}

/// The result's pixels, at the image's resolution: each taken from the
/// nearest pixel of `image` where geometry places it. Positions past the
/// photo's edge, the corners a turned photo leaves uncovered, are left out.
fn cropped_pixels(image: &LinearImage, settings: &Settings) -> Vec<[f32; 3]> {
    let geometry = Geometry::new(settings, image.width, image.height);
    let (across, down) = geometry.output_size();
    let mut pixels = Vec::with_capacity(across as usize * down as usize);
    for j in 0..down {
        for i in 0..across {
            let (x, y) = geometry.upright_position(f64::from(i) + 0.5, f64::from(j) + 0.5);
            if x < 0.0 || y < 0.0 || x >= f64::from(image.width) || y >= f64::from(image.height) {
                continue;
            }
            let at = (y as usize * image.width as usize + x as usize) * 4;
            pixels.push([0, 1, 2].map(|k| image.pixels[at + k].to_f32()));
        }
    }
    pixels
}

#[cfg(test)]
mod tests {
    use half::f16;

    use super::*;

    /// A generated photo: a sky over the top 30%, then a gray wall beside
    /// green ground, and a gray ramp along the bottom, then `fault`
    /// applied.
    fn photo(fault: impl Fn([f32; 3]) -> [f32; 3]) -> LinearImage {
        let (width, height) = (240u32, 160u32);
        let mut pixels = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let (u, v) = (x as f32 / width as f32, y as f32 / height as f32);
                let color = if v > 0.8 {
                    [u; 3]
                } else if v < 0.3 {
                    [0.22, 0.30, 0.45]
                } else if u < 0.3 {
                    [0.18, 0.18, 0.18]
                } else {
                    [0.07, 0.12, 0.04]
                };
                pixels.extend(fault(color).into_iter().chain([1.0]).map(f16::from_f32));
            }
        }
        LinearImage {
            width,
            height,
            pixels,
        }
    }

    #[test]
    fn a_well_exposed_photo_is_left_nearly_alone() {
        let found = enhance(&photo(|c| c), &Settings::NEUTRAL);
        assert!(found.exposure_ev.abs() <= 0.4, "{found:?}");
        assert_eq!((found.temperature, found.tint), (0.0, 0.0), "{found:?}");
    }

    #[test]
    fn a_dark_photo_is_brightened_and_a_bright_one_darkened() {
        let dark = enhance(&photo(|c| c.map(|v| v * 0.15)), &Settings::NEUTRAL);
        assert!(dark.exposure_ev > 0.8, "{dark:?}");
        let bright = enhance(
            &photo(|c| c.map(|v| (v * 3.0).min(1.0))),
            &Settings::NEUTRAL,
        );
        assert!(bright.exposure_ev < -0.3, "{bright:?}");
    }

    #[test]
    fn a_backlit_photo_gets_its_sky_recovered_and_its_shadows_lifted() {
        // A sky near white over a ground in deep shadow.
        let backlit = enhance(
            &photo(|c| {
                if c[2] > 0.4 {
                    [0.9, 0.93, 0.97]
                } else {
                    c.map(|v| v * 0.12)
                }
            }),
            &Settings::NEUTRAL,
        );
        assert!(backlit.highlights < 0.0, "{backlit:?}");
        // The dark ground is lifted, by brightness or by shadows.
        assert!(
            backlit.exposure_ev > 0.3 || backlit.shadows > 10.0,
            "{backlit:?}"
        );
    }

    #[test]
    fn a_color_cast_is_taken_out() {
        // Blue, as under shade: warmer. Green, as under fluorescent light:
        // toward magenta.
        let blue = enhance(
            &photo(|[r, g, b]| [r * 0.8, g, b * 1.25]),
            &Settings::NEUTRAL,
        );
        assert!(blue.temperature > 10.0, "{blue:?}");
        let green = enhance(&photo(|[r, g, b]| [r, g * 1.25, b]), &Settings::NEUTRAL);
        assert!(green.tint > 10.0, "{green:?}");
        // Opposite casts, opposite corrections: warm, as under tungsten
        // light indoors, where there's no sky but gray walls.
        let warm = enhance(
            &photo(|c| {
                let [r, g, b] = if c[2] > 0.4 { [0.3, 0.3, 0.3] } else { c };
                [r * 1.25, g, b * 0.8]
            }),
            &Settings::NEUTRAL,
        );
        assert!(warm.temperature < -10.0, "{warm:?}");
    }

    /// The limit of finding the cast from near-neutral colors: under a
    /// warm cast, a blue sky looks nearly gray, and outweighs the true
    /// grays, so the warmth stays. Outdoors that's usually what's wanted,
    /// the warm light of a late afternoon. The correction's strength and
    /// limit keep it from going far wrong either way.
    #[test]
    fn a_large_blue_sky_under_a_warm_cast_misleads_it_within_limits() {
        let mut image = photo(|[r, g, b]| [r * 1.25, g, b * 0.8]);
        let sky = [0.22 * 1.25, 0.30, 0.45 * 0.8].map(f16::from_f32);
        for row in image.pixels.chunks_mut(4 * 240).take(80) {
            for pixel in row.chunks_mut(4) {
                pixel[..3].copy_from_slice(&sky);
            }
        }
        let found = enhance(&image, &Settings::NEUTRAL);
        assert!(found.temperature.abs() <= WHITE_BALANCE_LIMIT, "{found:?}");
    }

    #[test]
    fn a_flat_photo_gets_contrast_and_a_dull_one_saturation() {
        // Everything squeezed toward middle gray, and colors faded.
        let flat = enhance(
            &photo(|c| {
                let y = luminance(c);
                c.map(|v| 0.12 + (y + (v - y) * 0.4) * 0.25)
            }),
            &Settings::NEUTRAL,
        );
        assert!(flat.contrast > 10.0, "{flat:?}");
        assert!(flat.saturation > 0.0, "{flat:?}");
    }

    #[test]
    fn it_reads_the_photo_as_cropped_and_ignores_its_tone_settings() {
        // The left half dark, the right half bright.
        let image = photo(|c| c);
        let mut halves = image.clone();
        for (i, pixel) in halves.pixels.chunks_mut(4).enumerate() {
            let x = i as u32 % image.width;
            let scale = if x < image.width / 2 { 0.05 } else { 1.0 };
            for value in &mut pixel[..3] {
                *value = f16::from_f32(value.to_f32() * scale);
            }
        }
        let left = Settings {
            crop_width: 0.5,
            ..Settings::NEUTRAL
        };
        let right = Settings {
            crop_x: 0.5,
            crop_width: 0.5,
            ..Settings::NEUTRAL
        };
        assert!(enhance(&halves, &left).exposure_ev > enhance(&halves, &right).exposure_ev);
        // Pressing again gives the same: the tone settings aren't read.
        let toned = Settings {
            exposure_ev: 1.5,
            contrast: 40.0,
            temperature: -30.0,
            ..left
        };
        assert_eq!(enhance(&halves, &left), enhance(&halves, &toned));
    }
}
