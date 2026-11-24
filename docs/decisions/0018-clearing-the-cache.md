# 0018. Clearing the cache deletes it directly

Status: Accepted · 2026-11-24 · Adds to CLAUDE.md rule 5's list of direct deletions (decision 0017)

## Context

CLN-4 said everything Clean up removes goes to the Recycle Bin or Trash. CLAUDE.md rule 5 lists the only files Seura may delete directly, and tests enforce that list.

The cache is different from everything else Clean up removes. It holds thumbnails, previews, the index snapshot, and the web view's data: all Seura's own, all rebuilt automatically, and promised as "safe to delete" (decision 0013). In a large library it runs to gigabytes. Moving that to the Recycle Bin fills it with files nobody will restore, and on Windows a batch larger than the Recycle Bin's limit makes Windows ask to delete it permanently anyway.

## Decision

Clearing the cache, from Settings or Clean up, deletes it directly. Edit files, settings, and presets, which hold the user's work, still go to the Recycle Bin or Trash. Rule 5's list of direct deletions gains Seura's own cache.

## Consequences

- CLAUDE.md rule 5 and CLN-4 say so, and the tests that enforce rule 5 allow deleting inside the cache folder, and only there.
- The confirmation for clearing the cache says it's rebuilt automatically, rather than that it can be restored.
