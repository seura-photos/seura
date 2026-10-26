# Seura Photos: the interface's text (ACC-5). Project Fluent; see
# https://projectfluent.org. English only in v1. Numbers passed in are
# formatted in the operating system's region (ACC-6).

app-name = Seura Photos
app-short-name = Seura
app-version = Version { $version }

## The Welcome screen (SET-1, SET-4)

welcome-title = Welcome to Seura Photos
welcome-what = Browse, fix, and organize the photos already on your computer.
welcome-originals = Your originals are never changed. Edits are saved in small files next to each photo, so you can undo them at any time.
welcome-seura-folder = Seura keeps its settings and thumbnails in the Seura Photos folder in your home folder: { $folder }
welcome-which-folders = Which folders hold your photos?
welcome-add-folder = Add another folder…
welcome-cache-synced = Your Seura Photos folder is synced to { $service }. Thumbnails take up space and would be uploaded too.
welcome-cache-still-synced = { $folder } is synced to { $service } too.
welcome-cache-elsewhere = Store thumbnails somewhere else…
welcome-cache-chosen = Thumbnails will be stored in { $folder }.
welcome-check-updates = Check for updates automatically
welcome-get-started = Get started
welcome-problem = Something went wrong: { $reason }

## The Library

library-add-folder = Add folder
library-empty = Add a folder to see its photos.
library-added = Added. Scanning for photos…
library-already-included = That folder is already in your library, in { $folder }.
library-absorbed =
    { $count ->
        [one] Added. It includes a folder that was already in your library.
       *[other] Added. It includes { $count } folders that were already in your library.
    }
library-add-failed = That folder couldn't be added: { $reason }
library-folders = Folders
library-photo-count =
    { $count ->
        [one] { $count } photo
       *[other] { $count } photos
    }
library-folder-meta =
    { $count ->
        [one] { $path } · { $count } photo
       *[other] { $path } · { $count } photos
    }
library-pick-folder = Choose a folder on the left.
library-tags = Tags
# The sidebar, for screen readers: its folders and tags.
library-sidebar = Folders and tags
# The TAGS section before anything is tagged.
tags-none = No tags yet. Select photos and press T to tag them.
# Under a tag's name above the grid: how many photos have it, from every folder.
library-tag-meta =
    { $count ->
        [one] { $count } photo, from every folder
       *[other] { $count } photos, from every folder
    }
# A tag tree node no photo has exactly, such as "family" when only "family/kids" is used (TAG-5).
library-tag-empty = No photo has exactly “{ $tag }”. Its photos are under the tags beneath it.
library-selected = { $count } selected
library-unavailable = Not connected
library-unavailable-title = { $path } (drive not connected)
library-remove-folder = Remove from library…
library-remove-title = Remove “{ $name }” from your library?
library-remove-body = Its photos stay where they are, and so do their edits. Seura just stops showing them and clears their thumbnails.
library-remove-confirm = Remove
library-remove-cancel = Cancel
library-removed = Removed from your library. Nothing on disk was changed.
library-remove-failed = That folder couldn't be removed: { $reason }

## The status bar (LIB-3)

status-scanning = Looking for photos in { $folder }…
status-thumbnails = Making thumbnails · { $done } of { $total }

## The grid

grid-label = Photos in { $folder }
grid-cloud-only = In the cloud
grid-needs-heif-extensions = Needs Microsoft's HEIF and HEVC extensions
grid-failed = Can't show this photo
grid-sort = Sort by
grid-sort-date-taken = Date taken
grid-sort-file-name = File name
grid-sort-date-modified = Date modified
grid-ascending = Oldest or A to Z first
grid-descending = Newest or Z to A first
grid-size = Size
grid-edited = Edited
grid-unreadable = Its edit file can't be read
grid-cell-label =
    { $rating ->
        [0] { $name }, { $date }
        [one] { $name }, { $date }, 1 star
       *[other] { $name }, { $date }, { $rating } stars
    }

## The Info panel (BRW-6)

info-label = Info
info-nothing = Select a photo to see its details.
info-size = Size
info-dimensions-size = { $width } × { $height } · { $size }
info-camera = Camera
info-lens = Lens
info-focal-length = { $mm } mm
info-focal-length-actual = { $mm } mm actual
info-aperture = ƒ/{ $value }
info-exposure = Exposure
info-shutter-fraction = 1/{ $denominator } s
info-shutter-seconds = { $seconds } s
info-iso = ISO { $iso }
info-exposure-bias = { $ev } EV
info-modified = Modified { $date }
info-location = Location
info-coordinates = { $latitude }, { $longitude }
info-folder = Folder
info-edits = Edits
info-edited = Edited
info-not-edited = None
info-unreadable = Its edit file can't be read
info-cloud-only = This photo is in the cloud. Its camera, lens, and location show once it's on this computer.
info-rating = Rating
info-stars =
    { $rating ->
        [0] Not rated
        [one] 1 star
       *[other] { $rating } stars
    }
# Beside empty stars when the selected photos' ratings differ (TAG-8).
info-rating-mixed = Mixed
# A star button: clicking it sets that rating.
info-rate =
    { $rating ->
        [one] Rate 1 star
       *[other] Rate { $rating } stars
    }
