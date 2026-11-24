# 2026-11-24: Filling in a photo's size later (M7 slice 8b, #264)

- **Nevada said to start #264,** Claude Code's suggestion for the first fix before the beta.
- **The fault was in the caller, not the edit file library.** `fill_in_photo` already kept an existing fingerprint and added only what was missing. But Seura only called it when the fingerprint was missing, so a file written with a fingerprint and no size never got one. It's now called when either is missing.
- **A guard Claude Code added:** if the file's fingerprint isn't the photo's any more, the photo has changed since (FIL-7), and the size decoded now may not be its own. So nothing is filled in then, and the existing notice handles it. A test holds this.
- **Sizes are learned sooner too.** The Viewer and Review now tell the index a photo's upright size when they decode it, and it's saved with the index's other details (#99). In Nevada's run on 2026-11-12, the four photos enhanced before their thumbnails were made would have had their size from the start.
- **The 51 files in Nevada's library** get their size the next time each is changed in Seura. Nothing goes through the library to fill them in, since a change Nevada didn't ask for would write files beside their photos.
- **No check on the Mac:** the change is the same on both platforms, and CI runs its tests on macOS. Nevada offered the Mac session; Claude Code kept it for #217, which is about the Mac.
