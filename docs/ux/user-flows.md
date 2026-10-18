# Seura Photos — User Flows and Screen Map (v1)

Status: draft · Last updated: 2026-10-17

This document describes the screens of Seura Photos v1 and how a user moves between them. It turns the [functional spec](../functional-spec.md) into screens and steps; requirement IDs refer to it. Wireframes are on the design canvas and will be summarized here as they settle.

## Design approach

The app is **opinionated** (product brief principle 7, [decision 0007](../decisions/0007-opinionated-interface.md)): it defines how it's used, following the process most people go through with photos:

**find → sort → fix → organize → share**

- **One window, four views.** Library (grid), Viewer (one photo), Editor (one photo with tools), and Review (a guided pass through a batch). Esc always goes back one step.
- **One layout per view.** No panels to rearrange, hide, or show. The Info panel is always shown in the Library and Viewer; the Editor's tools take its place in the Editor; Review shows only the photo and its actions.
- **Frequent things are fast.** Every action has a tier with a maximum cost (below); the most common are one click or key from wherever the user is.
- **The photo is the biggest thing on screen.**
- **Nothing to save.** Edits, tags, and ratings apply as the user works (EDT-11).
- **Destructive actions are confirmed and recoverable.** Revert, Delete, and Cleanup confirm first; Delete and Cleanup go to the Trash; Revert is undoable.

## Action tiers (INT-1)

Cost is counted from the view where the user most likely is when they want the action.

