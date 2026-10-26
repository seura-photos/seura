# Select All selected the window's text

- **Nevada found it:** Ctrl+A (Cmd+A on a Mac) selected the app's text, highlighting the window like a web page, rather than the photos. The grid handled Select All itself, but only while it had the keyboard focus. After a click on a folder, a tag, the Info panel, or empty space, the key went to the web view.
- **The fix, Claude Code's proposal:**
  - Select All selects the grid's photos wherever the focus is in the Library, except in a text field, where it selects the field's text as usual. In the Viewer it does nothing.
  - The app's interface text can't be selected at all, by dragging or by Select All, as in other desktop apps. Text fields can still be selected, and so can the Info panel's details, which are worth copying (a file name, a camera model), as Finder's Get Info allows.
- **Fixed in the same session, so no issue was opened.**
- **Checked by Nevada on both machines,** on the branch: Select All selects the photos with the focus anywhere in the Library, and the window's text stays unselected.
