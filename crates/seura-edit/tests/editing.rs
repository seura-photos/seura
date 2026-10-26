//! Editing a document: canonical order, neutral settings, tags, rating,
//! history, undo and redo, and preserving what this version doesn't know.

use serde_json::{Value, json};
use seura_edit::{
    Action, Amount, CoreOpId, CoreOperation, Edit, EditDocument, EditError, Exposure,
    HISTORY_LIMIT, PhotoIdentity, Step, Vignette, clean_tags, tag_key,
};
use time::OffsetDateTime;

const HASH: &str = "4e1c0b9a77d3f2e85a6b4c3d2e1f0a9b8c7d6e5f4a3b2c1d0e9f8a7b6c5d4e3f";

fn photo() -> PhotoIdentity {
    PhotoIdentity {
        file: "IMG_0001.jpg".into(),
        size: 1234,
        sha256: Some(HASH.into()),
        width: Some(4032),
        height: Some(3024),
    }
}

fn now() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_792_000_000).unwrap()
}

fn act(label: &str, edits: Vec<Edit>) -> Action {
    Action {
        label: label.into(),
        edits,
    }
}

fn exposure(ev: f64) -> Edit {
    Edit::Set(CoreOperation::Exposure(Exposure { ev }))
}

fn ids(document: &EditDocument) -> Vec<String> {
    document.operations().into_iter().map(|op| op.id).collect()
}

fn parse(value: Value) -> EditDocument {
    EditDocument::parse(&serde_json::to_vec(&value).unwrap()).unwrap()
}

#[test]
fn operations_are_inserted_in_canonical_order() {
    let mut document = EditDocument::new_for_photo(&photo());
    let vignette = CoreOperation::Vignette(Vignette {
        amount: -20.0,
        size: 50.0,
    });
    let contrast = CoreOperation::Contrast(Amount { amount: 10.0 });
    document
        .apply(&act("Vignette", vec![Edit::Set(vignette)]), now())
        .unwrap();
    document
        .apply(&act("Exposure", vec![exposure(0.5)]), now())
        .unwrap();
    document
        .apply(&act("Contrast", vec![Edit::Set(contrast)]), now())
        .unwrap();
    assert_eq!(
        ids(&document),
        ["seura.exposure", "seura.contrast", "seura.vignette"]
    );
    document.to_bytes().expect("still valid");
}

/// Highlights and shadows go after white balance and before contrast,
/// whatever order they're set in (edit format: "Operations").
#[test]
fn highlights_and_shadows_take_their_place_in_the_tone_stage() {
    let mut document = EditDocument::new_for_photo(&photo());
    let edits = vec![
        Edit::Set(CoreOperation::Contrast(Amount { amount: 10.0 })),
        Edit::Set(CoreOperation::Shadows(Amount { amount: 35.0 })),
        exposure(0.5),
        Edit::Set(CoreOperation::Highlights(Amount { amount: -40.0 })),
    ];
    document.apply(&act("Light", edits), now()).unwrap();
    assert_eq!(
        ids(&document),
        [
            "seura.exposure",
            "seura.highlights",
            "seura.shadows",
            "seura.contrast"
        ]
    );
    assert_eq!(
        document.core(CoreOpId::Shadows),
        Some(CoreOperation::Shadows(Amount { amount: 35.0 }))
    );
    document.to_bytes().expect("still valid");
}

#[test]
fn neutral_settings_remove_the_operation() {
    let mut document = EditDocument::new_for_photo(&photo());
    document
        .apply(&act("Exposure", vec![exposure(0.5)]), now())
        .unwrap();
    document
        .apply(&act("Exposure", vec![exposure(0.0)]), now())
        .unwrap();
    assert!(document.operations().is_empty());
    assert!(!document.json().contains_key("operations"));
    assert_eq!(document.history_len(), 2);
}

#[test]
fn unchanged_settings_record_nothing() {
    let mut document = EditDocument::new_for_photo(&photo());
    document
        .apply(&act("Exposure", vec![exposure(0.5)]), now())
        .unwrap();
    let changed = document
        .apply(&act("Exposure", vec![exposure(0.5)]), now())
        .unwrap();
    assert!(!changed);
    assert_eq!(document.history_len(), 1);
}

