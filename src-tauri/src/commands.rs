//! Commands the UI can call. Each one is also listed in `build.rs`, which
//! generates the permission the UI's capability grants for it.

/// The app's version, from `Cargo.toml`.
#[tauri::command]
#[specta::specta]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn app_version_matches_the_package() {
        assert_eq!(super::app_version(), env!("CARGO_PKG_VERSION"));
    }
}
