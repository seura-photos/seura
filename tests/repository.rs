//! Checks that the repository is set up the way the other suites expect.

use seura_tests::edit_format_dir;

#[test]
fn edit_format_submodule_is_checked_out() {
    let schema = edit_format_dir().join("schema").join("1.0.json");
    assert!(
        schema.is_file(),
        "{} is missing. Run `git submodule update --init` to fetch the edit format.",
        schema.display()
    );
}
