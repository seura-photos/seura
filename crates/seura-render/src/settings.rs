//! The settings a render uses, taken from an edit file, and the uniform
//! block that carries them to the shaders.

use bytemuck::{Pod, Zeroable};
use seura_edit::{CoreOperation, EditDocument};

use crate::Output;

/// Every core operation's parameters but red-eye's regions, each neutral by
/// default (technical design, section 6). The shaders skip an operation
/// whose parameters are neutral, so an operation missing from the edit file
/// costs nothing. Red-eye's regions are a list, which slice 7 of M4 gives a
/// buffer of its own.
///
/// So far only exposure is rendered; the other operations arrive with the
/// Editor's tools in M4, each as one function in the shaders.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// `seura.orient`: clockwise, 0, 90, 180, or 270 degrees, then flips.
    pub rotate: u16,
    pub flip_h: bool,
    pub flip_v: bool,
    /// `seura.straighten`'s angle, in degrees, clockwise positive.
    pub straighten: f32,
    /// `seura.crop`'s rectangle, as fractions of the straightened photo.
    pub crop_x: f32,
    pub crop_y: f32,
    pub crop_width: f32,
    pub crop_height: f32,
    /// `seura.exposure`'s `ev`, in stops.
    pub exposure_ev: f32,
    /// `seura.whiteBalance`, −100 to 100.
    pub temperature: f32,
    pub tint: f32,
    /// `seura.highlights`, `seura.shadows`, `seura.contrast`, and
    /// `seura.saturation`'s amounts, −100 to 100.
    pub highlights: f32,
    pub shadows: f32,
    pub contrast: f32,
    pub saturation: f32,
    /// `seura.blackAndWhite` and `seura.sepia`'s strengths, 0 to 100.
    pub black_and_white: f32,
    pub sepia: f32,
    /// `seura.vignette`: amount −100 to 100, size 0 to 100.
    pub vignette_amount: f32,
    pub vignette_size: f32,
}

impl Settings {
    /// Nothing changed: the photo as it is.
    pub const NEUTRAL: Settings = Settings {
        rotate: 0,
        flip_h: false,
        flip_v: false,
        straighten: 0.0,
        crop_x: 0.0,
        crop_y: 0.0,
        crop_width: 1.0,
        crop_height: 1.0,
        exposure_ev: 0.0,
        temperature: 0.0,
        tint: 0.0,
        highlights: 0.0,
        shadows: 0.0,
        contrast: 0.0,
        saturation: 0.0,
        black_and_white: 0.0,
        sepia: 0.0,
        // Without an amount, the size changes nothing.
        vignette_amount: 0.0,
        vignette_size: 0.0,
    };

    /// Disabled operations are skipped, as are ones Seura can't render
    /// (edit format: "Operations").
    pub fn from_document(document: &EditDocument) -> Settings {
        let mut settings = Settings::NEUTRAL;
        for operation in document.operations() {
            if !operation.enabled {
                continue;
            }
            let Some(core) = &operation.core else {
                continue;
            };
            match core {
                CoreOperation::Orient(orient) => {
                    settings.rotate = orient.rotate;
                    settings.flip_h = orient.flip_h;
                    settings.flip_v = orient.flip_v;
                }
                CoreOperation::Straighten(straighten) => {
                    settings.straighten = straighten.angle as f32;
                }
                CoreOperation::Crop(crop) => {
                    settings.crop_x = crop.x as f32;
                    settings.crop_y = crop.y as f32;
                    settings.crop_width = crop.width as f32;
                    settings.crop_height = crop.height as f32;
                }
                // Its regions arrive with red-eye, in M4 slice 7.
                CoreOperation::RedEye(_) => {}
                CoreOperation::Exposure(exposure) => settings.exposure_ev = exposure.ev as f32,
                CoreOperation::WhiteBalance(balance) => {
                    settings.temperature = balance.temperature as f32;
                    settings.tint = balance.tint as f32;
                }
                CoreOperation::Highlights(amount) => settings.highlights = amount.amount as f32,
                CoreOperation::Shadows(amount) => settings.shadows = amount.amount as f32,
                CoreOperation::Contrast(amount) => settings.contrast = amount.amount as f32,
                CoreOperation::Saturation(amount) => settings.saturation = amount.amount as f32,
                CoreOperation::BlackAndWhite(strength) => {
                    settings.black_and_white = strength.strength as f32;
                }
                CoreOperation::Sepia(strength) => settings.sepia = strength.strength as f32,
                CoreOperation::Vignette(vignette) => {
                    settings.vignette_amount = vignette.amount as f32;
                    settings.vignette_size = vignette.size as f32;
                }
            }
        }
        settings
    }

    /// The shaders' `Params` for these settings, encoded for `output`.
    pub(crate) fn uniforms(&self, output: Output) -> Params {
        Params {
            output_srgb: u32::from(output == Output::Srgb),
            rotate: u32::from(self.rotate),
            flip_h: u32::from(self.flip_h),
            flip_v: u32::from(self.flip_v),
            straighten: self.straighten,
            crop_x: self.crop_x,
            crop_y: self.crop_y,
            crop_width: self.crop_width,
            crop_height: self.crop_height,
            exposure_ev: self.exposure_ev,
            temperature: self.temperature,
            tint: self.tint,
            highlights: self.highlights,
            shadows: self.shadows,
            contrast: self.contrast,
            saturation: self.saturation,
            black_and_white: self.black_and_white,
            sepia: self.sepia,
            vignette_amount: self.vignette_amount,
            vignette_size: self.vignette_size,
        }
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings::NEUTRAL
    }
}

