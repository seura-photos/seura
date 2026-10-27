# Drives with no Recycle Bin: the 0.2.0 bug, and what Seura may delete (design session)

- **Nevada's USB-stick test found the worst bug so far:** on Windows, 0.2.0's Delete removed a photo on a USB stick permanently, with no prompt (#115). The design session had written into BRW-9 that such drives would be left out "once measured", and slice 9's journal had assumed Windows would warn; neither was measured until Nevada plugged in a stick. Claude Code found the cause (`FOF_NO_UI` in the `trash` crate suppresses the warning) and fixed it in #113 by refusing such paths before calling the system.
- **Lesson:** an assumption about a destructive path is tested before it ships, not after. Rule 5 now says tests enforce it.
- **Nevada proposed** deleting an emptied edit file directly on such drives, since nothing in it can be lost. Claude agreed, and added the same for Start over's original, only after a byte-for-byte check of the copy (decision 0017).
- **Claude recommended against** a "Delete anyway" for photos on memory cards, and against importing, since both cross principle 1. Nevada can overrule after trying 0.2.1.
- **0.2.0 is public** on GitHub, so its release notes get a warning, and the website's early-build section waits for 0.2.1.
