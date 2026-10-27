//! Rule 5: Seura deletes a file directly, without the Recycle Bin or Trash,
//! only where nothing in it can be lost (decision 0017). Every direct
//! deletion in the app's code is listed here with why it's allowed, so a
//! new one fails this test until it's reviewed against the rule. Test code
//! is left out: it deletes in its own temporary folders.

use std::fs;
use std::path::{Path, PathBuf};

/// Calls that delete for good.
const DELETING: [&str; 3] = ["remove_file(", "remove_dir(", "remove_dir_all("];

/// Each source file that deletes directly, how many times, and why.
const ALLOWED: [(&str, usize, &str); 4] = [
    (
        "crates/seura-edit/src/file.rs",
        2,
        "a failed save's temporary file; an edit file with nothing in it, \
         read again just before, where there's no Trash (FIL-1)",
    ),
    (
        "crates/seura-library/src/start_over.rs",
        3,
        "Start over's own copy in Seura's folder when the operation fails \
         (twice); an unreadable edit file once that copy matches it byte \
         for byte, where there's no Trash (FIL-4)",
    ),
    (
        "crates/seura-library/src/thumbnails.rs",
        3,
        "thumbnails in the cache, Seura's own and rebuilt from the photos, \
         and the cache's emptied folders",
    ),
    (
        "src-tauri/src/library.rs",
        1,
        "thumbnails in the cache, for photos no longer in the library",
    ),
];

#[test]
fn only_the_allowed_files_are_ever_deleted_directly() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut sources = Vec::new();
    for folder in ["crates", "src-tauri/src"] {
        collect(&root.join(folder), &mut sources);
    }
    let mut found = Vec::new();
    for path in sources {
        let text = fs::read_to_string(&path).unwrap().replace("\r\n", "\n");
        let count = DELETING
            .iter()
            .map(|call| app_code(&text).matches(call).count())
            .sum::<usize>();
        let name = path
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if count > 0 || ALLOWED.iter().any(|(allowed, ..)| *allowed == name) {
            found.push((name, count));
        }
    }
    found.sort();
    let mut expected: Vec<_> = ALLOWED
        .iter()
        .map(|(name, count, _)| (name.to_string(), *count))
        .collect();
    expected.sort();
    assert_eq!(
        found, expected,
        "a direct deletion was added or removed: check it against CLAUDE.md \
         rule 5 and decision 0017, then update ALLOWED"
    );
}

/// The source's app code: without its test module, comments, and anything
/// in `examples`, `benches`, or `tests` folders, which `collect` skips.
fn app_code(text: &str) -> String {
    let code = match text.find("#[cfg(test)]\nmod tests") {
        Some(at) => &text[..at],
        None => text,
    };
    code.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn collect(folder: &Path, sources: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy();
        if path.is_dir() {
            if !matches!(&*name, "examples" | "benches" | "tests" | "target") {
                collect(&path, sources);
            }
        } else if name.ends_with(".rs") {
            sources.push(path);
        }
    }
}
