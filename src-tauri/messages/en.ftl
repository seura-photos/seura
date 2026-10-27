# Text the core shows itself: the macOS menu (ACC-4, ACC-5). Project Fluent;
# see https://projectfluent.org. English only in v1.

app-name = Seura Photos

menu-about = About Seura Photos
menu-hide = Hide Seura Photos
menu-hide-others = Hide Others
menu-show-all = Show All
menu-quit = Quit Seura Photos
menu-file = File
menu-add-folder = Add Folder…
menu-show-in-finder = Show in Finder
menu-open-in-default-app = Open in Default App
menu-move-to-trash = Move to Trash…
menu-close-window = Close Window
menu-edit = Edit
menu-undo = Undo
menu-redo = Redo
# The Edit menu's Undo and Redo name the Library's last action and how many
# photos it touched (BRW-11).
menu-undo-action =
    { $kind ->
        [rating] Undo Rating
        [addTag] Undo Add Tag “{ $tag }”
       *[removeTag] Undo Remove Tag “{ $tag }”
    } ({ $count ->
        [one] 1 photo
       *[other] { $count } photos
    })
menu-redo-action =
    { $kind ->
        [rating] Redo Rating
        [addTag] Redo Add Tag “{ $tag }”
       *[removeTag] Redo Remove Tag “{ $tag }”
    } ({ $count ->
        [one] 1 photo
       *[other] { $count } photos
    })
menu-cut = Cut
menu-copy = Copy
menu-paste = Paste
menu-select-all = Select All
menu-window = Window
menu-minimize = Minimize
menu-fullscreen = Enter Full Screen
