//! The data-safety suite: the rules in CLAUDE.md that protect users' files,
//! checked by running real operations on a temporary photo library and
//! comparing every file before and after (technical design, section 11).
//!
//! - Rule 1: originals are never modified. They're made read-only here, so
//!   opening one for writing would fail, and their bytes, size, and
//!   modification time are compared.
//! - Rule 2: nothing is written except the edit file beside each photo.
//! - Rule 3: edit files are written atomically; no temporary files remain.
//! - Rule 4: an edit file that can't be read is never overwritten, and
//!   unknown content survives a save.
//!
//! - Scanning reads the library and writes only the cache: the index
//!   snapshot and its README.
//!
//! Suites for later write paths (thumbnails, export, Trash) join this one as
//! they're built.

mod library;

use std::fs;

use library::{Library, Snapshot};
use serde_json::{Value, json};
use seura_edit::{
    Action, CoreOperation, Edit, EditDocument, EditFile, Exposure, SaveError, Writer,
    edit_file_path,
};
use time::OffsetDateTime;

const WRITER: Writer = Writer {
    app: "Seura Photos",
    version: "0.0.0-test",
};

fn now() -> OffsetDateTime {
    OffsetDateTime::now_utc()
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

/// What a user session does to every photo: edit, tag, rate, save, reopen,
/// change again, undo, save.
fn edit_every_photo(library: &Library) {
    for photo in library.photos() {
        let path = edit_file_path(&photo);
        let document = EditDocument::new_for_photo(&library.identity(&photo));
        let mut file = EditFile::create(&path, document);
        let edits = vec![
            exposure(0.5),
            Edit::Tags(vec!["family".into()]),
            Edit::Rating(4),
        ];
        file.document.apply(&act("Edit", edits), now()).unwrap();
        file.save(WRITER, now()).unwrap();

        let mut file = EditFile::load(&path)
            .unwrap()
            .expect("the edit file was saved");
        file.document
            .apply(&act("Exposure", vec![exposure(1.25)]), now())
            .unwrap();
        file.save(WRITER, now()).unwrap();
        file.document.undo().unwrap();
        file.save(WRITER, now()).unwrap();
    }
}

#[test]
fn rules_1_and_2_scanning_writes_only_to_the_cache() {
    let library = Library::new();
    for photo in library.photos().iter().take(2) {
        let mut file = EditFile::create(
            &edit_file_path(photo),
            EditDocument::new_for_photo(&library.identity(photo)),
        );
        file.document
            .apply(&act("Rating", vec![Edit::Rating(3)]), now())
            .unwrap();
        file.save(WRITER, now()).unwrap();
    }
    let before = Snapshot::of(library.root());
    let cache = tempfile::tempdir().unwrap();

    let mut index = seura_library::Index::new();
    index.add_root(library.root()).unwrap();
    let root = index.roots()[0].clone();
    let report = index.scan(&root);
    assert_eq!(
        report.photos,
        library.photos().len(),
        "every stand-in is a photo"
    );
    index.save_snapshot(cache.path()).unwrap();
    let mut index = seura_library::Index::load_snapshot(cache.path()).unwrap();
    index.scan(&root);
    for id in index.photo_ids().collect::<Vec<_>>() {
        index.fingerprint(id).unwrap();
    }
    index.save_snapshot(cache.path()).unwrap();

    let after = Snapshot::of(library.root());
    assert!(
        after.added_since(&before).is_empty(),
        "added: {:?}",
        after.added_since(&before)
    );
    assert!(
        after.changed_since(&before).is_empty(),
        "changed: {:?}",
        after.changed_since(&before)
    );
    assert!(after.removed_since(&before).is_empty());
    let cache_files: Vec<String> = Snapshot::of(cache.path()).paths().collect();
    assert_eq!(cache_files, ["README.txt", seura_library::SNAPSHOT_FILE]);
}

#[test]
fn rule_1_editing_never_touches_originals() {
    let library = Library::new();
    let before = Snapshot::of(library.root());

    edit_every_photo(&library);

    let after = Snapshot::of(library.root());
    for photo in library.photos() {
        let relative = library.relative(&photo);
        assert_eq!(
            before.get(&relative),
            after.get(&relative),
            "{relative} changed: bytes, size, modification time, or read-only flag"
        );
    }
}

#[test]
fn rule_2_only_edit_files_beside_photos_are_written() {
    let library = Library::new();
    let before = Snapshot::of(library.root());

    edit_every_photo(&library);

    let after = Snapshot::of(library.root());
    let expected: Vec<String> = library
        .photos()
        .iter()
        .map(|photo| library.relative(&edit_file_path(photo)))
        .collect();
    assert_eq!(after.added_since(&before), sorted(expected));
    assert!(
        after.changed_since(&before).is_empty(),
        "existing files changed"
    );
    assert!(
        after.removed_since(&before).is_empty(),
        "files were removed"
    );
}

#[test]
fn rule_3_saves_leave_no_temporary_files() {
    let library = Library::new();
    edit_every_photo(&library);
    let leftovers: Vec<String> = Snapshot::of(library.root())
        .paths()
        .filter(|path| !path.ends_with(".seura.json") && !library.is_photo(path))
        .collect();
    assert!(leftovers.is_empty(), "left behind: {leftovers:?}");
}

#[test]
fn rule_3_a_save_the_system_refuses_leaves_everything_as_it_was() {
    let library = Library::new();
    let photo = &library.photos()[0];
    let path = edit_file_path(photo);
    let mut file = EditFile::create(&path, EditDocument::new_for_photo(&library.identity(photo)));
    file.document
        .apply(&act("Rating", vec![Edit::Rating(2)]), now())
        .unwrap();
    file.save(WRITER, now()).unwrap();
    let before = Snapshot::of(library.root());

    file.document
        .apply(&act("Rating", vec![Edit::Rating(5)]), now())
        .unwrap();
    let result = {
        let _blocked = library.block_writes(&path);
        file.save(WRITER, now())
    };

    assert!(
        matches!(result, Err(SaveError::Io(_))),
        "expected a write error, got {result:?}"
    );
    let after = Snapshot::of(library.root());
    assert!(
        after.added_since(&before).is_empty(),
        "left behind: {:?}",
        after.added_since(&before)
    );
    assert!(
        after.changed_since(&before).is_empty(),
        "changed: {:?}",
        after.changed_since(&before)
    );
    assert!(after.removed_since(&before).is_empty());
}

#[test]
fn rule_4_unreadable_edit_files_are_never_overwritten() {
    let library = Library::new();
    let photo = &library.photos()[0];
    let path = edit_file_path(photo);
    let identity = library.identity(photo);
    let mut valid = serde_json::to_value(EditDocument::new_for_photo(&identity).json()).unwrap();
    valid["rating"] = json!(6);
    let out_of_range = serde_json::to_vec(&valid).unwrap();
    valid["rating"] = json!(3);
    valid["formatVersion"] = json!("2.0");
    let newer_major = serde_json::to_vec(&valid).unwrap();

    let unreadable: [(&str, Vec<u8>); 6] = [
        ("empty", Vec::new()),
        ("not JSON", b"\x00\x01binary".to_vec()),
        (
            "truncated",
            br#"{"format": "seura-edit", "formatVersion": "1.0", "pho"#.to_vec(),
        ),
        (
            "another format",
            br#"{"format": "picasa", "formatVersion": "1.0"}"#.to_vec(),
        ),
        ("out of range", out_of_range),
        ("newer major version", newer_major),
    ];

    for (case, bytes) in unreadable {
        fs::write(&path, &bytes).unwrap();
        assert!(
            EditFile::load(&path).is_err(),
            "{case}: should fail to load"
        );

        // Saving a fresh document over it is refused.
        let mut file = EditFile::create(&path, EditDocument::new_for_photo(&identity));
        file.document
            .apply(&act("Rating", vec![Edit::Rating(5)]), now())
            .unwrap();
        let result = file.save(WRITER, now());
        assert!(
            matches!(result, Err(SaveError::Unreadable(_))),
            "{case}: expected Unreadable, got {result:?}"
        );
        assert_eq!(
            fs::read(&path).unwrap(),
            bytes,
            "{case}: the file was changed"
        );
    }
}

#[test]
fn rule_4_a_file_damaged_after_loading_is_not_overwritten() {
    let library = Library::new();
    let photo = &library.photos()[0];
    let path = edit_file_path(photo);
    let mut file = EditFile::create(&path, EditDocument::new_for_photo(&library.identity(photo)));
    file.document
        .apply(&act("Rating", vec![Edit::Rating(2)]), now())
        .unwrap();
    file.save(WRITER, now()).unwrap();

    let damaged = b"{ half a file".to_vec();
    fs::write(&path, &damaged).unwrap();
    file.document
        .apply(&act("Rating", vec![Edit::Rating(3)]), now())
        .unwrap();
    assert!(matches!(
        file.save(WRITER, now()),
        Err(SaveError::Unreadable(_))
    ));
    assert_eq!(fs::read(&path).unwrap(), damaged);
}

#[test]
fn changes_made_on_disk_since_loading_are_never_overwritten() {
    let library = Library::new();
    let photo = &library.photos()[0];
    let path = edit_file_path(photo);
    let mut file = EditFile::create(&path, EditDocument::new_for_photo(&library.identity(photo)));
    file.document
        .apply(&act("Rating", vec![Edit::Rating(2)]), now())
        .unwrap();
    file.save(WRITER, now()).unwrap();

    // Another computer syncs a different, valid version (FIL-3).
    let mut other = EditFile::load(&path).unwrap().unwrap();
    other
        .document
        .apply(&act("Rating", vec![Edit::Rating(5)]), now())
        .unwrap();
    other
        .save(
            Writer {
                app: "Seura Photos (laptop)",
                version: "0.0.0",
            },
            now(),
        )
        .unwrap();
    let synced = fs::read(&path).unwrap();

    file.document
        .apply(&act("Exposure", vec![exposure(1.0)]), now())
        .unwrap();
    assert!(matches!(
        file.save(WRITER, now()),
        Err(SaveError::ChangedOnDisk)
    ));
    assert_eq!(fs::read(&path).unwrap(), synced);

    // A file that appears where none was expected isn't overwritten either.
    let fresh = &library.photos()[1];
    let fresh_path = edit_file_path(fresh);
    let mut file = EditFile::create(
        &fresh_path,
        EditDocument::new_for_photo(&library.identity(fresh)),
    );
    fs::copy(&path, &fresh_path).unwrap();
    file.document
        .apply(&act("Rating", vec![Edit::Rating(1)]), now())
        .unwrap();
    assert!(matches!(
        file.save(WRITER, now()),
        Err(SaveError::ChangedOnDisk)
    ));
}

#[test]
fn rule_4_unknown_content_survives_a_save() {
    let library = Library::new();
    let photo = library.add_photo("IMG_1234.jpg");
    let path = edit_file_path(&photo);
    let example = library::format_example("IMG_1234.jpg.seura.json");
    fs::write(&path, &example).unwrap();
    let original: Value = serde_json::from_slice(&example).unwrap();

    let mut file = EditFile::load(&path).unwrap().unwrap();
    file.document
        .apply(&act("Exposure", vec![exposure(0.8)]), now())
        .unwrap();
    file.save(WRITER, now()).unwrap();

    let saved: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    // Everything outside what this save changes is identical, in the same order.
    let keys = |value: &Value| {
        value
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(keys(&saved), keys(&original));
    for key in [
        "$schema",
        "format",
        "formatVersion",
        "photo",
        "tags",
        "rating",
    ] {
        assert_eq!(saved[key], original[key], "{key} changed");
    }
    // Only exposure changed among the operations; the plugin's is untouched.
    let mut expected = original["operations"].clone();
    expected[2]["params"] = json!({ "ev": 0.8 });
    assert_eq!(saved["operations"], expected);
    // The old history is intact, with one record added.
    let history = saved["history"].as_array().unwrap();
    assert_eq!(
        history[..history.len() - 1],
        original["history"].as_array().unwrap()[..]
    );
}

fn sorted(mut paths: Vec<String>) -> Vec<String> {
    paths.sort();
    paths
}
