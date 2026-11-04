//! Color descriptions a photo can carry instead of an ICC profile, as ICC
//! profiles, so one conversion serves them all: HEIC's `nclx` and PNG's
//! `cICP` (ITU-T H.273's code points), and PNG's `gAMA` and `cHRM`.

use moxcms::{
    Chromaticity, CicpColorPrimaries, CicpProfile, ColorPrimaries, ColorProfile,
    MatrixCoefficients, ToneReprCurve, TransferCharacteristics,
};

/// "Unspecified", in both code point lists.
const UNSPECIFIED: u8 = 2;
const BT709_PRIMARIES: u8 = 1;
const SRGB_TRANSFER: u8 = 13;

/// A profile for code points, or `None` for sRGB and for code points that
/// don't describe colors Seura can convert, which are taken as sRGB.
/// Unspecified primaries or transfer take sRGB's.
pub(crate) fn from_code_points(primaries: u8, transfer: u8) -> Option<Vec<u8>> {
    let primaries = if primaries == UNSPECIFIED {
        BT709_PRIMARIES
    } else {
        primaries
    };
    let transfer = if transfer == UNSPECIFIED {
        SRGB_TRANSFER
    } else {
        transfer
    };
    if (primaries, transfer) == (BT709_PRIMARIES, SRGB_TRANSFER) {
        return None;
    }
    let color_primaries = CicpColorPrimaries::try_from(primaries).ok()?;
    let transfer_characteristics = TransferCharacteristics::try_from(transfer).ok()?;
    // Not every code point has values, and the profile's update reports
    // neither.
    ColorPrimaries::try_from(color_primaries).ok()?;
    ToneReprCurve::try_from(transfer_characteristics).ok()?;
    let mut profile = ColorProfile::new_srgb();
    profile.update_rgb_colorimetry_from_cicp(CicpProfile {
        color_primaries,
        transfer_characteristics,
        matrix_coefficients: MatrixCoefficients::Bt709,
        full_range: true,
    });
    profile.encode().ok()
}

/// A chromaticity's x and y, as PNG's `cHRM` gives them.
pub(crate) type Xy = (f32, f32);

/// The white point and the red, green and blue primaries.
pub(crate) type Chromaticities = [Xy; 4];

/// A profile for PNG's `gAMA` and `cHRM`, or `None` when neither says
/// anything usable. Without one of them, the other takes sRGB's.
pub(crate) fn from_gamma_and_chromaticities(
    gamma: Option<f32>,
    chromaticities: Option<Chromaticities>,
) -> Option<Vec<u8>> {
    // A file's gamma is its encoding exponent, so 0.45455 is a curve of 2.2.
    let gamma = gamma.filter(|gamma| (0.05..=5.0).contains(gamma));
    let chromaticities = chromaticities.filter(|&c| valid(c));
    if gamma.is_none() && chromaticities.is_none() {
        return None;
    }
    let mut profile = ColorProfile::new_srgb();
    // Its code points would be used in place of the curve set here.
    profile.cicp = None;
    if let Some([white, red, green, blue]) = chromaticities {
        let point = |(x, y): Xy| Chromaticity { x, y };
        profile.update_rgb_colorimetry(
            point(white).to_xyyb(),
            ColorPrimaries {
                red: point(red),
                green: point(green),
                blue: point(blue),
            },
        );
    }
    if let Some(gamma) = gamma {
        let curve = ToneReprCurve::Parametric(vec![1.0 / gamma]);
        profile.red_trc = Some(curve.clone());
        profile.green_trc = Some(curve.clone());
        profile.blue_trc = Some(curve);
    }
    profile.encode().ok()
}

