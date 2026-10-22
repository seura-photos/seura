# Build 0.1's release workflow (M2 slice 10)

- **Agreed with Nevada:** pre-release builds come from GitHub Actions, not from a developer's machine. Pushing a tag like `v0.1.0` builds the Windows installer and the macOS disk image on GitHub's runners and attaches them to a draft pre-release; Nevada looks it over and publishes it.
- **Unsigned for now:** Windows SmartScreen warns, and the Mac app is opened the first time with right-click → Open. Signing and notarization come with the public beta (M7).
- **The version** becomes 0.1.0 (it was 0.0.0): Cargo's workspace version, which Tauri uses, and the UI's package. The workflow refuses to build a tag that doesn't match it.
- **Checked locally:** the full release build makes a 5.6 MB Windows installer; the macOS disk image is built only on a Mac, so the first tagged run is its check.
