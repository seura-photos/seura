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
# Between a folder's path and its photo count above the grid.
heading-separator = ·
library-pick-folder = Choose a folder on the left.
library-tags = Tags
# The search box in the header (FND-2).
search-label = Search
search-placeholder = Search file names and tags
# Above the grid while searching, with the number found beside it.
search-title = Results for “{ $text }”
search-none = No photos match “{ $text }”.
# What the Viewer goes back to from search results.
search-back = Results

## Problems with files: the notice bar above the grid (user flows, flow 11).

# The selected photos' edit files can't be read (FIL-4).
start-over-unreadable =
    { $count ->
        [one] The edit file for { $name } can't be read, so the photo is shown unedited. Start over keeps that file aside, in Seura's folder, and begins again with no edits, tags, or rating.
       *[other] { $count } of the selected photos have edit files that can't be read, so they're shown unedited. Start over keeps those files aside, in Seura's folder, and begins again with no edits, tags, or rating.
    }
start-over = Start over
# Another program changed the photo since its edits were made (FIL-7).
notice-original-changed = This photo was changed by another program; edits may not line up.
# Its edit file has operations Seura can't render, from a newer version or another app (FIL-5).
notice-not-shown =
    { $count ->
        [one] This photo has an edit Seura Photos can't show, made by a newer version or another app. It's kept when you change the photo.
       *[other] This photo has { $count } edits Seura Photos can't show, made by a newer version or another app. They're kept when you change the photo.
    }
start-over-kept =
    { $count ->
        [one] The old edit file is kept in “Unreadable edit files” in Seura's folder.
       *[other] The { $count } old edit files are kept in “Unreadable edit files” in Seura's folder.
    }
start-over-show = Show in folder
start-over-readable =
    { $count ->
        [one] One edit file can be read now, so it was left as it is.
       *[other] { $count } edit files can be read now, so they were left as they are.
    }
start-over-failed =
    { $count ->
        [one] One photo couldn't be started over.
       *[other] { $count } photos couldn't be started over.
    }

## Filters (FND-1): they narrow whatever the grid shows.

filter = Filter
# The Filter button while filters are set, with how many (FND-1).
filter-with-count = Filter · { $count }
filter-controls = Filters
filter-rating = Rating
filter-any = Any
filter-min-rating =
    { $stars ->
        [one] At least 1 star
        [5] 5 stars
       *[other] At least { $stars } stars
    }
filter-tags = Tags
# The first entry of the tag list; choosing a tag adds it to the filter.
filter-add-tag = Add a tag…
filter-edits = Edits
filter-edited = Edited
filter-unedited = Not edited
filter-taken-from = Taken from
filter-taken-to = to
# Chips for the date taken; $from and $to are dates, formatted for the region.
filter-taken-between = Taken { $from } to { $to }
filter-taken-since = Taken since { $date }
filter-taken-until = Taken until { $date }
filter-active = Active filters
# A chip's × button.
filter-remove = Clear { $filter }
filter-clear-all = Clear all
filter-none = No photos here match the filters.
filter-clear = Clear filters
# The count while filtered: how many are shown, of how many there are.
filter-count =
    { $count ->
        [one] { $shown } of { $count } photo
       *[other] { $shown } of { $count } photos
    }
# The sidebar, for screen readers: its folders and tags.
library-sidebar = Folders and tags
# The divider between the sidebar and the grid (#74): its name, its value
# for screen readers, and its tooltip.
sidebar-width = Sidebar width
sidebar-width-value = { $width } pixels
sidebar-resize = Drag to resize. Double-click to reset.
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
# The size slider's tooltip.
grid-size-reset = Double-click for the default size
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
# The photo menu's Delete (BRW-9). $system is "mac" or "windows".
photo-delete =
    { $system ->
        [mac] Move to Trash…
       *[other] Move to Recycle Bin…
    }

## Deleting photos (BRW-9). $system is "mac" or "windows".

delete-title =
    { $count ->
        [one]
            { $system ->
                [mac] Move this photo to the Trash?
               *[other] Move this photo to the Recycle Bin?
            }
       *[other]
            { $system ->
                [mac] Move { $count } photos to the Trash?
               *[other] Move { $count } photos to the Recycle Bin?
            }
    }
