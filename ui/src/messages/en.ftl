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
library-edit = Edit
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
# The sidebar trees (LIB-2, TAG-5).
tree-collapse-all = Collapse all
tree-expand-beneath = Expand all beneath
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
grid-downloading = Downloading…
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
grid-cell-label-cloud =
    { $rating ->
        [0] { $name }, { $date }, in the cloud
        [one] { $name }, { $date }, 1 star, in the cloud
       *[other] { $name }, { $date }, { $rating } stars, in the cloud
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
        [rotateLeft] Undid rotating left
        [rotateRight] Undid rotating right
        [enhance] Undid Enhance
        [paste] Undid pasting edits
        [preset] Undid the preset “{ $tag }”
        [unmark] Undid unmarking
        [addTag] Undid adding “{ $tag }”
       *[removeTag] Undid removing “{ $tag }”
    } on { $count ->
        [one] 1 photo.
       *[other] { $count } photos.
    }
history-redid =
    { $kind ->
        [rating] Redid rating
        [rotateLeft] Redid rotating left
        [rotateRight] Redid rotating right
        [enhance] Redid Enhance
        [paste] Redid pasting edits
        [preset] Redid the preset “{ $tag }”
        [unmark] Redid unmarking
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
# After a change made with a key: 0 to 5, or T and the tag picker (INT-6).
change-done-rated =
    { $count ->
        [one] Rated 1 photo
       *[other] Rated { $count } photos
    } { $stars ->
        [one] 1 star.
       *[other] { $stars } stars.
    }
change-done-cleared =
    { $count ->
        [one] Cleared the rating on 1 photo.
       *[other] Cleared the rating on { $count } photos.
    }
change-done-rotated =
    { $count ->
        [one] Rotated 1 photo
       *[other] Rotated { $count } photos
    } { $clockwise ->
        [true] right.
       *[false] left.
    }
change-done-enhanced =
    { $count ->
        [one] Enhanced 1 photo.
       *[other] Enhanced { $count } photos.
    }
# Enhance on a selection that includes photos in the cloud (BRW-10, LIB-9).
enhance-confirm-title =
    { $count ->
        [one] Enhance 1 photo?
       *[other] Enhance { $count } photos?
    }
enhance-confirm-body =
    { $count ->
        [one] 1 of these photos is in the cloud ({ $size }). Download it to enhance it?
       *[other] { $count } of these photos are in the cloud ({ $size }). Download them to enhance them?
    }
enhance-confirm-download = Download and enhance
enhance-confirm-on-computer =
    { $count ->
        [one] Enhance only the 1 on this computer
       *[other] Enhance only the { $count } on this computer
    }
enhance-confirm-cancel = Cancel
enhance-downloading = Downloading { $at } of { $total }…
enhance-stop = Stop
enhance-done-some = Enhanced { $done } of { $count } photos.
enhance-stopped =
    { $count ->
        [one] 1 photo wasn't enhanced, because you stopped.
       *[other] { $count } photos weren't enhanced, because you stopped.
    }
enhance-still-downloading = One that was already downloading may still arrive on this computer.
enhance-not-downloaded =
    { $count ->
        [one] 1 photo couldn't be downloaded, so it wasn't enhanced. Check your connection.
       *[other] { $count } photos couldn't be downloaded, so they weren't enhanced. Check your connection.
    }
# Chosen in Enhance's question: only the photos on this computer (BRW-10).
change-cloud-only =
    { $count ->
        [one] 1 photo is in the cloud, so it wasn't enhanced.
       *[other] { $count } photos are in the cloud, so they weren't enhanced.
    }
change-done-added =
    { $count ->
        [one] Added the tag { $tag } to 1 photo.
       *[other] Added the tag { $tag } to { $count } photos.
    }
change-done-removed =
    { $count ->
        [one] Removed the tag { $tag } from 1 photo.
       *[other] Removed the tag { $tag } from { $count } photos.
    }
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
# Said once per cloud service, the first time an edit file goes to the Trash
# from a folder it syncs (FIL-1): the service may report a deletion of its
# own. $removal is "rating", "tag", or "other" (an edit reset, undo, redo,
# revert, or paste); $service is its name, such as "OneDrive"; $system is
# "mac" or "windows".
deletion-explained =
    { $count ->
        [one]
            { $removal ->
                [rating] Removing this photo's last rating moved its small edit file to the { $system ->
                        [mac] Trash
                       *[other] Recycle Bin
                    }.
                [tag] Removing this photo's last tag moved its small edit file to the { $system ->
                        [mac] Trash
                       *[other] Recycle Bin
                    }.
               *[other] This photo has no edits, tags or rating left, so its small edit file moved to the { $system ->
                        [mac] Trash
                       *[other] Recycle Bin
                    }.
            } { $service } may say a file was deleted; the photo itself isn't affected.
       *[other]
            { $removal ->
                [rating] Removing these photos' ratings moved { $count } small edit files to the { $system ->
                        [mac] Trash
                       *[other] Recycle Bin
                    }.
                [tag] Removing these photos' tags moved { $count } small edit files to the { $system ->
                        [mac] Trash
                       *[other] Recycle Bin
                    }.
               *[other] { $count } photos have no edits, tags or ratings left, so their small edit files moved to the { $system ->
                        [mac] Trash
                       *[other] Recycle Bin
                    }.
            } { $service } may say files were deleted; the photos themselves aren't affected.
    }
