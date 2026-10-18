# Seura Photos — guide for Claude Code

Seura Photos is a cross-platform (Windows and macOS) desktop photo organizer and non-destructive editor, built with Tauri: a Rust core and a TypeScript web UI. It is meant for real users, not a demo: correctness, safety of the user's files, and simplicity matter more than speed of delivery.

## Read before working

- `docs/product-brief.md`: purpose, users, guiding principles. Every change must fit the principles.
- `docs/functional-spec.md`: v1 requirements with IDs (LIB-1, EDT-3, ...). Implement against these; reference IDs in tests and commit messages.
- `docs/decisions/`: decision records. Do not reverse a recorded decision silently. If a change conflicts with one, stop and raise it; if the user agrees, add a new record that supersedes the old.
- The edit file format is specified in the separate `seura-photos/edit-format` repository (`spec/edit-format.md`, `schema/`). The app must conform to it exactly.

## Rules that protect user data

These are not negotiable, and tests should enforce them:

1. Never modify, move, rename, or copy a user's original photo. The only exception is BRW-9 (move to Trash on explicit, confirmed request).
2. Never write anywhere except: the edit file beside a photo, `Documents/Seura Photos/`, a cache location the user chose, and export destinations the user chose. Nothing in hidden app-data folders.
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

## Status

Design phase. The Tauri project has not been scaffolded yet; build, test, and lint commands will be added here when it is.
