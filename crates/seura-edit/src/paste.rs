//! Copying one photo's edits and pasting them onto others (PRE-1, PRE-2; edit
//! format, "Presets"): what's copied, and each target's action. Each chosen
//! stage replaces the target's; only what the app understands is replaced, so
//! other apps' operations stay on the target and the source's aren't added.

use crate::document::{Action, Edit, EditDocument, Operation};
use crate::operations::{CoreOpId, CoreOperation, Crop, Orient, Stage, Straighten};

/// The label recorded in each edit file's history.
pub const PASTE: &str = "Paste edits";

/// How close two frames' shapes must be to count as the same, as a share of
/// their ratio: closer than the rounding of a crop's edges would notice.
const SAME_SHAPE: f64 = 1e-3;

/// The groups a paste can change (PRE-1). Red-eye is never pasted: it belongs
/// to one photo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PasteGroups {
    /// The tone stage: light and color.
    pub adjustments: bool,
    /// The effects stage.
    pub effects: bool,
    /// The geometry stage: rotate, flip, straighten, and crop.
    pub geometry: bool,
}

/// One photo's edits, as copied.
#[derive(Clone, Debug, PartialEq)]
pub struct CopiedEdits {
    /// The photo's frame: its upright size as its own edits turn it, if known.
    frame: Option<(u32, u32)>,
    operations: Vec<CoreOperation>,
}

impl CopiedEdits {
    /// What can be copied from `document`, or `None` if there's nothing to
    /// copy: operations that are on, except red-eye. `upright` is the photo's
    /// upright size, if it's known.
    pub fn of(document: &EditDocument, upright: Option<(u32, u32)>) -> Option<CopiedEdits> {
        let operations: Vec<CoreOperation> = document
            .operations()
            .into_iter()
            .filter(|operation| operation.enabled)
            .filter_map(|operation| operation.core)
            .filter(|core| core.id() != CoreOpId::RedEye)
            .collect();
        if operations.is_empty() {
            return None;
        }
        let mut copied = CopiedEdits {
            frame: None,
            operations,
        };
        copied.frame = upright.map(|upright| oriented(upright, copied.orient()));
        Some(copied)
    }

    /// Whether anything copied is in `stage`.
    pub fn holds(&self, stage: Stage) -> bool {
        self.operations
            .iter()
            .any(|core| core.id().stage() == stage)
    }

    /// Whether pasting `groups` needs each target's shape: a ratio-locked
    /// crop or a straighten angle.
    pub fn needs_shape(&self, groups: PasteGroups) -> bool {
        groups.geometry && (self.lock().is_some() || self.angle() != 0.0)
    }

    /// A preset named `name` holding the copied stages in `groups` (PRE-3),
    /// as they are: a crop is fitted to each photo the preset is applied to.
    /// `None` if none of those stages holds anything, since such a preset
    /// would change nothing.
    pub fn preset(&self, name: &str, groups: PasteGroups) -> Option<EditDocument> {
        let chosen = |stage: Stage| match stage {
            Stage::Tone => groups.adjustments,
            Stage::Effects => groups.effects,
            Stage::Geometry => groups.geometry,
            Stage::Retouch => false,
        };
        let edits: Vec<Edit> = self
            .operations
            .iter()
            .filter(|core| chosen(core.id().stage()))
            .cloned()
            .map(Edit::Set)
            .collect();
        if edits.is_empty() {
            return None;
        }
        let mut preset = EditDocument::new_preset(name);
        let action = Action {
            label: String::new(),
            edits,
        };
        // A preset records no history, so the time isn't used.
        preset
            .apply(&action, time::OffsetDateTime::UNIX_EPOCH)
            .ok()?;
        Some(preset)
    }

    fn get(&self, id: CoreOpId) -> Option<&CoreOperation> {
        self.operations.iter().find(|core| core.id() == id)
    }

    fn orient(&self) -> Option<&Orient> {
        match self.get(CoreOpId::Orient) {
            Some(CoreOperation::Orient(orient)) => Some(orient),
            _ => None,
        }
    }

    fn angle(&self) -> f64 {
        match self.get(CoreOpId::Straighten) {
            Some(CoreOperation::Straighten(Straighten { angle })) => *angle,
            _ => 0.0,
        }
    }

