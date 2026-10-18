# 0005. App data in Documents, keys in the OS credential store

Status: Accepted · 2026-10-17

## Context

Desktop apps usually keep settings and caches in hidden locations (AppData on Windows, Library/Application Support on macOS). Seura Photos' "nothing hidden" principle means users should be able to see, back up, and delete everything the app writes. API keys for AI providers are the exception: a key in a plain file can leak if the folder is synced or shared.

## Decision

- Settings, presets, and (by default) the cache live in a visible folder, `Documents/Seura Photos`, which the app always checks first. If the user moves the cache, the settings file records where.
- If `Documents` is synced to a cloud drive, the app suggests a non-synced location for the cache.
- API keys, when AI arrives, are stored in the operating system's credential store (Windows Credential Manager, macOS Keychain) through the Rust `keyring` crate, and removed by cleanup.
- The cache is never the source of truth and is documented as safe to delete.

## Alternatives considered

- **Standard hidden app-data folders.** Conventional, but invisible to the user.
- **A small pointer file in the hidden settings location.** Workable, but unnecessary once Documents is treated as the fixed home.
- **Keys in the visible settings file.** Simple, but a security risk.

## Consequences

- Users can find and remove everything; cleanup and uninstall can list exactly what exists.
- The app writes into the user's Documents folder, which some users may find unexpected; the first-run screen explains it.
