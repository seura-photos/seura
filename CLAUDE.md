# Seura Photos — guide for Claude Code

Seura Photos is a cross-platform (Windows and macOS) desktop photo organizer and non-destructive editor, built with Tauri: a Rust core and a TypeScript web UI. It is meant for real users, not a demo: correctness, safety of the user's files, and simplicity matter more than speed of delivery.

## Read before working

- `docs/product-brief.md`: purpose, users, guiding principles. Every change must fit the principles.
- `docs/functional-spec.md`: v1 requirements with IDs (LIB-1, EDT-3, ...). Implement against these; reference IDs in tests and commit messages.
- `docs/technical-design.md`: architecture, crate layout, libraries, rendering pipeline, and testing strategy. Follow it; propose changes rather than diverging.
- `docs/milestone-plan.md` and the current milestone's work plan (`docs/m1-plan.md`): what to build next, in what order, and when it's done.
- `docs/decisions/`: decision records. Do not reverse a recorded decision silently. If a change conflicts with one, stop and raise it; if the user agrees, add a new record that supersedes the old.
- The edit file format is specified in the separate `seura-photos/edit-format` repository (`spec/edit-format.md`, `schema/`). The app must conform to it exactly.

## Who changes what

- **Product and design documents** (the brief, functional spec, UX documents, visual design, technical design) and the edit format are written in Nevada's design sessions with Claude in the Claude app, and arrive as pull requests.
- **Claude Code** implements them. When the implementation shows a gap, a contradiction, or a requirement that can't be met, raise it with Nevada rather than changing the meaning of a document. Small factual corrections that implementation settled (a library version, a measured number, a detail the design left open and the code decided) may go in Claude Code's own pull requests, said plainly in the description.
- **Milestone work plans** (`docs/m2-plan.md` and later): Claude Code drafts them from the milestone plan and the specs, and Nevada reviews them in the pull request. Questions about behavior or design that come up while planning go to a design session.
- Decision records can be written by either, as now.

## Rules that protect user data

These are not negotiable, and tests should enforce them:

1. Never modify, move, rename, or copy a user's original photo. The only exception is BRW-9 (move to Trash on explicit, confirmed request).
2. Seura's own data is written only to: the edit file beside a photo, the `Seura Photos` folder in the user's home folder (decision 0015), a cache location the user chose, and export destinations the user chose. Never in hidden app-data folders. Data the operating system or the web view runtime keeps on its own is minimized, documented in decision 0013, and removed by the uninstaller where possible. The UI never relies on browser storage (the web view runs incognito).
3. Write edit files atomically: temporary file in the same folder, then rename.
4. Never overwrite an edit file the app couldn't parse. Preserve unknown operations, fields, and extensions exactly when saving.
5. Deletion of a user's photo or of an edit file always goes to the system Trash or Recycle Bin, never permanent. Where a drive has none (a USB stick, a memory card, a network share), a photo is left as it is. Seura deletes a file directly only when nothing in it can be lost (decision 0017): a temporary file it created during a save that failed; an edit file that holds nothing (FIL-1), checked again just before; and an unreadable edit file once an identical copy is safely in Seura's folder (FIL-4). Leftover files Seura didn't create in the current operation, such as temporaries from a crash, go to the Trash, or are left as they are where there is none. Tests enforce that no other file is ever deleted directly.
6. No telemetry. Network use only for update checks and, later, user-enabled AI.

## Working conventions

- Keep code platform-neutral. Platform-specific code goes behind a small trait or module using `#[cfg(target_os = "...")]`.
- Edit operations are implemented once, in WGSL shaders shared by preview and export (decision 0004). Don't add a second implementation of any operation.
- Edits are settings rendered in a fixed order, never a replayed sequence (decision 0003).
- Prefer small, reviewable changes: one requirement or slice at a time, with tests.
- UI text lives outside code (ACC-5) so it can be translated later.

## Project journal and devlog

