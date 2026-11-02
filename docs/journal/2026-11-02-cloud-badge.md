# Cloud-only photos in the grid, and printing (design session)

- **Nevada asked** for a cloud icon on cloud-only photos, and a download prompt when a grid action, such as the hover Enhance button, needs a photo's pixels.
- **The icon:** only a tile with no thumbnail said "In the cloud"; with a system thumbnail, a cloud-only photo looked like any other, though opening, Enhance and Delete treat it differently. Claude agreed: a small cloud badge, top right, only on photos still in the cloud.
- **The download:** LIB-9 already allowed downloading a photo the user edits, so Enhance was stricter than the spec. Claude's recommendation, which Nevada accepted: one photo downloads without a prompt, as opening it does; a selection asks once with the count and size, since an unasked bulk download is what LIB-9 rules out (Claude Code's point in slice 9).
- **Printing** came up too. The product brief lists it under "later"; Claude suggested keeping it out of v1 and, after 1.0, handing the edited photo to the system's own photo printing rather than building layouts. Not written into the brief yet.
