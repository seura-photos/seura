//! The settings a render uses, taken from an edit file, and the uniform
//! block that carries them to the shaders.

use bytemuck::{Pod, Zeroable};
use seura_edit::{CoreOperation, EditDocument, Operation};

use crate::Output;
use crate::geometry::Geometry;

/// Every core operation's parameters but red-eye's regions, each neutral by
/// default (technical design, section 6). The shaders skip an operation
/// whose parameters are neutral, so an operation missing from the edit file
/// costs nothing. Red-eye's regions are a list, which slice 7 of M4 gives a
/// buffer of its own.
///
/// Operations arrive with the Editor's tools in M4, each as one function in
/// the shaders.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// `seura.orient`: clockwise, 0, 90, 180, or 270 degrees, then flips.
    pub rotate: u16,
    pub flip_h: bool,
    pub flip_v: bool,
    /// `seura.straighten`'s angle, in degrees, clockwise positive.
    pub straighten: f32,
    /// `seura.crop`'s rectangle, as fractions of the oriented photo, as
    /// the file has them: rounding them to pixels needs every digit.
    pub crop_x: f64,
    pub crop_y: f64,
    pub crop_width: f64,
    pub crop_height: f64,
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
        Settings::from_operations(
            document
                .operations()
                .iter()
                .filter(|operation| operation.enabled)
                .filter_map(|operation: &Operation| operation.core.as_ref()),
        )
    }

    /// The settings these core operations make, the rest neutral.
    pub fn from_operations<'a>(
        operations: impl IntoIterator<Item = &'a CoreOperation>,
    ) -> Settings {
        let mut settings = Settings::NEUTRAL;
        for core in operations {
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
                    settings.crop_x = crop.x;
                    settings.crop_y = crop.y;
                    settings.crop_width = crop.width;
                    settings.crop_height = crop.height;
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

    /// Whether `seura.orient` turns the photo on its side, so its width and
    /// height trade places.
    pub fn sideways(&self) -> bool {
        self.rotate % 180 == 90
    }

    /// The oriented photo's size (edit format, "Geometry frame and
    /// sampling"), for a photo of `width` × `height` upright: turned by
    /// `seura.orient`. Straighten and crop are measured in it.
    pub fn oriented_size(&self, width: u32, height: u32) -> (u32, u32) {
        if self.sideways() {
            (height, width)
        } else {
            (width, height)
        }
    }

    /// The size of the photo drawn with these settings, for a photo of
    /// `width` × `height` upright: its crop, in whole pixels of the oriented
    /// photo.
    pub fn output_size(&self, width: u32, height: u32) -> (u32, u32) {
        Geometry::new(self, width, height).output_size()
    }

    /// The shaders' `Params` for these settings, encoded for `output`, for
    /// one pass, over `draw`.
    pub(crate) fn uniforms(&self, output: Output, pass: Pass, draw: &Draw) -> Params {
        let crop = Geometry::new(self, draw.full.0, draw.full.1).crop();
        Params {
            output_srgb: u32::from(output == Output::Srgb),
            rotate: u32::from(self.rotate),
            flip_h: u32::from(self.flip_h),
            flip_v: u32::from(self.flip_v),
            straighten: self.straighten,
            crop_x: self.crop_x as f32,
            crop_y: self.crop_y as f32,
            crop_width: self.crop_width as f32,
            crop_height: self.crop_height as f32,
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
            draw_pass: pass as u32,
            tile_x: draw.part[0],
            tile_y: draw.part[1],
            tile_width: draw.part[2],
            tile_height: draw.part[3],
            full_width: draw.full.0 as f32,
            full_height: draw.full.1 as f32,
            view_x: draw.view[0],
            view_y: draw.view[1],
            view_width: draw.view[2],
            view_height: draw.view[3],
            crop_left: crop.left as f32,
            crop_top: crop.top as f32,
            crop_pixels_width: crop.width as f32,
            crop_pixels_height: crop.height as f32,
        }
    }
}

/// What one draw covers: the part of the upright photo its photo texture
/// holds, and the part of the result it draws, each as x, y, width and
/// height in fractions; and the upright photo's full size in pixels, which
/// geometry is measured in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Draw {
    pub full: (u32, u32),
    pub part: [f32; 4],
    pub view: [f32; 4],
}

impl Draw {
    pub const ALL: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

    /// All of a photo of this size, drawn whole.
    pub fn whole(full: (u32, u32)) -> Draw {
        Draw {
            full,
            part: Draw::ALL,
            view: Draw::ALL,
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
    pub draw_pass: u32,
    pub tile_x: f32,
    pub tile_y: f32,
    pub tile_width: f32,
    pub tile_height: f32,
    pub full_width: f32,
    pub full_height: f32,
    pub view_x: f32,
    pub view_y: f32,
    pub view_width: f32,
    pub view_height: f32,
    pub crop_left: f32,
    pub crop_top: f32,
    pub crop_pixels_width: f32,
    pub crop_pixels_height: f32,
}

/// What a draw does (`Params`' `draw_pass`): draw the photo, or one of the
/// luminance guide's passes (`shaders/guide.wgsl`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Pass {
    Draw = 0,
    Luminance = 1,
    Across = 2,
    Coefficients = 3,
    Down = 4,
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
    fn a_photo_turned_on_its_side_trades_width_and_height() {
        let turned = |rotate| Settings {
            rotate,
            ..Settings::NEUTRAL
        };
        assert_eq!(turned(0).output_size(4, 3), (4, 3));
        assert_eq!(turned(90).output_size(4, 3), (3, 4));
        assert_eq!(turned(180).output_size(4, 3), (4, 3));
        assert_eq!(turned(270).output_size(4, 3), (3, 4));
        // Cropped: in whole pixels of the turned photo.
        let cropped = Settings {
            crop_width: 0.5,
            crop_height: 0.3,
            ..turned(90)
        };
        assert_eq!(cropped.output_size(40, 30), (15, 12));
        // Flips keep the size.
        let flipped = Settings {
            flip_h: true,
            flip_v: true,
            ..Settings::NEUTRAL
        };
        assert_eq!(flipped.output_size(4, 3), (4, 3));
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
            vignette_size,
            draw_pass,
            tile_x,
            tile_y,
            tile_width,
            tile_height,
            full_width,
            full_height,
            view_x,
            view_y,
            view_width,
            view_height,
            crop_left,
            crop_top,
            crop_pixels_width,
            crop_pixels_height
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
        let params =
            Settings::NEUTRAL.uniforms(Output::DisplayP3, Pass::Draw, &Draw::whole((4, 3)));
        assert_eq!(params.output_srgb, 0);
        assert_eq!(
            (params.crop_width, params.crop_height, params.exposure_ev),
            (1.0, 1.0, 0.0)
        );
        assert_eq!(
            Settings::NEUTRAL
                .uniforms(Output::Srgb, Pass::Draw, &Draw::whole((4, 3)))
                .output_srgb,
            1
        );
    }
}
