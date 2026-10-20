//! Conformance with the edit-format repository (M1 slice 2): every file in
//! `examples/` and `tests/valid/` loads and round-trips unchanged, and every
//! file in `tests/invalid/` is rejected.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use seura_edit::{Action, Edit, EditDocument, Kind, ReadError, Step};

fn format_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../third_party/edit-format")
}

fn files_in(folder: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(format_dir().join(folder))
        .unwrap_or_else(|e| panic!("{folder} is missing ({e}); run `git submodule update --init`"))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no test files found in {folder}");
    files
}

#[test]
fn valid_files_load_and_round_trip_unchanged() {
    let valid = [files_in("examples"), files_in("tests/valid")].concat();
    for path in valid {
        let bytes = fs::read(&path).unwrap();
        let original: Value = serde_json::from_slice(&bytes).unwrap();
        let document = EditDocument::parse(&bytes)
            .unwrap_or_else(|e| panic!("{} should load: {e}", path.display()));

        // Writing it back gives the same JSON: same fields, same order, same values.
        let written = document.to_bytes().unwrap();
        let rewritten: Value = serde_json::from_slice(&written).unwrap();
        assert_eq!(
            rewritten,
            original,
            "{} changed on round trip",
            path.display()
        );
        assert_eq!(
            serde_json::to_string(&rewritten).unwrap(),
            serde_json::to_string(&original).unwrap(),
            "{} changed key order on round trip",
            path.display()
        );

        // And writing is stable: a second round trip is byte-for-byte identical.
        let again = EditDocument::parse(&written).unwrap().to_bytes().unwrap();
        assert_eq!(
            again,
            written,
            "{} isn't stable when rewritten",
            path.display()
        );
        assert!(written.ends_with(b"\n") && !written.starts_with(b"\xEF\xBB\xBF"));
    }
}

#[test]
fn examples_are_read_as_their_kind() {
    for path in files_in("examples") {
        let document = EditDocument::parse(&fs::read(&path).unwrap()).unwrap();
        let is_preset = path.to_string_lossy().ends_with(".seura-preset.json");
        let expected = if is_preset { Kind::Preset } else { Kind::Photo };
        assert_eq!(document.kind(), expected, "{}", path.display());
        assert_eq!(document.photo().is_some(), !is_preset, "{}", path.display());
    }
}

#[test]
fn invalid_files_are_rejected() {
    for path in files_in("tests/invalid") {
        match EditDocument::parse(&fs::read(&path).unwrap()) {
            Ok(_) => panic!("{} should be rejected", path.display()),
            Err(ReadError::Syntax(e)) => {
                panic!(
                    "{} should be rejected by a rule, not a syntax error: {e}",
                    path.display()
                )
            }
            Err(_) => {}
        }
    }
}

#[test]
fn the_full_example_is_read_correctly() {
    let bytes = fs::read(format_dir().join("examples/IMG_1234.jpg.seura.json")).unwrap();
    let document = EditDocument::parse(&bytes).unwrap();
    let photo = document.photo().unwrap();
    assert_eq!(
        (photo.file.as_str(), photo.width, photo.height),
        ("IMG_1234.jpg", Some(4032), Some(3024))
    );
    assert_eq!(document.tags(), ["family/kids", "vacation/2026"]);
    assert_eq!(document.rating(), 4);
    assert_eq!(document.history_len(), 9);

    let operations = document.operations();
    assert_eq!(operations.len(), 7);
    // Six core operations Seura renders, and a plugin's it keeps but can't.
    assert!(operations[..6].iter().all(|op| op.core.is_some()));
    assert_eq!(operations[6].id, "com.example.filmGrain");
    assert!(operations[6].core.is_none());
}

#[test]
fn undone_changes_in_the_example_can_be_redone() {
    let bytes = fs::read(format_dir().join("examples/IMG_2207.jpg.seura.json")).unwrap();
    let mut document = EditDocument::parse(&bytes).unwrap();
    assert_eq!((document.undo_steps(), document.redo_steps()), (2, 2));

    assert_eq!(document.redo().unwrap(), Step::Done("Contrast".into()));
    assert_eq!(document.redo().unwrap(), Step::Done("Saturation".into()));
    let ids: Vec<String> = document.operations().into_iter().map(|op| op.id).collect();
    // Contrast and saturation take their places after shadows (canonical order).
    assert_eq!(
        ids,
        [
            "seura.exposure",
            "seura.shadows",
            "seura.contrast",
            "seura.saturation"
        ]
    );
    assert_eq!(document.redo().unwrap(), Step::Nothing);
    document.to_bytes().expect("still valid");
}

#[test]
fn files_from_a_newer_minor_version_keep_their_version() {
    let path = format_dir().join("tests/valid/newer-minor-version.json");
    let original: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let mut document = EditDocument::parse(&fs::read(&path).unwrap()).unwrap();
    let rating = Action {
        label: "Rating".into(),
        edits: vec![Edit::Rating(5)],
    };
    document
        .apply(&rating, time::OffsetDateTime::now_utc())
        .unwrap();

    let saved: Value = serde_json::from_slice(&document.to_bytes().unwrap()).unwrap();
    assert_eq!(saved["formatVersion"], "1.1");
    // What 1.0 doesn't know is kept: a top-level field, a photo field, and a
    // core operation from the newer version.
    assert_eq!(saved["favorite"], original["favorite"]);
    assert_eq!(saved["photo"], original["photo"]);
    assert_eq!(saved["operations"], original["operations"]);
}

#[test]
fn shadows_after_contrast_is_rejected_for_its_order() {
    let path = format_dir().join("tests/invalid/shadows-after-contrast.json");
    let error = EditDocument::parse(&fs::read(path).unwrap()).unwrap_err();
    let message = error.to_string();
    assert!(message.contains("order"), "{message}");
}