# $edited is "yes" when any of the photos has an edit file, which goes too,
# as a separate file to restore (design answers, 2026-10-27).
delete-body =
    { $edited ->
        [yes]
            { $count ->
                [one]
                    { $system ->
                        [mac] Its edits, tags, and rating are in a small file beside it, which goes too. To get everything back, put both back from the Trash.
                       *[other] Its edits, tags, and rating are in a small file beside it, which goes too. To get everything back, restore both from the Recycle Bin.
                    }
               *[other]
                    { $system ->
                        [mac] Their edits, tags, and ratings are in small files beside them, which go too. To get everything back, put the photos and their files back from the Trash.
                       *[other] Their edits, tags, and ratings are in small files beside them, which go too. To get everything back, restore the photos and their files from the Recycle Bin.
                    }
            }
       *[no]
            { $count ->
                [one]
                    { $system ->
                        [mac] You can put it back from the Trash.
                       *[other] You can restore it from the Recycle Bin.
                    }
               *[other]
                    { $system ->
                        [mac] You can put them back from the Trash.
                       *[other] You can restore them from the Recycle Bin.
                    }
            }
    }
# Every photo being deleted is in a folder $service syncs, such as OneDrive.
delete-synced-all =
    { $count ->
        [one] This photo is in { $service }. It'll be removed from { $service } on all your devices too. { $service } keeps deleted files in its own recycle bin for a while.
       *[other] These photos are in { $service }. They'll be removed from { $service } on all your devices too. { $service } keeps deleted files in its own recycle bin for a while.
    }
delete-synced-some =
    { $count ->
        [one] 1 of these photos is in { $service }. It'll be removed from { $service } on all your devices too. { $service } keeps deleted files in its own recycle bin for a while.
       *[other] { $count } of these photos are in { $service }. They'll be removed from { $service } on all your devices too. { $service } keeps deleted files in its own recycle bin for a while.
    }
# Cloud-only photos can't go to the Recycle Bin or Trash, so they're left as they are.
delete-cloud-only =
    { $count ->
        [one]
            { $system ->
                [mac] 1 photo is only in the cloud, so it can't go to the Trash and is left as it is. Delete it from its cloud service, or download it first.
               *[other] 1 photo is only in the cloud, so it can't go to the Recycle Bin and is left as it is. Delete it from its cloud service, or download it first.
            }
       *[other]
            { $system ->
                [mac] { $count } photos are only in the cloud, so they can't go to the Trash and are left as they are. Delete them from their cloud service, or download them first.
               *[other] { $count } photos are only in the cloud, so they can't go to the Recycle Bin and are left as they are. Delete them from their cloud service, or download them first.
            }
    }
delete-read-only =
    { $count ->
        [one] 1 photo is in a read-only folder, so it's left as it is.
       *[other] { $count } photos are in read-only folders, so they're left as they are.
    }
# Photos on a drive with no Recycle Bin, where Windows would offer to delete
# them permanently. Not on the Mac, where the system refuses instead.
delete-no-trash =
    { $count ->
        [one]
            { $system ->
                [mac] 1 photo is on a drive with no Trash, so it's left as it is.
               *[other] 1 photo is on a drive with no Recycle Bin, such as a USB stick or a network share, so it's left as it is.
            }
       *[other]
            { $system ->
                [mac] { $count } photos are on a drive with no Trash, so they're left as they are.
               *[other] { $count } photos are on a drive with no Recycle Bin, such as a USB stick or a network share, so they're left as they are.
            }
    }
delete-confirm =
    { $system ->
        [mac] Move to Trash
       *[other] Move to Recycle Bin
    }
delete-cancel = Cancel
# None of the photos can be deleted; the dialog says why, with OK.
delete-nothing-title =
    { $count ->
        [one]
            { $system ->
                [mac] This photo can't be moved to the Trash
               *[other] This photo can't be moved to the Recycle Bin
            }
       *[other]
            { $system ->
                [mac] These photos can't be moved to the Trash
               *[other] These photos can't be moved to the Recycle Bin
            }
    }
delete-ok = OK
delete-done =
    { $count ->
        [one]
            { $system ->
                [mac] Moved 1 photo to the Trash.
               *[other] Moved 1 photo to the Recycle Bin.
            }
       *[other]
            { $system ->
                [mac] Moved { $count } photos to the Trash.
               *[other] Moved { $count } photos to the Recycle Bin.
            }
    }
delete-failed =
    { $count ->
        [one]
            { $system ->
                [mac] 1 photo couldn't be moved to the Trash, so it's where it was.
               *[other] 1 photo couldn't be moved to the Recycle Bin, so it's where it was.
            }
       *[other]
            { $system ->
                [mac] { $count } photos couldn't be moved to the Trash, so they're where they were.
               *[other] { $count } photos couldn't be moved to the Recycle Bin, so they're where they were.
            }
    }

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
