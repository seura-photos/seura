# 0015. App data in a Seura Photos folder in the home folder

Status: Accepted · 2026-10-20 · Supersedes the location in decision 0005

## Context

Decision 0005 put settings, presets, and the default cache in `Documents/Seura Photos`, a visible folder, to keep the promise that nothing Seura owns is hidden. Planning M2 showed the flaw: Documents is very often synced to the cloud. Windows pushes hard to back Documents up to OneDrive (Nevada had to refuse it several times on his own PC), and macOS offers to sync Desktop and Documents to iCloud.

Nothing in that folder should sync:

- **The cache** (thumbnails and the index, about 500 MB per 10,000 photos, rewritten constantly) is specific to one machine and always rebuildable. Syncing it wastes bandwidth and cloud storage, and iCloud may evict it to save space, making Seura slow.
- **The settings file** holds full paths to library folders, which differ between machines. Two computers syncing one settings file would overwrite each other's library or produce conflict copies.
- **Presets** would be nice to sync, but nothing depends on it.

## Decision

Seura keeps its own data in a visible folder named `Seura Photos` directly in the user's home folder:

- Windows: `C:\Users\<name>\Seura Photos` (the profile folder, found with the Known Folder API, `FOLDERID_Profile`).
- macOS: `/Users/<name>/Seura Photos` (the home directory).

It holds `settings.json`, `Presets/`, and, by default, `Cache/`. The cache can still be moved (SET-2), and the settings file records where.

- **Not synced by default.** OneDrive's folder backup covers Desktop, Documents, and Pictures (and on some versions Music and Videos), never the profile folder itself. iCloud's option covers Desktop and Documents.
- **Visible.** No hidden folder anywhere: one level above Documents, reachable in Explorer and Finder.
- **No macOS privacy prompt.** macOS asks before an app writes to Documents, Desktop, or Downloads; the home folder itself isn't protected.
- **Shown, not left to be found.** The Welcome screen names the folder, Settings shows it with an Open folder button, Cleanup and the uninstaller remove it.
- **Still checked for sync.** First run checks whether the home folder is itself inside a synced or backed-up location (some tools sync a whole profile); if so, it suggests another place for the cache.

Unchanged from 0005: API keys go in the operating system's credential store, and the cache is never the source of truth.

## Alternatives considered

- **Keep Documents, and suggest another cache location when it's synced.** The settings file would still sync between machines, with conflicting library paths.
- **The app's install folder.** On Windows a per-user install goes under `AppData\Local\Programs` (hidden) and a machine-wide one under `Program Files` (not writable by normal users). On macOS the app is a signed bundle, and writing inside it breaks the signature.
- **Standard app-data folders** (`AppData`, `~/Library`). Hidden; rejected in 0005 and still rejected.
- **Marking the folder not to sync.** iCloud skips names ending in `.nosync`, but OneDrive has no reliable per-folder exclusion inside a backed-up folder.

## Consequences

- Fewer people ever open their home folder than Documents, so discoverability depends on Seura showing the folder, which the Welcome screen, Settings, and Cleanup do.
- CLAUDE.md rule 2, the brief, SET-1, SET-3, PRE-3, the technical design, decision 0013's paths, and the edit format's note on presets change to match.
- M1 builds wrote to `Documents/Seura Photos`. That was only on Nevada's machines; the first build with the new location may read the old settings once to carry the library over, and leaves the old folder for him to delete.
