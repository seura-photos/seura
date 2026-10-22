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
info-aperture = ƒ/{ $value }
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
info-rating-mixed = Not all the same
info-tags = Tags
info-shared-tags = Tags they share
info-no-tags = None
info-selected =
    { $count ->
        [one] 1 photo selected
       *[other] { $count } photos selected
    }
info-date-range = { $first } to { $last }

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
viewer-keys = ← → next and previous · + − zoom · Esc back to { $folder }
viewer-keys-edit = ← → next and previous · + − zoom · E edit · Esc back to { $folder }
viewer-cloud-only = This photo is in the cloud and hasn't been downloaded to this computer.
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
