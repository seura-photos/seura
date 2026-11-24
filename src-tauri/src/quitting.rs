//! Quitting saves what's waiting first (EDT-11, #174). The Editor holds a
//! change for a moment before saving it, so closing the window or choosing
//! Quit asks the UI to save, and the app exits once it answers. A UI that
//! doesn't answer can't keep the app open: it exits after a short limit.
//!
//! On macOS, a quit can also come from the system: the Dock's Quit, logging
//! out or restarting, or any other quit Apple Event (#217). Those arrive as
//! `applicationShouldTerminate:`, which Tauri's windowing library doesn't
//! answer, so Seura adds it: the reply is "later", the same save runs, and
//! then the system is told to go ahead, so a logout carries on by itself.

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

/// Set once the app may exit: the UI has saved, or the limit has passed.
static FINISHING: AtomicBool = AtomicBool::new(false);

/// Starts quitting. Asking again while waiting changes nothing.
pub fn request<R: Runtime>(app: &AppHandle<R>) {
    static WAITING: AtomicBool = AtomicBool::new(false);
    if WAITING.swap(true, Ordering::SeqCst) {
        return;
    }
    if QuitRequested.emit(app).is_err() {
        finish(app);
        return;
    }
    let app = app.clone();
    thread::spawn(move || {
        thread::sleep(LIMIT);
        finish(&app);
    });
}

/// Lets the app exit, once: when the UI has saved, or at the limit.
pub fn finish<R: Runtime>(app: &AppHandle<R>) {
    if FINISHING.swap(true, Ordering::SeqCst) {
        return;
    }
    #[cfg(target_os = "macos")]
    if mac::take_waiting_reply() {
        // The system asked, and waits for the answer: it then quits as it
        // would have, through `applicationWillTerminate:`.
        mac::reply(app);
        return;
    }
    app.exit(0);
}

#[cfg(target_os = "macos")]
pub use mac::install;

#[cfg(target_os = "macos")]
mod mac {
    use super::FINISHING;
    use objc2::runtime::{AnyClass, AnyObject, Bool, Imp, Sel};
    use objc2::{class, ffi, msg_send, sel};
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tauri::{AppHandle, Runtime, Wry};

    /// `NSTerminateNow` and `NSTerminateLater` (`NSApplicationTerminateReply`).
    const TERMINATE_NOW: usize = 1;
    const TERMINATE_LATER: usize = 2;

    static APP: OnceLock<AppHandle<Wry>> = OnceLock::new();
    /// The system asked to quit and is waiting for the reply.
    static REPLY_WAITING: AtomicBool = AtomicBool::new(false);

    /// Adds `applicationShouldTerminate:` to the app's delegate. Called once
    /// the app is ready, when the delegate exists.
    pub fn install(app: &AppHandle<Wry>) {
        if APP.set(app.clone()).is_err() {
            return;
        }
        // SAFETY: on the main thread, once AppKit is running. The method is
        // added to the delegate's own class, with the signature AppKit calls
        // it with: an NSUInteger reply, and the NSApplication as sender.
        unsafe {
            let application: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
            let delegate: *mut AnyObject = msg_send![application, delegate];
            if delegate.is_null() {
                eprintln!("Quitting from the Dock won't wait for a save: no app delegate");
                return;
            }
            let class = ffi::object_getClass(delegate).cast_mut();
            let method = std::mem::transmute::<
                extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject) -> usize,
                Imp,
            >(should_terminate);
            let added: Bool = ffi::class_addMethod(
                class.cast::<AnyClass>(),
                sel!(applicationShouldTerminate:),
                method,
                c"Q@:@".as_ptr(),
            );
            if !added.as_bool() {
                eprintln!("Quitting from the Dock won't wait for a save: the delegate has its own");
            }
        }
    }

    /// AppKit asks whether to quit: not yet, the first time, while the UI
    /// saves what's waiting; at once after that.
    extern "C-unwind" fn should_terminate(
        _delegate: *mut AnyObject,
        _command: Sel,
        _sender: *mut AnyObject,
    ) -> usize {
        if FINISHING.load(Ordering::SeqCst) {
            return TERMINATE_NOW;
        }
        let Some(app) = APP.get() else {
            return TERMINATE_NOW;
        };
        REPLY_WAITING.store(true, Ordering::SeqCst);
        super::request(app);
        TERMINATE_LATER
    }

    /// Whether the system is waiting for the reply, taking the wait.
    pub fn take_waiting_reply() -> bool {
        REPLY_WAITING.swap(false, Ordering::SeqCst)
    }

    /// Tells AppKit to go ahead and quit, on the main thread as it requires.
    pub fn reply<R: Runtime>(app: &AppHandle<R>) {
        let _ = app.run_on_main_thread(|| {
            // SAFETY: on the main thread, answering the earlier
            // `NSTerminateLater`.
            unsafe {
                let application: *mut AnyObject =
                    msg_send![class!(NSApplication), sharedApplication];
                let _: () = msg_send![application, replyToApplicationShouldTerminate: Bool::YES];
            }
        });
    }
}
