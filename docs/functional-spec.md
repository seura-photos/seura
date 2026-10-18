# Seura Photos — Functional Spec (v1)

Status: approved · Last updated: 2026-10-17

This spec defines what Seura Photos v1 does, as behaviors with testable acceptance criteria. It builds on the [product brief](product-brief.md). Screen layout comes in the design phase, and the edit file format has its own specification in [seura-photos/edit-format](https://github.com/seura-photos/edit-format). Each requirement has an ID (LIB-1, EDT-3) so tests, commits, and code can reference it.

## Library

The library is the set of folders the user adds; the app works on them in place and never moves, copies, or modifies the photos.

- **LIB-1 Add a folder.** The user picks a folder with the system folder dialog; it and all its subfolders join the library. Adding a folder already inside the library does nothing and says so. Adding a parent of a library folder absorbs the child.
- **LIB-2 Folder tree.** The sidebar shows library folders and every subfolder that contains supported photos, directly or further down. Folders with no photos are hidden. Each folder shows its photo count.
- **LIB-3 First scan.** Thumbnails appear progressively as they are generated, the app stays usable throughout, and a progress indicator shows photos processed out of found.
- **LIB-4 Watching for changes.** While the app runs, new, changed, deleted, and renamed photos are reflected within 5 seconds. On launch, changes made while the app was closed are detected.
- **LIB-5 Remove a folder.** Removes it from the library only. No files are deleted; its cache entries are dropped; its edit files stay (removing them is Cleanup's job).
- **LIB-6 Unavailable folders.** A folder on a disconnected drive is shown as unavailable, not removed, and returns when the drive reconnects.
- **LIB-7 Skipped items.** Hidden files and folders, and system folders such as the Recycle Bin and Trash, are ignored.
- **LIB-8 Formats.** JPEG, HEIC/HEIF, and PNG are shown. Other files are ignored silently. For iPhone Live Photos, the still image is shown and its paired video ignored.

## Browsing

Selecting a folder or tag shows its photos in a thumbnail grid; opening one shows it full size.

- **BRW-1 Grid.** Shows the photos directly in the selected folder, not its subfolders. Thumbnails show photos with their edits applied, plus small markers for edited photos and star ratings.
- **BRW-2 Sorting.** By date taken (default; falls back to file date when the photo has none), file name, or date modified, ascending or descending.
- **BRW-3 Thumbnail size.** Adjustable with a slider; the setting is remembered.
- **BRW-4 Selection.** Click, Ctrl/Cmd-click, Shift-click, and Select All, following platform conventions. The number selected is always visible.
- **BRW-5 Viewer.** Double-click or Enter opens a photo full size. Arrow keys move to the next or previous photo in grid order. Zoom to fit, zoom to 100%, zoom in and out, and pan when zoomed. Esc returns to the grid at the same scroll position.
- **BRW-6 Photo info.** A panel, always visible in the Library and Viewer, shows file name, date taken, dimensions, file size, camera and lens, location as coordinates, tags, and rating.
- **BRW-7 Correct display.** Orientation stored in the photo's metadata is honored, and embedded color profiles are applied, so iPhone photos show their true colors.
- **BRW-8 Open elsewhere.** Show in Explorer or Finder, and open in the system's default app.
- **BRW-9 Delete.** On explicit request and after confirmation, moves the selected photos and their edit files to the system Recycle Bin or Trash. Never permanent deletion. Not available in read-only folders.
- **BRW-10 Quick actions.** Rotate left and right, Enhance, and rating are available directly in the Library (for the selection) and the Viewer, as one click or one key, without opening the Editor. In the Library, rotate and Enhance also appear on a thumbnail while it's hovered, so the pointer doesn't leave the grid (INT-5).

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

- **EDT-1 Edit mode.** Opened from the viewer or grid. A compare control (button, or holding a key) shows the original while held.
- **EDT-2 Crop.** Freeform, or fixed ratios: original, 1:1, 4:3, 3:2, 16:9, and 5:4, each switchable between landscape and portrait. A rule-of-thirds overlay shows while cropping. Reopening crop shows the whole photo with the current crop box, so the crop can be widened again.
- **EDT-3 Straighten.** From −45° to +45° in 0.1° steps. The crop shrinks automatically so no empty corners appear.
- **EDT-4 Rotate and flip.** Rotate 90° left or right, and flip horizontally or vertically. Also available in the grid for a whole selection.
- **EDT-5 Adjustments.** Exposure, contrast, color temperature, tint, and saturation. Each slider starts at a neutral center, and double-clicking it resets it. The preview updates live while dragging.
- **EDT-6 Effects.** Black and white, sepia, and vignette (amount and size), each with a strength control. Effects combine with adjustments in a fixed order, defined in the edit format spec.
- **EDT-7 Red-eye.** The user clicks or drags over each eye; each correction is stored as a region that can be moved or removed. No automatic detection in v1.
- **EDT-8 Reopening edits.** Every control shows its stored value when the photo is reopened, in this session or any later one.
- **EDT-9 Undo and redo.** Per photo, for the last 50 changes. The history is kept in the edit file as small change records, so undo and redo work across sessions. Plain, uncompressed JSON, so the file stays readable.
- **EDT-10 Revert to original.** Removes all edits after confirmation, keeping tags and rating. Undoable like any other change.
- **EDT-11 Autosave.** There is no Save button. Changes are written to the edit file within 1 second of the last change.
- **EDT-12 One-click enhance.** Analyzes the photo's brightness and color and sets exposure, contrast, temperature, tint, and saturation to corrected values. The result is ordinary slider settings the user can adjust or undo. Also available in the grid for a selection.

## Copying edits and presets

Edits from one photo can be reused on others, either directly or through a saved preset.

- **PRE-1 Copy and paste.** Copy the edits of one photo, then paste onto one photo or a selection. The user picks which groups to paste: adjustments and effects are on by default; crop and rotation are off by default; red-eye is never pasted, since it belongs to one photo.
- **PRE-2 Paste replaces.** Pasted groups replace the target's settings for those groups rather than adding to them. Pasting is undoable.
- **PRE-3 Save a preset.** The current edits can be saved as a named preset, with the same group choices as copying. Each preset is a file in `Documents/Seura Photos/Presets`, in the edit format.
- **PRE-4 Apply a preset.** The preset list sits in the edit panel. Hovering a preset previews it on the current photo; clicking applies it. Presets can also be applied to a grid selection.
- **PRE-5 Manage presets.** Presets can be renamed and deleted in the app. Preset files the user copies into the Presets folder appear in the list.

## Tags and ratings

Tags and ratings are stored in each photo's edit file, so they travel with the photo.

- **TAG-1 Tag structure.** A tag is one string; slashes mark levels for display only. `family`, `kids`, and `family/kids` are three separate tags. In the tag tree, `family/kids` appears as `kids` under `family`. The `family` node exists in the tree even when no photo has the plain tag `family`.
- **TAG-2 Clean-up of typed tags.** Leading and trailing spaces and slashes, and empty levels such as `family//kids`, are removed. Matching is case-insensitive: typing `Family` when `family` exists selects the existing tag.
- **TAG-3 Tag picker.** Lists all existing tags for the user to click. Typing filters the list to tags containing the typed text. Enter adds the highlighted tag, or creates a new tag when nothing matches. With several photos selected, the tag is added to all of them.
- **TAG-4 Removing tags.** A tag can be removed from one photo or a selection. A tag no photo uses disappears from the tree. There is no renaming or merging.
- **TAG-5 Tag tree.** Selecting a node shows the photos with exactly that tag, not tags beneath it. Each node shows its photo count.
- **TAG-6 Ratings.** Zero to five stars, set by clicking or with keys 0 to 5, on one photo or a selection.
- **TAG-7 Read-only folders.** Photos there cannot be tagged or rated, and the controls say why.

## Finding photos

Filters narrow whatever the grid is showing, a folder or a tag, and a search box looks across the whole library.

- **FND-1 Filters.** Minimum rating, one or more tags (a photo must have all of them), edited or unedited, and date range. Active filters are always visible and can be cleared in one click.
- **FND-2 Search.** Matches file names and tags across the whole library.
- **FND-3 Speed.** Results update as the user types, within 200 milliseconds for a library of 10,000 photos.

## Export

Export is the only way edits leave the app: it writes new files and never touches the originals.

- **EXP-1 What and where.** Exports the selected photos, edited or not, to a folder the user picks. The last folder used is remembered.
- **EXP-2 Format and size.** JPEG with a quality setting (default 90), or PNG. Size: original, or the long edge limited to 2048, 1600, or 1024 pixels, or a custom value. HEIC originals export as JPEG or PNG.
- **EXP-3 Naming.** Files keep their original names with the new extension. A name already taken gets a number, as in `IMG_1234 (1).jpg`. Export never overwrites any file.
- **EXP-4 Metadata.** Date, camera, and other metadata are kept. Orientation is applied to the pixels and the orientation tag reset, so every viewer shows the photo the right way up. Options: remove location data; include tags and rating as keywords other photo apps can read.
- **EXP-5 Color.** Wide-color originals, such as iPhone photos, are converted to the standard sRGB color space with its profile embedded, so the export looks right everywhere.
- **EXP-6 Progress.** Batch exports show progress and can be canceled. A summary at the end reports results and offers to open the folder.

## Edit files on disk

Edit files are the user's data, so the app treats them with the same care as photos: never corrupted, never silently discarded.

- **FIL-1 Lifecycle.** An edit file is created when a photo first gets an edit, tag, or rating, and deleted when the photo has none of these again.
- **FIL-2 Safe writes.** Each write goes to a temporary file that then replaces the old one, so a crash or power loss never leaves a half-written edit file.
- **FIL-3 Outside changes.** When an edit file changes on disk, for example synced from another computer, the app reloads it. If that photo is open in the editor, the app keeps the user's current work and says the file changed.
- **FIL-4 Unreadable files.** If an edit file can't be read, the photo is shown unedited with a warning marker, and the file is never overwritten without the user's confirmation. When the user confirms, the old file is kept as a backup.
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

- **SET-1 First run.** A short welcome asks which folders to add, suggesting the Pictures folder. One sentence explains that edits are saved in small files beside each photo and originals are never changed. If the default cache location is in a cloud-synced folder, the app suggests another.
- **SET-2 Settings.** Library folders, cache location and size (move or clear), default export options, and update checks. Nothing else: appearance follows the operating system (ACC-3), and the layout isn't configurable (INT-2).
- **SET-3 Storage.** Settings are a readable file in `Documents/Seura Photos`, alongside Presets and, by default, the cache.
- **SET-4 Updates.** The app checks for new versions and offers to install them; nothing installs without the user's say. First run asks whether to check, defaulting to on, and the choice can be changed in settings. Updates are published as releases in the seura-photos GitHub organization, and the app verifies each update's signature before installing it.

## Interaction model

The app is opinionated (product brief principle 7; decision 0007): it defines one way to do each thing, ordered by how often it's done and by the process of working with photos.

- **INT-1 Action tiers.** Every action belongs to a tier with a maximum cost, measured from the view the user is most likely in when they want it:
  - **Tier 1, one click or key:** next or previous photo, open a photo, back, rotate, Enhance, rate, compare with original, start a review, and crop (from the Editor).
  - **Tier 2, at most three clicks or keys:** adjustment sliders, add or remove a tag, export, copy and paste edits, delete, apply a preset.
  - **Tier 3, no limit:** effect strengths, red-eye, save or manage presets, cleanup, settings.
  The user flows document lists each action's tier and path; wireframes and the finished app are checked against it.
- **INT-2 Fixed layout.** Each view has one layout, decided by the app. There are no options to rearrange, hide, or show panels or toolbars, and no separate beginner or advanced modes.
- **INT-3 Process order.** Wherever the app presents a set of steps, they follow the order find, sort, fix, organize, share. In the Editor, tools are ordered Enhance; crop and straighten; rotate and flip; light; color; effects; red-eye; presets.
- **INT-4 Progressive disclosure.** Detail appears when it becomes relevant, decided by the app: for example, an effect's strength slider appears once the effect is turned on.
- **INT-5 Pointer travel.** Layouts minimize how far the pointer moves through a flow (Fitts's law: time to hit a target grows with distance and shrinks with size):
  - Controls revealed by an action (popovers, follow-up options, a mode's Done button) appear next to the control or selection that revealed them, ordered by likelihood, the most likely nearest.
  - Persistent controls keep fixed positions so they can be learned, arranged so each flow moves in one direction.
  - Frequent targets are large: at least 32 px for tier-1 actions, and at least 44 px for the main action of a flow, such as Next in Review or Done in the Editor.
  - Confirmations of destructive actions place the safe choice where the pointer already is, never the destructive one.
  - Wireframes and builds are checked by tracing the pointer's path through the main flows: Review, fixing a photo, and tagging.

## Performance targets

These make "feels instant" testable. They are measured on a mid-range laptop from about 2020 with integrated graphics, on each platform, with libraries tested up to 100,000 photos.

| ID | Action | Target |
| --- | --- | --- |
| PRF-1 | Launch to a usable grid, cache already built | Under 2 s |
| PRF-2 | Open a folder of 1,000 photos, cache already built | Grid full in under 1 s |
| PRF-3 | First scan, generating thumbnails | At least 30 photos per second |
| PRF-4 | Scrolling the grid | 60 frames per second |
| PRF-5 | Next or previous photo in the viewer | Under 150 ms |
| PRF-6 | Live preview while dragging a slider, 12 to 48 MP photo | At least 30 frames per second |
| PRF-7 | Memory use with a 50,000-photo library | Under 1 GB |

## Accessibility and platform conventions

The app should feel at home on each platform and be usable without a mouse or with a screen reader.

- **ACC-1 Keyboard.** Everything can be done from the keyboard. Shortcuts follow each platform: Cmd on macOS, Ctrl on Windows.
- **ACC-2 Screen readers.** Every control has a label, and thumbnails are announced by file name, date, and rating.
- **ACC-3 Display.** Text and controls meet WCAG AA contrast, scale with the system text size, and respect light or dark mode and reduced-motion settings.
- **ACC-4 Native pieces.** System menus, file and folder dialogs, notifications, and Trash or Recycle Bin.
- **ACC-5 Language.** English only in v1, with all text kept separate from code so translations can be added later.

## Decisions log

| Question | Decision |
| --- | --- |
| Deleting photos | Allowed, to the Trash, with confirmation (BRW-9). |
| One-click enhance | In v1 (EDT-12). Analyzes a brightness histogram and near-neutral colors, and outputs ordinary slider values. |
| Selecting a tag node | Exact tag only (TAG-5). |
| Undo across sessions | Yes: a capped history of 50 change records in the edit file (EDT-9). The cap may be lowered to 20 if file size ever matters; compression is avoided to keep files readable. |
| Grid scope | The folder's own photos only (BRW-1). |
| Update checks | Asked during first run, defaulting to on (SET-4). |
| Reference hardware | A mid-range laptop from about 2020 with integrated graphics, on each platform. |
| Interface model | Opinionated: one fixed layout per view, action tiers by frequency, process order, no modes (INT-1 to INT-4, decision 0007). |
| Guided flow | A Review flow for sorting and lightly fixing batches of photos (REV-1 to REV-6). |
