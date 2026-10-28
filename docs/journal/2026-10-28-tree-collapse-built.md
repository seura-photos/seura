# Collapse all and expanding beneath, built (M4 slice 0c; #124)

- **As the design session specified** (LIB-2, TAG-5; wireframes, "Shared structure"):
  - each tree heading has a Collapse all button (chevrons-down-up, quiet), left of FOLDERS' +;
  - Collapse all closes everything except the way to the selected folder or tag, and is disabled when nothing else is open;
  - Alt-click (Option-click) on an arrow, and "Expand all beneath" in the right-click menu, open a node and everything beneath it;
  - `*` opens the focused node and its siblings.
- **The folder menu now opens for any folder with folders in it,** not only library folders. Expand all beneath shows for those, and Remove from library… still only for library folders. The tag tree gained a menu of its own, with Expand all beneath, opened by right-click or the context-menu key.
- **Caught by the new test:** "everything beneath" at first also marked folders with nothing in them as open. Nothing showed, but Collapse all then counted them as open and stayed enabled. Only nodes with something beneath them are opened now.
- **A disabled quiet button** borrows the disabled menu item's look: secondary text, and no hover.
- **Nevada checked it by eye** on the branch, with a generated sample library of nested folders and tags (its own `SEURA_DATA`): Alt-click, the menu's Expand all beneath, `*`, and Collapse all keeping the way to the selection open, in both trees. "All working perfectly."
