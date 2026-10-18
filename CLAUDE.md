# Seura Photos — guide for Claude Code

Seura Photos is a cross-platform (Windows and macOS) desktop photo organizer and non-destructive editor, built with Tauri: a Rust core and a TypeScript web UI. It is meant for real users, not a demo: correctness, safety of the user's files, and simplicity matter more than speed of delivery.

## Read before working

- `docs/product-brief.md`: purpose, users, guiding principles. Every change must fit the principles.
- `docs/functional-spec.md`: v1 requirements with IDs (LIB-1, EDT-3, ...). Implement against these; reference IDs in tests and commit messages.
- `docs/technical-design.md`: architecture, crate layout, libraries, rendering pipeline, and testing strategy. Follow it; propose changes rather than diverging.
- `docs/milestone-plan.md` and the current milestone's work plan (`docs/m1-plan.md`): what to build next, in what order, and when it's done.
- `docs/decisions/`: decision records. Do not reverse a recorded decision silently. If a change conflicts with one, stop and raise it; if the user agrees, add a new record that supersedes the old.
- The edit file format is specified in the separate `seura-photos/edit-format` repository (`spec/edit-format.md`, `schema/`). The app must conform to it exactly.

## Rules that protect user data

These are not negotiable, and tests should enforce them:

1. Never modify, move, rename, or copy a user's original photo. The only exception is BRW-9 (move to Trash on explicit, confirmed request).
2. Seura's own data is written only to: the edit file beside a photo, `Documents/Seura Photos/`, a cache location the user chose, and export destinations the user chose. Never in hidden app-data folders. Data the operating system or the web view runtime keeps on its own is minimized, documented in decision 0013, and removed by the uninstaller where possible. The UI never relies on browser storage (the web view runs incognito).
3. Write edit files atomically: temporary file in the same folder, then rename.
4. Never overwrite an edit file the app couldn't parse. Preserve unknown operations, fields, and extensions exactly when saving.
5. Deletion always goes to the system Trash or Recycle Bin, never permanent.
6. No telemetry. Network use only for update checks and, later, user-enabled AI.

## Working conventions

- Keep code platform-neutral. Platform-specific code goes behind a small trait or module using `#[cfg(target_os = "...")]`.
- Edit operations are implemented once, in WGSL shaders shared by preview and export (decision 0004). Don't add a second implementation of any operation.
- Edits are settings rendered in a fixed order, never a replayed sequence (decision 0003).
- Prefer small, reviewable changes: one requirement or slice at a time, with tests.
- UI text lives outside code (ACC-5) so it can be translated later.

## Project journal and devlog

The project is built in the open, with a public devlog on the website (`seura-photos/seura-photos.github.io`).

- While working, add short dated notes to `docs/journal/` when a decision is made, something fails or surprises, or a mistake is caught. Say who suggested or decided what. See `docs/journal/README.md`.
- At the end of each milestone, draft a devlog post in the website repository following its CLAUDE.md, with `draft: true`. Never publish a post Nevada hasn't approved.

## Build, test, and lint

Clone with `git clone --recurse-submodules`, or run `git submodule update --init` after a plain clone. The Rust toolchain is pinned in `rust-toolchain.toml`. Install the tools once with `cargo install --locked tauri-cli@2.12.1 cargo-deny@0.20.2`, then `npm ci` in `ui/`.

Run from the repository root:

| Task | Command |
| --- | --- |
| Run the app | `cargo tauri dev` |
| Rust format check | `cargo fmt --all --check` (fix with `cargo fmt --all`) |
| Rust lint | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| Rust tests | `cargo test --workspace --locked` |
| Licenses and advisories | `cargo deny check` |
| Check where the app writes (rule 2) | build with `npm run tauri -- build --debug --no-bundle` in `ui/` (macOS: `--bundles app`), then `python3 scripts/app_data_audit.py` |

Run from `ui/`:

| Task | Command |
| --- | --- |
| Format check | `npm run format:check` (fix with `npm run format`) |
| Lint | `npm run lint` |
| Typecheck and build | `npm run build` |
| Tests | `npm test` |
| UI licenses | `npm run licenses` |
| Regenerate `src/bindings.ts` after changing a command | `npm run bindings` |
| Regenerate app icons after changing `design/brand/app-icon*.svg` | `npm run icons` |

CI (`.github/workflows/ci.yml`) runs all of these on Windows and macOS for every pull request.

Adding a command: write it in `src-tauri/src/commands.rs` with `#[tauri::command]` and `#[specta::specta]`, add it to `collect_commands!` in `src-tauri/src/lib.rs` and to the list in `src-tauri/build.rs`, grant `allow-<command-name>` in `src-tauri/capabilities/main-window.json`, then run `npm run bindings`.

Dependencies: use versions at least two weeks old, check licenses first (decision 0012), and keep `ui/package.json` pinned exactly.
