# Seura's folder moves to the home folder (M2 slice 1a)

Claude Code built the first part of M2 slice 1, following decision 0015.

- **Seura's files now live in `~/Seura Photos`**: `settings.json`, and the cache (with the web view's data) in `Cache/`. On Windows the home folder comes from `FOLDERID_Profile`; Claude checked that Tauri's home folder resolves through it, as the decision says.
- **The settings file holds the library folders** (SET-3). Before, they lived only in the index snapshot inside the cache, so clearing the cache would have forgotten the library. A snapshot whose folders don't match the settings is now set aside, which costs a rescan but never shows a wrong library.
- **Carried over once:** on the first run of the new build on Nevada's PC, his M1 library folder was read from the old snapshot in `Documents/Seura Photos/Cache` into the new settings. A fingerprint taken before the run confirmed the old snapshot was left byte for byte as it was; the old folder is his to delete.
- **A settings file that can't be read is never overwritten**: the app runs on defaults and keeps changes in memory. Fields from a newer version are kept when saving.
- **The app-data audit** now allows only `~/Seura Photos`, and also checks `Documents/Seura Photos`, so writing to the old place again would fail CI.
