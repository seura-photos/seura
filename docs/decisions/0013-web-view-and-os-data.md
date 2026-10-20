# 0013. Data the web view and operating system keep on their own

Status: Accepted · 2026-10-18 · Paths updated by [0015](0015-app-data-in-home-folder.md): read `Documents/Seura Photos` below as `~/Seura Photos`.

Clarifies CLAUDE.md rule 2 and decision 0005; supersedes neither.

## Context

Rule 2 says Seura writes only beside photos (edit files), to `Documents/Seura Photos`, to the cache location the user chose, and to export destinations, and never to hidden app-data folders. The first run of the scaffolded app (M1 slice 1) broke that without any Seura code asking for it: the web view runtime kept its own data, 79 MB in 286 files, in `%LOCALAPPDATA%\app.seuraphotos.desktop\EBWebView` on Windows. On macOS, WKWebView keeps website data under `~/Library` by default.

That data belongs to the web view (WebView2 on Windows, WKWebView on macOS), not to Seura. The two platforms differ in how far it can be controlled:

- **WebView2** lets the app choose the data folder (`data_directory`) and run InPrivate, which keeps no profile data between runs. It still needs a data folder.
- **WKWebView** offers no data folder setting. It can run with a non-persistent data store, which keeps website data in memory only.

## Decision

1. **Seura's own data never goes in hidden folders.** No exceptions: settings, presets, the cache, and anything Seura writes deliberately follow rule 2.
2. **Data the operating system or web view runtime keeps on its own is minimized, documented here, and removed by the uninstaller where possible.**
3. **Both platforms run the web view incognito** (Tauri's `WebviewWindowBuilder::incognito(true)`): InPrivate in WebView2, `WKWebsiteDataStore.nonPersistentDataStore` in WKWebView. As a result, the UI never relies on browser storage (`localStorage`, IndexedDB, cookies); anything that must persist goes through a Rust command to a visible file.
4. **On Windows, WebView2's data folder is inside the cache**: `<cache>/WebView`, by default `Documents/Seura Photos/Cache/WebView`. It's visible, moves with the cache, and is covered by the cache's "safe to delete" promise.
5. **CI enforces it.** `scripts/app_data_audit.py` runs the built app on both platforms (the real `.app` bundle on macOS, so the system sees the true identifier), quits it, and fails if any file named for the app (product name, binary, or identifier) appeared outside `Documents/Seura Photos`. The same script can be run by hand on a real machine.

## What is still written, as measured

Measured 2026-10-18 on Tauri 2.12.1 with a 15-second run of the app: on Nevada's Windows 11 PC, on his MacBook Pro (macOS 26.6.2, Apple Silicon, the bundled `.app`), and in CI on `windows-latest` and `macos-latest`:

| Where | Windows | macOS |
| --- | --- | --- |
| `Documents/Seura Photos/Cache/WebView` | 132 files, about 2 to 6 MB (WebView2's own browser data, empty of website data because of InPrivate) | Not used |
| Hidden app-data folders (`%LOCALAPPDATA%`, `%APPDATA%`, `AppData\LocalLow`; `~/Library`) | Nothing | Nothing named for the app, on the MacBook Pro and in CI |

Things that are outside this decision because they aren't data:

- **The installed program.** Tauri's per-user NSIS installer puts the program itself in `%LOCALAPPDATA%\Seura Photos`. That's where per-user Windows programs live; it's removed by uninstalling. On macOS the app lives wherever the user put the `.app`.
- **System-wide records** the operating system keeps about every app, in its own files rather than ones named for Seura: Spotlight's index, the macOS "reopen at login" list, Windows' recent-programs lists. Seura can't control or remove these.

## Watch list

Not seen in the measurements, but known to appear for some apps, and caught by the CI audit if they ever do:

- **macOS:** `~/Library/Saved Application State/app.seuraphotos.desktop.savedState` (window restoration when "Close windows when quitting an application" is off), `~/Library/Preferences/app.seuraphotos.desktop.plist` (`NSUserDefaults`), and `~/Library/Caches`, `~/Library/HTTPStorages`, or `~/Library/WebKit` folders named for the identifier (likely once the updater makes network requests, SET-4).
- **Windows:** `%LOCALAPPDATA%\app.seuraphotos.desktop`, if WebView2 or a Tauri plugin ever falls back to its default folder.

If one appears, minimize it if the platform allows, then add it to the table above with what removes it.

## Removal

- **Windows:** Tauri's NSIS uninstaller already offers to delete `%APPDATA%\app.seuraphotos.desktop` and `%LOCALAPPDATA%\app.seuraphotos.desktop`. That checkbox stays, as a safety net, since normally nothing is there. The WebView folder goes with the cache under CLN-2 and CLN-5.
- **macOS:** there is no uninstaller; users drag the app to the Trash. If anything from the watch list ever appears, the full reset in Cleanup (CLN-2) removes it too.

## Alternatives considered

- **Accept the defaults and document them.** Simplest, but 79 MB of hidden data after a single launch is exactly what "nothing hidden" exists to prevent.
- **Put WebView2's data in `Documents/Seura Photos` outside the cache.** Visible, but then it isn't covered by the cache's "safe to delete, rebuilt automatically" promise, and it wouldn't move when the user moves the cache off a synced Documents folder (SET-1).
- **Rely on incognito alone on Windows.** InPrivate still needs a data folder, so the default `%LOCALAPPDATA%` location would remain.

## Consequences

- No browser storage in the UI. UI state that must survive a restart (such as the thumbnail size, BRW-3) is stored through Rust in the settings file.
- When settings let the user move the cache (M2), the WebView folder moves with it. WebView2 doesn't support data folders on network drives, so if the chosen cache location is on one, the WebView folder needs a local fallback inside `Documents/Seura Photos`. To be handled with SET-3.
- Clearing the cache while the app is running can't remove the WebView folder, because WebView2 holds its files open. Clearing it at the next launch, before the web view starts, is part of CLN-2.
- The CI audit also runs every time a dependency or Tauri version changes, so a new hidden write is caught in the pull request that introduces it.
