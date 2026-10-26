//! Tagging and rating from the Library (M3 slice 1): the one path that writes
//! edit files there, checked against CLAUDE.md's rules 1 to 5 on a scanned
//! temporary library.

use std::cell::RefCell;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde_json::Value;
use seura_edit::{EditDocument, EditFile, edit_file_path};
use seura_library::{Change, ChangeReport, Index, PhotoId, Problem};

use crate::library::{self, Library, Snapshot};
use crate::{WRITER, act, now};

/// The library, scanned, with every photo's id in `Library::photos` order.
fn scanned(library: &Library) -> (RwLock<Index>, Vec<PhotoId>) {
    let mut index = Index::new();
    index.add_root(library.root()).unwrap();
    let root = index.roots()[0].clone();
    index.scan(&root);
    let ids = library
        .photos()
        .iter()
        .map(|photo| {
            index
                .photo_ids()
                .find(|id| {
                    index.path(*id).is_some_and(|path| {
                        fs::canonicalize(path).unwrap() == fs::canonicalize(photo).unwrap()
                    })
                })
                .expect("every stand-in is in the index")
        })
        .collect();
    (RwLock::new(index), ids)
}

/// A stand-in for the Recycle Bin or Trash: moves files to a folder outside
/// the library, and remembers them.
struct Trash {
    folder: tempfile::TempDir,
    moved: RefCell<Vec<PathBuf>>,
}

impl Trash {
    fn new() -> Trash {
        Trash {
            folder: tempfile::tempdir().unwrap(),
            moved: RefCell::new(Vec::new()),
        }
    }

    fn take(&self, path: &Path) -> io::Result<()> {
        let n = self.moved.borrow().len();
        fs::rename(path, self.folder.path().join(n.to_string()))?;
        self.moved.borrow_mut().push(path.to_path_buf());
        Ok(())
    }
}

fn change(index: &RwLock<Index>, ids: &[PhotoId], change: Change, trash: &Trash) -> ChangeReport {
    Index::change_shared(index, ids, &change, WRITER, now(), &|path| trash.take(path))
}

fn saved(photo: &Path) -> EditDocument {
    EditDocument::parse(&fs::read(edit_file_path(photo)).unwrap()).unwrap()
}

/// The files Seura has written, other than edit files: there should be none.
fn leftovers(library: &Library) -> Vec<String> {
    Snapshot::of(library.root())
        .paths()
        .filter(|path| !path.ends_with(".seura.json") && !library.is_photo(path))
        .collect()
}

#[test]
fn rules_1_to_3_tagging_and_rating_write_only_edit_files_beside_photos() {
    let library = Library::new();
    let (index, ids) = scanned(&library);
    let trash = Trash::new();
    let before = Snapshot::of(library.root());

    let rated = change(&index, &ids, Change::Rating(4), &trash);
    change(&index, &ids, Change::AddTag("family/kids".into()), &trash);
    change(&index, &ids, Change::AddTag("beach".into()), &trash);
    change(&index, &ids[..2], Change::RemoveTag("beach".into()), &trash);

    assert_eq!(rated.changed, ids);
    let after = Snapshot::of(library.root());
    let mut edit_files: Vec<String> = library
        .photos()
        .iter()
        .map(|photo| library.relative(&edit_file_path(photo)))
        .collect();
    edit_files.sort();
    // Rule 1: every original as it was; rule 2: only edit files added.
    assert!(after.changed_since(&before).is_empty(), "changed");
    assert!(after.removed_since(&before).is_empty(), "removed");
    assert_eq!(after.added_since(&before), edit_files);
    // Rule 3: no temporary files left.
    assert!(leftovers(&library).is_empty(), "{:?}", leftovers(&library));
    assert!(trash.moved.borrow().is_empty());

    for (n, photo) in library.photos().iter().enumerate() {
        let document = saved(photo);
        assert_eq!(document.rating(), 4);
        let tags = if n < 2 {
            vec!["family/kids"]
        } else {
            vec!["family/kids", "beach"]
        };
        assert_eq!(document.tags(), tags);
        // One history record per action.
        assert_eq!(document.history_len(), if n < 2 { 4 } else { 3 });
        // The photo's identity, with its real fingerprint.
        assert_eq!(
            document.photo().unwrap().sha256,
            library.identity(photo).sha256
        );
        let index = index.read().unwrap();
        assert_eq!(index.photo(ids[n]).unwrap().rating, 4);
    }
}

