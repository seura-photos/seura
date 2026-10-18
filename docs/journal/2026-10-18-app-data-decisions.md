# 2026-10-18: Decisions on PR #1 (web view data, MPL-2.0)

Nevada reviewed the open questions from the scaffold (see `2026-10-18-scaffold.md`) and decided:

- **Web view data** (Nevada's decision): put WebView2's data folder in the cache, run the web view incognito on both platforms, and write decision [0013](../decisions/0013-web-view-and-os-data.md) to clarify CLAUDE.md rule 2: Seura's own data never goes in hidden folders; data the OS or web view runtime keeps on its own is minimized, documented, and removed by the uninstaller where possible. Rule 2 in CLAUDE.md was updated to match.
- **MPL-2.0** approved for unmodified dependencies, recorded as an amendment to [decision 0012](../decisions/0012-no-gpl-dependencies.md).
- **Identifier** stays `app.seuraphotos.desktop`.
- **Submodule** stays pinned to a commit until edit-format has a release.

## Doing it

- Claude checked the current API in the Tauri 2.12.1 and wry 0.57 sources rather than from memory: `incognito(true)` maps to WebView2's InPrivate and to WKWebView's non-persistent data store; `data_directory` is Windows-only and needs an absolute path, so the main window is now created in code (`"create": false` in `tauri.conf.json`).
- Result on Windows: nothing in AppData. WebView2's folder in `Documents/Seura Photos/Cache/WebView` holds 132 files and 2 to 6 MB on first run, down from 79 MB in AppData.
- macOS couldn't be measured on Nevada's PC, so Claude added `scripts/app_data_audit.py` and a CI step that runs the built `.app` on a macOS runner. Nothing named for the app appeared under `~/Library`. Nevada can also run the script on his Mac.
- The audit started as a measurement and became a check: it now fails CI if any file named for the app appears outside `Documents/Seura Photos`, so rule 2 is enforced on both platforms, as CLAUDE.md asks of the data rules.

## Mistakes caught along the way

- **Windows-only code broke the macOS build.** The cache-path helpers are used only inside a `#[cfg(target_os = "windows")]` block, so clippy's dead-code check failed on macOS in CI. Running clippy for macOS locally doesn't work on Windows (a dependency compiles C code), so CI is the only check for the other platform. Fixed with a scoped `expect(dead_code)` that will itself fail once slice 3 uses the helpers.
- **The first macOS audit proved less than it seemed.** It reported "nothing written", but didn't check the app had actually been running. Claude added a check that the process is alive before quitting it.
- **The audit's failure path silently didn't exist.** A scripted edit meant to add it didn't match (Python's `str.replace` doesn't complain), so the audit would have passed anything. Claude caught it by planting a probe file in `%LOCALAPPDATA%\app.seuraphotos.desktop` during a run, expecting a failure, and getting none. Lesson: test a check by making it fail.
- **The matching was too loose twice.** `seura` matched Claude's own session folder (the project path), and `ebwebview` matched Outlook's WebView2 folder. The audit now matches only the app's own names: `Seura Photos`, `seura-photos`, and the identifier.
- **`npm run tauri` didn't work from `ui/`**, because the Tauri CLI looks for its config below the current folder. Fixed by running it from the repository root.

## Slice 1 done

- Nevada ran `cargo tauri dev` on his Mac and the app skeleton launched. With CI green on Windows and macOS, slice 1's "done when" is met.

## Node pinned to 24.21.0

- Nevada asked which Node to use; his Mac had Node 25, and Claude Code there had installed 24 alongside it with nvm. Claude checked the official schedule: 25 reached end of life on 2026-06-01 (odd versions never become LTS); 24 is LTS with security fixes until April 2028; 26 becomes LTS on 2026-10-28.
- Claude recommended 24, pinned to 24.21.0 (24.22.0 is under two weeks old), in `.nvmrc`, which nvm reads and CI now uses (`node-version-file`). `engines` is `>=24.21 <25`, so npm refuses the wrong Node.
- With Node at 24.15 or later, jsdom goes back to 30.1.2, the version first chosen in the scaffold.
- Moving to Node 26 is planned for when it has been LTS for a few weeks, around the start of M2, as its own change.

## Checked on the Mac through a second Claude session

- Nevada enabled Remote Control on both machines, so the Windows session (Claude) could message a Claude Code session on his MacBook Pro. Nevada asked for it to set Node, pull, and run the checks there; it worked under the Mac's own permission prompts and reported back.
- Results on macOS 26.6.2 (Apple Silicon): Node 24.21.0; UI and Rust checks all pass; the app-data audit of the bundled `.app` found nothing named for the app under `~/Library`. Added to decision 0013 as a real-machine measurement.
- **The Mac session found a flaw in the audit:** it took about 10 minutes instead of 20 seconds, because it walked Nevada's OneDrive folder in `~/Library/CloudStorage`, where the sync client was touching 18,687 files. Its suggested fix was adopted: the audit now skips `~/Library/CloudStorage` and `~/Library/Mobile Documents` (iCloud Drive), where Seura never writes. CI runners have no cloud storage, so CI couldn't have caught this.
