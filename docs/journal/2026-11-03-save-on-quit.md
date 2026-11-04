# Saving on quit (#174, M5 slice 0)

- **The fix:** closing the window now holds the close and asks the UI to save what's waiting, and the app exits when the UI answers, or after 3 seconds if it doesn't. A quit with nothing waiting took 26 ms on Windows.
- **On the Mac, Quit is now Seura's own menu item,** since the predefined one exits without asking anyone. This doesn't cover the Dock's Quit or logging out (#217).
- **Checked on Windows by a real close, with the same script on both builds:** a Brightness change followed at once by closing the window, on a scratch library. The fix saved it and exited in 65 ms. The installed 0.3.1 lost it, as the issue says.
- **The issue said a change waits about a second; it's 300 ms** (`SAVE_DELAY`).
- **A correction to the plan:** it said Ctrl+Q on Windows, which Seura doesn't have. Windows has no menu bar, so the check there is closing the window. Claude Code changed the plan's wording and didn't add a key.
- **A failed save at quit:** the message has no time to be read, because the app exits anyway. Claude Code left that as it is, and it's said in the pull request.
- **Checked on the Mac by the Mac session,** on a scratch library with a build it didn't install, with Accessibility turned on by Nevada for the test. A change made about 55 ms before quitting was saved 3 of 3 times from the Quit menu item (exit in 59 to 70 ms) and 3 of 3 from the red close button (54 to 66 ms). The standard quit Apple Event, which it sent to that process alone, as the Dock and logging out do, lost it 3 of 3 times, as Claude Code had expected (#217). The original photo's hash didn't change.
