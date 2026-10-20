//! Render stages, the core operations of format 1.0, and canonical order
//! (edit format: "Operations" and "Core operations").

use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

/// The four render stages, in render order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stage {
    Geometry,
    Retouch,
    Tone,
    Effects,
}

impl Stage {
    /// The stage named by a non-core operation's `group`.
    pub fn from_group(group: &str) -> Option<Stage> {
        match group {
            "geometry" => Some(Stage::Geometry),
            "retouch" => Some(Stage::Retouch),
            "tone" => Some(Stage::Tone),
            "effects" => Some(Stage::Effects),
            _ => None,
        }
    }
}

/// The core operations of format 1.0, in canonical order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CoreOpId {
    Orient,
    Straighten,
    Crop,
    RedEye,
    Exposure,
    WhiteBalance,
    Highlights,
    Shadows,
    Contrast,
    Saturation,
    BlackAndWhite,
    Sepia,
    Vignette,
}

impl CoreOpId {
    /// Every core operation, in canonical render order.
    pub const ALL: [CoreOpId; 13] = [
        CoreOpId::Orient,
        CoreOpId::Straighten,
        CoreOpId::Crop,
        CoreOpId::RedEye,
        CoreOpId::Exposure,
        CoreOpId::WhiteBalance,
        CoreOpId::Highlights,
        CoreOpId::Shadows,
        CoreOpId::Contrast,
        CoreOpId::Saturation,
        CoreOpId::BlackAndWhite,
        CoreOpId::Sepia,
        CoreOpId::Vignette,
    ];

    /// The operation's `id` in the file.
    pub fn id(self) -> &'static str {
        match self {
            CoreOpId::Orient => "seura.orient",
            CoreOpId::Straighten => "seura.straighten",
            CoreOpId::Crop => "seura.crop",
            CoreOpId::RedEye => "seura.redEye",
            CoreOpId::Exposure => "seura.exposure",
            CoreOpId::WhiteBalance => "seura.whiteBalance",
            CoreOpId::Highlights => "seura.highlights",
            CoreOpId::Shadows => "seura.shadows",
            CoreOpId::Contrast => "seura.contrast",
            CoreOpId::Saturation => "seura.saturation",
            CoreOpId::BlackAndWhite => "seura.blackAndWhite",
            CoreOpId::Sepia => "seura.sepia",
            CoreOpId::Vignette => "seura.vignette",
        }
    }

    pub fn from_id(id: &str) -> Option<CoreOpId> {
        CoreOpId::ALL.into_iter().find(|op| op.id() == id)
    }

    pub fn stage(self) -> Stage {
        match self {
            CoreOpId::Orient | CoreOpId::Straighten | CoreOpId::Crop => Stage::Geometry,
            CoreOpId::RedEye => Stage::Retouch,
            CoreOpId::Exposure
            | CoreOpId::WhiteBalance
            | CoreOpId::Highlights
            | CoreOpId::Shadows
            | CoreOpId::Contrast
            | CoreOpId::Saturation => Stage::Tone,
            CoreOpId::BlackAndWhite | CoreOpId::Sepia | CoreOpId::Vignette => Stage::Effects,
        }
    }

    fn rank(self) -> usize {
        CoreOpId::ALL
            .iter()
            .position(|op| *op == self)
            .unwrap_or(usize::MAX)
    }
}

/// The only version of each core operation this crate can render and edit.
pub const CORE_VERSION: u64 = 1;

