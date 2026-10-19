# Dropping libheif: HEIC through the operating system only

A decision for the next devlog post (Nevada asked for it to be included). Decision [0014](../decisions/0014-heic-system-decoders-only.md).

- **The question came from Nevada,** after slice 4 was merged: would removing the libheif fallback on Windows be erring on the side of caution, and simpler? Claude agreed and recommended it; Nevada decided, noting it can be added back later.
- **Why:** libde265, the HEVC decoder under libheif, has no patent license; Microsoft's and Apple's decoders do. The fallback also brought vcpkg to every Windows setup and to CI, two DLLs and LGPL notices in the installer, delay-loaded linking in four build scripts, and two traps (vcpkg's default libheif includes a GPL encoder, and vcpkg sends telemetry).
- **What made it easy to let go:** slice 4 showed that HEIC is uncommon on Windows (OneDrive, iCloud for Windows, and USB transfers all hand over JPEGs by default), and that on real iPhone photos the two Windows decoders gave the same picture. Microsoft's extensions are already on Nevada's PC (HEIF 1.2.48, HEVC 2.5.33), and Explorer's own HEIC thumbnails come from them too.
- **What users without the extensions see:** a tile saying "Needs Microsoft's HEIF and HEVC extensions" instead of "Can't show this photo". Claude made Seura check which extension is missing before blaming one, since a WIC error alone doesn't say whether the file or the system is at fault.
- **The cost:** Nevada can no longer test HEIC on Windows without the extensions installed, and CI's Windows runners may skip the HEIC tests. `SEURA_HEIF_DECODER=none` shows the missing-extension path on a PC that has them; the Mac runs the HEIC tests on every pull request.
- Five files gone, and about 80 fewer lines of code and build configuration.
