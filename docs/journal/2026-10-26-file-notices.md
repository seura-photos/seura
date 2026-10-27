# Notices about files (M3 slice 7b)

- **FIL-3's Editor part moves to M4.** Claude Code suggested it in slice 7a's pull request, and Nevada agreed: "Keep yours or load theirs?" only applies while a photo is open in the Editor, which is development-only until M4. The Library side of FIL-3 was already done. The M3 plan now says so, and M4's plan will carry it.
- **The notice bar now also says, for one selected photo,** when its edit file has edits Seura can't show, from a newer version or another app (FIL-5), and when another program has changed the photo since its edits were made (FIL-7), in flow 11's words: "This photo was changed by another program; edits may not line up."
- **Only enabled operations Seura doesn't render count for FIL-5.** A disabled one isn't shown by any app, and unknown fields and extensions don't change how the photo looks, so they're kept without a notice.
- **FIL-7 checks only photos with edits.** Tags and a rating don't depend on the pixels. A cloud-only photo isn't checked, since fingerprinting would download it (LIB-9).
- **Fingerprinting reads the whole photo the first time,** so it runs off the UI's thread and without holding the index's lock (a new `fingerprint_shared`, like the change path's). The result is cached while the photo is unchanged, so selecting it again is quick.
- **Only one selected photo is checked,** since fingerprinting many photos at once could take a long time. Several selected photos show only the FIL-4 notice, which needs no reading.
- **The Viewer doesn't show these notices yet.** Flow 11 places the bar in the main area. Whether the Viewer should show them too is a question for a design session.