    fn crop(&self) -> Option<&Crop> {
        match self.get(CoreOpId::Crop) {
            Some(CoreOperation::Crop(crop)) => Some(crop),
            _ => None,
        }
    }

    fn lock(&self) -> Option<f64> {
        self.crop()?.aspect.as_deref().and_then(ratio)
    }

    /// The geometry stage for a target of upright size `upright`, with its
    /// crop made to suit it. `None` if the target's shape is needed and isn't
    /// known, so it keeps the whole stage it has (PRE-1; plan, question 15).
    fn geometry(&self, upright: Option<(u32, u32)>) -> Option<Vec<(CoreOpId, CoreOperation)>> {
        if upright.is_none() && (self.lock().is_some() || self.angle() != 0.0) {
            return None;
        }
        let mut stage: Vec<(CoreOpId, CoreOperation)> = self
            .operations
            .iter()
            .filter(|core| core.id().stage() == Stage::Geometry && core.id() != CoreOpId::Crop)
            .map(|core| (core.id(), core.clone()))
            .collect();
        let frame = upright.map(|upright| oriented(upright, self.orient()));
        let whole = Crop {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
            aspect: None,
        };
        // Straightening without a crop still needs one, to hide the corners.
        let crop = match (self.crop(), self.angle() != 0.0) {
            (Some(crop), _) => Some(crop),
            (None, true) => Some(&whole),
            (None, false) => None,
        };
        if let Some(crop) = crop {
            let crop = match frame {
                Some(frame) => fitted_crop(crop, self.lock(), self.angle(), frame, self.frame),
                None => crop.clone(),
            };
            stage.push((CoreOpId::Crop, CoreOperation::Crop(crop)));
        }
        Some(stage)
    }
}

/// What pasting does to one target.
#[derive(Clone, Debug, PartialEq)]
pub struct Pasted {
    pub action: Action,
    /// The target's shape was needed and isn't known, so its crop and
    /// rotation weren't changed.
    pub kept_geometry: bool,
}

/// The action that pastes `copied`'s `groups` onto a target with the edit
/// file `target` (none if it has none yet) and upright size `upright`, if
/// that's known.
pub fn paste_action(
    copied: &CopiedEdits,
    groups: PasteGroups,
    target: Option<&EditDocument>,
    upright: Option<(u32, u32)>,
) -> Pasted {
    let theirs = target.map(EditDocument::operations).unwrap_or_default();
    let mut edits = Vec::new();
    let mut replace = |id: CoreOpId, wanted: Option<&CoreOperation>| {
        let entry = theirs.iter().find(|operation| operation.id == id.id());
        if let Some(edit) = replacement(id, wanted, entry) {
            edits.push(edit);
        }
    };
    let mut kept_geometry = false;
    if groups.geometry {
        match copied.geometry(upright) {
            Some(stage) => {
                for id in in_stage(Stage::Geometry) {
                    let wanted = stage.iter().find(|(each, _)| *each == id);
                    replace(id, wanted.map(|(_, core)| core));
                }
            }
            None => kept_geometry = true,
        }
    }
    for (chosen, stage) in [
        (groups.adjustments, Stage::Tone),
        (groups.effects, Stage::Effects),
    ] {
        if chosen {
            for id in in_stage(stage) {
                replace(id, copied.get(id));
            }
        }
    }
    Pasted {
        action: Action {
            label: PASTE.into(),
            edits,
        },
        kept_geometry,
    }
}

/// What replacing one operation takes, if anything. An entry this app can't
/// read stays, being another version's (FIL-5). One that's off stays unless
/// there's a value for it: it isn't drawn, and undo would bring it back on,
/// as with revert (EDT-10).
fn replacement(
    id: CoreOpId,
    wanted: Option<&CoreOperation>,
    entry: Option<&Operation>,
) -> Option<Edit> {
    if entry.is_some_and(|entry| entry.core.is_none()) {
        return None;
    }
    match (wanted, entry) {
        (Some(wanted), _) => Some(Edit::Set(wanted.clone())),
        (None, Some(entry)) if entry.enabled => Some(Edit::Remove(id)),
        (None, _) => None,
    }
}

