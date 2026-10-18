# 0001. Tauri with a Rust core and a web UI

Status: Accepted · 2026-10-13

## Context

Seura Photos must run on Windows and macOS from one codebase, start quickly, stay small to download, and handle heavy image work: decoding large photos, generating thousands of thumbnails, and re-rendering edits fast enough for live sliders.

## Decision

Use Tauri: a Rust backend for file scanning, metadata, thumbnails, edit files, and export, with the UI in HTML, CSS, and TypeScript running in the operating system's web view (WebView2 on Windows, WKWebView on macOS).

## Alternatives considered

- **Electron.** Familiar, but ships Chromium, so downloads are 100 MB or more and memory use is high. Heavy image work would still need native code.
- **Qt (C++ or PySide6).** Mature and used by digiKam, but C++ raises the cost of every feature, and Python is hard to package cleanly and slow for pixel work.
- **Avalonia (.NET).** Capable, but a smaller ecosystem for image formats.

## Consequences

- Small, fast-starting app; nearly all code shared between platforms.
- The two web views differ slightly, so the UI must be tested on both (see 0006).
- Some imaging libraries are C or C++ (libheif, LibRaw, exiv2), used through Rust bindings whose maturity varies; each is checked in the technical design.
- Tauri's updater plugin provides signed updates from GitHub Releases without a server of our own.
