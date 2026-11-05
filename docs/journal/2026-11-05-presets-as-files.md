# 2026-11-05: Presets as files

M5 slice 6 (#211; PRE-3, PRE-5): saving a preset, the Presets folder as a list, and renaming and deleting.

- **Two questions the wireframes left open went to Nevada,** since Nevada was there. Claude Code suggested an answer for each, and Nevada took both:
  - **Rename and Delete are in each preset's right-click menu,** which the context-menu key and Shift+F10 also open, as for photos. A ⋯ button on each row, or a Manage presets dialog, were the alternatives.
  - **A preset's name is its file's name.** The name is stored twice, in the file name and in the `name` field, and the two differ once someone renames the file by hand. The file name is what the folder shows, and it's always unique. When Seura saves or renames, it keeps the two the same, and it never rewrites a file just to make them match.
- **Names are cleaned for both systems (Claude Code's rules).** Characters Windows or the Mac can't hold in a file name become `-`. Leading and trailing dots and spaces go, since a leading dot hides a file on the Mac and Windows drops trailing ones. A Windows device name such as `CON` gets a `_`, and names are cut at 100 characters. When the name changed, the message says so.
- **A name in use is compared without letter case,** since both systems' usual drives ignore it. Saving under one asks first, and the button becomes Replace. A replace keeps the existing preset's name. A preset that can't be read is never replaced, renamed or rewritten, only listed with "Can't be read" and offered for deletion (rule 4).
- **Renaming moves the file, then updates its `name` field.** Writing a new file and removing the old one would have meant deleting a file directly, which rule 5 doesn't allow. Renaming onto another preset's name is refused rather than asked, since that would replace a preset.
- **A group with nothing in it can't be chosen when saving.** This differs from paste, where a chosen empty group clears the target's (Nevada's decision today). The edit format says applying a preset leaves alone the stages it doesn't hold, so a preset with an empty stage would mean nothing. The save dialog offers the same three groups and defaults as paste, with those groups turned off.
- **The folder is watched** with the library's own watcher, from launch if the folder exists, or from the first save that makes it. Before then, Seura writes nothing to its folder.
- **Checked in the real app on Windows** through the web view's debug port, on a scratch Seura folder:
  - saving, the name-in-use question and replacing;
  - a preset copied in from the format's examples, and a broken one, appearing in about two seconds;
  - renaming, including the refusal of another preset's name;
  - deleting to the Recycle Bin, with Cancel focused.

  The saved file passed the format's `check.py`, and the broken file was unchanged at the end. The deleted test preset is in Nevada's Recycle Bin.
- **A mistake caught at once:** one edit to the save dialog inserted a long run of empty event handlers into the file. Claude Code spotted it in the edit's result and replaced the block before building.
