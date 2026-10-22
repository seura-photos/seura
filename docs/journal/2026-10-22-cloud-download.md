# Opening a cloud-only photo (M2 slice 7c)

- **Built to the settled LIB-9 (#44).** Opening a cloud-only photo in the Viewer shows its cached thumbnail, enlarged and blurred, with a turning indicator and "Downloading 9.4 MB from the cloud…", and the photo replaces it when it's ready. No progress bar and no cancel button: neither Windows nor macOS reports a download's progress to the app reading it, as measured the day before on both machines.
- **Moving on is how a download is abandoned.** The arrow keys, the buttons, and Esc work at any time; the Viewer stops waiting and drops the result. On the Mac that stops the download; on Windows it finishes in the background. Seura promises neither.
- **A download that fails, or takes more than three minutes**, says "This photo is in the cloud, and it couldn't be downloaded. Check your connection." with Try again. Three minutes is about a 20 MB photo at 1 Mbit/s; nothing ever spins forever.
- **Caught while building it:** when the Viewer moved on from a photo still being decoded ahead, a request waiting for it would have started reading it itself, which for a cloud-only photo means downloading it again. It now gives up instead, with a test that sets up the race deliberately.
- **Asking the file, not the index:** whether a photo is still in the cloud is now checked on the file's attributes when it's opened, which reads nothing. A photo downloaded since the last scan opens at once, and the Info panel shows its details.
- **The look-ahead still skips cloud-only photos**, so browsing never downloads one the user hasn't opened.
- **M1's Editor** (development builds only) still opens a cloud-only photo without the downloading state. The Editor is rebuilt in M4, with the same state the Viewer has, as the spec now asks.