#[test]
fn rule_5_an_edit_file_left_with_nothing_goes_to_the_trash() {
    let library = Library::new();
    let (index, ids) = scanned(&library);
    let trash = Trash::new();
    let photo = &library.photos()[0];
    let before = Snapshot::of(library.root());

    change(&index, &ids[..1], Change::Rating(3), &trash);
    change(&index, &ids[..1], Change::AddTag("beach".into()), &trash);
    change(&index, &ids[..1], Change::Rating(0), &trash);
    assert!(trash.moved.borrow().is_empty(), "the tag is still there");
    let report = change(&index, &ids[..1], Change::RemoveTag("beach".into()), &trash);

    // FIL-1: sent to the Trash, not deleted, and nothing else touched.
    assert_eq!(report.changed, ids[..1]);
    // Compared by real path: the index holds the folder's (on macOS,
    // /private/var for /var).
    let real = |path: &Path| {
        let folder = fs::canonicalize(path.parent().unwrap()).unwrap();
        folder.join(path.file_name().unwrap())
    };
    let moved: Vec<PathBuf> = trash.moved.borrow().iter().map(|path| real(path)).collect();
    assert_eq!(moved, [real(&edit_file_path(photo))]);
    let after = Snapshot::of(library.root());
    assert!(after.added_since(&before).is_empty());
    assert!(after.changed_since(&before).is_empty());
    assert!(after.removed_since(&before).is_empty());
    let index = index.read().unwrap();
    let record = index.photo(ids[0]).unwrap();
    assert_eq!((record.rating, record.tags.len()), (0, 0));
    assert_eq!(record.edit_file_stamp, None);
}

#[test]
fn rule_4_what_another_app_wrote_is_kept_and_never_sent_to_the_trash() {
    let library = Library::new();
    let photo = library.add_photo("IMG_1234.jpg");
    let edit_file = edit_file_path(&photo);
    let example = library::format_example("IMG_1234.jpg.seura.json");
    fs::write(&edit_file, &example).unwrap();
    // Without its edits, tags, and rating, the example still holds an
    // extension, which isn't Seura's to throw away.
    let mut file = EditFile::load(&edit_file).unwrap().unwrap();
    let operations: Vec<_> = file
        .document
        .operations()
        .into_iter()
        .filter_map(|op| op.core.map(|core| seura_edit::Edit::Remove(core.id())))
        .collect();
    file.document
        .apply(
            &act(
                "Clear",
                [operations, vec![seura_edit::Edit::Tags(Vec::new())]].concat(),
            ),
            now(),
        )
        .unwrap();
    file.save(WRITER, now()).unwrap();
    let (index, _) = scanned(&library);
    let id = index
        .read()
        .unwrap()
        .photo_ids()
        .find(|id| {
            index
                .read()
                .unwrap()
                .path(*id)
                .unwrap()
                .ends_with("IMG_1234.jpg")
        })
        .unwrap();
    let trash = Trash::new();
    let unknown_before: Value = serde_json::from_slice(&fs::read(&edit_file).unwrap()).unwrap();

    change(&index, &[id], Change::Rating(0), &trash);

    assert!(trash.moved.borrow().is_empty());
    let after: Value = serde_json::from_slice(&fs::read(&edit_file).unwrap()).unwrap();
    assert_eq!(after["extensions"], unknown_before["extensions"]);
    assert!(after.get("rating").is_none());
}

#[test]
fn rule_4_an_unreadable_edit_file_is_reported_and_never_overwritten() {
    let library = Library::new();
    let (index, ids) = scanned(&library);
    let photo = &library.photos()[0];
    fs::write(edit_file_path(photo), b"{ not json").unwrap();
    let trash = Trash::new();

    let report = change(&index, &ids[..1], Change::Rating(5), &trash);

    assert_eq!(report.problems, [(ids[0], Problem::Unreadable)]);
    assert_eq!(fs::read(edit_file_path(photo)).unwrap(), b"{ not json");
    assert!(trash.moved.borrow().is_empty());
}

