//! Shared helpers for the cross-crate tests in this folder: data safety,
//! render references, and benchmarks (technical design, section 11).
//!
//! Each suite is a `[[test]]` target in this crate's `Cargo.toml`.

use std::path::PathBuf;

/// The repository root, found from this crate's manifest directory.
pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the tests crate lives one level below the repository root")
        .to_path_buf()
}

/// The checked-out `seura-photos/edit-format` submodule.
pub fn edit_format_dir() -> PathBuf {
    repo_root().join("third_party").join("edit-format")
}