/// The shaders' `Params` (`shaders/main.wgsl`), field for field. A test
/// checks this layout against naga's reading of the WGSL; the preview's is
/// generated from it (`ui/src/preview/generated/params.json`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct Params {
    pub output_srgb: u32,
    pub rotate: u32,
    pub flip_h: u32,
    pub flip_v: u32,
    pub straighten: f32,
    pub crop_x: f32,
    pub crop_y: f32,
    pub crop_width: f32,
    pub crop_height: f32,
    pub exposure_ev: f32,
    pub temperature: f32,
    pub tint: f32,
    pub highlights: f32,
    pub shadows: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub black_and_white: f32,
    pub sepia: f32,
    pub vignette_amount: f32,
    pub vignette_size: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(operations: &str) -> EditDocument {
        let json = format!(
            r#"{{"format": "seura-edit", "formatVersion": "1.0",
                "photo": {{"file": "a.jpg", "size": 1}}, "operations": {operations}}}"#
        );
        EditDocument::parse(json.as_bytes()).unwrap()
    }

    #[test]
    fn exposure_comes_from_the_file() {
        let doc = document(r#"[{"id": "seura.exposure", "version": 1, "params": {"ev": 0.5}}]"#);
        assert_eq!(Settings::from_document(&doc).exposure_ev, 0.5);
    }

    #[test]
    fn every_core_operation_s_parameters_come_from_the_file() {
        let doc = document(
            r#"[
            {"id": "seura.orient", "version": 1, "params": {"rotate": 90, "flipH": true, "flipV": false}},
            {"id": "seura.straighten", "version": 1, "params": {"angle": -2.5}},
            {"id": "seura.crop", "version": 1, "params": {"x": 0.1, "y": 0.2, "width": 0.5, "height": 0.6, "aspect": "4:3"}},
            {"id": "seura.exposure", "version": 1, "params": {"ev": 0.5}},
            {"id": "seura.whiteBalance", "version": 1, "params": {"temperature": 10, "tint": -5}},
            {"id": "seura.highlights", "version": 1, "params": {"amount": -30}},
            {"id": "seura.shadows", "version": 1, "params": {"amount": 40}},
            {"id": "seura.contrast", "version": 1, "params": {"amount": 15}},
            {"id": "seura.saturation", "version": 1, "params": {"amount": -100}},
            {"id": "seura.blackAndWhite", "version": 1, "params": {"strength": 100}},
            {"id": "seura.sepia", "version": 1, "params": {"strength": 50}},
            {"id": "seura.vignette", "version": 1, "params": {"amount": -20, "size": 70}}
            ]"#,
        );
        assert_eq!(
            Settings::from_document(&doc),
            Settings {
                rotate: 90,
                flip_h: true,
                flip_v: false,
                straighten: -2.5,
                crop_x: 0.1,
                crop_y: 0.2,
                crop_width: 0.5,
                crop_height: 0.6,
                exposure_ev: 0.5,
                temperature: 10.0,
                tint: -5.0,
                highlights: -30.0,
                shadows: 40.0,
                contrast: 15.0,
                saturation: -100.0,
                black_and_white: 100.0,
                sepia: 50.0,
                vignette_amount: -20.0,
                vignette_size: 70.0,
            }
        );
    }

    #[test]
    fn a_disabled_operation_is_skipped() {
        let doc = document(
            r#"[{"id": "seura.exposure", "version": 1, "enabled": false, "params": {"ev": 0.5}}]"#,
        );
        assert_eq!(Settings::from_document(&doc), Settings::NEUTRAL);
    }

    #[test]
    fn params_is_laid_out_as_the_shader_s() {
        let module = naga::front::wgsl::parse_str(crate::SHADER).unwrap();
        let (members, span) = module
            .types
            .iter()
            .find_map(|(_, ty)| match &ty.inner {
                naga::TypeInner::Struct { members, span }
                    if ty.name.as_deref() == Some("Params") =>
                {
                    Some((members.clone(), *span))
                }
                _ => None,
            })
            .unwrap();
        macro_rules! offsets {
            ($($field:ident),*) => {
                vec![$((stringify!($field), std::mem::offset_of!(Params, $field) as u32)),*]
            };
        }
        let ours = offsets!(
            output_srgb,
            rotate,
            flip_h,
            flip_v,
            straighten,
            crop_x,
            crop_y,
            crop_width,
            crop_height,
            exposure_ev,
            temperature,
            tint,
            highlights,
            shadows,
            contrast,
            saturation,
            black_and_white,
            sepia,
            vignette_amount,
            vignette_size
        );
        let theirs: Vec<(&str, u32)> = members
            .iter()
            .map(|member| (member.name.as_deref().unwrap(), member.offset))
            .collect();
        assert_eq!(ours, theirs);
        assert_eq!(size_of::<Params>() as u32, span);
    }

    #[test]
    fn neutral_is_the_whole_photo_unchanged() {
        let params = Settings::NEUTRAL.uniforms(Output::DisplayP3);
        assert_eq!(params.output_srgb, 0);
        assert_eq!(
            (params.crop_width, params.crop_height, params.exposure_ev),
            (1.0, 1.0, 0.0)
        );
        assert_eq!(Settings::NEUTRAL.uniforms(Output::Srgb).output_srgb, 1);
    }
}
