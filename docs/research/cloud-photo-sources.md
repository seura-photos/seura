# Cloud-synced photos: services, clients, and what Seura can see

Status: research for LIB-9 · 2026-10-18

Many people's phone photos reach their computer through a cloud service's sync client, not a cable. Those folders usually hold *cloud-only* files: they appear in the folder with their name, size, and date, but their contents stay in the cloud until something reads them. Reading one, even its first few bytes, downloads the whole file. A photo app that treats such a folder like any other could download a whole library just by scanning it.

This note covers which services matter for Seura's users, which clients exist on which platform, and whether Seura can tell a cloud-only file from a local one. Nevada asked for it on 2026-10-18 after pointing out that his own phone photos arrive this way. Per his direction, it covers the four services phone users are likely to use: OneDrive, iCloud, Google, and Dropbox.

## The matrix

| Service | Phone camera backup | Windows client | Mac client | Cloud-only files detectable? |
| --- | --- | --- | --- | --- |
| **OneDrive** | iPhone and Android apps (Camera Upload). Samsung Gallery's direct sync to OneDrive ended on 2026-09-30; Microsoft points those users to the OneDrive app's camera backup instead. | Built into Windows. Files On-Demand, using Windows' Cloud Files API. | OneDrive app, using macOS File Provider (`~/Library/CloudStorage/OneDrive-…`). | **Yes, both platforms.** Windows: a per-file attribute (`RECALL_ON_DATA_ACCESS`). Mac: the "dataless" flag. **Verified on Nevada's PC.** |
| **iCloud Photos** | iPhone (default for most iPhone users). | iCloud for Windows: an *iCloud Photos* entry in File Explorer, photos downloaded on demand. iCloud for Windows uses the Cloud Files API. | **Not a folder.** iCloud Photos lives inside the Photos app's library, a package whose layout is private and changes between macOS versions. | Windows: **likely**, through the same Cloud Files attributes, but not yet verified for iCloud Photos specifically. Mac: not applicable; see below. |
| **iCloud Drive** | Only if the user saves photos into Files, which is uncommon. | iCloud for Windows, Cloud Files API. | Built in; files are dataless until downloaded. | **Yes, both platforms**, by the same attribute and flag. |
| **Google Photos** | Android (default for most Android users); also an iPhone app. | **No sync client.** The only ways to get the photos onto a PC are manual: downloading from photos.google.com, or Google Takeout. | **None.** | Not applicable: there's no synced folder. |
| **Google Drive** | Not used for camera backup by Google's apps; only if the user or another app puts photos there. | Drive for desktop. By default it *streams* through a virtual drive (usually `G:`) with its own file system driver, which reports itself as FAT32. Not the Cloud Files API. | Drive for desktop, using File Provider (`~/Library/CloudStorage/GoogleDrive-…`). | Mac: **yes** (dataless flag). Windows: **not per file.** The virtual drive has no documented "online-only" attribute, so Seura must treat everything on it as cloud-only. The drive is identifiable: Drive for desktop records its mount point in the registry (`HKCU\Software\Google\DriveFS\Share`). |
| **Dropbox** | iPhone and Android apps (Camera Uploads, including on the free plan if the desktop app is installed). | Dropbox desktop app with online-only files. | Dropbox app, using File Provider (`~/Library/CloudStorage/Dropbox`). | Mac: **yes** (dataless flag). Windows: **likely** through the Cloud Files attributes; Dropbox doesn't document its implementation, so this needs a check on a real install. |

**Which matter most for phone photos:**

1. **iCloud Photos**, for iPhone users. Very common, but on a Mac the photos are inside the Photos library, not in a folder (see "Two gaps" below).
2. **Google Photos**, for Android users. Very common, but it has no desktop sync at all.
3. **OneDrive**, on both kinds of phone. It's the one that reliably produces a synced folder of phone photos on both Windows and Mac, and it's built into Windows. Nevada's own library arrives this way: his OneDrive camera roll holds 3,947 files, 3,885 of them cloud-only, 21.4 GB in all.
4. **Dropbox**, less common for photos, but its camera uploads produce a synced folder like OneDrive's.

Photos from dedicated cameras usually arrive by card reader or cable straight into a local folder, so they're unaffected.

## What Seura can rely on

- **Windows Cloud Files API** (OneDrive, iCloud for Windows, very likely Dropbox): every cloud-only file carries `FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS`. Listing a folder and reading attributes, sizes, and dates doesn't download anything; reading the contents does. Any service built on this API is covered without special handling, including ones not listed here.
- **macOS File Provider and iCloud Drive** (OneDrive, Google Drive, Dropbox, iCloud Drive): cloud-only files carry the `SF_DATALESS` flag, with the same rule: metadata is free, contents download.
- **Google Drive's virtual drive on Windows** is the exception: no per-file signal, so the whole drive is treated as cloud-only.

**Checked on Nevada's PC (2026-10-18)**, with `cargo run --release -p seura-library --example cloud_check`: Seura's detection matched Windows' count exactly (3,810 cloud-only photos, plus 75 cloud-only non-photo files the scanner ignores), a full scan took 231 ms, and the folder's 3,885 cloud-only files were all still cloud-only afterwards. Nothing was downloaded.

A public bug report against another tool shows what goes wrong otherwise: a directory scan on Windows with iCloud for Windows installed triggered large downloads, because it read files without checking for the attribute first.

