# What the Info panel shows, measured against Explorer and Finder (M2 slice 8)

- **Nevada listed** everything Explorer's details pane and Finder's Get Info show for a photo, about twenty items between them, and asked which Seura should add.
- **Claude's recommendation, which Nevada accepted:** two additions.
  - An **Exposure** row: shutter speed and ISO, and the exposure compensation when there was some ("1/120 s · ISO 64", "2 s · ISO 800 · -0.7 EV"). With the aperture already in the Lens row, that's the three settings photographers look at first; Apple's Photos and the iPhone show the same group.
  - **Date modified, only when there's no date taken**, so scans and screenshots aren't left without any date.
- **Left out on purpose:** type (the file name says it), location on disk (the Viewer shows the folder), title and subject (rarely filled in, and a feature of their own if ever), maximum aperture, metering mode, flash, created and last-opened dates (file-system dates, not the photo's), colour space and profile (Seura's job is to show colour correctly without anyone thinking about profiles; it may come back in the Editor or export), alpha channel, and red-eye. The panel stays short; Explorer and Finder remain a right-click away.
- **The spec's list (BRW-6)** gains the two; written up for the design session.
