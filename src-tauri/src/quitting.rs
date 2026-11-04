//! Quitting saves what's waiting first (EDT-11, #174). The Editor holds a
//! change for a moment before saving it, so closing the window or choosing
//! Quit asks the UI to save, and the app exits once it answers. A UI that
//! doesn't answer can't keep the app open: it exits after a short limit.

use serde::Serialize;
use specta::Type;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Runtime};
use tauri_specta::Event;

/// How long a quit waits for the UI: long enough for a small file to be
/// written to a slow drive, and short enough not to look stuck.
const LIMIT: Duration = Duration::from_secs(3);

/// The user asked to quit: the UI saves what's waiting and calls `finish_quit`.
#[derive(Clone, Debug, Serialize, Type, Event)]
pub struct QuitRequested;

/// Starts quitting. Asking again while waiting changes nothing.
pub fn request<R: Runtime>(app: &AppHandle<R>) {
    static WAITING: AtomicBool = AtomicBool::new(false);
    if WAITING.swap(true, Ordering::SeqCst) {
        return;
    }
    if QuitRequested.emit(app).is_err() {
        app.exit(0);
        return;
    }
    let app = app.clone();
    thread::spawn(move || {
        thread::sleep(LIMIT);
        app.exit(0);
    });
}