/// Whether each chromaticity is a real one, since y is divided by, and the
/// primaries make a triangle, since a line has no inverse.
fn valid(chromaticities: Chromaticities) -> bool {
    let [_, red, green, blue] = chromaticities;
    let area = (green.0 - red.0) * (blue.1 - red.1) - (green.1 - red.1) * (blue.0 - red.0);
    area.abs() > 1e-3
        && chromaticities
            .iter()
            .all(|&(x, y)| x.is_finite() && y.is_finite() && x >= 0.0 && y > 0.0 && x + y <= 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raster::Rgba8;

    /// A pixel's values, converted from the description to the working space.
    fn converted(icc: &[u8], color: [u8; 3]) -> [f32; 3] {
        let image = Rgba8::new(1, 1, vec![color[0], color[1], color[2], 255]);
        let pixels = image.to_linear_p3(Some(icc)).pixels;
        [pixels[0].to_f32(), pixels[1].to_f32(), pixels[2].to_f32()]
    }

    fn close(found: [f32; 3], expected: [f32; 3]) {
        for (found, expected) in found.iter().zip(expected) {
            assert!((found - expected).abs() < 0.002, "{found} for {expected}");
        }
    }

    #[test]
    fn code_points_for_srgb_need_no_profile() {
        assert_eq!(from_code_points(1, 13), None);
        assert_eq!(from_code_points(2, 2), None);
        assert_eq!(from_code_points(1, 2), None);
        assert_eq!(from_code_points(2, 13), None);
    }

    #[test]
    fn code_points_that_mean_nothing_are_taken_as_srgb() {
        assert_eq!(from_code_points(200, 13), None);
        assert_eq!(from_code_points(9, 200), None);
    }

    #[test]
    fn bt2020_code_points_keep_colors_outside_display_p3() {
        let icc = from_code_points(9, 13).unwrap();
        // BT.2020's green, on the sRGB curve, worked out from the primaries.
        close(converted(&icc, [0, 255, 0]), crate::raster::BT2020_GREEN);
    }

    #[test]
    fn the_transfer_code_point_is_honored() {
        // Linear: 128 is 128 / 255, not the sRGB curve's 0.2159.
        let linear = from_code_points(1, 8).unwrap();
        let half = 128.0 / 255.0;
        close(converted(&linear, [128, 128, 128]), [half, half, half]);
        // BT.709's curve: 128 is ((128 / 255 + 0.099) / 1.099) ^ (1 / 0.45).
        let bt709 = from_code_points(1, 1).unwrap();
        let curved = ((half + 0.099) / 1.099_f32).powf(1.0 / 0.45);
        close(converted(&bt709, [128, 128, 128]), [curved, curved, curved]);
    }

    #[test]
    fn hdr_transfers_convert_without_failing() {
        for transfer in [16, 18] {
            let icc = from_code_points(9, transfer).unwrap();
            assert!(
                converted(&icc, [128, 128, 128])
                    .iter()
                    .all(|v| v.is_finite())
            );
        }
    }

    /// sRGB's primaries and white, as a PNG's `cHRM` gives them.
    const SRGB_CHROMATICITIES: Chromaticities =
        [(0.3127, 0.329), (0.64, 0.33), (0.3, 0.6), (0.15, 0.06)];

    #[test]
    fn a_png_gamma_sets_the_curve() {
        let icc = from_gamma_and_chromaticities(Some(1.0), None).unwrap();
        let half = 128.0 / 255.0;
        close(converted(&icc, [128, 128, 128]), [half, half, half]);
        let icc = from_gamma_and_chromaticities(Some(0.45455), None).unwrap();
        let curved = (128.0 / 255.0_f32).powf(2.2);
        close(converted(&icc, [128, 128, 128]), [curved, curved, curved]);
    }

    #[test]
    fn png_chromaticities_set_the_primaries() {
        // Adobe RGB's, on sRGB's curve.
        let adobe = [(0.3127, 0.329), (0.64, 0.33), (0.21, 0.71), (0.15, 0.06)];
        let icc = from_gamma_and_chromaticities(None, Some(adobe)).unwrap();
        close(converted(&icc, [0, 255, 0]), crate::raster::ADOBE_GREEN);
        // sRGB's own, on a gamma of 1: green is as `srgb_green` below.
        let icc = from_gamma_and_chromaticities(Some(1.0), Some(SRGB_CHROMATICITIES)).unwrap();
        close(converted(&icc, [0, 255, 0]), crate::raster::SRGB_GREEN);
    }

    #[test]
    fn png_color_chunks_that_make_no_sense_are_ignored() {
        assert_eq!(from_gamma_and_chromaticities(None, None), None);
        assert_eq!(from_gamma_and_chromaticities(Some(0.0), None), None);
        assert_eq!(from_gamma_and_chromaticities(Some(-1.0), None), None);
        // A y of zero would divide by zero; primaries in a line have no inverse.
        let zero = [(0.3, 0.3), (0.64, 0.0), (0.3, 0.6), (0.15, 0.06)];
        assert_eq!(from_gamma_and_chromaticities(None, Some(zero)), None);
        let line = [(0.3, 0.3), (0.2, 0.2), (0.4, 0.4), (0.1, 0.1)];
        assert_eq!(from_gamma_and_chromaticities(None, Some(line)), None);
    }
}
