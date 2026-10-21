# Seura Photos: the interface's text (ACC-5). Project Fluent; see
# https://projectfluent.org. English only in v1. Numbers passed in are
# formatted in the operating system's region (ACC-6).

app-name = Seura Photos
app-short-name = Seura
app-version = Version { $version }

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

## The grid

grid-label = Photos in { $folder }
grid-cloud-only = In the cloud
grid-needs-heif-extensions = Needs Microsoft's HEIF and HEVC extensions
grid-failed = Can't show this photo

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
