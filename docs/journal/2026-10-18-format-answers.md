# 2026-10-18: The format questions, answered in the format itself

Slice 2 raised three questions the edit format didn't settle (see `2026-10-18-edit-files.md`). Nevada took them to the Claude Desktop conversation where the project started; it updated `seura-photos/edit-format` (commit `1065a9a`) and FIL-1 on `main`, and Nevada passed the answers back to Claude Code. The questions came from implementing the format, which is the point of building the reference implementation early: the spec's decisions log now credits both additions to it.

- **Newer minor versions.** The format now publishes a reader schema, `schema/1.x-reader.json`, generated from `1.0.json`: it checks every field 1.0 defines and accepts the rest. A reader must never lower `formatVersion` when saving. `seura-edit` dropped its own relaxed schema and uses the published one, so the app and `check.py` can't drift apart.
- **Redo across restarts.** History records can now be marked `"undone": true`; undone records stay at the end until a new change discards them, and redo has its own consistency check. `seura-edit` dropped its session-only redo list: undo marks the record, redo clears the mark, and both survive closing the app (EDT-9 as written).
- **A change Seura doesn't understand** (a kind of history change from a later minor version) now fails the consistency check, so it's never half-applied. The format's new valid example exercises this with a 1.1 file that changes a field 1.0 doesn't know.
- **Empty edit files go to the Trash** (FIL-1 updated). Still M3 work; a note in `seura-edit`'s file code says there is deliberately no delete, so it isn't added later with a permanent `remove_file`.

Claude also aligned how `formatVersion` is read with `check.py` (no leading zeros, so `"1.01"` is invalid) and confirmed each of the four new invalid files is rejected for its own reason, not by accident.

Two small things noticed along the way:

- The format commit accidentally includes a compiled Python cache file (`tools/__pycache__/make_reader_schema.cpython-313.pyc`). It should be removed there and `__pycache__/` added to that repository's `.gitignore`.
- When a save fails, `seura-edit` deletes its own half-written temporary file permanently rather than sending it to the Trash. Claude reads rule 5 as covering the user's files and Seura's saved data, not a temporary file that never became one; noted here in case Nevada reads it differently.

## Follow-up: rule 5 clarified

- Claude Desktop removed the stray `.pyc` from edit-format and added a `.gitignore` (commit `e26d7e0`); the submodule now points there.
- Nevada (through Claude Desktop) agreed that deleting Seura's own half-written temporary file is correct, and clarified rule 5 in CLAUDE.md: deleting a user's photo or an edit file always goes to the Trash; the one exception is a temporary file Seura created during a save that failed; leftovers Seura didn't create in the current operation, such as temporaries from a crash, go to the Trash.
- Claude added two tests that a failed save leaves no temporary file and doesn't touch the existing edit file: one in `seura-edit` that makes only the final rename fail, after the temporary file is fully written, and one in the data-safety suite where the operating system refuses the save (a read-only edit file on Windows, which turned out to block replacing it; a read-only folder on macOS). Removing the clean-up line made both fail, naming the leftover `.tmp` file.
