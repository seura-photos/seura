# 0010. An in-memory index with a snapshot file, not a database

Status: Accepted, confirmed by the M1 benchmarks · 2026-10-17 · Amended 2026-10-18 (snapshot encoding)

## Context

The library needs fast browsing, filtering, and search across up to 100,000 photos (PRF-1, PRF-7, FND-3). The edit files beside each photo are the source of truth, so the index is a rebuildable cache. The project owner prefers to avoid SQLite unless performance requires it.

## Decision

Keep the index in memory, with secondary maps for folders, tags, and search, and persist it as a single binary snapshot file (`postcard`; originally `bincode`, see the amendment below) in the cache folder, written atomically. Load it at launch before anything else; reconcile with the disk in the background.

Estimated memory: about 30 MB for 100,000 photos. Searching 10,000 records in memory takes well under a millisecond.

## Benchmark gate

The walking skeleton measures launch time, memory, query time, and snapshot write time on generated 10,000 and 100,000 photo libraries. If PRF-1, PRF-7, or FND-3 is missed, the index moves to SQLite (`rusqlite`, MIT) behind the same interface, recorded in a new decision that supersedes this one.

## Benchmark result (2026-10-18)

Every target passed, on Nevada's Windows PC and MacBook Pro and on CI's Windows and macOS runners, by a factor of at least 5: at 100,000 photos the index takes 34 MB (340 bytes per photo, close to the estimate above), loads from its snapshot in 36 to 95 ms, lists a 1,000-photo folder in microseconds, and answers a search keystroke in under 8 ms. The snapshot is 7.5 MB, smaller than the "tens of megabytes" expected below, and saves in under 70 ms. The in-memory index stays; SQLite isn't needed. Details, budgets, and every machine's numbers: [`docs/benchmarks/m1-index.md`](../benchmarks/m1-index.md). The benchmark runs in CI on every pull request and fails if a target is missed.

## Amendment: snapshot encoding (2026-10-18)

`bincode` became unmaintained in December 2025 (RustSec advisory RUSTSEC-2025-0141), so the snapshot is encoded with `postcard` (MIT or Apache-2.0) instead: serde-based like the rest of the project, with a documented, stable encoding. Found by Claude when adding the dependency in M1 slice 3. The decision itself is unchanged.

## Alternatives considered

- **SQLite from the start.** Proven and queryable, but an extra dependency and schema to maintain, for a cache whose queries are simple.
- **One cache file per photo.** Simple, but slow to load at launch on large libraries.

## Consequences

- No database dependency or migrations in v1.
- The whole index is rewritten on each snapshot; at 100,000 photos that's tens of megabytes, so snapshots are throttled rather than written on every change.
