//! The Seura Photos app shell: window, commands, events, the `seura://`
//! protocol, and (later) the updater (technical design, section 1).
//!
//! The web UI reaches the core only through the commands registered in
//! [`specta_builder`]; their TypeScript bindings are generated into
//! `ui/src/bindings.ts` (decision 0008).

mod about;
mod commands;
mod editor;
mod gpu;
mod library;
mod locale;
mod locations;
mod measure;
mod menu;
mod messages;
mod protocol;
mod synced;
mod viewer;
mod welcome;

use library::{AppSettings, Library, LibraryChanged, ScanProgress, ThumbnailsReady};
use std::sync::Mutex;
use tauri::{App, Manager, RunEvent, WebviewWindowBuilder};
use tauri_specta::{Builder, Event, collect_commands, collect_events};

/// Every command the UI may call and every event it may hear. The same list
/// drives the invoke handler and the generated TypeScript bindings, so the
/// two can't drift apart.
pub fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::app_version,
            commands::open_link,
            commands::add_folder,
            commands::folders,
            commands::remove_folder,
            commands::photos,
            commands::view_settings,
            commands::set_sort,
            commands::set_thumbnail_size,
            commands::set_sidebar_width,
            commands::show_thumbnails,
            commands::change_photos,
            commands::start_over,
            commands::deleting,
            commands::delete_photos,
            commands::photo_notices,
            commands::show_kept_files,
            commands::tag_choices,
            commands::tags,
            commands::tag_photos,
            commands::search_photos,
            commands::undo,
            commands::redo,
            commands::show_in_folder,
            commands::open_in_default_app,
            commands::opening,
            commands::photos_chosen,
            commands::photo_details,
            commands::selection_summary,
            commands::open_photo,
            commands::close_photo,
            commands::view_photo,
            commands::view_region,
            commands::close_viewer,
            commands::edit_photo,
            commands::step_edit,
            commands::revert_photo,
            commands::export_photo,
            commands::dev_options,
            commands::text_scale,
            commands::region,
            commands::welcome,
            commands::welcome_add_folder,
            commands::welcome_choose_cache,
            commands::finish_welcome,
            commands::check_render,
            commands::finish_render_check,
            commands::measure_elapsed,
            commands::scan_progress,
            commands::measure_report,
        ])
        .events(collect_events![
            ThumbnailsReady,
            LibraryChanged,
            ScanProgress,
            menu::MenuCommand
        ])
        // Nothing changed, as the preview draws the original (EDT-1).
        .constant(
            "NEUTRAL_SETTINGS",
            commands::RenderSettings::from(seura_render::Settings::NEUTRAL),
        )
}

/// Builds and runs the app. Called from `main`.
pub fn run() {
    measure::start();
    let builder = specta_builder();

    let app = tauri::Builder::default();
    #[cfg(target_os = "macos")]
    let app = app.menu(menu::build);
    app.on_menu_event(|app, event| menu::handle(app, event.id().as_ref()))
        .invoke_handler(builder.invoke_handler())
        .register_uri_scheme_protocol("seura", |context, request| {
            protocol::respond(context.app_handle(), &request)
        })
        .setup(move |app| {
            builder.mount_events(app);
            // Release builds on Windows have no console; the UI says it too.
            for switch in commands::active_switches() {
                eprintln!("Developer switch on: {switch}");
            }
            let (settings, origin) = locations::load_settings(app.handle())?;
            if let Some(problem) = &settings.problem {
                eprintln!(
                    "{} is left as it is, and changes aren't saved: {problem}",
                    settings.path().display()
                );
            }
            if origin == locations::Origin::CarriedOver {
                eprintln!(
                    "Library folders carried over from Documents/Seura Photos (decision 0015)"
                );
            }
            let seura = locations::seura_folder(app.handle())?;
            let cache = locations::cache_folder(&seura, &settings.settings);
            app.manage(Library::open(
                app.handle(),
                &cache,
                &settings.settings.library_folders,
            ));
            app.manage(AppSettings(Mutex::new(settings)));
            app.manage(welcome::Welcome::new(
                origin == locations::Origin::FirstRun || welcome::forced(),
            ));
            app.manage(editor::Editor::default());
            app.manage(viewer::Viewer::default());
            // Show the snapshot at once (PRF-1), then catch up with the disk.
            app.state::<Library>().start_watching(app.handle());
            // Finished thumbnails reach the UI in batches.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(library::THUMBNAILS_INTERVAL);
                    handle.state::<Library>().send_thumbnails(&handle);
                }
            });
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let library = handle.state::<Library>();
                library.rescan(&handle);
                let _ = LibraryChanged.emit(&handle);
                library.sweep_thumbnails();
                // A drive that's disconnected shows as unavailable and comes
                // back when it's reconnected (LIB-6), checked every few seconds.
                let mut available = library.availability();
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(4));
                    let now = library.availability();
                    if now != available {
                        if now
                            .iter()
                            .zip(&available)
                            .any(|(now, before)| *now && !before)
                        {
                            library.rescan(&handle);
                        }
                        available = now;
                        library.rewatch();
                        let _ = LibraryChanged.emit(&handle);
                    }
                }
            });
            create_main_window(app, &cache)?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while starting Seura Photos")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                app.state::<Library>().save();
            }
        });
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
fn create_main_window(app: &mut App, cache: &std::path::Path) -> tauri::Result<()> {
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
    let window = window.data_directory(locations::web_view_folder(cache));
    #[cfg(not(target_os = "windows"))]
    let _ = cache;

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