# The photo menu's Presets submenu (PRE-4), and its note when there are none.
photo-presets = Presets
photo-no-presets = No presets
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
# At the end of a review: how marked photos in the cloud get to the Trash (REV-4).
delete-will-download =
    { $count ->
        [one]
            { $system ->
                [mac] 1 photo is only in the cloud ({ $size }). It will be downloaded first, so it can go to the Trash. If it isn't downloaded, it stays as it is and stays marked.
               *[other] 1 photo is only in the cloud ({ $size }). It will be downloaded first, so it can go to the Recycle Bin. If it isn't downloaded, it stays as it is and stays marked.
            }
       *[other]
            { $system ->
                [mac] { $count } photos are only in the cloud ({ $size }). They'll be downloaded one at a time, so they can go to the Trash. Any that aren't downloaded stay as they are and stay marked.
               *[other] { $count } photos are only in the cloud ({ $size }). They'll be downloaded one at a time, so they can go to the Recycle Bin. Any that aren't downloaded stay as they are and stay marked.
            }
    }
delete-not-downloaded =
    { $count ->
        [one] 1 photo wasn't downloaded, so it stays in the cloud and stays marked.
       *[other] { $count } photos weren't downloaded, so they stay in the cloud and stay marked.
    }
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
# Photos on a drive with no Recycle Bin or Trash, where deleting would be
# permanent: Seura never deletes them (decision 0017).
delete-no-trash =
    { $count ->
        [one]
            { $system ->
                [mac] 1 photo is on a drive with no Trash, such as a network share. Deleting it there would be permanent, so it's left as it is. You can delete it in Finder.
               *[other] 1 photo is on a drive with no Recycle Bin, such as a USB stick or a network share. Deleting it there would be permanent, so it's left as it is. You can delete it in File Explorer.
            }
       *[other]
            { $system ->
                [mac] { $count } photos are on a drive with no Trash, such as a network share. Deleting them there would be permanent, so they're left as they are. You can delete them in Finder.
               *[other] { $count } photos are on a drive with no Recycle Bin, such as a USB stick or a network share. Deleting them there would be permanent, so they're left as they are. You can delete them in File Explorer.
            }
    }
# On the Mac, photos in a file provider's folder such as OneDrive's: macOS
# refuses to move them to the Trash for Seura (#112), so they're left as
# they are, and Finder can delete them. $service is "OneDrive", for example.
delete-file-provider =
    { $count ->
        [one] 1 photo is in { $service }, where macOS doesn't let Seura Photos move photos to the Trash, so it's left as it is. You can delete it in Finder.
       *[other] { $count } photos are in { $service }, where macOS doesn't let Seura Photos move photos to the Trash, so they're left as they are. You can delete them in Finder.
    }
delete-cloud-service = a cloud service's folder
delete-show-in-finder = Show in Finder
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
# The system's own words for why, after delete-failed (#112).
delete-failed-reason =
    { $system ->
        [mac] macOS said: { $reason }
       *[other] Windows said: { $reason }
    }

