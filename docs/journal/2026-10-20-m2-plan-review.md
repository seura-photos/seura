# Reviewing the M2 plan, and moving Seura's folder out of Documents

Claude Code drafted the M2 work plan (PR #15) and listed ten questions for a design session. Claude reviewed it there and recommended an answer for each; Nevada decided.

- **The big one: Seura's own folder moves out of Documents** (decision 0015). Claude Code's question was only where to suggest the cache when Documents is cloud-synced. Nevada pointed out how often Documents is synced: Windows pushes hard for OneDrive backup (he had to refuse it several times on his own PC), and macOS offers iCloud for Desktop and Documents. He asked to reconsider the location as a whole, with nothing in or under a hidden folder.
  - Claude's analysis: none of Seura's files should sync. The cache is huge and machine-specific; the settings file holds library paths that differ between machines, so syncing it would make two computers fight over it.
  - The app's install folder doesn't work (hidden or not writable on Windows; a signed bundle on macOS), and AppData and `~/Library` are hidden.
  - Chosen: a visible `Seura Photos` folder directly in the home folder. Not synced by default, not hidden, and on macOS it avoids the permission prompt for Documents. Seura names it on the Welcome screen and in Settings, since fewer people browse their home folder.
  - Nevada asked for this decision and its reasoning to be in the next devlog post.
- **Toolbar controls appear only as their features are built**, not as disabled buttons.
- **The M1 Editor is hidden in installed builds** during M2, which writes nothing; it stays available in development.
- **Thumbnails: 96 to 256 px, default 160**, which keeps the cached 512 px thumbnails sharp on 2× screens.
- **One sort order for the whole app**, as Picasa did.
- **HEIC and JPEG pairs are both shown**, as separate files.
- **The Viewer shows edits**, like the thumbnails.
- **Performance:** measured on Nevada's machines and CI, aiming for about half the target time; a reference-class laptop in the M7 beta.
- **Mockups in the repository** (`docs/ux/mockups/`), so slices can be checked against them.
- **GitHub issues per slice**, under a GitHub milestone, now that the repositories are public and others may follow along.
- Claude also asked for two additions to slice 1: UI text kept outside the code from the start (ACC-5), and the system text size and reduced-motion settings (ACC-3).
