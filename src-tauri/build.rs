use std::{env, path::PathBuf};

fn main() {
    // Listing the app's commands makes Tauri generate an `allow-<command>`
    // permission for each, and deny any command a capability doesn't grant
    // (technical design, section 10).
    let manifest = tauri_build::AppManifest::new().commands(&[
        "app_version",
        "add_folder",
        "folders",
        "photos",
        "show_thumbnails",
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

    // seura-image can call libheif on Windows; it's delay-loaded so the app
    // starts without heif.dll (see seura-image's build.rs).
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-arg=/DELAYLOAD:heif.dll");
        println!("cargo:rustc-link-lib=delayimp");
    }

    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(manifest)
            .windows_attributes(windows),
    )
    .expect("failed to run the Tauri build script");
}