photo-show-failed = Couldn't show that in the folder: { $reason }
photo-open-failed = Couldn't open that: { $reason }

## Commands for the whole app: Windows' ⋯ button at the right end of the
## toolbar (ACC-4), and About (SET-5)

more = More
more-about = About Seura Photos
about-early-build = An early build, before the beta.
about-copy = Copy
about-copied = Copied.
about-copy-failed = Couldn't copy: { $reason }
about-website = seuraphotos.app
about-project = The project on GitHub
about-license = Apache License 2.0
about-ok = OK
about-link-failed = Couldn't open that: { $reason }

## Zoom, in the Viewer and the Editor (BRW-5, EDT-1)

zoom = Zoom
zoom-out = Zoom out
zoom-in = Zoom in
zoom-fit = Fit
zoom-full = { $percent }

## Viewer (BRW-5)

viewer-back = Back to { $folder }
viewer-position = { $position } of { $total }
viewer-previous = Previous photo
viewer-next = Next photo
viewer-photo = { $name }
viewer-fit = Fit · { $percent }
viewer-zoomed = { $percent }
viewer-keys = ← → next and previous · + − zoom · T tag · 0–5 rate · E edit · Esc back to { $folder }
viewer-downloading = Downloading { $size } from the cloud…
viewer-download-failed = This photo is in the cloud, and it couldn't be downloaded. Check your connection.
viewer-try-again = Try again
viewer-edit = Edit
viewer-failed = This photo can't be shown: { $reason }
viewer-no-preview = This computer's graphics can't show photos here.

# Review (REV-1 to REV-6). The toolbar's button and its tooltips.
review-start = Review
review-start-all = Sort through these photos one at a time
review-start-selection =
    { $count ->
        [one] Sort through the selected photo
       *[other] Sort through the { $count } selected photos
    }
# Above the photo: what's being reviewed. $name is a folder or tag.
review-title = Reviewing { $name }
review-title-search = Reviewing search results
review-title-selection =
    { $count ->
        [one] Reviewing 1 photo
       *[other] Reviewing { $count } photos
    }
review-exit = Exit review
review-back = Back
review-next = Next
review-edit = Edit
review-edit-key = Edit (E)
review-tag = Tag
review-tag-key = Tag (T)
review-stars = Rating
# Under the photo, when Enhance is applied.
review-enhanced = Enhanced
# Marking for deletion (REV-4): always "mark", since nothing is removed yet.
review-mark = Mark to delete
review-marked = Marked ✓
review-mark-key = Mark to delete (X)
review-unmark-key = Unmark (X)
review-marked-banner = Marked for deletion · deleted only when you finish · X to unmark
review-left-marked =
    { $count ->
        [one] 1 photo is still marked for deletion. Review it again to finish.
       *[other] { $count } photos are still marked for deletion. Review them again to finish.
    }
mark-failed = Couldn't save that mark: { $reason }
# The banner offering to review photos that have just arrived (REV-1).
review-new-banner =
    { $count ->
        [one] 1 new photo in { $folder }. Sort through it?
       *[other]
            { $others ->
                [0] { $count } new photos in { $folder }. Sort through them one at a time?
                [one] { $count } new photos in { $folder } and 1 other folder. Sort through them one at a time?
               *[other] { $count } new photos in { $folder } and { $others } other folders. Sort through them one at a time?
            }
    }
review-new-start = Review
review-new-dismiss = Not now
review-title-new = Reviewing new photos
# A review of photos that are in the cloud (REV-2, LIB-9).
review-cloud-title =
    { $count ->
        [one] Review 1 photo?
       *[other] Review { $count } photos?
    }
review-cloud-body =
    { $cloudOnly ->
        [one] 1 of these photos is in the cloud ({ $size }). Download it when you reach it?
       *[other] { $cloudOnly } of these photos are in the cloud ({ $size }). Download them as you reach them?
    }
