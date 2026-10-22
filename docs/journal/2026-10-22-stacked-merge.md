# A stacked pull request that merged into the wrong branch

- **Slice 7c (#46) was stacked on slice 8 (#45)**, because both changed the Viewer, with #45's branch as its base. Claude Code told Nevada that GitHub would move #46 onto `main` once #45's branch was deleted, and suggested merging #45 first. But #45's branch was still there when Nevada merged #46, so #46 merged into it, not into `main`, and Claude Code then deleted that branch as usual.
- **Caught while rebasing slice 9 (#47):** its Viewer tests were the older set, without 7c's. Nothing was lost: the reviewed commit was put onto `main` unchanged as #48, its files checked identical to what #46 had merged, and #47 rebased on top of it.
- **The lesson, from Claude Code:** don't stack pull requests on each other's branches. When two slices touch the same files, open the second from `main` and rebase it after the first merges. That's a little rebasing for Claude, rather than a merge order for Nevada to get right.