fn in_stage(stage: Stage) -> impl Iterator<Item = CoreOpId> {
    CoreOpId::ALL
        .into_iter()
        .filter(move |id| id.stage() == stage)
}

/// A frame's size once `orient` has turned the photo.
fn oriented((width, height): (u32, u32), orient: Option<&Orient>) -> (u32, u32) {
    match orient {
        Some(orient) if orient.rotate % 180 == 90 => (height, width),
        _ => (width, height),
    }
}

/// A ratio lock, `"a:b"`, as width over height in pixels.
fn ratio(aspect: &str) -> Option<f64> {
    let (a, b) = aspect.split_once(':')?;
    let (a, b): (f64, f64) = (a.trim().parse().ok()?, b.trim().parse().ok()?);
    (a > 0.0 && b > 0.0 && a.is_finite() && b.is_finite()).then_some(a / b)
}

/// A pasted crop on a frame of size `frame`. A ratio lock keeps its ratio, on
/// a frame of another shape: the same center and share of the frame's area,
/// shrunk until it fits and then moved inward only as far as it must (M5 plan,
/// slice 5). A frame of the same shape gets exactly the source's crop, and one
/// with no lock its fractions. If the angle would then show empty corners,
/// the crop shrinks as it does in the Editor (EDT-3).
fn fitted_crop(
    crop: &Crop,
    lock: Option<f64>,
    angle: f64,
    frame: (u32, u32),
    source: Option<(u32, u32)>,
) -> Crop {
    let (across, down) = (f64::from(frame.0.max(1)), f64::from(frame.1.max(1)));
    let same_shape = source.is_some_and(|(width, height)| {
        let theirs = f64::from(width.max(1)) / f64::from(height.max(1));
        ((across / down) / theirs - 1.0).abs() < SAME_SHAPE
    });
    let mut fitted = crop.clone();
    let mut changed = false;
    if let Some(lock) = lock
        && !same_shape
    {
        let share = crop.width * crop.height;
        let width = (share * lock * down / across).sqrt();
        let height = share / width;
        let scale = (1.0 / width).min(1.0 / height).min(1.0);
        let (width, height) = (width * scale, height * scale);
        let (center_x, center_y) = (crop.x + crop.width / 2.0, crop.y + crop.height / 2.0);
        fitted.width = width;
        fitted.height = height;
        fitted.x = (center_x - width / 2.0).clamp(0.0, 1.0 - width);
        fitted.y = (center_y - height / 2.0).clamp(0.0, 1.0 - height);
        changed = true;
    }
    if angle != 0.0 {
        let mut scale = fit_scale(&fitted, frame, angle);
        if scale == 0.0 {
            // Its center has turned off the photo.
            fitted.x = (1.0 - fitted.width) / 2.0;
            fitted.y = (1.0 - fitted.height) / 2.0;
            scale = fit_scale(&fitted, frame, angle);
            changed = true;
        }
        // What the source's Editor fitted and rounded is left as it is.
        if scale < 1.0 - 1e-6 {
            let (center_x, center_y) = (
                fitted.x + fitted.width / 2.0,
                fitted.y + fitted.height / 2.0,
            );
            fitted.width *= scale;
            fitted.height *= scale;
            fitted.x = center_x - fitted.width / 2.0;
            fitted.y = center_y - fitted.height / 2.0;
            changed = true;
        }
    }
    if changed {
        let (x, width) = on_pixels(fitted.x, fitted.width, across);
        let (y, height) = on_pixels(fitted.y, fitted.height, down);
        (fitted.x, fitted.width, fitted.y, fitted.height) = (x, width, y, height);
    }
    fitted
}

/// A crop's edges along one side of `size` pixels, rounded inward to whole
/// pixels, so rendering's rounding can't move an edge outward onto an
/// uncovered corner (formulas appendix, geometry.md). A hair of slack keeps an
/// edge already on a pixel there. Returns the start and length, as fractions.
pub fn on_pixels(start: f64, length: f64, size: f64) -> (f64, f64) {
    let first = ((start.clamp(0.0, 1.0) * size) - 1e-6).ceil().max(0.0);
    let last = (((start + length).clamp(0.0, 1.0) * size) + 1e-6)
        .floor()
        .min(size)
        .max(first + 1.0);
    (first / size, (last - first) / size)
}

