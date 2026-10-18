# 0011. HEIC decoding: operating system first, libheif as the Windows fallback

Status: Proposed (needs the owner's decision) · 2026-10-17

## Context

HEIC is the default photo format on iPhones, so supporting it is essential (LIB-8). HEIC images are compressed with HEVC (H.265), which raises two issues:

- **Licenses.** The common open-source decoder, libheif, is LGPL-licensed, as is libde265, the HEVC decoder it uses. LGPL libraries can be used from an Apache-licensed app when dynamically linked, with their source and license notices provided. (To be confirmed against each project's license files when the dependency is added.)
- **Patents.** HEVC is covered by patent pools. Operating system vendors license this for their own decoders. Open-source apps that bundle an HEVC decoder (GIMP, digiKam, darktable, and others) generally do so without a license, and the risk for a free, non-commercial project has historically been low, but it isn't zero.

The operating systems already include licensed decoders:

- **macOS:** ImageIO decodes HEIC natively, with correct color handling.
- **Windows:** the WIC codec needs Microsoft's free "HEIF Image Extensions" plus "HEVC Video Extensions," which is preinstalled on many PCs and otherwise a small paid download from the Microsoft Store.

## Proposed decision

1. **macOS:** use ImageIO. No bundled HEVC decoder.
2. **Windows:** use WIC when the HEVC extension is installed.
3. **Windows fallback:** when it isn't, use a bundled libheif with libde265, dynamically linked (not the static "embedded" build), with license notices and source links in the About box and installer.

An alternative stance is to skip step 3 and instead tell Windows users without the extension how to install it. That avoids bundling an HEVC decoder entirely, at the cost of a worse first experience for some users.

## Consequences

- Most users get the operating system's licensed decoder.
- The Windows build ships two extra DLLs, and the third-party notices must be maintained.
- Decoding behavior can differ slightly between decoders; the reference-image tests include HEIC samples on both platforms.

This is not legal advice; if the project grows or is distributed commercially, the patent question should be revisited with someone qualified.