/// A core operation's settings, version 1. Unknown params make an entry
/// unknown rather than being dropped (`deny_unknown_fields`).
#[derive(Clone, Debug, PartialEq)]
pub enum CoreOperation {
    Orient(Orient),
    Straighten(Straighten),
    Crop(Crop),
    RedEye(RedEye),
    Exposure(Exposure),
    WhiteBalance(WhiteBalance),
    /// A neighborhood operation (edit format: "Neighborhood operations").
    Highlights(Amount),
    /// A neighborhood operation (edit format: "Neighborhood operations").
    Shadows(Amount),
    Contrast(Amount),
    Saturation(Amount),
    BlackAndWhite(Strength),
    Sepia(Strength),
    Vignette(Vignette),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Orient {
    pub rotate: u16,
    pub flip_h: bool,
    pub flip_v: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Straighten {
    #[serde(serialize_with = "number")]
    pub angle: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Crop {
    #[serde(serialize_with = "number")]
    pub x: f64,
    #[serde(serialize_with = "number")]
    pub y: f64,
    #[serde(serialize_with = "number")]
    pub width: f64,
    #[serde(serialize_with = "number")]
    pub height: f64,
    /// The user's ratio lock, such as `"4:3"`; editing only, not rendering.
    #[serde(default)]
    pub aspect: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedEye {
    pub regions: Vec<RedEyeRegion>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedEyeRegion {
    #[serde(serialize_with = "number")]
    pub x: f64,
    #[serde(serialize_with = "number")]
    pub y: f64,
    #[serde(serialize_with = "number")]
    pub radius: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exposure {
    #[serde(serialize_with = "number")]
    pub ev: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WhiteBalance {
    #[serde(serialize_with = "number")]
    pub temperature: f64,
    #[serde(serialize_with = "number")]
    pub tint: f64,
}

/// Highlights, shadows, contrast, and saturation: `amount`, −100 to 100.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Amount {
    #[serde(serialize_with = "number")]
    pub amount: f64,
}

/// Black and white, and sepia: `strength`, 0 to 100.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Strength {
    #[serde(serialize_with = "number")]
    pub strength: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vignette {
    #[serde(serialize_with = "number")]
    pub amount: f64,
    #[serde(serialize_with = "number")]
    pub size: f64,
}

impl CoreOperation {
    pub fn id(&self) -> CoreOpId {
        match self {
            CoreOperation::Orient(_) => CoreOpId::Orient,
            CoreOperation::Straighten(_) => CoreOpId::Straighten,
            CoreOperation::Crop(_) => CoreOpId::Crop,
            CoreOperation::RedEye(_) => CoreOpId::RedEye,
            CoreOperation::Exposure(_) => CoreOpId::Exposure,
            CoreOperation::WhiteBalance(_) => CoreOpId::WhiteBalance,
            CoreOperation::Highlights(_) => CoreOpId::Highlights,
            CoreOperation::Shadows(_) => CoreOpId::Shadows,
            CoreOperation::Contrast(_) => CoreOpId::Contrast,
            CoreOperation::Saturation(_) => CoreOpId::Saturation,
            CoreOperation::BlackAndWhite(_) => CoreOpId::BlackAndWhite,
            CoreOperation::Sepia(_) => CoreOpId::Sepia,
            CoreOperation::Vignette(_) => CoreOpId::Vignette,
        }
    }

    /// Reads an operation entry's params, if it is a core operation at the
    /// version this crate supports and its params are the ones defined for it.
    pub fn from_params(id: CoreOpId, version: u64, params: &Value) -> Option<CoreOperation> {
        if version != CORE_VERSION {
            return None;
        }
        let params = params.clone();
        let op = match id {
            CoreOpId::Orient => CoreOperation::Orient(serde_json::from_value(params).ok()?),
            CoreOpId::Straighten => CoreOperation::Straighten(serde_json::from_value(params).ok()?),
            CoreOpId::Crop => CoreOperation::Crop(serde_json::from_value(params).ok()?),
            CoreOpId::RedEye => CoreOperation::RedEye(serde_json::from_value(params).ok()?),
            CoreOpId::Exposure => CoreOperation::Exposure(serde_json::from_value(params).ok()?),
            CoreOpId::WhiteBalance => {
                CoreOperation::WhiteBalance(serde_json::from_value(params).ok()?)
            }
            CoreOpId::Highlights => CoreOperation::Highlights(serde_json::from_value(params).ok()?),
            CoreOpId::Shadows => CoreOperation::Shadows(serde_json::from_value(params).ok()?),
            CoreOpId::Contrast => CoreOperation::Contrast(serde_json::from_value(params).ok()?),
            CoreOpId::Saturation => CoreOperation::Saturation(serde_json::from_value(params).ok()?),
            CoreOpId::BlackAndWhite => {
                CoreOperation::BlackAndWhite(serde_json::from_value(params).ok()?)
            }
            CoreOpId::Sepia => CoreOperation::Sepia(serde_json::from_value(params).ok()?),
            CoreOpId::Vignette => CoreOperation::Vignette(serde_json::from_value(params).ok()?),
        };
        Some(op)
    }

    /// The params as written to the file: numbers rounded to four decimals,
    /// whole numbers without a decimal point.
    pub fn to_params(&self) -> Value {
        let value = match self {
            CoreOperation::Orient(p) => serde_json::to_value(p),
            CoreOperation::Straighten(p) => serde_json::to_value(p),
            CoreOperation::Crop(p) => serde_json::to_value(p),
            CoreOperation::RedEye(p) => serde_json::to_value(p),
            CoreOperation::Exposure(p) => serde_json::to_value(p),
            CoreOperation::WhiteBalance(p) => serde_json::to_value(p),
            CoreOperation::Highlights(p)
            | CoreOperation::Shadows(p)
            | CoreOperation::Contrast(p)
            | CoreOperation::Saturation(p) => serde_json::to_value(p),
            CoreOperation::BlackAndWhite(p) | CoreOperation::Sepia(p) => serde_json::to_value(p),
            CoreOperation::Vignette(p) => serde_json::to_value(p),
        };
        value.expect("operation params always serialize")
    }

    /// Whether the settings leave the photo unchanged. Writers remove neutral
    /// operations rather than store them (edit format: "Core operations").
    pub fn is_neutral(&self) -> bool {
        match self {
            CoreOperation::Orient(p) => p.rotate == 0 && !p.flip_h && !p.flip_v,
            CoreOperation::Straighten(p) => p.angle == 0.0,
            // A full-frame crop with a ratio lock still records the user's choice.
            CoreOperation::Crop(p) => {
                p.x == 0.0 && p.y == 0.0 && p.width == 1.0 && p.height == 1.0 && p.aspect.is_none()
            }
            CoreOperation::RedEye(p) => p.regions.is_empty(),
            CoreOperation::Exposure(p) => p.ev == 0.0,
            CoreOperation::WhiteBalance(p) => p.temperature == 0.0 && p.tint == 0.0,
            CoreOperation::Highlights(p)
            | CoreOperation::Shadows(p)
            | CoreOperation::Contrast(p)
            | CoreOperation::Saturation(p) => p.amount == 0.0,
            CoreOperation::BlackAndWhite(p) | CoreOperation::Sepia(p) => p.strength == 0.0,
            CoreOperation::Vignette(p) => p.amount == 0.0,
        }
    }
}

/// Rounds to four decimal places and writes whole numbers as integers, so
/// files stay readable and stable (edit format: "Rules for readers and writers").
fn number<S: Serializer>(value: &f64, serializer: S) -> Result<S::Ok, S::Error> {
    let rounded = round4(*value);
    if rounded.fract() == 0.0 && rounded.abs() < 1e15 {
        serializer.serialize_i64(rounded as i64)
    } else {
        serializer.serialize_f64(rounded)
    }
}

pub(crate) fn round4(value: f64) -> f64 {
    let rounded = (value * 10_000.0).round() / 10_000.0;
    // Avoid writing -0.
    if rounded == 0.0 { 0.0 } else { rounded }
}

/// Where an operation entry belongs in canonical order: by stage, core
/// operations first in their fixed order, then non-core ones. `None` for a
/// core-looking id this version doesn't define (from a newer minor version),
/// whose place is unknown.
pub(crate) fn order_key(entry: &Value) -> Option<(usize, u8, usize)> {
    let id = entry.get("id").and_then(Value::as_str).unwrap_or("");
    if let Some(core) = CoreOpId::from_id(id) {
        return Some((core.stage() as usize, 0, core.rank()));
    }
    if id.starts_with("seura.") {
        return None;
    }
    let stage = entry
        .get("group")
        .and_then(Value::as_str)
        .and_then(Stage::from_group)
        .map_or(4, |stage| stage as usize);
    Some((stage, 1, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn numbers_are_rounded_and_whole_numbers_stay_integers() {
        let op = CoreOperation::Contrast(Amount { amount: 35.0 });
        assert_eq!(op.to_params(), json!({ "amount": 35 }));
        let op = CoreOperation::Exposure(Exposure { ev: 0.123456 });
        assert_eq!(op.to_params(), json!({ "ev": 0.1235 }));
        let op = CoreOperation::Exposure(Exposure { ev: -0.00001 });
        assert_eq!(op.to_params(), json!({ "ev": 0 }));
    }

    #[test]
    fn unknown_params_or_versions_are_not_core() {
        let params = json!({ "ev": 1, "gamma": 2 });
        assert!(CoreOperation::from_params(CoreOpId::Exposure, 1, &params).is_none());
        let params = json!({ "ev": 1 });
        assert!(CoreOperation::from_params(CoreOpId::Exposure, 2, &params).is_none());
        assert!(CoreOperation::from_params(CoreOpId::Exposure, 1, &params).is_some());
    }

    #[test]
    fn canonical_order_follows_the_spec_table() {
        let ids: Vec<_> = CoreOpId::ALL.iter().map(|op| op.id()).collect();
        assert_eq!(
            ids,
            [
                "seura.orient",
                "seura.straighten",
                "seura.crop",
                "seura.redEye",
                "seura.exposure",
                "seura.whiteBalance",
                "seura.highlights",
                "seura.shadows",
                "seura.contrast",
                "seura.saturation",
                "seura.blackAndWhite",
                "seura.sepia",
                "seura.vignette",
            ]
        );
        let plugin = json!({ "id": "com.example.grain", "group": "tone" });
        let saturation = json!({ "id": "seura.saturation" });
        let black_and_white = json!({ "id": "seura.blackAndWhite" });
        assert!(order_key(&saturation) < order_key(&plugin));
        assert!(order_key(&plugin) < order_key(&black_and_white));
    }

    #[test]
    fn highlights_and_shadows_are_tone_operations_with_an_amount() {
        for id in [CoreOpId::Highlights, CoreOpId::Shadows] {
            assert_eq!(id.stage(), Stage::Tone);
            let op = CoreOperation::from_params(id, 1, &json!({ "amount": -35.5 })).unwrap();
            assert_eq!(op.id(), id);
            assert_eq!(op.to_params(), json!({ "amount": -35.5 }));
            assert!(!op.is_neutral());
            let neutral = CoreOperation::from_params(id, 1, &json!({ "amount": 0 })).unwrap();
            assert!(neutral.is_neutral());
        }
        let shadows = json!({ "id": "seura.shadows" });
        assert!(order_key(&json!({ "id": "seura.whiteBalance" })) < order_key(&shadows));
        assert!(order_key(&json!({ "id": "seura.highlights" })) < order_key(&shadows));
        assert!(order_key(&shadows) < order_key(&json!({ "id": "seura.contrast" })));
    }
}