#[test]
fn history_records_each_action_and_is_capped() {
    let mut document = EditDocument::new_for_photo(&photo());
    for step in 1..=(HISTORY_LIMIT + 5) {
        document
            .apply(&act("Exposure", vec![exposure(step as f64 / 100.0)]), now())
            .unwrap();
    }
    assert_eq!(document.history_len(), HISTORY_LIMIT);
    let history = document.json()["history"].as_array().unwrap();
    // The oldest were dropped: the first kept record goes from 0.05 to 0.06.
    assert_eq!(history[0]["changes"][0]["from"], json!({ "ev": 0.05 }));
    assert_eq!(history[0]["at"], json!("2026-10-14T17:46:40Z"));
}

#[test]
fn undo_and_redo_restore_settings_tags_and_rating() {
    let mut document = EditDocument::new_for_photo(&photo());
    document
        .apply(&act("Exposure", vec![exposure(0.5)]), now())
        .unwrap();
    let edits = vec![
        exposure(1.0),
        Edit::Tags(vec!["family".into()]),
        Edit::Rating(3),
    ];
    document.apply(&act("Paste edits", edits), now()).unwrap();

    assert_eq!(document.undo().unwrap(), Step::Done("Paste edits".into()));
    assert_eq!(
        document.core(CoreOpId::Exposure),
        Some(CoreOperation::Exposure(Exposure { ev: 0.5 }))
    );
    assert!(document.tags().is_empty());
    assert_eq!(document.rating(), 0);

    assert_eq!(document.redo().unwrap(), Step::Done("Paste edits".into()));
    assert_eq!(
        document.core(CoreOpId::Exposure),
        Some(CoreOperation::Exposure(Exposure { ev: 1.0 }))
    );
    assert_eq!(document.tags(), ["family"]);
    assert_eq!(document.rating(), 3);

    document.undo().unwrap();
    document.undo().unwrap();
    assert!(document.operations().is_empty());
    assert_eq!(document.undo().unwrap(), Step::Nothing);
    // Both records stay in the file, marked undone, ready to redo.
    assert_eq!((document.undo_steps(), document.redo_steps()), (0, 2));
    let history = document.json()["history"].as_array().unwrap();
    assert!(history.iter().all(|record| record["undone"] == json!(true)));
}

#[test]
fn undone_records_are_marked_in_place() {
    let mut document = EditDocument::new_for_photo(&photo());
    document
        .apply(&act("Exposure", vec![exposure(0.5)]), now())
        .unwrap();
    document.undo().unwrap();
    let record = document.json()["history"][0].as_object().unwrap();
    let keys: Vec<&str> = record.keys().map(String::as_str).collect();
    assert_eq!(keys, ["at", "label", "undone", "changes"]);
    document.redo().unwrap();
    let record = document.json()["history"][0].as_object().unwrap();
    assert!(!record.contains_key("undone"));
}

#[test]
fn redo_survives_a_restart() {
    let mut document = EditDocument::new_for_photo(&photo());
    document
        .apply(&act("Exposure", vec![exposure(0.5)]), now())
        .unwrap();
    document
        .apply(&act("Rating", vec![Edit::Rating(4)]), now())
        .unwrap();
    document.undo().unwrap();

    let mut reopened = EditDocument::parse(&document.to_bytes().unwrap()).unwrap();
    assert_eq!(reopened.rating(), 0);
    assert_eq!(reopened.redo().unwrap(), Step::Done("Rating".into()));
    assert_eq!(reopened.rating(), 4);
    assert_eq!(reopened.redo_steps(), 0);
}

#[test]
fn a_new_change_discards_redo() {
    let mut document = EditDocument::new_for_photo(&photo());
    document
        .apply(&act("Exposure", vec![exposure(0.5)]), now())
        .unwrap();
    document.undo().unwrap();
    document
        .apply(&act("Rating", vec![Edit::Rating(2)]), now())
        .unwrap();
    assert_eq!(document.redo().unwrap(), Step::Nothing);
    // The undone record is gone from the file, not just hidden.
    assert_eq!(document.history_len(), 1);
    assert_eq!(document.json()["history"][0]["label"], "Rating");
}