review-cloud-download = Download as I go
review-cloud-thumbnails = Show thumbnails only
review-cloud-cancel = Cancel
review-in-cloud = In the cloud ({ $size })
# The summary after a review's last photo (REV-5).
review-summary-title = End of review
review-summary-rated = Rated
review-summary-edited = Edited
review-summary-tagged = Tagged
review-summary-marked = Marked for deletion
review-summary-saved = Everything is saved, and nothing has been deleted yet.
review-summary-still-marked =
    { $count ->
        [one] 1 marked photo couldn't be moved, and stays marked.
       *[other] { $count } marked photos couldn't be moved, and stay marked.
    }
review-summary-move =
    { $count ->
        [one]
            { $system ->
                [mac] Move 1 marked photo to the Trash
               *[other] Move 1 marked photo to the Recycle Bin
            }
       *[other]
            { $system ->
                [mac] Move { $count } marked photos to the Trash
               *[other] Move { $count } marked photos to the Recycle Bin
            }
    }
review-summary-export =
    { $count ->
        [0] No photos to export
        [one] Export 1 photo
       *[other] Export { $count } photos
    }
review-summary-library = Back to the Library
# X in the Library or Viewer with no marked photo chosen (REV-6).
mark-outside-review =
    Marking for deletion happens in Review. To move photos to { $platform ->
        [mac] the Trash
       *[other] the Recycle Bin
    } now, use Delete.
# A marked photo in the Library and Viewer (REV-6).
grid-marked-tooltip = { $name } · Marked for deletion
grid-cell-label-marked = { $label }, marked for deletion
viewer-photo-marked = { $name }, marked for deletion
info-marked = Marked for deletion
info-marked-some = { $count } of { $total } marked for deletion
info-unmark = Unmark
# The rating, shown as stars under the photo.
review-rating =
    { $stars ->
        [one] 1 star
       *[other] { $stars } stars
    }

## The Editor (EDT-*)

editor-saved-automatically = Edits saved automatically
editor-not-saved = Edits can't be saved
editor-preview = { $name }, edited
editor-original = { $name }, original
editor-tools = Tools
editor-compare = Hold to compare
editor-done = Done
# Crop mode (EDT-2, EDT-3; wireframes, "Crop and straighten").
# The Export dialog, its progress, and its results (EXP-1 to EXP-6).
toolbar-export = Export
toolbar-copy-edits = Copy edits
# Copying and pasting edits (PRE-1, PRE-2).
copy-done = Copied the edits of { $name }.
copy-no-edits = { $name } has no edits to copy, so what you copied before stays.
copy-unreadable = The edits of { $name } can't be read, so what you copied before stays.
copy-failed = Couldn't copy those edits: { $reason }
paste-title =
    { $count ->
        [one] Paste edits onto 1 photo
       *[other] Paste edits onto { $count } photos
    }
paste-from = Copied from { $name }.
paste-adjustments = Adjustments (tone and color)
paste-effects = Effects
paste-geometry = Crop and rotation
paste-none = None copied.
paste-none-removes =
    { $count ->
        [one] None copied, so pasting removes any on this photo.
       *[other] None copied, so pasting removes any on these photos.
    }
paste-cancel = Cancel
paste-go = Paste
paste-nothing = Nothing is copied yet. Copy a photo's edits first, with { $key }.
paste-done =
    { $count ->
        [one] Pasted edits onto 1 photo.
       *[other] Pasted edits onto { $count } photos.
    }
paste-no-change =
    { $count ->
        [one] Nothing changed: that photo already has these edits.
       *[other] Nothing changed: those photos already have these edits.
    }
paste-cloud-kept =
    { $count ->
        [one] 1 photo is in the cloud, so its crop and rotation weren't changed.
       *[other] { $count } photos are in the cloud, so their crop and rotation weren't changed.
    }
# Presets in the Editor (PRE-3, PRE-5; wireframes, "Editor", item 9).
# A preset's name is its file's name. $system is "mac" or "windows".
presets-title = Presets
presets-none = No presets yet.
presets-save = Save current edits as a preset…
presets-unreadable = Can't be read
presets-rename = Rename…
presets-delete =
    { $system ->
        [mac] Move to Trash…
       *[other] Move to Recycle Bin…
    }
