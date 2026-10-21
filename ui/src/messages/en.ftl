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