#[test]
fn a_failed_undo_check_discards_that_record_and_older_ones_only() {
    // Exposure was changed by another app; contrast is undone and redoable.
    let mut document = parse(json!({
        "format": "seura-edit",
        "formatVersion": "1.0",
        "photo": photo(),
        "operations": [ { "id": "seura.exposure", "version": 1, "params": { "ev": 2 } } ],
        "history": [
            { "at": "2026-10-17T19:37:30Z", "label": "Exposure",
              "changes": [ { "op": "seura.exposure", "from": null, "to": { "ev": 0.3 } } ] },
            { "at": "2026-10-17T19:38:00Z", "label": "Contrast", "undone": true,
              "changes": [ { "op": "seura.contrast", "from": null, "to": { "amount": 30 } } ] }
        ]
    }));
    assert_eq!(document.undo().unwrap(), Step::Discarded);
    assert_eq!((document.undo_steps(), document.redo_steps()), (0, 1));
    assert_eq!(document.redo().unwrap(), Step::Done("Contrast".into()));
}

#[test]
fn a_failed_redo_check_discards_all_undone_records() {
    // Contrast is undone, but another app has since set it.
    let mut document = parse(json!({
        "format": "seura-edit",
        "formatVersion": "1.0",
        "photo": photo(),
        "operations": [
            { "id": "seura.exposure", "version": 1, "params": { "ev": 0.3 } },
            { "id": "seura.contrast", "version": 1, "params": { "amount": 10 } }
        ],
        "history": [
            { "at": "2026-10-17T19:37:30Z", "label": "Exposure",
              "changes": [ { "op": "seura.exposure", "from": null, "to": { "ev": 0.3 } } ] },
            { "at": "2026-10-17T19:38:00Z", "label": "Contrast", "undone": true,
              "changes": [ { "op": "seura.contrast", "from": null, "to": { "amount": 30 } } ] },
            { "at": "2026-10-17T19:38:30Z", "label": "Saturation", "undone": true,
              "changes": [ { "op": "seura.saturation", "from": null, "to": { "amount": 15 } } ] }
        ]
    }));
    assert_eq!(document.redo().unwrap(), Step::Discarded);
    assert_eq!((document.undo_steps(), document.redo_steps()), (1, 0));
    // Nothing was applied.
    assert_eq!(
        document.core(CoreOpId::Contrast),
        Some(CoreOperation::Contrast(Amount { amount: 10.0 }))
    );
}

#[test]
fn a_change_this_version_does_not_understand_fails_the_check() {
    // From the format repository: a 1.1 file whose newest record changes a
    // field 1.0 doesn't know.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../third_party/edit-format/tests/valid/newer-minor-version.json");
    let mut document = EditDocument::parse(&std::fs::read(path).unwrap()).unwrap();
    let before = document.operations();
    assert_eq!(document.undo().unwrap(), Step::Discarded);
    assert_eq!(document.history_len(), 0);
    assert_eq!(document.operations(), before);
}

#[test]
fn malformed_versions_are_rejected() {
    for version in ["1.01", "01.0", "1", "1.0.0", "one"] {
        let bytes = serde_json::to_vec(&json!({
            "format": "seura-edit", "formatVersion": version, "photo": photo()
        }))
        .unwrap();
        let result = EditDocument::parse(&bytes);
        assert!(
            matches!(result, Err(seura_edit::ReadError::Invalid(_))),
            "{version}: {result:?}"
        );
    }
}

#[test]
fn undo_discards_history_that_no_longer_matches_the_file() {
    // Another app set exposure to 2 without updating the history.
    let mut document = parse(json!({
        "format": "seura-edit",
        "formatVersion": "1.0",
        "photo": photo(),
        "operations": [ { "id": "seura.exposure", "version": 1, "params": { "ev": 2 } } ],
        "history": [
            { "at": "2026-10-17T19:37:30Z", "label": "Exposure",
              "changes": [ { "op": "seura.exposure", "from": null, "to": { "ev": 0.3 } } ] }
        ]
    }));
    assert_eq!(document.undo().unwrap(), Step::Discarded);
    assert_eq!(document.history_len(), 0);
    // The setting another app made is left alone.
    assert_eq!(
        document.core(CoreOpId::Exposure),
        Some(CoreOperation::Exposure(Exposure { ev: 2.0 }))
    );
}

