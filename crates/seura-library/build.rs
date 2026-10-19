fn main() {
    // seura-image can call libheif on Windows; it's delay-loaded so these
    // binaries start without heif.dll (see seura-image's build.rs).
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-arg=/DELAYLOAD:heif.dll");
        println!("cargo:rustc-link-lib=delayimp");
    }
}
