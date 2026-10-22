//! Measuring the performance targets in a release build (M2 slice 10).
//!
//! `SEURA_MEASURE=1` makes the UI run a fixed script on itself (see
//! `ui/src/measure.ts`) and report what it measured here; the core adds its
//! own memory use, writes `measurement.json` to Seura's folder, and quits.
//! Meant to be run with `SEURA_DATA` pointing at a folder of its own, by
//! `scripts/measure.py`, so a real library is never touched.

use std::sync::OnceLock;
use std::time::Instant;

use tauri::{AppHandle, Runtime};

/// The developer switch that runs the measurement script.
pub const MEASURE_SWITCH: &str = "SEURA_MEASURE";

/// When the app started, as near as the core can tell.
static STARTED: OnceLock<Instant> = OnceLock::new();

/// Notes the start; called first thing in `run`.
pub fn start() {
    STARTED.get_or_init(Instant::now);
}

pub fn active() -> bool {
    std::env::var(MEASURE_SWITCH).as_deref() == Ok("1")
}

/// Milliseconds since the app started.
pub fn elapsed_ms() -> f64 {
    STARTED.get_or_init(Instant::now).elapsed().as_secs_f64() * 1000.0
}

/// The core process's memory, in bytes: what it holds now, and the most it
/// has held. The web view's own processes aren't included.
pub fn memory() -> (u64, u64) {
    platform::memory()
}

/// Writes the UI's report, with the core's memory added, to
/// `measurement.json` in Seura's folder, and quits.
pub fn report<R: Runtime>(app: &AppHandle<R>, ui: &str) -> Result<(), String> {
    let mut report: serde_json::Value =
        serde_json::from_str(ui).map_err(|error| error.to_string())?;
    let (now, peak) = memory();
    report["memory"] = serde_json::json!({ "coreBytes": now, "corePeakBytes": peak });
    let seura = crate::locations::seura_folder(app).map_err(|error| error.to_string())?;
    let text = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    std::fs::write(seura.join("measurement.json"), text).map_err(|error| error.to_string())?;
    app.exit(0);
    Ok(())
}

#[cfg(windows)]
mod platform {
    #![allow(unsafe_code, reason = "the process memory counters are a C API")]

    use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::Threading::GetCurrentProcess;

    pub fn memory() -> (u64, u64) {
        let mut counters = PROCESS_MEMORY_COUNTERS {
            cb: size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            ..Default::default()
        };
        let read = unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb) };
        if read.is_err() {
            return (0, 0);
        }
        (
            counters.WorkingSetSize as u64,
            counters.PeakWorkingSetSize as u64,
        )
    }
}

#[cfg(target_os = "macos")]
mod platform {
    #![allow(unsafe_code, reason = "getrusage is a C API")]

    /// The peak resident size, which macOS reports in bytes; the current
    /// size isn't available this way, so both are the peak.
    pub fn memory() -> (u64, u64) {
        let mut usage = std::mem::MaybeUninit::<libc::rusage>::zeroed();
        if unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) } != 0 {
            return (0, 0);
        }
        let peak = unsafe { usage.assume_init() }.ru_maxrss as u64;
        (peak, peak)
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod platform {
    pub fn memory() -> (u64, u64) {
        (0, 0)
    }
}