/// The largest scale, at most 1, of `crop` about its own center that keeps it
/// on a photo turned `angle` degrees in a frame of size `frame`; 0 if its
/// center isn't on it (formulas appendix, geometry.md, "Keeping the corners
/// out of view").
fn fit_scale(crop: &Crop, frame: (u32, u32), angle: f64) -> f64 {
    let (across, down) = (f64::from(frame.0), f64::from(frame.1));
    let (sin, cos) = angle.to_radians().sin_cos();
    let back = |x: f64, y: f64| (x * cos + y * sin, -x * sin + y * cos);
    let (half_across, half_down) = (across / 2.0, down / 2.0);
    let (ex, ey) = back(
        (crop.x + crop.width / 2.0) * across - half_across,
        (crop.y + crop.height / 2.0) * down - half_down,
    );
    if ex.abs() > half_across || ey.abs() > half_down {
        return 0.0;
    }
    let (a, b) = (crop.width * across / 2.0, crop.height * down / 2.0);
    let mut scale: f64 = 1.0;
    for (sx, sy) in [(-1.0, -1.0), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
        let (gx, gy) = back(sx * a, sy * b);
        for (g, e, limit) in [(gx, ex, half_across), (gy, ey, half_down)] {
            if g != 0.0 {
                scale = scale.min((limit - g.signum() * e) / g.abs());
            }
        }
    }
    scale.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::PhotoIdentity;
    use crate::operations::{Amount, Exposure, RedEye, RedEyeRegion, Strength};
    use serde_json::json;
    use time::OffsetDateTime;

    const ALL: PasteGroups = PasteGroups {
        adjustments: true,
        effects: true,
        geometry: true,
    };
    const DEFAULTS: PasteGroups = PasteGroups {
        adjustments: true,
        effects: true,
        geometry: false,
    };

    fn document(operations: Vec<CoreOperation>) -> EditDocument {
        let mut document = EditDocument::new_for_photo(&PhotoIdentity {
            file: "IMG_1.jpg".into(),
            size: 1,
            sha256: None,
            width: None,
            height: None,
        });
        let action = Action {
            label: "Test".into(),
            edits: operations.into_iter().map(Edit::Set).collect(),
        };
        document.apply(&action, OffsetDateTime::UNIX_EPOCH).unwrap();
        document
    }

    fn exposure(ev: f64) -> CoreOperation {
        CoreOperation::Exposure(Exposure { ev })
    }

    fn sepia() -> CoreOperation {
        CoreOperation::Sepia(Strength { strength: 50.0 })
    }

    fn crop(x: f64, y: f64, width: f64, height: f64, aspect: Option<&str>) -> CoreOperation {
        CoreOperation::Crop(Crop {
            x,
            y,
            width,
            height,
            aspect: aspect.map(String::from),
        })
    }

    /// `copied` from `operations`, pasted onto a photo with `theirs`.
    fn pasted(
        operations: Vec<CoreOperation>,
        source: Option<(u32, u32)>,
        groups: PasteGroups,
        theirs: Vec<CoreOperation>,
        upright: Option<(u32, u32)>,
    ) -> (EditDocument, bool) {
        let copied = CopiedEdits::of(&document(operations), source).unwrap();
        let mut target = document(theirs);
        let Pasted {
            action,
            kept_geometry,
        } = paste_action(&copied, groups, Some(&target), upright);
        target.apply(&action, OffsetDateTime::UNIX_EPOCH).unwrap();
        (target, kept_geometry)
    }

    fn crop_of(document: &EditDocument) -> Crop {
        match document.core(CoreOpId::Crop) {
            Some(CoreOperation::Crop(crop)) => crop,
            other => panic!("no crop: {other:?}"),
        }
    }

    fn close(found: f64, expected: f64) {
        assert!((found - expected).abs() < 1e-9, "{found} isn't {expected}");
    }

    #[test]
    fn a_preset_holds_the_chosen_stages_as_they_are_pre_3() {
        let locked = crop(0.1, 0.1, 0.6, 0.6, Some("3:2"));
        let copied = CopiedEdits::of(
            &document(vec![locked.clone(), exposure(0.5), sepia()]),
            Some((4000, 3000)),
        )
        .unwrap();

        let preset = copied.preset("Warm", DEFAULTS).unwrap();
        assert_eq!(preset.kind(), crate::Kind::Preset);
        assert_eq!(preset.preset_name(), Some("Warm"));
        assert_eq!(preset.core(CoreOpId::Exposure), Some(exposure(0.5)));
        assert_eq!(preset.core(CoreOpId::Sepia), Some(sepia()));
        assert_eq!(preset.core(CoreOpId::Crop), None);
        assert_eq!(preset.history_len(), 0);
        let bytes = preset.to_bytes().unwrap();
        assert!(EditDocument::parse(&bytes).is_ok());

        // The crop isn't fitted to anything yet.
        let with_crop = copied.preset("Framed", ALL).unwrap();
        assert_eq!(with_crop.core(CoreOpId::Crop), Some(locked));

        // Only stages with nothing in them: no preset.
        let effects_only = PasteGroups {
            adjustments: false,
            effects: false,
            geometry: false,
        };
        assert_eq!(copied.preset("Empty", effects_only).map(|_| ()), None);
        let tone_only = CopiedEdits::of(&document(vec![exposure(0.5)]), None).unwrap();
        let effects = PasteGroups {
            effects: true,
            ..effects_only
        };
        assert!(tone_only.preset("Empty", effects).is_none());
    }

    #[test]
    fn a_photo_with_only_red_eye_has_nothing_to_copy_pre_1() {
        let red_eye = CoreOperation::RedEye(RedEye {
            regions: vec![RedEyeRegion {
                x: 0.5,
                y: 0.5,
                radius: 0.1,
            }],
        });
        assert_eq!(CopiedEdits::of(&document(vec![red_eye]), None), None);
        assert_eq!(CopiedEdits::of(&document(Vec::new()), None), None);
    }

    #[test]
    fn red_eye_is_never_pasted_and_the_targets_stays_pre_1() {
        let circle = |x| RedEyeRegion {
            x,
            y: 0.5,
            radius: 0.1,
        };
        let red_eye = |x| {
            CoreOperation::RedEye(RedEye {
                regions: vec![circle(x)],
            })
        };
        let (target, _) = pasted(
            vec![exposure(1.0), red_eye(0.2)],
            None,
            ALL,
            vec![red_eye(0.8)],
            None,
        );
        assert_eq!(target.core(CoreOpId::RedEye), Some(red_eye(0.8)));
        assert_eq!(target.core(CoreOpId::Exposure), Some(exposure(1.0)));
    }

    #[test]
    fn chosen_groups_replace_and_the_others_stay_pre_2() {
        let contrast = CoreOperation::Contrast(Amount { amount: 20.0 });
        let source = vec![exposure(1.0), sepia()];
        let theirs = vec![
            exposure(-2.0),
            contrast.clone(),
            crop(0.1, 0.1, 0.5, 0.5, None),
        ];
        let (target, _) = pasted(source, None, DEFAULTS, theirs, Some((400, 300)));
        assert_eq!(target.core(CoreOpId::Exposure), Some(exposure(1.0)));
        // Replaced, not added to: the group's other settings are gone.
        assert_eq!(target.core(CoreOpId::Contrast), None);
        assert_eq!(target.core(CoreOpId::Sepia), Some(sepia()));
        // Crop and rotation weren't chosen.
        assert_eq!(crop_of(&target).x, 0.1);
    }

    #[test]
    fn a_chosen_group_with_nothing_copied_clears_the_targets_pre_2() {
        let (target, _) = pasted(vec![exposure(1.0)], None, DEFAULTS, vec![sepia()], None);
        assert_eq!(target.core(CoreOpId::Sepia), None);
        let groups = PasteGroups {
            effects: false,
            ..DEFAULTS
        };
        let (target, _) = pasted(vec![exposure(1.0)], None, groups, vec![sepia()], None);
        assert_eq!(target.core(CoreOpId::Sepia), Some(sepia()));
    }

    #[test]
    fn nothing_chosen_changes_nothing() {
        let (target, _) = pasted(
            vec![exposure(1.0)],
            None,
            PasteGroups::default(),
            vec![exposure(-1.0)],
            None,
        );
        assert_eq!(target.core(CoreOpId::Exposure), Some(exposure(-1.0)));
        assert_eq!(target.history_len(), 1);
    }

    /// A document with one other app's operation in the tone stage, and a
    /// version of `seura.contrast` this app can't read.
    fn with_others() -> EditDocument {
        let mut json = serde_json::to_value(document(vec![exposure(-1.0)]).json()).unwrap();
        let operations = json["operations"].as_array_mut().unwrap();
        operations.push(json!({
            "id": "seura.contrast",
            "version": 99,
            "params": { "amount": 3 },
        }));
        operations.push(json!({
            "id": "com.example.glow",
            "version": 1,
            "group": "tone",
            "params": { "amount": 3 },
        }));
        EditDocument::parse(json.to_string().as_bytes()).unwrap()
    }

    #[test]
    fn other_apps_operations_stay_on_the_target_and_arent_added_pre_1() {
        let mut target = with_others();
        let copied = CopiedEdits::of(
            &document(vec![
                exposure(1.0),
                CoreOperation::Contrast(Amount { amount: 9.0 }),
            ]),
            None,
        )
        .unwrap();
        let before = target.operations().len();
        let pasted = paste_action(&copied, DEFAULTS, Some(&target), None);
        // Another version of an operation isn't ours to replace, and saying
        // so doesn't fail the rest of the paste.
        target
            .apply(&pasted.action, OffsetDateTime::UNIX_EPOCH)
            .unwrap();
        assert_eq!(target.core(CoreOpId::Exposure), Some(exposure(1.0)));
        let ids: Vec<String> = target.operations().into_iter().map(|op| op.id).collect();
        assert!(ids.contains(&"com.example.glow".to_string()));
        assert_eq!(target.operations().len(), before);

        // The source's others aren't copied.
        let copied = CopiedEdits::of(&with_others(), None).unwrap();
        let mut plain = document(vec![sepia()]);
        let pasted = paste_action(&copied, ALL, Some(&plain), None);
        plain
            .apply(&pasted.action, OffsetDateTime::UNIX_EPOCH)
            .unwrap();
        let ids: Vec<String> = plain.operations().into_iter().map(|op| op.id).collect();
        assert_eq!(ids, ["seura.exposure"]);
    }

    #[test]
    fn operations_that_are_off_arent_copied_or_removed() {
        let mut json = serde_json::to_value(document(vec![sepia(), exposure(1.0)]).json()).unwrap();
        for entry in json["operations"].as_array_mut().unwrap() {
            if entry["id"] == "seura.sepia" {
                entry["enabled"] = json!(false);
            }
        }
        let off = EditDocument::parse(json.to_string().as_bytes()).unwrap();
        let copied = CopiedEdits::of(&off, None).unwrap();
        assert!(!copied.holds(Stage::Effects));
        let mut target = off.clone();
        let only_tone = CopiedEdits::of(&document(vec![exposure(2.0)]), None).unwrap();
        let pasted = paste_action(&only_tone, DEFAULTS, Some(&target), None);
        target
            .apply(&pasted.action, OffsetDateTime::UNIX_EPOCH)
            .unwrap();
        assert!(target.operations().iter().any(|op| op.id == "seura.sepia"));
    }

    #[test]
    fn a_locked_crop_on_another_shape_keeps_its_ratio_center_and_share() {
        // 3:2 at the center of a 3:2 photo, onto a 4:3 one.
        let source = vec![crop(0.25, 0.25, 0.5, 0.5, Some("3:2"))];
        let (target, _) = pasted(
            source,
            Some((6000, 4000)),
            ALL,
            Vec::new(),
            Some((4000, 3000)),
        );
        let found = crop_of(&target);
        let share = 0.25;
        let width = (share * 1.5 * 3000.0_f64 / 4000.0).sqrt();
        // Each edge rounds inward by at most a pixel.
        assert!((found.width - width).abs() < 2.0 / 4000.0);
        assert!((found.height - share / width).abs() < 2.0 / 3000.0);
        assert!((found.x + found.width / 2.0 - 0.5).abs() < 1.0 / 4000.0);
        assert!((found.y + found.height / 2.0 - 0.5).abs() < 1.0 / 3000.0);
        // The ratio, in pixels, within a pixel's rounding.
        let pixels = (found.width * 4000.0) / (found.height * 3000.0);
        assert!((pixels - 1.5).abs() < 1e-3, "{pixels}");
        assert_eq!(found.aspect.as_deref(), Some("3:2"));
    }

    #[test]
    fn a_locked_crop_shrinks_until_it_fits_then_moves_inward() {
        // Nearly the whole of a wide photo, onto a tall one: at the same share
        // it would be wider than the photo, so it shrinks.
        let source = vec![crop(0.0, 0.0, 1.0, 0.9, Some("16:9"))];
        let (target, _) = pasted(
            source,
            Some((1600, 1000)),
            ALL,
            Vec::new(),
            Some((1000, 2000)),
        );
        let found = crop_of(&target);
        assert!(found.width <= 1.0 && found.height <= 1.0);
        close(found.width, 1.0);
        let pixels = (found.width * 1000.0) / (found.height * 2000.0);
        assert!((pixels - 16.0 / 9.0).abs() < 0.01, "{pixels}");
        assert!(found.x >= 0.0 && found.x + found.width <= 1.0);
        assert!(found.y >= 0.0 && found.y + found.height <= 1.0);

        // Off to a corner: it moves inward only as far as it must.
        let source = vec![crop(0.0, 0.0, 0.4, 0.2, Some("3:1"))];
        let (target, _) = pasted(
            source,
            Some((1500, 1000)),
            ALL,
            Vec::new(),
            Some((1000, 1000)),
        );
        let found = crop_of(&target);
        close(found.x, 0.0);
        assert!(found.y >= 0.0);
        let pixels = found.width / found.height;
        assert!((pixels - 3.0).abs() < 0.05, "{pixels}");
    }

    #[test]
    fn a_target_of_the_same_shape_gets_exactly_the_sources_crop() {
        let source = vec![crop(0.1234, 0.2345, 0.5, 0.5, Some("3:2"))];
        let (target, _) = pasted(
            source,
            Some((6000, 4000)),
            ALL,
            Vec::new(),
            Some((3000, 2000)),
        );
        let found = crop_of(&target);
        assert_eq!((found.x, found.y), (0.1234, 0.2345));
        assert_eq!((found.width, found.height), (0.5, 0.5));
    }

    #[test]
    fn a_crop_with_no_lock_keeps_its_fractions_on_any_shape() {
        let source = vec![crop(0.1, 0.2, 0.3, 0.4, None)];
        let (target, _) = pasted(
            source,
            Some((6000, 4000)),
            ALL,
            Vec::new(),
            Some((1000, 2000)),
        );
        let found = crop_of(&target);
        assert_eq!(
            (found.x, found.y, found.width, found.height),
            (0.1, 0.2, 0.3, 0.4)
        );
    }

    #[test]
    fn the_frame_is_the_targets_after_the_pasted_turn() {
        let orient = CoreOperation::Orient(Orient {
            rotate: 90,
            flip_h: false,
            flip_v: false,
        });
        // Turned, the 6000 × 4000 source's frame is tall, and so is the 4000
        // × 3000 target's: 3:4, against 2:3.
        let source = vec![orient.clone(), crop(0.25, 0.25, 0.5, 0.5, Some("2:3"))];
        let (target, _) = pasted(
            source,
            Some((6000, 4000)),
            ALL,
            Vec::new(),
            Some((4000, 3000)),
        );
        let found = crop_of(&target);
        let pixels = (found.width * 3000.0) / (found.height * 4000.0);
        assert!((pixels - 2.0 / 3.0).abs() < 1e-3, "{pixels}");
        assert_eq!(target.core(CoreOpId::Orient), Some(orient));
    }

    #[test]
    fn straightening_on_another_shape_keeps_the_corners_out_of_view() {
        let angle = CoreOperation::Straighten(Straighten { angle: 10.0 });
        let source = vec![angle.clone(), crop(0.0, 0.0, 1.0, 1.0, None)];
        let (target, _) = pasted(
            source,
            Some((6000, 4000)),
            ALL,
            Vec::new(),
            Some((1000, 2000)),
        );
        let found = crop_of(&target);
        assert!(found.width < 1.0 && found.height < 1.0);
        assert!(fit_scale(&found, (1000, 2000), 10.0) >= 1.0 - 1e-6);

        // A straighten with no crop still gets one.
        let (target, _) = pasted(vec![angle], None, ALL, Vec::new(), Some((1000, 2000)));
        assert!(crop_of(&target).width < 1.0);
    }

    #[test]
    fn the_target_keeps_its_geometry_when_its_shape_is_needed_and_unknown() {
        let theirs = || vec![crop(0.1, 0.1, 0.5, 0.5, None), sepia()];
        let locked = vec![crop(0.2, 0.2, 0.5, 0.5, Some("3:2")), exposure(1.0)];
        let (target, kept) = pasted(locked, None, ALL, theirs(), None);
        assert!(kept);
        assert_eq!(crop_of(&target).x, 0.1);
        // The other groups paste as usual.
        assert_eq!(target.core(CoreOpId::Exposure), Some(exposure(1.0)));
        assert_eq!(target.core(CoreOpId::Sepia), None);

        let angle = vec![CoreOperation::Straighten(Straighten { angle: 2.0 })];
        let (target, kept) = pasted(angle, None, ALL, theirs(), None);
        assert!(kept);
        assert_eq!(target.core(CoreOpId::Straighten), None);
        assert_eq!(crop_of(&target).x, 0.1);
    }

    #[test]
    fn rotation_and_an_unlocked_crop_need_no_shape() {
        let orient = CoreOperation::Orient(Orient {
            rotate: 90,
            flip_h: false,
            flip_v: false,
        });
        let source = vec![orient.clone(), crop(0.2, 0.2, 0.5, 0.5, None)];
        let (target, kept) = pasted(source, None, ALL, Vec::new(), None);
        assert!(!kept);
        assert_eq!(target.core(CoreOpId::Orient), Some(orient));
        assert_eq!(crop_of(&target).x, 0.2);
    }

    #[test]
    fn geometry_not_chosen_needs_no_shape() {
        let source = vec![crop(0.2, 0.2, 0.5, 0.5, Some("3:2")), exposure(1.0)];
        let (_, kept) = pasted(source.clone(), None, DEFAULTS, Vec::new(), None);
        assert!(!kept);
        let copied = CopiedEdits::of(&document(source), None).unwrap();
        assert!(!copied.needs_shape(DEFAULTS));
        assert!(copied.needs_shape(ALL));
    }

    #[test]
    fn pasting_onto_a_photo_with_no_edit_file_needs_only_what_was_copied() {
        let copied = CopiedEdits::of(&document(vec![exposure(1.0)]), None).unwrap();
        let pasted = paste_action(&copied, DEFAULTS, None, None);
        assert_eq!(pasted.action.edits, vec![Edit::Set(exposure(1.0))]);
        // A group with nothing, on a photo with nothing, changes nothing.
        let groups = PasteGroups {
            effects: true,
            ..PasteGroups::default()
        };
        assert!(
            paste_action(&copied, groups, None, None)
                .action
                .edits
                .is_empty()
        );
    }

    #[test]
    fn the_fit_matches_the_edit_formats_vectors() {
        let vectors: serde_json::Value = serde_json::from_str(include_str!(
            "../../../third_party/edit-format/tests/formulas/geometry/crop-inside.json"
        ))
        .unwrap();
        let cases = vectors["cases"].as_array().unwrap();
        assert!(!cases.is_empty());
        for case in cases {
            let frame = (
                case["frame"][0].as_u64().unwrap() as u32,
                case["frame"][1].as_u64().unwrap() as u32,
            );
            let number = |name: &str| case["crop"][name].as_f64().unwrap();
            let crop = Crop {
                x: number("x"),
                y: number("y"),
                width: number("width"),
                height: number("height"),
                aspect: None,
            };
            let scale = fit_scale(&crop, frame, case["angle"].as_f64().unwrap());
            let expected = case["scale"].as_f64().unwrap();
            assert!((scale - expected).abs() < 1e-3, "{case}: {scale}");
        }
    }

    #[test]
    fn edges_round_inward_to_whole_pixels() {
        let (start, length) = on_pixels(0.1234, 0.5, 100.0);
        close(start, 0.13);
        close(length, 0.49);
        // An edge already on a pixel stays there.
        let (start, length) = on_pixels(0.25, 0.5, 100.0);
        close(start, 0.25);
        close(length, 0.5);
    }
}