# The current rating's star: clicking it again clears the rating.
info-clear-rating = Clear rating
info-tags = Tags
info-no-tags = None
info-selected =
    { $count ->
        [one] 1 photo selected
       *[other] { $count } photos selected
    }
info-date-range = { $first } to { $last }
## Tags and ratings (TAG-1 to TAG-8)

# Where the rating and tags are in the Info panel, and as a notice when 0 to 5 or T is pressed there.
read-only-folder = This folder is read-only, so tags and ratings can't be saved here.
# The tag picker (TAG-3; wireframes, section 7).
picker-title =
    { $count ->
        [one] Tag 1 photo
       *[other] Tag { $count } photos
    }
picker-keys = Enter adds · Esc closes
picker-added = Tags added
picker-field = Tag
picker-list = Tags
# The first row when what's typed isn't a tag yet.
picker-create = Create tag “{ $tag }”
# Beside a tag only some of the photos have (TAG-8).
picker-some = { $count } of { $total }
info-add-tag = Add tag
# A chip's × button.
tag-remove = Remove { $tag }
# A chip for a tag only some of the selected photos have (TAG-8).
info-tag-partial = { $tag } · { $count } of { $total }
# After undo or redo in the Library (BRW-11).
history-undid =
    { $kind ->
        [rating] Undid rating
        [addTag] Undid adding “{ $tag }”
       *[removeTag] Undid removing “{ $tag }”
    } on { $count ->
        [one] 1 photo.
       *[other] { $count } photos.
    }
history-redid =
    { $kind ->
        [rating] Redid rating
        [addTag] Redid adding “{ $tag }”
       *[removeTag] Redid removing “{ $tag }”
    } on { $count ->
        [one] 1 photo.
       *[other] { $count } photos.
    }
history-skipped =
    { $count ->
        [one] 1 photo was changed since, so it was left alone.
       *[other] { $count } photos were changed since, so they were left alone.
    }
history-nothing-to-undo = Nothing to undo.
history-nothing-to-redo = Nothing to redo.
history-failed = Couldn't undo or redo that: { $reason }
change-read-only-some =
    { $count ->
        [one] 1 photo is in a read-only folder, so it wasn't changed.
       *[other] { $count } photos are in read-only folders, so they weren't changed.
    }
change-unreadable =
    { $count ->
        [one] 1 photo's edit file can't be read, so it wasn't changed.
       *[other] { $count } photos' edit files can't be read, so they weren't changed.
    }
change-failed =
    { $count ->
        [one] Couldn't save the change to 1 photo.
       *[other] Couldn't save the change to { $count } photos.
    }

## Opening photos elsewhere (BRW-8)

photo-show-in-explorer = Show in Explorer
photo-show-in-finder = Show in Finder
photo-open =
    { $count ->
        [one] Open in default app
       *[other] Open { $count } in default app
    }
photo-open-title = Open { $count } photos?
photo-open-windows = Depending on the app, each may open in its own window.
photo-open-cloud =
    { $count ->
        [one] 1 of them is in the cloud and will be downloaded to this computer.
       *[other] { $count } of them are in the cloud and will be downloaded to this computer.
    }
photo-open-confirm = Open
photo-open-cancel = Cancel
photo-show-failed = Couldn't show that in the folder: { $reason }
photo-open-failed = Couldn't open that: { $reason }

## Viewer (BRW-5)

viewer-back = Back to { $folder }
viewer-position = { $position } of { $total }
viewer-previous = Previous photo
viewer-next = Next photo
viewer-photo = { $name }
viewer-fit = Fit · { $percent }
viewer-zoomed = { $percent }
viewer-zoom = Zoom
viewer-zoom-out = Zoom out
viewer-zoom-in = Zoom in
viewer-zoom-fit = Fit
viewer-zoom-full = { $percent }
viewer-keys = ← → next and previous · + − zoom · T tag · 0–5 rate · Esc back to { $folder }
viewer-keys-edit = ← → next and previous · + − zoom · T tag · 0–5 rate · E edit · Esc back to { $folder }
viewer-downloading = Downloading { $size } from the cloud…
viewer-download-failed = This photo is in the cloud, and it couldn't be downloaded. Check your connection.
viewer-try-again = Try again
viewer-failed = This photo can't be shown: { $reason }
viewer-no-preview = This computer's graphics can't show photos here.

## M1's Editor (development builds only until M4)

editor-back = Back to photos
editor-preview = { $name }, edited
editor-exposure = Exposure
editor-ev = { $value } EV
editor-export = Export…
editor-exporting = Exporting…
editor-exported = Exported { $file } to { $folder }.
editor-export-failed = The photo couldn't be exported: { $reason }
editor-open-failed = This photo couldn't be opened: { $reason }
editor-no-preview = This computer's graphics can't show the preview.
editor-save-failed = Your change couldn't be saved: { $reason }
editor-read-only = This photo's edit file can't be read, so Seura won't change it: { $reason }

## Developer tools

dev-switches-on = Developer switches on: { $switches }
check-running = Checking that the preview matches export…
