# 0014. HEIC decoding: the operating system's decoders only

Status: Accepted · 2026-10-19 · Supersedes the Windows fallback in [0011](0011-heic-decoding.md)

## Context

Decision 0011 chose the operating system's HEIC decoders (ImageIO on macOS, WIC on Windows with Microsoft's HEIF and HEVC extensions), and a bundled libheif with libde265 on Windows PCs without those extensions. M1 slice 4 built all three and tested them against each other.

Building and testing the fallback showed what it costs and how often it's needed:

- **Patents.** libde265 is an HEVC decoder without a patent license. 0011 accepted the risk as low for a free project; it isn't zero, and it's the one part of Seura that carries it.
- **Weight.** The fallback needs vcpkg on every Windows development PC and in CI, two DLLs in the installer with LGPL notices and source links, delay-loaded linking in four build scripts, and care with vcpkg's defaults (its libheif build includes the GPL x265 encoder, and it sends telemetry).
- **HEIC is uncommon on Windows.** iPhone photos usually reach a PC as JPEG: OneDrive's camera upload, iCloud for Windows, and the iPhone's USB transfer all convert by default, and getting real HEICs onto Nevada's PC took effort. HEIC turns up on Windows mainly through settings that keep originals, Google Photos downloads, and libraries shared with a Mac.
- **The system decoders agree.** On real iPhone photos, WIC and libheif gave the same result. Microsoft's extensions are preinstalled on many PCs (Nevada's has both) and are a free or very cheap Store install otherwise.

## Decision

1. **macOS:** ImageIO, as before.
2. **Windows:** WIC, with Microsoft's HEIF Image Extensions and HEVC Video Extensions. No bundled HEVC decoder.
3. **Without them,** a HEIC photo's tile says it needs Microsoft's HEIF and HEVC extensions, rather than "Can't show this photo". Seura checks which one is missing, so a damaged file isn't blamed on a missing extension. The user guide explains how to install them (0011's step 4 stands).
4. **Developer switch:** `SEURA_HEIF_DECODER=none` makes Seura behave as if the extensions were missing, so that path can be checked on a PC that has them.

Nevada decided this, at Claude's recommendation. libheif can be added back later if users need it; the history of slice 4 has a working version.

## Consequences

- No HEVC decoder ships with Seura, and no LGPL code; the patent question in 0011 no longer applies to Seura's own files.
- No vcpkg: development on Windows needs only the usual tools, CI is faster, and the installer has two fewer DLLs and notices.
- Some Windows users see a message instead of their HEIC photos until they install the extensions.
- CI's Windows runners may lack the extensions, so there the HEIC tests are skipped; the Mac runs them on every pull request, and Nevada's PC runs them locally. Seura's own reading of the container's orientation and color (decision 0011) is unchanged and tested on both.
