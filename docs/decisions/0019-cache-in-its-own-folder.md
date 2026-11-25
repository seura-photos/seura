# 0019. A cache location gets a folder of its own

Status: Accepted · 2026-11-24 · Refines SET-1 and SET-2's cache location; works with decision 0018

## Context

The Welcome screen lets the user store thumbnails somewhere else (SET-1), and Settings will let them move the cache (SET-2). Until now, the folder the user picked became the cache itself. Seura's `thumbs` folder, `index.bin`, `README.txt` and, on Windows, the web view's `WebView` folder went straight into it, beside whatever else it held. Someone who picked their Desktop or a drive's top level got Seura's files mixed with their own.

That matters more now that the cache can be cleared, and is deleted directly (decision 0018). Clearing and measuring the cache already touch only Seura's own entries, by name. Even so, a folder shared with the user's files is easy to misread, and hard to delete whole.

## Decision

When the user picks a place for the cache, Seura makes a folder of its own there, `Seura Photos Cache`, and keeps the cache in it. This applies to the Welcome screen and to Settings' Move…. If the picked folder is already named `Seura Photos Cache`, it's used as it is, so moving back to an earlier one doesn't nest a second.

Caches already set to a picked folder stay where they are. Clearing or measuring one still touches only Seura's own entries in it. Moving one copies only those entries.

Nevada's decision, 2026-11-24, raised by Claude Code while planning M7 slice 3b.

## Consequences

- Seura's cache is always a folder that holds nothing else, unless an older one was set up before this decision.
- The cache's README, which explains the folder to anyone who opens it, sits in a folder that's obviously Seura's.
- Deleting the folder by hand is safe, as the README says.
