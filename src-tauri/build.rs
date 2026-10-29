use std::{env, path::PathBuf};

fn main() {
    // Listing the app's commands makes Tauri generate an `allow-<command>`
    // permission for each, and deny any command a capability doesn't grant
    // (technical design, section 10).
    let manifest = tauri_build::AppManifest::new().commands(&[
        "app_version",
        "open_link",
        "add_folder",
        "folders",
        "remove_folder",
        "photos",
        "view_settings",
        "set_sort",
        "set_thumbnail_size",
        "set_sidebar_width",
        "show_thumbnails",
        "change_photos",
        "start_over",
        "deleting",
        "delete_photos",
        "photo_notices",
        "show_kept_files",
        "tag_choices",
        "tags",
        "tag_photos",
        "search_photos",
        "undo",
        "redo",
        "show_in_folder",
        "open_in_default_app",
        "opening",
        "photos_chosen",
        "photo_details",
        "selection_summary",
        "open_photo",
        "close_photo",
        "view_photo",
        "view_region",
        "close_viewer",
        "edit_photo",
        "step_edit",
        "revert_photo",
        "resolve_outside_change",
        "export_photo",
        "dev_options",
        "text_scale",
        "region",
        "welcome",
        "welcome_add_folder",
        "welcome_choose_cache",
        "finish_welcome",
        "check_render",
        "finish_render_check",
        "measure_elapsed",
        "scan_progress",
        "measure_report",
    ]);

    // tauri-build would embed the Windows application manifest into the app
    // binary only, and test binaries then fail to start because they lack the
    // Common Controls v6 dependency. Embedding it through the linker instead
    // covers every binary, tests included.
    let mut windows = tauri_build::WindowsAttributes::new();
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        windows = tauri_build::WindowsAttributes::new_without_app_manifest();
        let path =
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("windows-app-manifest.xml");
        println!("cargo:rerun-if-changed={}", path.display());
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", path.display());
    }

    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(manifest)
            .windows_attributes(windows),
    )
    .expect("failed to run the Tauri build script");
}