| Tier | Budget | Action | Path |
| --- | --- | --- | --- |
| 1 | 1 click or key | Next or previous photo | Arrow keys; buttons in Viewer, Review; filmstrip in Editor |
| 1 | | Open a photo | Double-click or Enter in the Library |
| 1 | | Back | Esc |
| 1 | | Rotate left or right | `[` `]`, or toolbar buttons in Library, Viewer, Review, Editor |
| 1 | | Enhance | `A`, or toolbar button in Library, Viewer, Review, Editor |
| 1 | | Rate | `0`–`5`, or stars in the Info panel and Review |
| 1 | | Compare with original | Hold `\`, or the Compare button in the Editor |
| 1 | | Start a review | Banner in the Library, or the Review button |
| 1 | | Mark for deletion (in Review) | `X` |
| 1 | | Open the Editor | `E`, or the Edit button |
| 1 | | Crop | `C` in the Editor, or the Crop button |
| 2 | ≤ 3 | Adjust a slider | Editor → drag |
| 2 | | Add a tag | `T` → type or click → Enter |
| 2 | | Remove a tag | × on the tag in the Info panel |
| 2 | | Export | Ctrl+E → Export |
| 2 | | Copy and paste edits | Shift+Ctrl+C, select, Shift+Ctrl+V → Paste |
| 2 | | Delete | Delete → Confirm |
| 2 | | Apply a preset | Editor → click preset |
| 3 | — | Effect strength, red-eye, save or manage presets, cleanup, settings | Editor sections, menus |

## Screen map

| Screen | Kind | Reached from | Requirements |
| --- | --- | --- | --- |
| Welcome | Full window, first run only | First launch | SET-1, SET-4 |
| Library | Main view | Launch; Esc from Viewer or Review | LIB-*, BRW-1–4, BRW-10, TAG-*, FND-* |
| Viewer | Main view | Double-click or Enter on a photo | BRW-5, BRW-6, BRW-10 |
| Editor | Main view | `E` or Edit, from any view | EDT-*, PRE-* |
| Review | Main view | Banner or Review button in the Library | REV-* |
| Review summary | Panel at the end of a review | Last photo of a review | REV-5 |
| Info panel | Fixed side panel | Always in Library and Viewer | BRW-6, TAG-3, TAG-6 |
| Tag picker | Popover | `T`, or Add tag | TAG-3 |
| Paste edits | Dialog | Paste edits | PRE-1 |
| Save preset | Dialog | Save as preset in the Editor | PRE-3 |
| Export | Dialog, then progress | Export | EXP-* |
| Delete confirmation | Dialog | Delete | BRW-9 |
| Cleanup | Dialog with preview | File menu, folder context menu | CLN-* |
| Settings | Window (macOS) or dialog (Windows) | App or Edit menu | SET-2 |
| Notices | Bar at the top of the main area | Problems with files or folders | FIL-3, FIL-4, FIL-7, LIB-6 |

## Layout of the main window

```
┌──────────────────────────────────────────────────────────────────────┐
│ Toolbar: search · Review · rotate · Enhance · Edit · Export · view   │
├────────────┬───────────────────────────────────────────┬─────────────┤
│ Sidebar    │ Main area                                 │ Info panel  │
│            │                                           │ (always)    │
│ FOLDERS    │  Library: thumbnail grid                  │             │
│  ▸ Pictures│  Viewer:  one photo, full size            │ file, date, │
│    ▸ 2025  │                                           │ camera,     │
│    ▸ 2026  │                                           │ rating,     │
│            │                                           │ tags        │
│ TAGS       │                                           │             │
│  ▸ family  │                                           │             │
│     kids   │                                           │             │
├────────────┴───────────────────────────────────────────┴─────────────┤
│ Status bar: N photos · N selected · scan progress                    │
└──────────────────────────────────────────────────────────────────────┘
```

The Editor keeps the toolbar, drops the sidebar, and puts the tool panel where the Info panel was, with a filmstrip under the photo. Review drops both side panels.

## Flows

### 1. First run (SET-1, SET-4)

1. The Welcome screen: the app's name, what it does, and one sentence explaining that edits are saved in small files beside each photo and originals are never changed.
2. "Which folders hold your photos?" lists the Pictures folder, pre-checked, with Add another folder.
3. A checkbox: "Check for updates automatically," checked by default.
4. If `Documents` is cloud-synced, a short note offers another location for the thumbnail cache.
5. Get started → the Library opens on the first folder and thumbnails fill in as the scan runs (LIB-3).
6. When the first scan finishes, the Library offers a review of the most recent folder (REV-1).

Target: under a minute from launch to photos on screen.

### 2. Review a batch (REV-*)

The guided path, and the main way new photos get sorted.

1. The Library banner says "84 new photos in Summer. Review them?" Or the user chooses Review for any folder, tag, search, or selection.
2. The Review screen shows one photo, large, with "12 of 84" and one row of actions: previous, rotate left, rotate right, Enhance, stars, Tag, Mark for deletion, Edit, next.
3. The user works through the batch with single keys: `→` next, `←` previous, `0`–`5` rate, `[` `]` rotate, `A` enhance, `T` tag, `X` mark for deletion, `E` edit.
4. Edit opens the Editor on that photo; Done returns to the same place in the review.
5. After the last photo, the summary: "Rated 41 · Edited 12 · Tagged 30 · Marked for deletion 9," with Move 9 to Trash and Export photos rated 4★ or more.
6. Esc leaves at any time; everything done so far is already saved.

### 3. Browse and view (LIB-2, BRW-1–10)

1. Select a folder in the sidebar; the grid shows its photos.
2. Sort and thumbnail size are in the view controls.
3. Rotate, Enhance, and rating work directly on the selection, without opening anything (BRW-10).
4. Double-click or Enter opens the Viewer; arrow keys move through the folder.
5. In the Viewer, zoom with the wheel or pinch, or Fit and 100%; drag to pan.
6. Esc returns to the Library at the same scroll position, with the last viewed photo selected.

### 4. Fix a photo (EDT-*)

The basis of the design test: fix a crooked, dark photo in under five minutes.

1. `E` or Edit opens the Editor.
2. The tool panel, in process order (INT-3):
   - **Enhance** (one click)
   - **Crop and straighten** (opens crop mode)
   - **Rotate left, rotate right, flip**
   - **Light:** exposure, contrast
   - **Color:** temperature, tint, saturation
   - **Effects:** black and white, sepia, vignette, each a one-click look whose strength slider appears once it's on (INT-4)
   - **Red-eye** (opens red-eye mode)
   - **Presets** (list, with Save as preset)
3. Sliders update the photo live; double-click resets one.
4. Crop mode shows the whole photo with the crop box, ratio choices, and a straighten slider. Done or Enter applies; Esc cancels.
5. Red-eye mode: click each eye; corrections can be dragged or removed.
6. Hold `\` or Compare to see the original.
7. Undo and redo work as usual, including across restarts (EDT-9). Revert to original confirms first and keeps tags and rating.
8. The filmstrip moves to the next photo without leaving the Editor.
9. Done or Esc returns to where the user came from: Viewer, Library, or Review.

### 5. Tag and rate (TAG-*)

1. Select photos in the Library, or open one in any view.
2. `T` opens the tag picker with all existing tags listed. Typing filters; Enter adds the highlighted tag or creates the typed one. The picker stays open for more; Esc closes it.
3. Tags show as chips in the Info panel; × removes one.
4. `0`–`5` or the stars set the rating.
5. Selecting a tag in the sidebar shows the photos with exactly that tag, from every folder (TAG-5).

### 6. Find photos (FND-*)

1. The search box searches file names and tags across the library; Esc clears it.
2. Filter opens controls above the grid: minimum rating, tags, edited or unedited, date range. Active filters show as chips with Clear all.

### 7. Reuse edits (PRE-*)

1. Copy edits (Shift+Ctrl+C) copies the current photo's edits.
2. Select targets, Paste edits (Shift+Ctrl+V): a small dialog with adjustments and effects checked and crop and rotation unchecked.
3. Save as preset in the Editor asks for a name and the same choices.
4. Presets in the Editor preview on hover and apply on click; in the Library, the Presets menu applies one to the selection.

### 8. Export (EXP-*)

1. Select photos, Ctrl+E.
2. The dialog: destination, format and quality, size, and options to remove location data and include tags as keywords.
3. Progress with Cancel, then a summary with Show in Explorer or Finder.

### 9. Delete (BRW-9)

1. Select photos, press Delete (Cmd+Delete on macOS).
2. Confirm; the photos and their edit files go to the Recycle Bin or Trash.

### 10. Cleanup (CLN-*)

1. File → Clean up, or right-click a folder → Clean up this folder.
2. Choose scope and what to remove; a preview shows counts.
3. Export edited photos first, or Move to Trash.

### 11. Problems with files (FIL-*, LIB-6)

Shown as a notice bar at the top of the main area, never a blocking dialog:

- **Changed on disk while editing:** "This photo's edits were changed outside Seura Photos. Keep yours or load theirs?"
- **Unreadable edit file:** the photo shows unedited with a warning badge; the notice offers Start over, keeping a backup.
- **Original changed:** "This photo was changed by another program; edits may not line up."
- **Unavailable folder:** dimmed in the sidebar, with "Drive not connected."

## Keyboard shortcuts

Ctrl on Windows, Cmd on macOS. Single-letter keys work in every view except while typing in a text field.

| Action | Shortcut |
| --- | --- |
| Open photo | Enter, double-click |
| Back | Esc |
| Next or previous photo | → ← |
| Rotate left, right | `[` `]` |
| Enhance | A |
| Rate | 0–5 |
| Tag | T |
| Edit | E |
| Crop (in Editor) | C |
| Mark for deletion (in Review) | X |
| Compare with original | Hold `\` |
| Undo, redo | Ctrl+Z, Shift+Ctrl+Z |
| Copy edits, paste edits | Shift+Ctrl+C, Shift+Ctrl+V |
| Export | Ctrl+E |
| Search | Ctrl+F |
| Select all | Ctrl+A |
| Delete | Delete (Windows), Cmd+Delete (macOS) |

## Decisions

| Question | Decision |
| --- | --- |
| Single window or a separate Viewer window | One window. |
| Editor tool panel: one column or tabs | One column, in process order. |
| Where tags live | The Info panel in Library and Viewer, and under the photo in Review. |
| Thumbnail badges | Always visible, small and quiet. |
| Modes and customization | None: one fixed layout per view, no beginner or advanced modes (INT-2, decision 0007). |
