# Keys in dialogs stay in them (Claude Code)

- **Found while building About** (#120): the Delete, Open, and Remove folder confirmations stopped only Esc from reaching the views behind them. The Library and the Viewer listen for keys on the window, so pressing a number while confirming a delete would rate the selected photos behind the dialog. T opened the tag picker, and Ctrl+Z undid the last action. Tab also took the focus out of the dialog, which says it's modal.
- **Found by reading the code, not seen in the app.** The new tests confirm it: all three failed before the fix.
- **Fixed with one shared handler** (`ui/src/dialogKeys.ts`), which About uses too. No key leaves a dialog, Esc closes it, and Tab and Shift+Tab go round its buttons. Nevada agreed to do it as the next slice.
- **Nevada checked it by hand** from the branch: pressing 3 while Delete was being confirmed left the photo's rating as it was, and Tab stayed among the dialog's buttons.