#[test]
fn numbers_match_by_value_in_the_consistency_check() {
    let mut document = parse(json!({
        "format": "seura-edit",
        "formatVersion": "1.0",
        "photo": photo(),
        "operations": [ { "id": "seura.contrast", "version": 1, "params": { "amount": 35.0 } } ],
        "history": [
            { "at": "2026-10-17T19:37:30Z", "label": "Contrast",
              "changes": [ { "op": "seura.contrast", "from": null, "to": { "amount": 35 } } ] }
        ]
    }));
    assert_eq!(document.undo().unwrap(), Step::Done("Contrast".into()));
}

#[test]
fn tags_are_cleaned_as_writers_must() {
    let tags = ["  /family//kids/ ", "Family/Kids", "vacation", "", " / "].map(String::from);
    assert_eq!(clean_tags(&tags), ["family/kids", "vacation"]);
}

#[test]
fn invalid_edits_change_nothing() {
    let mut document = EditDocument::new_for_photo(&photo());
    document
        .apply(&act("Exposure", vec![exposure(0.5)]), now())
        .unwrap();
    let before = document.json().clone();
    let result = document.apply(&act("Too much", vec![exposure(1.0), exposure(9.0)]), now());
    assert!(matches!(result, Err(EditError::Invalid(_))));
    assert_eq!(document.json(), &before);
    let result = document.apply(&act("Rating", vec![Edit::Rating(6)]), now());
    assert!(matches!(result, Err(EditError::Invalid(_))));
    assert_eq!(document.json(), &before);
}

#[test]
fn unknown_content_is_kept_in_place_when_editing() {
    let mut document = parse(json!({
        "format": "seura-edit",
        "formatVersion": "1.0",
        "writer": { "app": "Other App", "version": "3", "build": 77 },
        "photo": photo(),
        "operations": [
            { "id": "seura.exposure", "version": 1, "enabled": false, "params": { "ev": 0.25 } },
            { "id": "com.example.toneCurve", "version": 4, "group": "tone", "required": true,
              "params": { "points": [[0, 0], [0.5, 0.6], [1, 1]] } },
            { "id": "com.example.filmGrain", "version": 2, "group": "effects", "params": { "amount": 12 } }
        ],
        "extensions": { "com.example.faces": { "people": ["Ann"] } }
    }));

    let contrast = CoreOperation::Contrast(Amount { amount: 20.0 });
    let edits = vec![exposure(0.75), Edit::Set(contrast)];
    document.apply(&act("Edit", edits), now()).unwrap();

    let json = document.json();
    let operations = json["operations"].as_array().unwrap();
    // Exposure changed in place and stays disabled; contrast went before the
    // plugin in the same stage; the plugins are untouched.
    assert_eq!(
        operations[0],
        json!({ "id": "seura.exposure", "version": 1, "enabled": false, "params": { "ev": 0.75 } })
    );
    assert_eq!(operations[1]["id"], "seura.contrast");
    assert_eq!(
        operations[2]["params"],
        json!({ "points": [[0, 0], [0.5, 0.6], [1, 1]] })
    );
    assert_eq!(operations[3]["id"], "com.example.filmGrain");
    assert_eq!(
        json["extensions"],
        json!({ "com.example.faces": { "people": ["Ann"] } })
    );
    assert_eq!(json["writer"]["build"], 77);
    // History went in its usual place, before the extensions.
    let keys: Vec<&str> = json.keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        [
            "format",
            "formatVersion",
            "writer",
            "photo",
            "operations",
            "history",
            "extensions"
        ]
    );
}

