# Downloading cloud-only photos: what the systems actually allow

- LIB-9 promised that opening a cloud-only photo would show "progress" and offer "a way to cancel". While building the Viewer (M2 slice 7c), Claude Code and Nevada measured what Windows and macOS really do, with OneDrive on both: Claude Code on Nevada's PC, and a Claude session on his Mac. Two cloud-only JPEGs of about 9 MB each.
- **Windows:** the download happens while the file is being opened (about 3 seconds); the size on disk stays at 0, then jumps to full. Killing the reader after a second didn't stop it: the download finished anyway.
- **Mac:** opening is instant and the download happens on the first read (about 3.5 seconds), again with no visible progress. Killing the reader did stop it; the photo stayed in the cloud.
- So neither platform tells an app how far a download has got, and "cancel" means different things on each.
- **Nevada's proposal:** show that the photo is downloading, and replace the indicator with the photo when it's done; no progress bar and no cancel button. Claude agreed in the design session, and added:
  - the photo's size in the message ("Downloading 9.4 MB from the cloud…"), so a slow connection doesn't look like a hang;
  - the cached thumbnail, enlarged and blurred, behind the indicator;
  - the user can always move on, which is how a download is abandoned; Seura just stops waiting, the same on both platforms, and makes no promise about what the system does next;
  - a plain message with Try again when a download fails;
  - the Viewer's look-ahead skips cloud-only photos, so browsing never downloads one.
- The same indicator applies in the Editor and in export, where a cloud-only photo shows as downloading and the summary lists any that failed.
- A good example of measuring before building: the spec's original wording couldn't have been met on either platform.
