# 0010. An in-memory index with a snapshot file, not a database

Status: Accepted, subject to the first milestone's benchmarks · 2026-10-17

## Context

The library needs fast browsing, filtering, and search across up to 100,000 photos (PRF-1, PRF-7, FND-3). The edit files beside each photo are the source of truth, so the index is a rebuildable cache. The project owner prefers to avoid SQLite unless performance requires it.

## Decision

Keep the index in memory, with secondary maps for folders, tags, and search, and persist it as a single binary snapshot file (`bincode`) in the cache folder, written atomically. Load it at launch before anything else; reconcile with the disk in the background.

Estimated memory: about 30 MB for 100,000 photos. Searching 10,000 records in memory takes well under a millisecond.

## Benchmark gate

The walking skeleton measures launch time, memory, query time, and snapshot write time on generated 10,000 and 100,000 photo libraries. If PRF-1, PRF-7, or FND-3 is missed, the index moves to SQLite (`rusqlite`, MIT) behind the same interface, recorded in a new decision that supersedes this one.

## Alternatives considered

- **SQLite from the start.** Proven and queryable, but an extra dependency and schema to maintain, for a cache whose queries are simple.
- **One cache file per photo.** Simple, but slow to load at launch on large libraries.

## Consequences

- No database dependency or migrations in v1.
- The whole index is rewritten on each snapshot; at 100,000 photos that's tens of megabytes, so snapshots are throttled rather than written on every change.