presets-nothing-to-save = This photo has no edits to save as a preset.
presets-save-title = Save as preset
presets-name = Name
presets-group-empty = None in this photo's edits.
presets-cancel = Cancel
presets-save-go = Save
presets-replace = Replace
presets-name-in-use = A preset named “{ $name }” already exists. Replace it, or choose another name.
presets-name-in-use-unreadable = A preset named “{ $name }” already exists and can't be read, so it isn't replaced. Choose another name.
presets-name-invalid = That name can't be used for a file. Use some letters or numbers.
presets-save-failed = Couldn't save the preset: { $reason }
presets-saved = Saved the preset “{ $name }”.
presets-replaced = Replaced the preset “{ $name }”.
# The name had characters a file name can't hold, which were changed.
presets-saved-as = Saved the preset as “{ $name }”, since file names can't hold some characters.
presets-rename-title = Rename preset
presets-rename-go = Rename
presets-rename-in-use = A preset named “{ $name }” already exists. Choose another name.
presets-rename-failed = Couldn't rename the preset: { $reason }
presets-renamed-as = Renamed the preset “{ $name }”, since file names can't hold some characters.
presets-gone = The preset “{ $name }” is no longer in the Presets folder.
presets-delete-title =
    { $system ->
        [mac] Move the preset “{ $name }” to the Trash?
       *[other] Move the preset “{ $name }” to the Recycle Bin?
    }
presets-delete-body =
    { $system ->
        [mac] You can restore it from the Trash.
       *[other] You can restore it from the Recycle Bin.
    }
presets-delete-go =
    { $system ->
        [mac] Move to Trash
       *[other] Move to Recycle Bin
    }
presets-deleted =
    { $system ->
        [mac] Moved the preset “{ $name }” to the Trash.
       *[other] Moved the preset “{ $name }” to the Recycle Bin.
    }
presets-delete-no-trash =
    { $system ->
        [mac] The preset “{ $name }” is on a drive with no Trash, so it's left as it is.
       *[other] The preset “{ $name }” is on a drive with no Recycle Bin, so it's left as it is.
    }
# Applying a preset (PRE-4), from the Library or the Editor.
preset-applied =
    { $count ->
        [one] Applied “{ $name }” to 1 photo.
       *[other] Applied “{ $name }” to { $count } photos.
    }
preset-no-change =
    { $count ->
        [one] Nothing changed: that photo already looks like this preset.
       *[other] Nothing changed: those photos already look like this preset.
    }
preset-gone = The preset “{ $name }” is no longer in the Presets folder.
preset-unreadable = The preset “{ $name }” can't be read, so it wasn't applied.
preset-empty = The preset “{ $name }” has nothing Seura can apply.
presets-delete-failed =
    { $system ->
        [mac] Couldn't move the preset to the Trash: { $reason }
       *[other] Couldn't move the preset to the Recycle Bin: { $reason }
    }
export-title =
    { $count ->
        [one] Export 1 photo
       *[other] Export { $count } photos
    }
export-save-to = Save to
export-no-folder = Choose a folder
export-choose = Choose…
export-format = Format
export-jpeg = JPEG
export-png = PNG
export-quality = Quality
export-size = Size
export-size-original = Original size
export-size-long-edge = Long edge up to { $edge } px
export-size-custom = Custom, up to
export-size-custom-pixels = Custom maximum long edge, in pixels
export-size-px = px
export-size-note = Photos smaller than the limit keep their size and aren't enlarged.
export-size-custom-hint = Enter a whole number of pixels, { $min } or more.
export-details = Details
export-remove-location = Remove location data
export-include-keywords = Include tags and rating as keywords
export-originals = Edits are applied to the exported copies. Originals aren't changed, and existing files are never overwritten: a name that's taken gets a number, like "IMG_4417 (1).jpg".
export-cloud =
    { $total ->
        [one] This photo is in the cloud ({ $size }) and will be downloaded.
       *[other]
            { $count ->
                [one] 1 of these photos is in the cloud ({ $size }) and will be downloaded.
               *[other] { $count } of these photos are in the cloud ({ $size }) and will be downloaded.
            }
    }