## Downloading a cloud-only photo

When the user opens a cloud-only photo, Seura reads it, which downloads it (LIB-9). How that download behaves was measured on 2026-10-22 with OneDrive on both platforms, on two cloud-only JPEGs of about 9 MB each from Nevada's library, by reading only:

| | Windows (Cloud Files API) | macOS 26.6.2 (File Provider) |
| --- | --- | --- |
| When it downloads | While the file is being opened: 3.0 s for 9.4 MB. Reading afterwards took 7 ms. | During the first read: about 3.5 s for 9.4 MB. Opening is instant. |
| Progress visible to the app | None. The size on disk stays at 0, then jumps to the full size. | None. The size on disk stays at 0, then jumps to the full size (sampled every 200 ms). |
| The reader abandons it after 1 s | The download finishes anyway, about a second later. | The download stops, and the file stays cloud-only. |

So neither platform tells a reading app how far a download has got, and abandoning one means different things on each. LIB-9 was settled accordingly (#44): an indicator with the photo's size, no progress bar, no cancel button; moving on just stops waiting, and Seura relies on neither platform's behavior.

Measured by Claude Code on Nevada's PC (PowerShell, opening with `FileShare.ReadWrite` and timing each 1 MB read, and killing a separate process mid-open), and by the Mac Claude session (Python, `os.open` and `os.read`, sampling `st_flags` and `st_blocks` from a second thread). On the Mac the abandoned reader was killed about 1 s after it started; Python starts in about 30 ms, so it was almost certainly blocked in the read, though the script didn't prove it.

## Two gaps outside Seura's reach

These aren't about downloading; they're about where the photos are at all.

1. **iPhone users on a Mac.** iCloud Photos keeps its photos inside the Photos app's library, which Seura doesn't read: it works on folders, and the library's layout is private to Apple and changes between versions. With "Optimize Mac Storage" on, most originals aren't even on the Mac. Such users would have to export from Photos, or move to a folder-based service. A read-only source built on Apple's photo library interface (PhotoKit) would be possible later. **Decided by Nevada (2026-10-18): not supported in v1.**
2. **Android users with Google Photos.** There is no desktop sync client, and since August 2026 Drive for desktop no longer connects to Google Photos at all. Getting the photos onto a computer is manual. The practical advice for these users is OneDrive's (or Dropbox's) camera backup on the phone. **Decided by Nevada (2026-10-18):** the user guide's topic on cloud-synced folders includes a brief summary of what each service offers, including this.

## Sources

- Microsoft: [Changes to Samsung Gallery sync and OneDrive](https://support.microsoft.com/en-us/onedrive/changes-to-samsung-gallery-sync-and-onedrive); [Build a cloud sync engine that supports placeholder files](https://learn.microsoft.com/en-us/windows/win32/cfapi/build-a-cloud-file-sync-engine).
- Apple: [Use iCloud Photos on your PC](https://support.apple.com/en-us/108994); [iCloud for Windows user guide](https://support.apple.com/guide/icloud-windows/toc).
- Computerworld: [iCloud for Windows: what is it](https://www.computerworld.com/article/1668721/icloud-for-windows-what-is-it-and-how-do-you-use-it.html) (iCloud for Windows uses the Cloud Files API).
- Google: [Use Drive for desktop on macOS](https://support.google.com/drive/answer/12178485) (File Provider, `~/Library/CloudStorage`); [Stream and mirror files with Drive for desktop](https://support.google.com/drive/answer/13401938); [Drive sync FAQ for admins](https://knowledge.workspace.google.com/admin/drive/drive-sync-faq-for-admins) (the virtual drive appears as a FAT file system); [Advanced Drive for desktop configuration](https://support.google.com/a/answer/7644837) (registry settings).
- TechRepublic: [Google Drive stops automatic photo backups to Google Photos on desktop](https://www.techrepublic.com/article/news-google-drive-google-photos-desktop-backup-change/).
- Dropbox: [Expected changes with Dropbox for macOS on File Provider](https://help.dropbox.com/installs/macos-faqs); [Troubleshoot camera uploads](https://help.dropbox.com/installs-integrations/photos/camera-uploads-fix); [Dropbox Basic](https://www.dropbox.com/basic).
- TidBITS: [Apple's File Provider forces Mac cloud storage changes](https://tidbits.com/2023/03/10/apples-file-provider-forces-mac-cloud-storage-changes/).
- GitHub: [hermes-agent issue #97898](https://github.com/NousResearch/hermes-agent/issues/97898) (directory scans hydrating iCloud and OneDrive placeholders).
- Mac Photos library layout: [MacMost, Photos storage options](https://macmost.com/mac-photos-app-storage-options.html).

## Still to verify

- iCloud for Windows: that iCloud Photos files carry the Cloud Files attribute (needs a PC with iCloud for Windows signed in).
- Dropbox on Windows: the same.
- Google Drive on Windows: reading the mount point from the registry, and whether the system thumbnail works there without downloading.
- ~~System thumbnails for cloud-only files~~: checked. The Windows shell on Nevada's OneDrive camera roll (M1 slice 4), and macOS Quick Look on four of the 5,997 cloud-only photos in his OneDrive Pictures folder (M2 slice 6b, 2026-10-22): all got thumbnails and stayed cloud-only.
