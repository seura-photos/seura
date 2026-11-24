# M7 planning questions (design session)

- **Claude Code drafted the M7 plan** (#265) with design questions for release readiness. Claude answered, mostly as Claude Code suggested:
  - **Settings** on one page that applies at once; **Clean up** with scope, what to remove, a live preview, and a full reset that confirms again and returns to the Welcome screen. Both are now in the wireframes.
  - **On the Mac,** Clean up's full reset stands in for an uninstaller.
  - **Updates:** checked at launch at most once a day, offered with a quiet notice. Claude added a pre-release channel: pre-release builds read a second manifest, so beta testers are offered 0.6.1 and later 1.0, while release builds only ever see releases.
  - **One rule change, Claude's catch:** CLN-4 sent the cache to the Recycle Bin, which for gigabytes of thumbnails fills it with nothing anyone will restore, and on Windows past its size limit asks to delete permanently anyway. Clearing the cache now deletes it directly (decision 0018, CLAUDE.md rule 5).
  - **The user guide** is a website page written in a design session from Claude Code's fact sheet.
- Also caught up: decision 0015 lists `marks.json`; BRW-11 and REV-6 say undo can put back a review's marks; decision 0013 lists the GPU shader caches (#231) and the update installer.