export-cancel = Cancel
export-go = Export
export-busy = An export is still running. Wait for it to finish, or cancel it.
export-folder-failed = The folder dialog couldn't open: { $reason }
export-failed = The export couldn't run: { $reason }
export-starting = Getting ready to export…
export-exporting = Exporting { $at } of { $total }…
export-exporting-one = Exporting { $name }…
export-downloading = Downloading { $at } of { $total }: { $name } ({ $size })…
export-downloading-one = Downloading { $name } ({ $size })…
export-done-one = Exported { $file }.
export-failed-one = { $name } couldn't be exported. { $reason }
export-stopped =
    { $count ->
        [one] 1 photo wasn't exported, because you canceled.
       *[other] { $count } photos weren't exported, because you canceled.
    }
export-still-downloading = One that was already downloading may still arrive on this computer.
export-folder-full = The drive is full, so the export stopped.
export-folder-notWritable = That folder can't be written to, so the export stopped.
export-left =
    { $count ->
        [one] 1 photo wasn't exported.
       *[other] { $count } photos weren't exported.
    }
export-problem-notDownloaded = It couldn't be downloaded. Check your connection.
export-problem-unreadable = It's missing, damaged, or not a photo Seura can read.
export-problem-needsCodec = This computer needs Microsoft's HEIF and HEVC extensions to read it.
export-problem-editsUnreadable = Its edit file can't be read, so its edits aren't known.
export-problem-failed = Something went wrong while making the file. Too many tags for a JPEG is one cause; PNG has no such limit.
export-summary-all =
    { $count ->
        [one] Exported 1 photo
       *[other] Exported { $count } photos
    }
export-summary-some = Exported { $done } of { $count } photos
export-summary-failures =
    { $count ->
        [one] 1 photo couldn't be exported:
       *[other] { $count } photos couldn't be exported:
    }
export-renamed =
    { $count ->
        [one] 1 name was already taken in that folder, so that file got a number. Nothing was replaced, so you may have two copies of that photo.
       *[other] { $count } names were already taken in that folder, so those files got a number. Nothing was replaced, so you may have two copies of those photos.
    }
export-renamed-one = That name was already taken in the folder, so nothing was replaced.
export-summary-ok = OK
# One-click enhance (EDT-12).
editor-enhance = Enhance
# Quick actions in the Library and the Viewer (BRW-10), and on a hovered
# thumbnail.
quick-actions = Quick fixes
quick-rotate-left = Rotate left
quick-rotate-right = Rotate right
quick-enhance = Enhance
quick-rotate-left-key = Rotate left ([)
quick-rotate-right-key = Rotate right (])
quick-enhance-key = Enhance (A)
editor-crop = Crop and straighten
crop-title = Crop and straighten
crop-hint = Enter to apply · Esc to cancel
crop-aspect-ratio = Aspect ratio
crop-original = Original
crop-freeform = Freeform
crop-orientation = Orientation
crop-landscape = Landscape
crop-portrait = Portrait
crop-straighten = Straighten
crop-straighten-note = The crop shrinks as needed so no empty corners show.
crop-reset = Reset crop
crop-cancel = Cancel
crop-done = Done
# The box and its corners take the focus in turn; arrow keys move them.
crop-box = Crop box
crop-handle = crop handle
crop-top-left = Top-left corner
crop-top-right = Top-right corner
crop-bottom-left = Bottom-left corner
crop-bottom-right = Bottom-right corner
# The box's size, in the photo's own pixels, such as "3:2 · 3720 × 2480".
crop-size = { $ratio } · { $width } × { $height }
# Red-eye mode (EDT-7; wireframes, "Red-eye mode").
editor-red-eye = Red-eye
editor-fix-red-eye = Fix red-eye…
red-eye-title = Fix red-eye
red-eye-hint = Enter to apply · Esc to cancel
red-eye-instruction = Click each red eye, or drag across it. Drag a circle to move it.
red-eye-full = That's the most corrections a photo can have. Remove one to add another.
red-eye-count =
    { $total ->
        [0] No corrections yet
        [one] 1 correction
       *[other] { $total } corrections
    }
