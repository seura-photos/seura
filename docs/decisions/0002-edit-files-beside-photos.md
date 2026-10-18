# 0002. Edits stored in visible sidecar files beside each photo

Status: Accepted · 2026-10-17

## Context

Non-destructive editing needs somewhere to store edits. Picasa used a hidden `.picasa.ini` per folder; Lightroom and digiKam use a central database; darktable uses sidecar files. A core principle of Seura Photos is "nothing hidden," and users' work should outlive the app.

## Decision

Each photo with edits, tags, or a rating gets a visible JSON file in the same folder, named after the full photo file name: `IMG_1234.jpg` → `IMG_1234.jpg.seura.json`. The format is published as an open specification in [seura-photos/edit-format](https://github.com/seura-photos/edit-format).

Edit files are the source of truth. The thumbnail cache and search index are derived from them and can be deleted and rebuilt at any time.

## Alternatives considered

- **Central database.** Fast to query, but invisible, tied to one app, and lost if the photos move to another computer.
- **One hidden file per folder.** Fewer files, but hidden, and a single corrupt file loses a whole folder's edits.
- **Writing into the photo's own metadata.** Modifies the original, which breaks the first principle.

## Consequences

- Edits travel with photos when folders are copied, backed up, or synced.
- Photos in read-only folders can be viewed and exported but not edited or tagged.
- Folders gain visible extra files; the app provides cleanup to remove them.
- Writes must be atomic (temporary file, then rename) to survive crashes.
