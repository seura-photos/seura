# Quick Look thumbnails for cloud-only photos (M2 slice 6b)

- **On macOS, cloud-only photos now show real thumbnails** instead of the placeholder, as on Windows: Quick Look asks the sync service for its preview, the one Finder shows. Seura asks for the thumbnail only, never the file's icon (LIB-9).
- **Written blind, checked twice.** Claude can't build macOS code on Nevada's PC, so CI found two type errors and a lint before it compiled. A test asks Quick Look for a thumbnail of an ordinary JPEG, so CI now checks the request works, not just that it compiles.
- **The real check was on Nevada's Mac**, by the Mac Claude session, with Nevada's go-ahead: all 5,997 photos in his OneDrive Pictures folder were cloud-only. Four of them got thumbnails and stayed in the cloud (the dataless flag still set afterwards). It found them by their file flags only, never opening a photo.
- **Cold starts are slow:** OneDrive's first two answers took 3.4 and 6.1 s, later ones under a second. Claude raised the wait from 10 to 30 s, since a miss leaves the placeholder until the next launch.
