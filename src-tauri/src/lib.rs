//! The Seura Photos app shell: window, commands, and (later) events, the
//! `seura://` protocol, and the updater (technical design, section 1).
//!
//! The web UI reaches the core only through the commands registered in
//! [`specta_builder`]; their TypeScript bindings are generated into
//! `ui/src/bindings.ts` (decision 0008).

mod commands;
mod locations;

use tauri::{App, WebviewWindowBuilder};
use tauri_specta::{Builder, collect_commands};

/// Every command the UI may call. The same list drives the invoke handler and
/// the generated TypeScript bindings, so the two can't drift apart.
pub fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![commands::app_version])
}

/// Builds and runs the app. Called from `main`.
pub fn run() {
    let builder = specta_builder();

    tauri::Builder::default()
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            create_main_window(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Seura Photos");
}

/// Creates the main window from its entry in `tauri.conf.json` (which has
/// `"create": false`), keeping the web view's own data out of hidden folders
/// as far as each platform allows (decision 0013):
///
/// - Incognito on both platforms: WebView2's InPrivate mode on Windows, a
///   non-persistent `WKWebsiteDataStore` on macOS. Nothing the UI stores in
///   the browser survives a restart, so the UI must never rely on it.
/// - On Windows, WebView2's data folder goes in the cache instead of
///   `%LOCALAPPDATA%`. WKWebView has no equivalent setting.
fn create_main_window(app: &mut App) -> tauri::Result<()> {
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "main")
        .cloned()
        .expect("tauri.conf.json defines the main window");

    let window = WebviewWindowBuilder::from_config(app.handle(), &config)?.incognito(true);

    #[cfg(target_os = "windows")]
    let window = window.data_directory(locations::web_view_folder(&locations::cache_folder(
        app.handle(),
    )?));

    window.build()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use specta_typescript::Typescript;

    /// The committed bindings must match the Rust command signatures.
    /// Run with `SEURA_UPDATE_BINDINGS=1` to regenerate them.
    #[test]
    fn typescript_bindings_are_current() {
        let committed = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ui/src/bindings.ts");
        let scratch =
            std::env::temp_dir().join(format!("seura-bindings-{}.ts", std::process::id()));

        super::specta_builder()
            .export(Typescript::default(), &scratch)
            .expect("failed to export TypeScript bindings");
        let generated = fs::read_to_string(&scratch).expect("failed to read exported bindings");
        let _ = fs::remove_file(&scratch);

        if std::env::var_os("SEURA_UPDATE_BINDINGS").is_some() {
            fs::write(&committed, &generated).expect("failed to write ui/src/bindings.ts");
            return;
        }

        let current = fs::read_to_string(&committed).unwrap_or_default();
        assert!(
            current == generated,
            "ui/src/bindings.ts is out of date. Run `npm run bindings` in ui/ to regenerate it."
        );
    }
}