# { $corrections } is red-eye-count.
red-eye-count-outside = { $corrections }, { $outside } outside the crop
red-eye-remove-all = Remove all
red-eye-cancel = Cancel
red-eye-done = Done
red-eye-correction = Correction { $number }
red-eye-remove = Remove correction { $number }
# The Editor's rotate and flip buttons (EDT-4).
editor-turn = Rotate and flip
editor-rotate-left = Rotate left
editor-rotate-right = Rotate right
editor-flip-horizontal = Flip horizontal
editor-flip-vertical = Flip vertical
editor-light = Light
editor-brightness = Brightness
editor-contrast = Contrast
editor-highlights = Highlights
editor-shadows = Shadows
editor-color = Color
editor-temperature = Temperature
editor-tint = Tint
editor-saturation = Saturation
# The ends of the temperature and tint sliders (EDT-5).
editor-cool = Cool
editor-warm = Warm
editor-green = Green
editor-magenta = Magenta
# The Editor's effects (EDT-6): each turned on and off by its button, which
# shows On or Off, and its sliders shown beneath once it's on.
editor-effects = Effects
editor-black-and-white = Black and white
editor-sepia = Sepia
editor-vignette = Vignette
editor-effect-on = On
editor-effect-off = Off
editor-strength = Strength
editor-vignette-amount = Amount
editor-vignette-size = Size
# The ends of the vignette's Amount slider.
editor-darker = Darker
editor-lighter = Lighter
editor-filmstrip = Filmstrip
editor-open-failed = This photo couldn't be opened: { $reason }
editor-no-preview = This computer's graphics can't show the preview.
editor-save-failed = Your change couldn't be saved: { $reason }
editor-save-read-only = This photo's folder is read-only, so your change couldn't be saved.
editor-save-unreadable = This photo's edit file can't be read, so your change wasn't saved.
editor-read-only = This photo's edit file can't be read, so Seura won't change it: { $reason }
# Undo and redo name the change by the label its edit file's history gives
# it, such as "Brightness" or "Rating" (EDT-9).
editor-undo = Undo
editor-redo = Redo
editor-undo-change = Undo { $change }
editor-redo-change = Redo { $change }
editor-undid = Undid { $change }
editor-redid = Redid { $change }
# Undoing or redoing a rating or tag change made in the Library, which
# the Editor doesn't show.
editor-stepped-rating =
    { $undo ->
        [true] { $stars ->
            [0] Undid removing the rating
           *[other] Undid rating of { $stars ->
                [one] 1 star
               *[other] { $stars } stars
            }
        }
       *[false] { $stars ->
            [0] Redid removing the rating
           *[other] Redid rating of { $stars ->
                [one] 1 star
               *[other] { $stars } stars
            }
        }
    }
editor-stepped-tag =
    { $undo ->
        [true] { $kind ->
            [addTag] Undid adding “{ $tag }”
           *[removeTag] Undid removing “{ $tag }”
        }
       *[false] { $kind ->
            [addTag] Redid adding “{ $tag }”
           *[removeTag] Redid removing “{ $tag }”
        }
    }
editor-revert = Revert to original…
editor-revert-question = Remove every edit to this photo? Its tags and rating stay, and you can undo this.
# Added to the question when the photo has operations from other apps,
# which revert keeps (EDT-10).
editor-revert-other-apps = Edits made in other apps are kept.
editor-revert-confirm = Revert to original
editor-revert-cancel = Cancel
editor-reverted = Reverted to original
editor-revert-failed = The photo couldn't be reverted: { $reason }
# The open photo's edit file changed outside Seura (FIL-3; user flows,
# flow 11).
editor-changed-outside = This photo's edits were changed outside Seura Photos. Keep yours or load theirs?
editor-keep-yours = Keep yours
editor-load-theirs = Load theirs
editor-kept-yours = Kept your edits
editor-loaded-theirs = Loaded the edits made outside Seura Photos
editor-resolve-failed = That didn't work: { $reason }
# Said when the user leaves the photo before choosing.
editor-left-outside = Your changes to this photo since it changed elsewhere weren't kept.
editor-history-discarded = This photo's edits were changed outside Seura Photos without their history, so the history before that change is gone.
editor-step-failed =
    { $undo ->
        [true] The change couldn't be undone: { $reason }
       *[false] The change couldn't be redone: { $reason }
    }

## Developer tools

dev-switches-on = Developer switches on: { $switches }
check-running = Checking that the preview matches export…
