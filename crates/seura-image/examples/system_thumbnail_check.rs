//! Asks the system for each file's thumbnail and reports whether the file
//! stayed cloud-only (Windows attribute RECALL_ON_DATA_ACCESS) throughout.
//! `cargo run -p seura-image --example system_thumbnail_check -- <files>`
#[cfg(windows)]
fn cloud_only(path: &std::path::Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    std::fs::metadata(path).is_ok_and(|m| m.file_attributes() & 0x40_0000 != 0)
}
#[cfg(not(windows))]
fn cloud_only(_path: &std::path::Path) -> bool {
    false
}
fn main() {
    for file in std::env::args().skip(1) {
        let path = std::path::Path::new(&file);
        let before = cloud_only(path);
        let start = std::time::Instant::now();
        let thumbnail = seura_image::system_thumbnail(path, 512);
        let took = start.elapsed().as_millis();
        let after = cloud_only(path);
        let result = thumbnail.map_or("none".to_string(), |t| format!("{}x{}", t.width, t.height));
        println!(
            "{result:>9} in {took:>5} ms  cloud-only before {before}, after {after}  {}",
            path.file_name().unwrap().to_string_lossy()
        );
    }
}
