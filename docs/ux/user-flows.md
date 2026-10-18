# Seura Photos — User Flows and Screen Map (v1)

Status: draft · Last updated: 2026-10-17

This document describes the screens of Seura Photos v1 and how a user moves between them. It turns the [functional spec](../functional-spec.md) into screens and steps; visual design comes after, in the wireframes. Requirement IDs refer to the functional spec.

## Design approach

- **One window, three views.** Library (grid), Viewer (one photo, full size), and Editor (one photo, with tools). The user is always in exactly one, and Esc always goes back one step: Editor → Viewer → Library.
- **The photo is the biggest thing on screen.** Panels are narrow and can be hidden.
- **Nothing to save.** Edits apply as the user works (EDT-11), so no view has a Save button or an "unsaved changes" prompt.
- **Familiar to Picasa users.** Folders on the left, photos in the middle, editing tools in a side panel, a filmstrip to move between photos while editing.
- **Every destructive action is confirmed and recoverable.** Revert, Delete, and Cleanup confirm first; Delete and Cleanup go to the Trash; Revert is undoable.

## Screen map

| Screen | Kind | Reached from | Requirements |
| --- | --- | --- | --- |
| Welcome | Full window, first run only | First launch | SET-1, SET-4 |
| Library | Main view | Launch; Esc from Viewer | LIB-*, BRW-1–4, TAG-*, FND-* |
| Viewer | Main view | Double-click or Enter on a photo | BRW-5, BRW-6 |
| Editor | Main view | Edit button or E key, from Viewer or Library | EDT-*, PRE-* |
| Info panel | Side panel, toggled | Library, Viewer, Editor | BRW-6, TAG-3, TAG-6 |
| Tag picker | Popover | T key, or Add tag in the Info panel | TAG-3 |
| Paste edits | Dialog | Paste edits command | PRE-1 |
| Save preset | Dialog | Save as preset in the Editor | PRE-3 |
| Export | Dialog, then progress | Export command | EXP-* |
| Delete confirmation | Dialog | Delete key or command | BRW-9 |
| Cleanup | Dialog with preview | File menu, folder context menu | CLN-* |
| Settings | Window (macOS) or dialog (Windows) | App or Edit menu | SET-2 |
| Notices | Bar at top of the main area | Problems with files or folders | FIL-3, FIL-4, FIL-7, LIB-6 |

## Layout of the main window

```
┌──────────────────────────────────────────────────────────────────────┐
│ Toolbar: search · view controls · Edit · Export · Info toggle        │
├────────────┬───────────────────────────────────────────┬─────────────┤
│ Sidebar    │ Main area                                 │ Info panel  │
│            │                                           │ (optional)  │
│ FOLDERS    │  Library: thumbnail grid                  │             │
│  ▸ Pictures│  Viewer:  one photo, full size            │ file, date, │
│    ▸ 2025  │  Editor:  photo + tool panel + filmstrip  │ camera,     │
│    ▸ 2026  │                                           │ tags,       │
│            │                                           │ rating      │
│ TAGS       │                                           │             │
│  ▸ family  │                                           │             │
│     kids   │                                           │             │
│  vacation  │                                           │             │
├────────────┴───────────────────────────────────────────┴─────────────┤
│ Status bar: N photos · N selected · scan progress                    │
└──────────────────────────────────────────────────────────────────────┘
```

In the Editor, the sidebar collapses so the photo gets more room, and the tool panel takes the right side in place of the Info panel.

## Flows

### 1. First run (SET-1, SET-4)

1. The app opens to the Welcome screen: the name, one sentence about what it does, and one sentence explaining that edits are saved in small files beside each photo and originals are never changed.
2. "Add folders" lists the Pictures folder, pre-checked, with an Add another folder button.
3. A checkbox: "Check for updates automatically," checked by default.
4. If `Documents` is cloud-synced, a short note offers a different location for the thumbnail cache, with a Choose button. Otherwise nothing is shown.
5. Get started → the Library opens on the first folder, and thumbnails fill in as the scan runs (LIB-3). The status bar shows progress.

Target: under a minute from launch to photos on screen.

### 2. Browse and view (LIB-2, BRW-1–8)

1. In the Library, the user selects a folder in the sidebar. The grid shows that folder's own photos.
2. The view controls sort the grid (BRW-2) and resize thumbnails (BRW-3).
3. Double-click or Enter opens the Viewer. Arrow keys move through the photos in grid order (BRW-5).
4. In the Viewer, zoom with the scroll wheel or pinch, or with the Fit and 100% buttons; drag to pan.
5. I toggles the Info panel (BRW-6).
6. Esc returns to the Library at the same scroll position, with the last viewed photo selected.

### 3. Fix a photo (EDT-*)

The core flow, and the basis of the design test: fix a crooked, dark photo in under five minutes.

1. From the Viewer or Library, Edit (or E) opens the Editor on that photo.
2. The tool panel shows, top to bottom:
   - **Enhance** (one click, EDT-12)
   - **Crop and straighten** (opens crop mode)
   - **Rotate left, rotate right, flip**
   - **Light:** exposure, contrast
   - **Color:** temperature, tint, saturation
   - **Effects:** black and white, sepia, vignette
   - **Red-eye** (opens red-eye mode)
   - **Presets** (list, with Save as preset)
