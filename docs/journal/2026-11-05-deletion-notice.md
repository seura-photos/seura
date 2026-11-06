# 2026-11-05: Explaining a cloud service's deletion notice

M5 slice 8 (#200; FIL-1). This follows the design session's answer in #202: a message, not a dialog, once per service, with no setting. The details below are Claude Code's.

- **Nevada checked #229 on Windows and the Mac** before merging it.
- **What's recorded:** the Trash function the app passes to its writes now notes each edit file it moves. Four kinds of write use it: the Library's changes, the Library's undo and redo, the Editor's changes, and the Editor's undo and redo. Enhance only adds edits, so it never empties a file and isn't wrapped. Delete and Start over have their own confirmations and are left as they are. A file on a drive with no Trash isn't moved, so it isn't explained.
- **Once per service, remembered in `settings.json`:** after the batch, the first service among the moved files that isn't in `explainedServices` is explained and saved there. A batch spanning two new services explains one, and the other waits for its own next time: one message per batch. If the settings file can't be saved, the service is explained again after a restart, which does no harm.
- **The words** name what removed the files: a rating, a tag, or something else (an edit reset, undo, redo, revert or paste). They have a one-photo form and a several-photos form. On the Mac, the one-photo rating wording is FIL-1's sentence word for word, and on Windows it says Recycle Bin.
- **The action's own message joins the explanation** rather than replacing it, whichever arrives first. The explanation comes as an event while the change finishes, and the action's message comes when the command returns. Messages within one and a half seconds of each other show as one.
- **Not tried in a real synced folder on Windows:** a scratch folder there would sync to Nevada's OneDrive account. The plan's "done when" is Nevada on the Mac with OneDrive.