#[test]
fn rule_3_a_write_the_system_refuses_leaves_everything_as_it_was() {
    let library = Library::new();
    let (index, ids) = scanned(&library);
    let trash = Trash::new();
    let photo = &library.photos()[0];
    change(&index, &ids[..1], Change::Rating(2), &trash);
    let before = Snapshot::of(library.root());

    let report = {
        let _blocked = library.block_writes(&edit_file_path(photo));
        change(&index, &ids[..1], Change::Rating(5), &trash)
    };

    assert_eq!(report.changed, []);
    assert_eq!(report.problems.len(), 1, "{report:?}");
    let after = Snapshot::of(library.root());
    assert!(
        after.added_since(&before).is_empty(),
        "{:?}",
        after.added_since(&before)
    );
    assert!(after.changed_since(&before).is_empty());
    assert!(after.removed_since(&before).is_empty());
    assert_eq!(saved(photo).rating(), 2);
    assert_eq!(index.read().unwrap().photo(ids[0]).unwrap().rating, 2);
}

#[test]
fn tag_7_photos_in_a_read_only_folder_are_not_changed() {
    let library = Library::new();
    let winter = library.root().join("Winter");
    let _read_only = library::read_only_folder(&winter);
    // Found by the scan, before anything is tried.
    let (index, ids) = scanned(&library);
    let snow = ids[4];
    {
        let index = index.read().unwrap();
        let folder = |id: PhotoId| index.folder(index.photo(id).unwrap().folder).unwrap();
        assert!(folder(snow).read_only);
        assert!(!folder(ids[0]).read_only);
    }
    let trash = Trash::new();
    let before = Snapshot::of(library.root());

    let report = change(&index, &ids, Change::Rating(3), &trash);

    assert_eq!(report.problems, [(snow, Problem::ReadOnly)]);
    assert_eq!(report.changed, ids[..4]);
    let after = Snapshot::of(library.root());
    assert!(
        !after
            .added_since(&before)
            .iter()
            .any(|path| path.starts_with("Winter/")),
        "{:?}",
        after.added_since(&before)
    );
    assert!(after.changed_since(&before).is_empty());
}

/// The real Recycle Bin or Trash, on CI only, so running the tests doesn't
/// fill a developer's. The file leaves its folder and nothing else does.
#[test]
fn rule_5_the_system_trash_takes_the_file() {
    if std::env::var_os("CI").is_none() {
        return;
    }
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("IMG_1.jpg.seura.json");
    let keep = folder.path().join("IMG_1.jpg");
    fs::write(&file, b"{}").unwrap();
    fs::write(&keep, b"photo").unwrap();

    seura_library::move_to_trash(&file).unwrap();

    assert!(!file.exists());
    assert_eq!(fs::read(&keep).unwrap(), b"photo");
}

#[test]
fn brw_11_undo_and_redo_write_only_edit_files_and_bring_back_what_went_to_the_trash() {
    let library = Library::new();
    let (index, ids) = scanned(&library);
    let trash = Trash::new();
    let before = Snapshot::of(library.root());
    let mut history = seura_library::History::default();
    let mut act = |change: Change, ids: &[PhotoId]| {
        let (_, action) = Index::change_recorded(&index, ids, &change, WRITER, now(), &|path| {
            trash.take(path)
        });
        history.record(action);
    };
    act(Change::Rating(3), &ids);
    act(Change::Rating(0), &ids[..2]);
    // The first two photos' files went to the Trash, emptied.
    assert_eq!(trash.moved.borrow().len(), 2);

    let undo = |history: &mut seura_library::History| {
        history.undo(&index, WRITER, now(), &|path| trash.take(path))
    };
    let redo = |history: &mut seura_library::History| {
        history.redo(&index, WRITER, now(), &|path| trash.take(path))
    };
    assert_eq!(undo(&mut history).unwrap().done, 2);
    for photo in &library.photos()[..2] {
        assert_eq!(
            EditDocument::parse(&fs::read(edit_file_path(photo)).unwrap())
                .unwrap()
                .rating(),
            3,
            "written back from memory, with its history"
        );
    }
    redo(&mut history).unwrap();
    undo(&mut history).unwrap();
    undo(&mut history).unwrap();

    // Every original as it was, nothing but edit files and nothing left behind.
    let after = Snapshot::of(library.root());
    assert!(after.changed_since(&before).is_empty(), "changed");
    assert!(after.removed_since(&before).is_empty(), "removed");
    assert!(
        after
            .added_since(&before)
            .iter()
            .all(|path| path.ends_with(".seura.json")),
        "{:?}",
        after.added_since(&before)
    );
    assert!(leftovers(&library).is_empty(), "{:?}", leftovers(&library));
    // Rule 5: everything removed went to the Trash, never deleted.
    assert!(trash.moved.borrow().len() >= 2);
}