3. Moving a slider updates the photo live. Double-click a slider to reset it.
4. Crop mode replaces the panel's contents with ratio choices and a straighten slider; the photo shows the full image with the crop box and a rule-of-thirds overlay. Done or Enter returns to the tool panel; Esc cancels the crop change.
5. Red-eye mode: click each eye; each correction shows as a circle that can be dragged or deleted. Done returns to the panel.
6. Holding the backslash key, or the Compare button, shows the original (EDT-1).
7. Undo and redo work as usual (Ctrl/Cmd+Z, Shift+Ctrl/Cmd+Z), including across restarts (EDT-9).
8. Revert to original (EDT-10) asks for confirmation and keeps tags and rating.
9. The filmstrip along the bottom moves to the next photo without leaving the Editor.
10. Esc or Done returns to the Viewer. There is nothing to save.

### 4. Tag and rate (TAG-*)

1. Select one or more photos in the Library, or open one in the Viewer or Editor.
2. Press T, or click Add tag in the Info panel. The tag picker opens with all existing tags listed.
3. Typing filters the list. Enter adds the highlighted tag, or creates the typed tag when nothing matches. The picker stays open to add more; Esc closes it.
4. Tags appear as removable chips in the Info panel; clicking a chip's × removes it.
5. Keys 0–5 set the rating, or click the stars in the Info panel.
6. Selecting a tag in the sidebar shows the photos with exactly that tag, from every folder (TAG-5).

### 5. Find photos (FND-*)

1. The search box in the toolbar searches file names and tags across the whole library; results replace the grid, labeled "Search results," and Esc clears the search.
2. The Filter button opens filter controls above the grid: minimum rating, tags, edited or unedited, date range.
3. Active filters show as chips above the grid, with Clear all.

### 6. Reuse edits (PRE-*)

1. In the Editor or Library, Copy edits (Ctrl/Cmd+Shift+C) copies the current photo's edits.
2. Select target photos, then Paste edits (Ctrl/Cmd+Shift+V). A small dialog lists the groups to paste, with adjustments and effects checked and crop and rotation unchecked. Paste applies them.
3. In the Editor, Save as preset asks for a name and the same group choices.
4. The Presets section of the tool panel lists presets. Hovering previews one on the photo; clicking applies it. In the Library, the Presets menu applies one to the selection.

### 7. Export (EXP-*)

1. Select photos, then Export (Ctrl/Cmd+E).
2. The Export dialog: destination folder (the last one used), format (JPEG with quality, or PNG), size (original, 2048, 1600, 1024, or custom long edge), and checkboxes for removing location data and including tags as keywords.
3. Export starts and the dialog shows progress, with Cancel.
4. When done: a summary, and Show in Explorer or Finder.

### 8. Delete (BRW-9)

1. Select photos, press Delete (Cmd+Delete on macOS).
2. A confirmation names the count and says they'll go to the Recycle Bin or Trash along with their edit files.
3. Confirm → the photos disappear from the grid, and the selection moves to the next photo.

### 9. Cleanup (CLN-*)

1. File → Clean up, or right-click a folder → Clean up this folder.
2. The dialog sets the scope (this folder, with or without subfolders; the whole library; only orphaned edit files) and what to remove (edit files, cache, and for a full reset, settings and presets).
3. A preview lists counts: "312 edit files, 87 of them with edits," and so on.
4. Export edited photos first, or Move to Trash. Both are explicit buttons; nothing happens on close.

### 10. Problems with files (FIL-*, LIB-6)

These appear as a notice bar at the top of the main area, never as a blocking dialog:

- **Changed on disk** while open in the Editor: "This photo's edits were changed outside Seura Photos. Keep yours or load theirs?"
- **Unreadable edit file:** the photo shows unedited, with a warning badge on its thumbnail; the notice offers Start over (keeping a backup of the old file).
- **Original changed:** "This photo was changed by another program; edits may not line up."
- **Unavailable folder:** the folder is dimmed in the sidebar, with "Drive not connected."

## Keyboard shortcuts (initial set)

Ctrl on Windows, Cmd on macOS.

| Action | Shortcut |
| --- | --- |
| Open photo | Enter, double-click |
| Back | Esc |
| Next or previous photo | Arrow keys |
| Edit | E |
| Info panel | I |
| Tag | T |
| Rate | 0–5 |
| Compare with original | Hold \\ |
| Undo, redo | Ctrl+Z, Shift+Ctrl+Z |
| Copy edits, paste edits | Shift+Ctrl+C, Shift+Ctrl+V |
| Rotate left, right | Ctrl+[, Ctrl+] |
| Export | Ctrl+E |
| Search | Ctrl+F |
| Select all | Ctrl+A |
| Delete | Delete (Windows), Cmd+Delete (macOS) |

## Open questions

- [ ] **Single window or separate Viewer window?** Picasa used one window. [One window]
- [ ] **Editor tool panel: one scrolling column, or tabs** (Basic, Effects, Presets)? Picasa used tabs. [One column; it's short enough, and nothing is hidden]
- [ ] **Where tags live in the UI.** Info panel only, or also a small tag strip under the photo in the Viewer? [Info panel only in v1]
- [ ] **Thumbnail badges.** Show edited marker and rating always, or only on hover? [Always, small and quiet]
