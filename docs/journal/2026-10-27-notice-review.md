# Reviewing the notices from screenshots (design session)

- **Claude Code sent screenshots** of every notice and the message, in light and dark, from the installed 0.2.1, with three questions.
- **Caution looked like information.** FIL-7 ("changed by another program") had the same blue bar as FIL-5, with only its icon different. Claude recommended a third level, amber, as Windows' own info bars have: information blue, caution amber, problem red. New tokens `caution` and `cautionSubtle`.
- **Found in the screenshots:** in dark mode, the info and caution icons on the blue bar used `accent`, only 2.3:1, the same mistake as the selected row's bar. Icons on accentSubtle now use accentText.
- **Messages after a key.** Nevada expected a message after pressing 3, and INT-6 had said no, since the badge changes. Claude's recommendation: a change made with a key gets a message, since nothing changes where the user is looking and the photos may be off screen; a click on a star doesn't.
- **Also settled:** a selection's date range written with the region's range format, collapsing shared parts (#125); the filter row fits one line at the default size with the Tags control capped at 200 px, wrapping only when it must (#118).