#[test]
fn operations_from_another_version_are_never_replaced() {
    let mut document = parse(json!({
        "format": "seura-edit",
        "formatVersion": "1.0",
        "photo": photo(),
        "operations": [ { "id": "seura.exposure", "version": 2, "params": { "stops": 1, "curve": "filmic" } } ]
    }));
    let operation = &document.operations()[0];
    assert!(
        operation.core.is_none(),
        "version 2 isn't rendered as version 1"
    );
    let result = document.apply(&act("Exposure", vec![exposure(0.5)]), now());
    assert_eq!(
        result,
        Err(EditError::UnsupportedOperation("seura.exposure".into()))
    );
    let result = document.apply(&act("Reset", vec![Edit::Remove(CoreOpId::Exposure)]), now());
    assert_eq!(
        result,
        Err(EditError::UnsupportedOperation("seura.exposure".into()))
    );
}

#[test]
fn files_from_a_newer_minor_version_are_read_and_kept() {
    // A hypothetical 1.1: a new top-level field, a new photo field, and a new
    // core operation this version doesn't know.
    let original = json!({
        "format": "seura-edit",
        "formatVersion": "1.1",
        "photo": {
            "file": "IMG_0001.jpg", "size": 1234, "sha256": HASH, "width": 4032, "height": 3024,
            "capturedWith": "iPhone"
        },
        "operations": [
            { "id": "seura.exposure", "version": 1, "params": { "ev": 0.5 } },
            { "id": "seura.sharpen", "version": 1, "params": { "amount": 30 } }
        ],
        "faces": [ { "x": 0.4, "y": 0.3 } ]
    });
    let mut document = parse(original.clone());
    assert_eq!(
        document
            .to_bytes()
            .map(|b| serde_json::from_slice::<Value>(&b).unwrap())
            .unwrap(),
        original
    );

    document
        .apply(&act("Exposure", vec![exposure(1.0)]), now())
        .unwrap();
    let json = document.json();
    assert_eq!(json["formatVersion"], "1.1");
    assert_eq!(json["faces"], original["faces"]);
    assert_eq!(json["photo"]["capturedWith"], "iPhone");
    assert_eq!(json["operations"][1], original["operations"][1]);
}

#[test]
fn newer_major_versions_are_refused() {
    let bytes = serde_json::to_vec(&json!({
        "format": "seura-edit", "formatVersion": "2.0", "photo": photo()
    }))
    .unwrap();
    assert!(matches!(
        EditDocument::parse(&bytes),
        Err(seura_edit::ReadError::UnsupportedVersion(v)) if v == "2.0"
    ));
}

#[test]
fn presets_have_no_tags_rating_or_history() {
    let mut document = parse(json!({
        "format": "seura-edit", "formatVersion": "1.0", "kind": "preset", "name": "Bright"
    }));
    document
        .apply(&act("Exposure", vec![exposure(0.5)]), now())
        .unwrap();
    assert_eq!(document.history_len(), 0);
    let result = document.apply(&act("Tags", vec![Edit::Tags(vec!["x".into()])]), now());
    assert_eq!(result, Err(EditError::NotForPresets));
    document.to_bytes().expect("still a valid preset");
}

#[test]
fn a_byte_order_mark_is_read_but_never_written() {
    let mut bytes = b"\xEF\xBB\xBF".to_vec();
    bytes.extend(
        serde_json::to_vec(
            &json!({ "format": "seura-edit", "formatVersion": "1.0", "photo": photo() }),
        )
        .unwrap(),
    );
    let document = EditDocument::parse(&bytes).unwrap();
    assert!(!document.to_bytes().unwrap().starts_with(b"\xEF\xBB\xBF"));
}

/// A cloud-only photo, tagged without downloading it (LIB-9).
fn pending_photo() -> PhotoIdentity {
    PhotoIdentity {
        file: "20260912_181502.jpg".into(),
        size: 3_264_118,
        sha256: None,
        width: None,
        height: None,
    }
}

