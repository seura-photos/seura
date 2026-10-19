fn main() {
    // libheif is delay-loaded on Windows, so binaries start without it and
    // only HEIC decoding needs it (see src/heif/windows.rs). This covers this
    // crate's own tests; src-tauri/build.rs does the same for the app.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-arg=/DELAYLOAD:heif.dll");
        println!("cargo:rustc-link-lib=delayimp");
    }
}
