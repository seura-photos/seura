# Seura Photos — Functional Spec (v1)

Status: approved · Last updated: 2026-10-27

This spec defines what Seura Photos v1 does, as behaviors with testable acceptance criteria. It builds on the [product brief](product-brief.md). Screen layout comes in the design phase, and the edit file format has its own specification in [seura-photos/edit-format](https://github.com/seura-photos/edit-format). Each requirement has an ID (LIB-1, EDT-3) so tests, commits, and code can reference it.

## Library

The library is the set of folders the user adds; the app works on them in place and never moves, copies, or modifies the photos.

- **LIB-1 Add a folder.** The user picks a folder with the system folder dialog; it and all its subfolders join the library. Adding a folder already inside the library does nothing and says so. Adding a parent of a library folder absorbs the child.
- **LIB-2 Folder tree.** The sidebar shows library folders and every subfolder that contains supported photos, directly or further down. Folders with no photos are hidden. Each folder shows its photo count. The tree starts with only its top level open; what the user opens stays open for the session.
  - **Collapse all:** a button in the FOLDERS heading closes every folder except those on the way to the selected one, so the selection stays in view. It's disabled when nothing else is open.
  - **Expand beneath:** Alt-click (Windows) or Option-click (macOS) on a folder's arrow opens it and everything beneath it, as in Finder; the folder's right-click menu has the same command, "Expand all beneath". From the keyboard, `*` opens the focused folder and its siblings (the ARIA tree pattern).
  - There is no command to expand the whole tree: on a large library it would make hundreds of rows.
- **LIB-3 First scan.** Thumbnails appear progressively as they are generated, the app stays usable throughout, and a progress indicator shows photos processed out of found.
- **LIB-4 Watching for changes.** While the app runs, new, changed, deleted, and renamed photos are reflected within 5 seconds. On launch, changes made while the app was closed are detected.
- **LIB-5 Remove a folder.** Removes it from the library only. No files are deleted; its cache entries are dropped; its edit files stay (removing them is Cleanup's job).
- **LIB-6 Unavailable folders.** A folder on a disconnected drive is shown as unavailable, not removed, and returns when the drive reconnects.
- **LIB-7 Skipped items.** Hidden files and folders, and system folders such as the Recycle Bin and Trash, are ignored.
- **LIB-8 Formats.** JPEG, HEIC/HEIF, and PNG are shown. Other files are ignored silently. For iPhone Live Photos, the still image is shown and its paired video ignored. When the same photo exists as both HEIC and JPEG (for example, from iCloud for Windows), both files are shown: they are separate files, each with its own edits and tags.
- **LIB-9 Cloud-only photos.** Folders synced by OneDrive, iCloud, Google Drive, or Dropbox often hold photos whose contents are still in the cloud. Adding, scanning, and browsing such a folder never downloads them: their thumbnails come from the system or the sync service, or show as a placeholder marked as in the cloud, and they sort by file date until their date taken is known (BRW-2). A cloud-only photo is downloaded only when the user opens it in the Viewer or Editor, edits it, or exports it, one photo at a time. While it downloads:
  - **The screen shows that it's downloading, and how big it is:** the photo's cached thumbnail, enlarged and slightly blurred, with an indicator and "Downloading 9.4 MB from the cloud…". There is no progress bar: neither Windows nor macOS tells an app how much of a download is done.
  - **The user is never held up.** They can move to the next or previous photo, or leave, at any time; moving on is how a download is abandoned, and there is no separate cancel button. Seura simply stops waiting. What the system then does with the download is up to it (on Windows it finishes in the background; on the Mac it stops), and Seura makes no promise either way.
  - **A failed download says so,** for example when the computer is offline: "This photo is in the cloud, and it couldn't be downloaded. Check your connection." with **Try again**. It never spins indefinitely.
  - **Nothing reads ahead:** the Viewer's look-ahead for next and previous (PRF-5) skips cloud-only photos, so browsing never downloads a photo the user hasn't opened. Tagging and rating don't download it: its edit file records the fingerprint and dimensions once the photo is read (edit format, "Photo identity"). Seura never downloads photos in bulk and never changes whether a file is kept on the device. PRF-5 applies to photos already on the device. See `docs/research/cloud-photo-sources.md`.

## Browsing

Selecting a folder or tag shows its photos in a thumbnail grid; opening one shows it full size.

- **BRW-1 Grid.** Shows the photos directly in the selected folder, not its subfolders. Thumbnails show photos with their edits applied, plus small markers for edited photos and star ratings.
- **BRW-2 Sorting.** By date taken (default; falls back to file date when the photo has none), file name, or date modified, ascending or descending. One sort order for the whole app, remembered; not per folder.
- **BRW-3 Thumbnail size.** Adjustable with a slider from 96 to 256 logical pixels per cell, 160 by default; the setting is remembered. Cached thumbnails are 512 pixels, sharp at the largest size on 2× screens (DSP-4).
- **BRW-4 Selection.** Click, Ctrl/Cmd-click, Shift-click, and Select All, following platform conventions. The number selected is always visible.
- **BRW-5 Viewer.** Double-click or Enter opens a photo full size. Arrow keys move to the next or previous photo in grid order. Zoom to fit, zoom to 100%, zoom in and out, and pan when zoomed. Esc returns to the grid at the same scroll position. A cloud-only photo downloads as LIB-9 describes. The Viewer shows the photo with its edits applied, like the thumbnails; comparing with the original is in the Editor (EDT-1).
- **BRW-6 Photo info.** A panel, always visible in the Library and Viewer, shows file name, date taken, dimensions, file size, camera and lens, exposure settings (shutter speed and ISO, plus exposure compensation when there was some), location as coordinates, tags, and rating. A photo with no date taken, such as a scan or a screenshot, shows its date modified instead, labeled as such. With several photos selected, the date is the range from the earliest to the latest, written the region's way for a range, with the parts they share not repeated: "Mar 26, 2007, 12:51 – 3:10 PM" for one day, and a single date and time when the two are the same as shown.
- **BRW-7 Correct display.** Orientation stored in the photo's metadata is honored, and embedded color profiles are applied, so iPhone photos show their true colors.
- **BRW-8 Open elsewhere.** Show in Explorer or Finder, and open in the system's default app.
- **BRW-9 Delete.** On explicit request and after confirmation, moves the selected photos and their edit files to the system Recycle Bin or Trash. Never permanent deletion. Not available in read-only folders. The confirmation says that a photo's edits, tags, and rating are in a small file beside it, which goes too, and that restoring both gets everything back. Deleting a photo in a cloud-synced folder also deletes it from the cloud and the user's other devices; the service keeps it in its own deleted items for a time. When the selection includes such photos, the confirmation says so, naming the service: "3 of these photos are in OneDrive. They'll be removed from OneDrive on all your devices too. OneDrive keeps deleted files in its own recycle bin for a while." It doesn't promise a number of days, which varies by service and plan. Photos that can't reach the Recycle Bin or Trash are left as they are, and the confirmation says which and why, so the user is never one step from a permanent deletion:
  - **Cloud-only photos:** measured on OneDrive, Windows sends them only to the service's recycle bin online, silently, and macOS refuses. A cloud-only photo is never downloaded just to delete it (LIB-9). The confirmation says to delete it from the service, or to open it first, which downloads it. Downloading them one at a time so they can go to the Trash is planned with Review's batched deletion (M6, REV-4).
  - **Drives with no Recycle Bin or Trash,** such as a USB stick, a memory card, or a network share: deleting there would be permanent, so their photos are left as they are, and the confirmation says they can be deleted in File Explorer or Finder, which warn that it's permanent. Seura itself never deletes a photo permanently, even after a warning (decision 0017). Copying photos off a memory card and clearing it are the camera's or the operating system's job; Seura works with photos where they are (product brief).
- **BRW-10 Quick actions.** Rotate left and right, Enhance, and rating are available directly in the Library (for the selection) and the Viewer, as one click or one key, without opening the Editor. In the Library, rotate and Enhance also appear on a thumbnail while it's hovered, so the pointer doesn't leave the grid (INT-5).
- **BRW-11 Undo in the Library.** Ctrl+Z undoes the last action, whatever is selected now; Shift+Ctrl+Z redoes it. An action is one change to every photo it touched, such as a rating or a tag applied to a selection. The app remembers, in memory only, the last 10 actions of the session and which photos each touched; the changes themselves are recorded only in each photo's edit file history (EDT-9). Undoing an action is the edit format's undo on each of those photos: its history record is marked undone, and the previous values are restored, so the Library and the Editor always agree on a photo's history. A photo whose latest change is no longer the one from that action, because it was changed since in the Editor or by another app, or whose history fails the edit format's consistency check, is left alone. What was undone is always said: in the Edit menu on macOS before it's chosen ("Undo Rating (5 photos)"), and as a brief notice afterwards on both platforms ("Undid rating on 5 photos", including how many photos were skipped). After a restart there is nothing to undo in the Library; each photo's own history still works in the Editor. Moving photos to the Trash (BRW-9) is not undoable; its confirmation is the safeguard.

## Review

A guided pass through a batch of photos: sort, lightly fix, and tag them one at a time, using only one-key actions. It's the main path for new photos copied from a phone or camera.

- **REV-1 Starting a review.** When a scan finds photos added since the previous launch, the Library shows a banner: "84 new photos in Summer. Review them?" A review can also be started from any folder, tag, search result, or selection.
- **REV-2 Review screen.** One photo at a time, as large as the window allows, with a progress indicator ("12 of 84") and a single row of actions beneath it. The sidebar and Info panel are hidden; the photo's file name, date, rating, and tags show compactly under the photo.
- **REV-3 Actions.** Each is one click or one key: next and previous; rate 1 to 5; rotate left and right; Enhance; add a tag (opens the tag picker); mark for deletion; and Edit, which opens the Editor and returns to the review when done.
- **REV-4 Deletion is batched.** Marking for deletion is instant and reversible during the review. Marked photos are moved to the Trash only after one confirmation at the end of the review (BRW-9 rules apply). The control, its tooltip, and the photo's marked state all say "mark" rather than "delete," and make clear that nothing is removed until the review is finished.
- **REV-5 Finishing.** At the end, a summary shows how many photos were rated, edited, tagged, and marked for deletion, with buttons to confirm the deletions and to export the photos rated at or above a chosen number of stars.
- **REV-6 Stopping early.** The user can leave a review at any time with Esc. Nothing is lost: ratings, edits, and tags are saved as they're made, and marked photos stay marked until the next review of that batch or until the user unmarks them.

## Editing

Every edit is a stored setting, never a change to pixels in the original, so any edit can be adjusted or removed later.

- **EDT-1 Edit mode.** Opened from the viewer or grid. A compare control (button, or holding a key) shows the original while held. Opening a cloud-only photo shows the same downloading state as the Viewer (LIB-9). The photo can be zoomed as in the Viewer (BRW-5): Fit, 100%, in and out, with the same toolbar controls, wheel, pinch, and + and − keys. Dragging pans, except while a tool uses the drag, such as red-eye; then holding Space and dragging pans, as do the scroll wheel and trackpad. Crop and straighten always show the whole photo; zoom returns when they're done. Holding Compare keeps the zoom and position: it shows the photo without its red-eye, tone, color, and effects edits, but framed as edited, turned, straightened, and cropped, so the two line up. Moving to another photo returns to Fit.
- **EDT-2 Crop.** Freeform, or fixed ratios: original, 1:1, 4:3, 3:2, 16:9, and 5:4, each switchable between landscape and portrait. A rule-of-thirds overlay shows while cropping. Reopening crop shows the whole photo with the current crop box, so the crop can be widened again.
- **EDT-3 Straighten.** From −45° to +45° in 0.1° steps. The crop shrinks automatically so no empty corners appear. While crop mode is open, bringing the angle back toward zero lets the crop grow again, up to the crop the user set; what's kept when the user presses Done is the crop as it is then.
- **EDT-4 Rotate and flip.** Rotate 90° left or right, and flip horizontally or vertically. Also available in the grid for a whole selection. A crop follows the photo: the same part of the photo stays in it, and a fixed ratio turns with it, so a 3:2 landscape crop becomes 2:3 portrait. Flips mirror what's on screen, and a flip also mirrors the straighten angle. Rotating or flipping returns the zoom to Fit, since the photo changes shape.
- **EDT-5 Adjustments.** Light: brightness, contrast, highlights, and shadows. Color: temperature, tint, and saturation. Brightness is the edit format's exposure operation (in photographic stops) under a friendlier name. Highlights recovers or brightens the bright areas of a photo, such as a sky, with little effect on dark areas; shadows lifts or deepens the dark areas, such as a backlit face, with little effect on bright ones. Each slider starts at a neutral center, and double-clicking it resets it. The preview updates live while dragging. Every slider shows its value as a signed whole number, with a true minus sign and "0" at neutral: −100 to 100, or 0 to 100 for an effect's strength. Brightness uses the same scale, 25 to a stop (100 is +4 stops, the edit format's limit), so all the light sliders read alike. Temperature and tint also name their ends: Cool and Warm, Green and Magenta. Double-clicking a slider resets it.
- **EDT-6 Effects.** Black and white, sepia, and vignette (amount and size), each with a strength control. Effects combine with adjustments in a fixed order, defined in the edit format spec.
- **EDT-7 Red-eye.** The user clicks or drags over each eye; each correction is stored as a region that can be moved or removed. No automatic detection in v1. Zoom (EDT-1) is how small eyes are reached.
- **EDT-8 Reopening edits.** Every control shows its stored value when the photo is reopened, in this session or any later one.
- **EDT-9 Undo and redo.** Per photo, for the last 50 changes. The history is kept in the edit file as small change records, so undo and redo work across sessions. Plain, uncompressed JSON, so the file stays readable.
- **EDT-10 Revert to original.** Removes all of Seura's own edits that are turned on, after confirmation, and keeps tags and rating. It also keeps operations from other apps and operations that are turned off, since they can't be brought back whole by undo and the edit format says to preserve them (FIL-5); when the photo has any from other apps, the confirmation says they're kept. Undoable like any other change. The confirmation replaces the Revert button in place, with Cancel where the pointer is and focused, and the question and Revert below it (INT-5); Esc cancels.
- **EDT-11 Autosave.** There is no Save button. Changes are written to the edit file within 1 second of the last change.
- **EDT-12 One-click enhance.** Analyzes the photo's brightness and color and sets brightness, contrast, highlights, shadows, temperature, tint, and saturation to corrected values. The result is ordinary slider settings the user can adjust or undo. Also available in the grid for a selection. It replaces only those seven settings; crop, rotation, straightening, red-eye, and effects stay as they are. It analyzes the photo as cropped, before any tone or color settings, so pressing it again gives the same result, not a stronger one. Each press is one change in the history (EDT-9). In the grid, each selected photo is analyzed on its own.

## Copying edits and presets

Edits from one photo can be reused on others, either directly or through a saved preset.

- **PRE-1 Copy and paste.** Copy the edits of one photo, then paste onto one photo or a selection. The user picks which groups to paste: adjustments and effects are on by default; crop and rotation are off by default; red-eye is never pasted, since it belongs to one photo.
- **PRE-2 Paste replaces.** Pasted groups replace the target's settings for those groups rather than adding to them. Pasting is undoable.
- **PRE-3 Save a preset.** The current edits can be saved as a named preset, with the same group choices as copying. Each preset is a file in `Seura Photos/Presets` in the user's home folder (decision 0015), in the edit format.
- **PRE-4 Apply a preset.** The preset list sits in the edit panel. Hovering a preset previews it on the current photo; clicking applies it. Presets can also be applied to a grid selection.
- **PRE-5 Manage presets.** Presets can be renamed and deleted in the app. Preset files the user copies into the Presets folder appear in the list.

## Tags and ratings

Tags and ratings are stored in each photo's edit file, so they travel with the photo.

- **TAG-1 Tag structure.** A tag is one string; slashes mark levels for display only. `family`, `kids`, and `family/kids` are three separate tags. In the tag tree, `family/kids` appears as `kids` under `family`. The `family` node exists in the tree even when no photo has the plain tag `family`.
- **TAG-2 Clean-up of typed tags.** Leading and trailing spaces and slashes, and empty levels such as `family//kids`, are removed. Matching is case-insensitive: typing `Family` when `family` exists selects the existing tag. Case-insensitive matching uses full Unicode case folding on normalized text (NFC), so it works for every language, not just A to Z: `Ärzte` matches `ärzte`, and `STRASSE` matches `straße`. The tag keeps the form the user first typed.
- **TAG-3 Tag picker.** Lists all existing tags for the user to click. Typing filters the list to tags containing the typed text, and the first match is highlighted. "Create tag "…"" is listed above the matches whenever the typed text isn't already a tag, and is highlighted only when nothing matches. Enter adds the highlighted tag, so adding an existing tag, the common case, takes the fewest keys; to create a new tag that other tags contain (typing "beach" when "beach house" exists), choose Create with the arrow keys or a click. With several photos selected, the tag is added to all of them.
- **TAG-4 Removing tags.** A tag can be removed from one photo or a selection. A tag no photo uses disappears from the tree. There is no renaming or merging.
- **TAG-5 Tag tree.** Selecting a node shows the photos with exactly that tag, not tags beneath it. Each node shows its photo count: the photos with exactly that tag. A node no photo has exactly, such as `family` when only `family/kids` is used, shows no count, not 0 and not a total of the tags beneath it, since selecting it shows no photos. The tag picker's counts follow the same rule. The tag tree starts closed and has the same Collapse all, Expand beneath, and `*` as the folder tree (LIB-2).
- **TAG-6 Ratings.** Zero to five stars, set by clicking or with keys 0 to 5, on one photo or a selection. Clicking the current rating's star clears the rating, as in Lightroom and Apple's Photos; it's the mouse's way to clear one, and its accessible name says so ("Clear rating").
- **TAG-8 Mixed selections.** With several photos selected, the Info panel shows what they have in common. When their ratings differ, the stars show empty with the word "Mixed" beside them, so the state doesn't depend on appearance alone; choosing a rating sets it on all of them. A tag only some of them have shows as a chip with a count, such as "beach · 3 of 7"; its × removes the tag from the photos that have it. The tag picker lists such tags first, and Enter adds the tag to the rest of the selection.
- **TAG-7 Read-only folders.** Photos there cannot be tagged or rated, and the controls say why: a line in the Info panel where the rating and tags are, such as "This folder is read-only, so tags and ratings can't be saved here." The stars and Add tag stay in place, disabled, so the layout doesn't change (INT-2). Pressing T or 0 to 5 there shows the same message as a notice. A tooltip alone isn't enough: many web views show no tooltip on a disabled control, and it never reaches a keyboard user.

## Finding photos

Filters narrow whatever the grid is showing (a folder, a tag, or search results), and a search box looks across the whole library.

- **FND-1 Filters.** Minimum rating, one or more tags (a photo must have all of them), edited or unedited, and date range. Active filters are always visible and can be cleared in one click, and the Filter button shows how many are set ("Filter · 2"), so an active filter doesn't rely on color alone; the status bar shows how many photos match ("3 of 214 photos"). They narrow whatever the grid shows, and stay set when the user moves to another folder, tag, or search, since they describe what the user is looking for, not where; they reset when the app restarts, so it never opens on a folder that looks empty. When nothing in view matches, the grid says so ("No photos here match the filters") with a Clear filters button.
- **FND-2 Search.** Matches file names and tags across the whole library. While searching, the grid shows the results from every folder, headed with the search and the number found ("Results for 'beach' · 42 photos"), and the sidebar has no selection. Esc clears the search and returns to the folder or tag selected before. Selecting a folder or tag in the sidebar ends the search.
- **FND-3 Speed.** Results update as the user types, within 200 milliseconds for a library of 10,000 photos.

## Export

Export is the only way edits leave the app: it writes new files and never touches the originals.

- **EXP-1 What and where.** Exports the selected photos, edited or not, to a folder the user picks. The last folder used is remembered.
- **EXP-2 Format and size.** JPEG with a quality setting (default 90), or PNG. Size: original, or the long edge limited to 2048, 1600, or 1024 pixels, or a custom value. HEIC originals export as JPEG or PNG.
- **EXP-3 Naming.** Files keep their original names with the new extension. A name already taken gets a number, as in `IMG_1234 (1).jpg`. Export never overwrites any file.
- **EXP-4 Metadata.** Date, camera, and other metadata are kept. Orientation is applied to the pixels and the orientation tag reset, so every viewer shows the photo the right way up. Options: remove location data; include tags and rating as keywords other photo apps can read.
- **EXP-5 Color.** Wide-color originals, such as iPhone photos, are converted to the standard sRGB color space with its profile embedded, so the export looks right everywhere.
- **EXP-6 Progress.** Batch exports show progress and can be canceled. A cloud-only photo shows as downloading, with its size, while the export waits for it (LIB-9); canceling stops waiting. A summary at the end reports results, including any photos that couldn't be downloaded, and offers to open the folder.

## Edit files on disk

Edit files are the user's data, so the app treats them with the same care as photos: never corrupted, never silently discarded.

- **FIL-1 Lifecycle.** An edit file is created when a photo first gets an edit, tag, or rating. When the photo has none of these again, and the file holds nothing else the app must preserve (such as another app's operations or extensions), the file is moved to the system Recycle Bin or Trash, like every other deletion (BRW-9, CLN-4). It is never deleted permanently. On a drive with no Recycle Bin or Trash, such as a USB stick, a memory card, or a network share, that file is deleted directly instead, since nothing in it can be lost: Seura reads it again just before, and deletes it only if it still holds no edits, tags, rating, or anything else the app must preserve (decision 0017).
- **FIL-2 Safe writes.** Each write goes to a temporary file that then replaces the old one, so a crash or power loss never leaves a half-written edit file.
- **FIL-3 Outside changes.** When an edit file changes on disk, for example synced from another computer, the app reloads it. If that photo is open in the editor, the app keeps the user's current work and says the file changed, asking "Keep yours or load theirs?". Leaving the photo before answering loads theirs, so nothing on disk is lost, and a message says the user's changes since were not kept.
- **FIL-4 Unreadable files.** If an edit file can't be read, the photo is shown unedited with a warning marker, and the file is never overwritten without the user's confirmation. When the user confirms ("Start over"), the old file is kept as a backup in the `Unreadable edit files` folder inside Seura's own folder (decision 0015), not beside the photo, named with the date and time and its original name: `2026-10-26 143012 IMG_1234.jpg.seura.json`. It is moved there unchanged, or copied there and then moved to the Recycle Bin or Trash where a move isn't possible. The notice afterwards says where it went, with Show in folder. The edit format doesn't need to know about this. On a drive with no Recycle Bin or Trash, the original is deleted directly only after its copy in Seura's folder has been read back and matches it byte for byte; otherwise it's left in place and Start over doesn't proceed (decision 0017).
- **FIL-5 Unknown content.** Operations the app doesn't recognize, from a newer version or another app, are kept unchanged when the file is saved, and the user is told they aren't shown.
- **FIL-6 Renamed photos.** Each edit file records a fingerprint of its photo. If a photo is renamed outside the app, its edit file is matched back up automatically when a photo in the same folder has the same fingerprint and no edit file of its own.
- **FIL-7 Changed originals.** If an original's fingerprint no longer matches, for example after another program edited it, the edits still apply, and a warning says results may be off.

## Cleanup

Cleanup removes what the app has written, on the user's request, and nothing else.

- **CLN-1 Scope.** One folder (optionally with its subfolders), the whole library, or only orphaned edit files whose photos no longer exist.
- **CLN-2 What can be removed.** Edit files; the cache; and, for a full reset, settings and presets. API keys join this list when AI arrives.
- **CLN-3 Preview.** Before anything is removed, the app lists what will go, with counts, and how many photos will lose edits. It offers to export the edited photos first.
- **CLN-4 Recoverable.** Removed files go to the system Recycle Bin or Trash, not permanent deletion.
- **CLN-5 Uninstall.** The uninstaller offers the same choices, defaulting to leaving edit files in place.

## First run and settings

The first run gets a new user to their photos in under a minute; settings stay few and plain.

- **SET-1 First run.** A short welcome asks which folders to add, suggesting the Pictures folder. One sentence explains that edits are saved in small files beside each photo and originals are never changed, and names the folder where Seura keeps its settings and cache (`Seura Photos` in the home folder). If that folder is inside a cloud-synced or backed-up location, the app suggests another place for the cache.
- **SET-2 Settings.** Library folders, cache location and size (move or clear), the location of Seura's own folder with an Open folder button, default export options, and update checks. Nothing else: appearance follows the operating system (ACC-3), and the layout isn't configurable (INT-2).
- **SET-3 Storage.** Settings are a readable file, `settings.json`, in a visible `Seura Photos` folder in the user's home folder, alongside Presets and, by default, the cache (decision 0015). The settings file, not the cache, holds the list of library folders, so clearing the cache never forgets the library.
- **SET-4 Updates.** The app checks for new versions and offers to install them; nothing installs without the user's say. First run asks whether to check, defaulting to on, and the choice can be changed in settings. Updates are published as releases in the seura-photos GitHub organization, and the app verifies each update's signature before installing it.
- **SET-5 About.** Shows the app's name and mark, its version, a line saying it's an early build while the version is below 1.0, and links to the website, the GitHub project, and the license; third-party notices are added for release (M7). The version can be selected, and a Copy button copies it, so a problem report can say which build it's about. On macOS it's the system's About panel, from the app menu; on Windows, a small dialog from the toolbar's ⋯ button (ACC-4).

## Interaction model

The app is opinionated (product brief principle 7; decision 0007): it defines one way to do each thing, ordered by how often it's done and by the process of working with photos.

- **INT-1 Action tiers.** Every action belongs to a tier with a maximum cost, measured from the view the user is most likely in when they want it:
  - **Tier 1, one click or key:** next or previous photo, open a photo, back, rotate, Enhance, rate, compare with original, start a review, and crop (from the Editor).
  - **Tier 2, at most three clicks or keys:** adjustment sliders, add or remove a tag, export, copy and paste edits, delete, apply a preset.
  - **Tier 3, no limit:** effect strengths, red-eye, save or manage presets, cleanup, settings.
  The user flows document lists each action's tier and path; wireframes and the finished app are checked against it.
- **INT-2 Fixed layout.** Each view has one layout, decided by the app. There are no options to rearrange, hide, or show panels or toolbars, and no separate beginner or advanced modes. Resizing the sidebar's width by dragging its edge isn't a layout option: it changes how much room folder and tag names get, not what's shown or where.
- **INT-3 Process order.** Wherever the app presents a set of steps, they follow the order find, sort, fix, organize, share. In the Editor, tools are ordered Enhance; crop and straighten; rotate and flip; light; color; effects; red-eye; presets.
- **INT-4 Progressive disclosure.** Detail appears when it becomes relevant, decided by the app: for example, an effect's strength slider appears once the effect is turned on.
- **INT-5 Pointer travel.** Layouts minimize how far the pointer moves through a flow (Fitts's law: time to hit a target grows with distance and shrinks with size):
  - Controls revealed by an action (popovers, follow-up options, a mode's Done button) appear next to the control or selection that revealed them, ordered by likelihood, the most likely nearest.
  - Persistent controls keep fixed positions so they can be learned, arranged so each flow moves in one direction.
  - Frequent targets are large: at least 32 px for tier-1 actions, and at least 44 px for the main action of a flow, such as Next in Review or Done in the Editor.
  - Confirmations of destructive actions place the safe choice where the pointer already is, never the destructive one.
  - Wireframes and builds are checked by tracing the pointer's path through the main flows: Review, fixing a photo, and tagging.
- **INT-6 Messages and notices.** Two kinds, kept apart so neither hides the other, and the same in the Library and the Viewer:
  - **Problems with the selected photos' files** (FIL-4, FIL-5, FIL-7) show in the notice bar at the top of the photo area for as long as those photos are selected. In the Viewer the photo fits below the bar, never under it.
  - **The result of the last action,** when it isn't visible on its own (photos moved to the Trash, an undo, photos skipped as read-only, a folder added), or when it was done with a key ("Rated 3 photos 3 stars", "Rotated 2 photos", "Added the tag beach to 5 photos"), shows as a brief message near the bottom of the photo area, centered, for about 6 seconds or until the next action. It never blocks anything and is announced to screen readers. A key gives no feedback where the user is looking, and the photos it changed may be off screen; a click on a star or a chip changes what's under the pointer, so it needs no message. A new message replaces the one before, so rating photo after photo by key shows only the latest.

## Performance targets

These make "feels instant" testable. They are defined for a mid-range laptop from about 2020 with integrated graphics, on each platform, with libraries tested up to 100,000 photos. During development they're measured on Nevada's machines and CI, which are much faster, so there the aim is about half the target time, to leave headroom; a reference-class laptop is tested during the M7 beta.

| ID | Action | Target |
| --- | --- | --- |
| PRF-1 | Launch to a usable grid, cache already built | Under 2 s |
| PRF-2 | Open a folder of 1,000 photos, cache already built | Grid full in under 1 s |
| PRF-3 | First scan, generating thumbnails | At least 30 photos per second |
| PRF-4 | Scrolling the grid | 60 frames per second |
| PRF-5 | Next or previous photo in the viewer | Under 150 ms |
| PRF-6 | Live preview while dragging a slider, 12 to 48 MP photo | At least 30 frames per second |
| PRF-7 | Memory use with a 50,000-photo library | Under 1 GB |

## Display

Today's screens are larger and denser than in Picasa's time. Windows and macOS scale high-density screens (a 4K monitor typically behaves like 1920×1080 or 2560×1440, only sharper), so sizes in this spec are logical pixels.

- **DSP-1 Crisp at any scaling.** Thumbnails, previews, and the edit view render at the screen's device pixel density, and all icons are vector graphics.
- **DSP-2 Minimum window.** Every view works at 1280×720 logical pixels without overlapping, clipped, or truncated controls.
- **DSP-3 Room goes to photos.** Layouts are designed for 1920×1080 and 2560×1440 logical. Extra space goes to the photos (more grid columns, larger default thumbnails, a larger photo area), not to additional controls. Side panels may widen modestly with the window; control sizes stay the same.
- **DSP-4 Thumbnail resolution.** Cached thumbnails are generated large enough for 2× displays at the largest thumbnail size.

## Accessibility and platform conventions

The app should feel at home on each platform and be usable without a mouse or with a screen reader.

- **ACC-1 Keyboard.** Everything can be done from the keyboard. Shortcuts follow each platform: Cmd on macOS, Ctrl on Windows.
- **ACC-2 Screen readers.** Every control has a label, and thumbnails are announced by file name, date, and rating.
- **ACC-3 Display.** Text and controls meet WCAG AA contrast, scale with the system text size, and respect light or dark mode and reduced-motion settings.
- **ACC-4 Native pieces.** File and folder dialogs, notifications, and the Trash or Recycle Bin. On macOS, the menu bar, with the standard app, File, Edit, and Window menus. On Windows, no menu bar, as in most current Windows apps: commands for the whole app (About, and Settings and Clean up when they exist) are under a ⋯ button at the right end of the toolbar, in every view. Each platform has one place for them, never both.
- **ACC-5 Language.** English only in v1, with all text kept in message catalogs, never in code, so translations can be added later. The interface and the core (menus, dialogs, notifications) share one catalog format, and it handles each language's plural forms and word order: "1 photo" and "214 photos" are one message, not a word with an "s" appended.
- **ACC-6 Regional formats.** Dates, times, numbers, and file sizes follow the operating system's region settings, even while the interface is in English. A German region shows `12.07.2026, 18:42` and `2,1 MB`; a US region shows `Jul 12, 2026, 6:42 PM` and `2.1 MB`. Date fields drawn by the web view itself, such as the filter's, follow the region too where the platform allows it; where one can't, that's recorded as a known limitation.
- **ACC-7 Room for translation.** Labels, buttons, menus, and panels allow text about 40% longer than English without truncating or overlapping at the minimum window size (DSP-2), since languages such as German and French often run that much longer. Checked with a pseudo-translated build: every string lengthened and accented, so clipped or untranslated text stands out.

## Decisions log

| Question | Decision |
| --- | --- |
| Deleting photos | Allowed, to the Trash, with confirmation (BRW-9). |
| One-click enhance | In v1 (EDT-12). Analyzes a brightness histogram and near-neutral colors, and outputs ordinary slider values. |
| Light sliders | Brightness, contrast, highlights, and shadows (EDT-5). Highlights and shadows were suggested by a friend of the project; Picasa had both. A separate middle-tone brightness was considered and left out, since it would overlap the exposure-based brightness slider. |
| Selecting a tag node | Exact tag only (TAG-5). |
| Expanding and collapsing the trees | Collapse all in each tree's heading, keeping the selection in view; expanding everything beneath one node, by Alt- or Option-click on its arrow or its right-click menu; no expand-all for a whole tree, which on a large library would make hundreds of rows (LIB-2, TAG-5). Suggested by Nevada; Claude's recommendation for the form, 2026-10-27. |
| Undo across sessions | Yes: a capped history of 50 change records in the edit file (EDT-9). The cap may be lowered to 20 if file size ever matters; compression is avoided to keep files readable. |
| Grid scope | The folder's own photos only (BRW-1). |
| Cloud-synced folders | Supported without downloading: cloud-only photos are fetched one at a time, only when needed (LIB-9). Raised by Nevada, 2026-10-18. |
| iCloud Photos on a Mac | Not supported in v1: the photos live inside the Photos app's library, not a folder. Users can export from Photos or use a folder-based service (Nevada, 2026-10-18). |
| Update checks | Asked during first run, defaulting to on (SET-4). |
| Reference hardware | A mid-range laptop from about 2020 with integrated graphics, on each platform. Until the M7 beta, measured on faster machines, aiming for about half the target time (Nevada, 2026-10-20). |
| Translations | English only in v1, built to be translated: one shared catalog format with plurals (ACC-5), regional formats from the operating system (ACC-6), room for longer text (ACC-7), and Unicode-aware tag matching (TAG-2). Translations, a language setting, and right-to-left languages come after v1 (product brief, "Not in v1"). Raised by Nevada, 2026-10-20. |
| Downloading cloud-only photos | An indicator and the photo's size, no progress bar and no cancel button: measured on OneDrive on both platforms, neither system reports download progress to the reading app, and abandoning a download finishes it on Windows but stops it on the Mac. Moving on is how a download is abandoned; Seura only stops waiting (LIB-9). Found by Claude Code and Nevada, 2026-10-22; Nevada's proposal. |
| What photo info shows | BRW-6's fields plus exposure settings, and date modified only when there's no date taken. Compared against Explorer's details pane and Finder's Get Info (about twenty items between them); the rest stays out to keep the panel short, with the reasons in the [journal](journal/2026-10-22-info-panel-fields.md). Nevada's review, Claude's recommendation, 2026-10-22. |
| Where Seura keeps its own files | A visible `Seura Photos` folder in the home folder, not Documents, because Documents is often cloud-synced and neither the cache nor the settings should sync (decision 0015). Raised by Nevada, 2026-10-20. |
| Undo in the Library | Ctrl+Z undoes the last action, not the last change to whatever is selected now, as in almost every app (BRW-11). The app remembers only which photos each of the session's last 10 actions touched; the changes live in each edit file's history, and undoing marks the record undone there rather than adding a new change, so each photo has one history that the Library and the Editor share. Nevada wanted the edit files as the source of record and asked how undo should behave with a selection spanning two earlier actions; Claude's recommendation, Nevada's decision, 2026-10-26. |
| Deleting cloud-synced photos | Allowed, as for any photo, with the confirmation saying the photos also leave the cloud and the user's other devices (BRW-9). Raised by Claude Code, 2026-10-26. |
| Deleting cloud-only photos | Left out, with the reason in the confirmation (BRW-9). Measured by Nevada on OneDrive: Windows sends them only to the service's online recycle bin, silently, and macOS refuses, so they can't reach the Recycle Bin or Trash. Deleting them anyway, as Finder does with a warning, was rejected: the photo would leave every device, recoverable only online, with Seura unable to tell afterwards. Downloading them first is planned for M6. The same rule keeps Delete off drives with no Recycle Bin. Claude's recommendation, Nevada's decision, 2026-10-27. |
| Messages and notices | Problems with files in the notice bar at the top of the photo area; the last action's result in a brief message at the bottom of it; both in the Library and the Viewer (INT-6). The sidebar and the status bar were tried first and were easy to miss (Nevada, testing M3). Claude's recommendation, Nevada's decision, 2026-10-27. |
| Sidebar width | Resizable by dragging, remembered, from 180 to 400 px (INT-2; wireframes). Nevada's decision, 2026-10-23; widths Claude's, 2026-10-27. |
| Drives with no Recycle Bin or Trash | Photos there are never deleted, even after a warning; a "Delete anyway" was considered for clearing memory cards and rejected, since rule 5 is the app's core promise and Explorer and Finder already do it with their own warning. Seura's own edit files there are deleted directly only when nothing in them can be lost (FIL-1, FIL-4). Found by Nevada's USB-stick test: 0.2.0 deleted such photos permanently without asking (#115), fixed in 0.2.1. Nevada proposed the edit-file rule; Claude's recommendations; decision 0017, 2026-10-27. |
| Importing from cards and USB sticks | Not in v1. Copying photos into a folder isn't a managed library, but it copies originals, which the product brief's first principle rules out; the operating system's own import (Windows Photos, Image Capture) does it, and the user then adds the folder. Revisit after 1.0 only with a change to that principle. 2026-10-27. |
| About, Settings, and Clean up on Windows | Under a ⋯ button at the right end of the toolbar, since Windows has no menu bar (ACC-4); macOS keeps its menus. Found by Nevada: 0.2 showed its version nowhere on Windows. Claude's recommendation, 2026-10-27; in 0.3. |
| Notice levels | Three, as in Windows' own info bars: information (blue), caution (amber), problem (red). Claude Code found that FIL-7's caution looked like information side by side; Claude's recommendation, 2026-10-27. |
| Messages after a key | A rating, tag, rotation, or Enhance done with a key shows a message; the same done with a click doesn't (INT-6). Nevada expected one after pressing 3. Claude's recommendation, 2026-10-27. |
| Editor details for M4 | Crops follow rotation and flips (EDT-4); the crop grows back as straightening is undone, until Done (EDT-3); every slider shows a signed whole number, brightness at 25 to a stop (EDT-5); Enhance replaces only its seven settings and gives the same result when pressed again (EDT-12). Raised by Claude Code while planning M4; Claude's recommendations, 2026-10-28. |
| Deleting OneDrive photos on the Mac | Not possible from Seura for now: the file provider refuses the installed app, ad-hoc or Apple Development signed (#112). The confirmation says so and offers Show in Finder. Asking Finder to do it through Apple Events was rejected: it needs a permission to control Finder, a broad and alarming ask for an app built on keeping the user's files safe. To test again with Developer ID signing before the beta (M7). Claude's recommendation, 2026-10-29. |
| Zoom in the Editor | As in the Viewer, with Space-drag to pan while red-eye uses the drag, and always Fit in crop (EDT-1). Raised by Nevada, 2026-10-29: never discussed before, and red-eye needs it. |
| Early builds | Public from 0.2, offered on the website with no promise of support; the beta stays in M7 (decision 0016). Nevada's decision, 2026-10-27. |
| Enter in the tag picker | Adds the first existing match; Create is chosen only when nothing matches, or by hand (TAG-3). Claude Code's choice while building, confirmed 2026-10-27. |
| Backups of unreadable edit files | In Seura's own folder, not beside the photo, so nothing new is written into the user's folders (FIL-4; CLAUDE.md rule 2). Raised by Claude Code, 2026-10-26. |
| Search, filters, and counts | Search replaces the grid until Esc (FND-2); filters persist across folders but not restarts (FND-1); parent tag nodes show no count rather than a total (TAG-5); mixed selections show "Mixed" and partial tags with a count (TAG-8); read-only folders say why in the Info panel (TAG-7). Raised by Claude Code while planning M3, 2026-10-26; Claude's recommendations, Nevada's decision. |
| Sort order and thumbnail size | One app-wide sort order (BRW-2); thumbnails 96 to 256 px, default 160 (BRW-3). |
| HEIC and JPEG pairs | Both shown, as separate files (LIB-8); may be revisited after 1.0. |
| Interface model | Opinionated: one fixed layout per view, action tiers by frequency, process order, no modes (INT-1 to INT-4, decision 0007). |
| Guided flow | A Review flow for sorting and lightly fixing batches of photos (REV-1 to REV-6). |