The project is built in the open, with a public devlog on the website (https://seuraphotos.app, repository `seura-photos/seura-photos.github.io`).

- While working, add short dated notes to `docs/journal/` when a decision is made, something fails or surprises, or a mistake is caught. Say who suggested or decided what. The design sessions add notes there too, so read the journal's recent entries before drafting a post. See `docs/journal/README.md`.
- At the end of each milestone, draft a devlog post in the website repository following its CLAUDE.md, with `draft: true`. Never publish a post Nevada hasn't approved.

## Build, test, and lint

Clone with `git clone --recurse-submodules`, or run `git submodule update --init` after a plain clone. The Rust toolchain is pinned in `rust-toolchain.toml` and Node in `.nvmrc` (24.21.0, LTS; with nvm, run `nvm use` in the repository). Install the tools once with `cargo install --locked tauri-cli@2.12.1 cargo-deny@0.20.2`, then `npm ci` in `ui/`.

On Windows, HEIC photos are decoded by Microsoft's HEIF Image Extensions and HEVC Video Extensions from the Microsoft Store (decision 0014); without them the HEIC tests are skipped. `SEURA_HEIF_DECODER=none` makes the app behave as if they were missing.

**Faster development builds (optional, local).** Unoptimized, `cargo tauri dev` takes many seconds to open a photo (13 s for 12 megapixels, against 0.23 s optimized). To optimize the libraries and Seura's image code in your development builds only, create `.cargo/config.toml` in the repository (git-ignored, so CI and other clones aren't affected) with:

```toml
[profile.dev.package."*"]
opt-level = 3

[profile.dev.package.seura-image]
opt-level = 3
```

The first build after adding it takes longer; later ones don't. It's kept out of `Cargo.toml` because it made CI's uncached builds take twice as long.

`SEURA_PREVIEW=webgl2` makes the Editor's preview use WebGL2 even where the web view has WebGPU, to check the fallback that macOS 13 to 15 use.

`SEURA_WELCOME=1` shows the first-run Welcome screen on a computer that already has settings; nothing is saved unless you press Get started.

`SEURA_DATA=<folder>` uses that folder as Seura's folder (settings, cache) instead of `Seura Photos` in the home folder, so measuring or testing never touches a real library. `SEURA_MEASURE=1` makes a build measure the performance targets on itself, write `measurement.json` to Seura's folder, and quit; `scripts/measure.py` sets both up.

`SEURA_LOCALE=pseudo` pseudo-translates every string, lengthened about 40% and accented, to check that translations would fit (ACC-7). Text lives in Project Fluent catalogs: `ui/src/messages/en.ftl` and `src-tauri/messages/en.ftl`.

Run from the repository root:

| Task | Command |
| --- | --- |
| Run the app | `cargo tauri dev` |
| Rust format check | `cargo fmt --all --check` (fix with `cargo fmt --all`) |
| Rust lint | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| Rust tests | `cargo test --workspace --locked` |
| Licenses and advisories | `cargo deny check` |
| Index benchmarks (decision 0010; fail if a target is missed) | `cargo bench -p seura-library --bench index` |
| Check where the app writes (rule 2) | build with `npm run tauri -- build --debug --no-bundle` in `ui/` (macOS: `--bundles app`), then `python3 scripts/app_data_audit.py` |
| Check the preview renders like export, in the real web view | build as above, then `python3 scripts/render_check.py` |
| Measure the performance targets (PRF-1 to PRF-5, PRF-7) | build for release (`npm run tauri -- build --no-bundle` in `ui/`; macOS: `--bundles app`), then `python3 scripts/measure.py` (about 5 GB of generated test photos in `target/measure`; `--small` for a quick check). In CI: the Measure workflow, run by hand |

Run from `ui/`:

| Task | Command |
| --- | --- |
| Format check | `npm run format:check` (fix with `npm run format`) |
| Lint | `npm run lint` |
| Typecheck and build | `npm run build` |
| Tests | `npm test` |
| UI licenses | `npm run licenses` |
| Regenerate `src/bindings.ts` after changing a command | `npm run bindings` |
| Regenerate `src/preview/generated/` after changing a shader | `npm run shaders` |
| Regenerate `src/styles/tokens.css` after changing `design/tokens.json` | `npm run tokens` |
| Regenerate app icons after changing `design/brand/app-icon*.svg` | `npm run icons` |

CI (`.github/workflows/ci.yml`) runs all of these on Windows and macOS for every pull request.

Adding a command: write it in `src-tauri/src/commands.rs` with `#[tauri::command]` and `#[specta::specta]`, add it to `collect_commands!` in `src-tauri/src/lib.rs` and to the list in `src-tauri/build.rs`, grant `allow-<command-name>` in `src-tauri/capabilities/main-window.json`, then run `npm run bindings`.

Dependencies: use versions at least two weeks old, check licenses first (decision 0012), and keep `ui/package.json` pinned exactly.
