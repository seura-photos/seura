# "Damaged" on the Mac: an unsigned app, not a broken one

- **Nevada installed build 0.1.1 on his Mac** from the disk image, and macOS refused to open it as "damaged", not with the expected warning about an unidentified developer.
- **The cause:** the app had no signature beyond the one the linker adds automatically, and macOS on Apple Silicon calls a downloaded app like that damaged. Claude Code's release notes had also said to open it the first time with right-click → Open, which recent macOS no longer allows.
- **For his copy:** removing the "downloaded" mark (`xattr -dr com.apple.quarantine`) let it open.
- **The fix:** ad-hoc signing (`signingIdentity: "-"`), free and done during the build. The Mac Claude session built it and simulated a download: macOS now shows its "Not Opened" dialog ("Apple could not verify…"), which System Settings → Privacy & Security → Open Anyway gets past, and Nevada read it on screen. The system log showed the difference: 0.1.1 carried only the linker's signature, under the binary's name, while the new build is a sealed bundle under Seura's own identifier.
- **Real signing and notarization**, which open without any warning, need an Apple Developer account and stay planned for the beta (M7). The installer question for Windows is separate (#70).
