# 2026-10-18 to 19: Decoding and thumbnails (M1 slice 4)

Claude built `seura-image`, the thumbnail cache and workers, and the first real screen: a folder list and a thumbnail grid. Nevada decided three things up front: a developer switch to test the libheif fallback on his one Windows PC, building libheif now as decision 0011 says, and Google's libwebp for lossy WebP thumbnails.

## Caught before they shipped

- **vcpkg's libheif turns on the x265 *encoder* by default, which is GPL.** A plain install would have pulled GPL code into Seura against decision 0012. Seura installs `libheif[core]`, decoding only (libde265, LGPL, dynamically linked).
- **`tauri-plugin-dialog` depends on the file-system plugin**, which decision 0009 keeps out. The folder dialog uses `rfd` from Rust instead.
- **libwebp looked vulnerable and wasn't.** The `webp` crate's bundled libwebp says "1.3.1", a version affected by the 2023 WebP exploit (CVE-2023-4863), but the fix is in the source; RustSec agrees. Claude checked the code rather than trusting either label.
- **vcpkg sends telemetry by default.** It's built with metrics off, here and in CI.

## HEIC, with test files made on Nevada's Mac

- Real HEIC files were needed, and Seura can't write HEIC. The Claude Code session on Nevada's Mac generated four tiny ones with Apple's ImageIO (upright, turned, mirrored, Display P3) and sent them over Remote Control as text. One arrived 25 bytes short; the SHA-256 checksums caught it, and it was resent as numbered hex lines with a CRC each, which arrived intact. They're committed as test fixtures.
- **Windows' decoder (WIC) applies HEIC's rotation itself.** A third-party crate said it didn't, so Claude's first version applied it again and turned photos twice. The Mac session had warned about exactly this risk; the test files showed it at once.
- **The mirror axis:** Apple writes `imir` axis 1 for a left-right mirror, and libheif reads it that way, though the standard's wording suggests the opposite. The Mac session flagged the doubt; libheif settled it.
- **WIC's colors drift** on HEIC files whose color box says "unspecified" (green comes out 215 instead of 255); libheif and files with a profile are exact. Recorded as a known deviation in a test, to check against Nevada's real iPhone photos.
- The macOS decoder (ImageIO, through Rust's objc2 crates) was written on Windows, where it can't be compiled. The Mac session built and tested it: it worked the first time, all four files right.

## Cloud-only photos (LIB-9)

- Thumbnails for cloud-only photos come from the Windows shell, which uses OneDrive's own previews. Claude tested on five of Nevada's cloud-only photos at a time, counting cloud-only files before and after. First result: no thumbnails at all. Changing one thing, the COM threading model (single-threaded apartment instead of multithreaded), gave real thumbnails in about 20 ms each, and nothing was downloaded.
- macOS still shows a placeholder for cloud-only photos; Quick Look thumbnails are follow-up work, since they need asynchronous Objective-C callbacks that can only be compiled through the Mac session.

## Smaller things

- `heif.dll` is delay-loaded and loaded by hand from beside the app (or vcpkg, for development), so a missing DLL disables HEIC instead of stopping the app from starting.
- CI builds libheif with vcpkg at the commit pinned in `.github/vcpkg-commit` and caches it: about five minutes the first time, seconds after.
- Claude tried the app with Windows UI Automation: seeded a test folder, clicked through the folder list, and captured the window. The grid, thumbnails, and all four HEIC test files showed correctly, and the app wrote only inside its cache.
- Nevada's PC still has Node 24.13; the project needs 24.21. One install used `--engine-strict=false` as a stopgap.

## Mistakes caught

- A test that "checked" P3 color conversion couldn't fail: fully saturated green looks the same converted or not. Rewritten with mid-tone colors and expected values computed independently in Python; checked that it fails with conversion turned off.
- A scripted edit hit an escape error on a Windows path, and the commit after it ran anyway, committing a file with a message describing changes that weren't there. Fixed with a follow-up commit that says so.
- A cleanup command was blocked by a safety check because of how a path variable expanded; nothing was removed, and the cleanup was redone with literal paths.

## Nevada's checks

- Nokia's HEIF conformance files (downloaded for testing, not committed) showed thumbnails in the app on both machines: Windows through WIC, the Mac through ImageIO.
- Getting real iPhone HEICs turned out to be the hard part: Nevada's iPhone is set to take JPEGs ("Most Compatible"), and OneDrive's camera upload, iCloud for Windows, and iPhone USB transfers all convert to JPEG by default. Nevada asked whether HEIC on Windows is worth supporting at all; the research says it's uncommon there but real, mostly through Google Photos, settings that keep originals, and libraries shared with a Mac. Kept for now; noted as a possible decision.
- On the Mac, `npm ci` was needed after the pull that added React Virtual, or Vite can't resolve it.
- Nevada made two real iPhone HEICs (iPhone 15 Pro, camera switched to High Efficiency) and put them in the git-ignored `test-libraries/`. A temporary probe compared Windows' two decoders on them, printing numbers only: same size and orientation, colors within one level on average, 99% of pixels within 8 levels. **The WIC color drift doesn't affect real iPhone photos**, which carry an ICC profile; it only hits files whose color box is "unspecified" with no profile. Both photos carry GPS location and an HDR gain map (ignored; the standard picture is shown), so they stay local rather than in the public repository (Claude's recommendation).
- Nevada noticed thumbnails of real HEICs were slow to appear, especially on Windows. `cargo tauri dev` builds Rust without optimizations, which matters most for decoding and resizing; the first-scan speed target (PRF-3) is measured with release builds in M2, along with faster thumbnail paths (smaller decodes, embedded previews).
