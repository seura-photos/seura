# 2026-11-24: Two new advisories, ignored until their fixes are two weeks old (#268)

- **The Licenses and advisories job failed on #267,** the Node 26 PR, though nothing it changed was involved. Two RustSec advisories had come out since main's last run, so every pull request would have failed the same way:
  - RUSTSEC-2026-0592, a stack overflow in `url` 2.5.8's `Url::origin()` on deeply nested `blob:` URLs, fixed in 2.5.9 (2026-11-20);
  - RUSTSEC-2026-0594, private cookie nonces that can repeat after a fork in `cookie` 0.18.2, fixed in 0.18.3 (2026-11-23).
- **Both fixes are under two weeks old,** so decision 0012's rule doesn't allow them yet: `url` 2.5.9 from 2026-12-04, and `cookie` 0.18.3 from 2026-12-07.
- **Claude Code's reading of the exposure:** very small. Both crates come only through Tauri. Seura's web view loads only its own pages, Seura uses no cookies, and its process doesn't fork that way.
- **Nevada's decision, on Claude Code's suggestion:** keep the rule, and ignore both advisories in `deny.toml`, with a reason and the date for each. The other choice was upgrading now, as a standing exception for security fixes, which would have needed a decision record. Nevada asked that this be revisited once the time has passed. #268 holds the dates and the steps.
