# 2026-11-09: The new-photos banner (M6 slice 6, REV-1)

- **Scans now say which photos are new,** and a small tracker holds them in memory until additions have been quiet for a minute. At launch they're offered at once. Nothing is saved, so after the next launch they're no longer new, as question 9 says.
- **No banner when there's nothing to compare with:** on first run, after clearing the cache, or when the snapshot is of other folders. The launch's scan is only counted when a snapshot of the same folders loaded.
- **A folder just added doesn't count,** because only the launch's scan and the watcher feed the tracker.
- **A rename isn't a new photo,** by two checks, Claude Code's addition to the plan's "recognized rename": a photo that got its edit file back, and, for a photo with no edit file, one gone in the same scan with the same size and date. Without the second, renaming a photo in Explorer would have offered it as new.
- **Exports are left out when additions settle, not when they're seen.** Seura only learns an export's final name once it's written, and the watcher may see the file first. Checking a minute later removes that race.
- **More photos can join a banner that's already up,** once they've settled. After Not now, later arrivals are a new offer.
- **Cancelling the question about the cloud keeps the banner,** Claude Code's reading of "until Review or Not now", since no review started.
- **Checked in a scratch build on Windows, on throwaway photos:** no banner on first run; two photos added with Seura closed gave "2 new photos in photos" and a review of exactly those, oldest first; two copied while open gave the banner 61 seconds after the last one; an export into the library gave none after 75 seconds; and clearing the cache gave none.