#[test]
fn a_photo_can_be_tagged_before_its_details_are_known() {
    let mut document = EditDocument::new_for_photo(&pending_photo());
    let edits = vec![Edit::Tags(vec!["family".into()]), Edit::Rating(4)];
    document.apply(&act("Tag and rate", edits), now()).unwrap();
    let saved: Value = serde_json::from_slice(&document.to_bytes().unwrap()).unwrap();
    assert_eq!(
        saved["photo"],
        json!({ "file": "20260912_181502.jpg", "size": 3_264_118 })
    );
}

#[test]
fn missing_details_are_filled_in_once_the_photo_is_read() {
    let mut document = EditDocument::new_for_photo(&pending_photo());
    assert!(document.fill_in_photo(HASH, 4032, 3024).unwrap());
    let photo = document.json()["photo"].as_object().unwrap();
    let keys: Vec<&str> = photo.keys().map(String::as_str).collect();
    assert_eq!(keys, ["file", "size", "sha256", "width", "height"]);
    assert_eq!(document.photo().unwrap().sha256.as_deref(), Some(HASH));
    assert_eq!(
        document.history_len(),
        0,
        "filling in details isn't a user action"
    );

    // Known values are never replaced: a different fingerprint means the
    // photo changed, which is reported, not papered over (FIL-7).
    let other = "a".repeat(64);
    assert!(!document.fill_in_photo(&other, 100, 100).unwrap());
    assert_eq!(document.photo().unwrap().sha256.as_deref(), Some(HASH));
    assert_eq!(document.photo().unwrap().width, Some(4032));
}

#[test]
fn a_malformed_fingerprint_is_refused() {
    let mut document = EditDocument::new_for_photo(&pending_photo());
    let before = document.json().clone();
    let result = document.fill_in_photo("not a hash", 4032, 3024);
    assert!(matches!(result, Err(EditError::Invalid(_))));
    assert_eq!(document.json(), &before);
}

#[test]
fn a_file_is_empty_once_nothing_in_it_needs_keeping() {
    let mut document = EditDocument::new_for_photo(&photo());
    assert!(document.is_empty());
    document
        .apply(&act("Rating", vec![Edit::Rating(3)]), now())
        .unwrap();
    assert!(!document.is_empty());
    document
        .apply(&act("Rating", vec![Edit::Rating(0)]), now())
        .unwrap();
    // Only history left: that's not worth keeping (FIL-1).
    assert_eq!(document.history_len(), 2);
    assert!(document.is_empty());
    document
        .apply(&act("Tags", vec![Edit::Tags(vec!["beach".into()])]), now())
        .unwrap();
    assert!(!document.is_empty());

    // What another app or a newer version wrote is kept.
    for kept in [
        json!({ "operations": [{ "id": "com.example.grain", "version": 1, "group": "effects", "params": {} }] }),
        json!({ "extensions": { "com.example.faces": {} } }),
        // Fields only a newer minor version can have.
        json!({ "formatVersion": "1.1", "note": "keep me" }),
    ] {
        let mut value = json!({ "format": "seura-edit", "formatVersion": "1.0", "photo": photo() });
        value
            .as_object_mut()
            .unwrap()
            .extend(kept.as_object().unwrap().clone());
        assert!(!parse(value).is_empty(), "{kept}");
    }
    let mut unknown_identity =
        json!({ "format": "seura-edit", "formatVersion": "1.1", "photo": photo() });
    unknown_identity["photo"]["serial"] = json!(7);
    assert!(!parse(unknown_identity).is_empty());
}

#[test]
fn tags_match_in_every_language_tag_2() {
    // TAG-2's own examples: full case folding, not just A to Z.
    assert_eq!(tag_key("Ärzte"), tag_key("ärzte"));
    assert_eq!(tag_key("STRASSE"), tag_key("straße"));
    assert_eq!(tag_key("Family"), tag_key("family"));
    // The same letter composed or decomposed (macOS often gives the latter).
    assert_eq!(tag_key("caf\u{e9}"), tag_key("cafe\u{301}"));
    assert_ne!(tag_key("family"), tag_key("family/kids"));

    // The first form typed wins, and is stored composed.
    let tags = ["Straße", "STRASSE", "cafe\u{301}", "Café"].map(String::from);
    assert_eq!(clean_tags(&tags), ["Straße", "caf\u{e9}"]);
}
