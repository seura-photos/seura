//! Where Seura writes its own data (CLAUDE.md rule 2; decisions 0005 and 0013).

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager, Runtime};

/// `Documents/Seura Photos`: settings, presets, and by default the cache.
pub fn seura_folder<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<PathBuf> {
    Ok(app.path().document_dir()?.join("Seura Photos"))
}

/// The cache folder. Until settings exist (SET-3, M2) this is always the
/// default, `Documents/Seura Photos/Cache`; then it becomes the user's choice.
pub fn cache_folder<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<PathBuf> {
    Ok(seura_folder(app)?.join("Cache"))
}

/// The web view's own data folder, inside the cache so it's visible, movable
/// with the cache, and safe to delete. Only WebView2 (Windows) honors it;
/// WKWebView (macOS) runs with a non-persistent store instead (decision 0013).
#[cfg(any(target_os = "windows", test))]
pub fn web_view_folder(cache: &Path) -> PathBuf {
    cache.join("WebView")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn web_view_data_lives_inside_the_cache() {
        let cache = Path::new("cache");
        assert!(super::web_view_folder(cache).starts_with(cache));
    }
}
