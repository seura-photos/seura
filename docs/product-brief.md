# Seura Photos — Product Brief

Status: approved · Last updated: 2026-10-17

## Summary

Seura Photos is a free, open-source photo app for Windows and macOS that brings back what Picasa did well: browse the photos already on your drive, make simple edits that never touch the originals, and organize with tags.

Every edit and tag is stored in a visible file next to the photo, in a documented format, so your work stays readable and portable even without the app. Optional AI assistance, using the user's own API key, suggests edits the user can preview, adjust, or discard.

## The problem

Picasa combined four things: it worked on your existing folders, it was simple, it was free, and it ran locally. Google discontinued it in 2016, and no open-source app on Windows and macOS offers all four today.

- **digiKam** is local and cross-platform, but complex for casual users.
- **darktable** and **RawTherapee** are built for photographers editing RAW camera files.
- **Shotwell** is simple, but Linux-only.
- **Immich** and **PhotoPrism** require running a server.
- The simple, local tools that do exist are closed source and often paid.

A second, smaller problem: every non-destructive editor stores its edits in its own private format. Switch apps, and you keep the originals but lose your edits.

## Who it's for

People who used Picasa or something like it and want an app that just works: easy to use, and not trying to do everything. They have photos in folders on their computer, many of them copied from phones, and want to find, tidy, lightly fix, and share them. They are comfortable with a computer but are not photo-software experts.

It is not for professionals, RAW workflows, or people who want cloud sync.

**Design test:** a former Picasa user can install the app and fix a crooked, dark photo within five minutes, without reading documentation.

## Guiding principles

Every feature and design decision is checked against these.

1. **Originals are never modified, moved, or copied.** The app only reads them. Edits live in separate files; edited results are saved as new files on export. (The one exception is deleting a photo, which moves it to the system Trash only on explicit request; see BRW-9 in the functional spec.)
2. **Nothing hidden.** Everything the app writes is a visible file in a documented format, in a place the user can see. A built-in cleanup removes all of it on request.
3. **Simple over complete.** Every feature must be usable by the target user without documentation. When in doubt, leave it out.
4. **Local first.** The app works fully offline, with no account and no telemetry. It uses the network only for features the user turns on, such as update checks and AI.
5. **Portable.** The edit format is open and documented, so other apps can read it and the user's work outlives this app.
6. **Feels instant.** Opening a folder, scrolling thumbnails, and moving an edit slider must respond immediately.
7. **Opinionated.** The app defines how it's used, following the process most people go through with photos: find, sort, fix, organize, share. There is one way to do each thing, the most frequent actions are the quickest to reach, and the layout isn't customizable. Settings exist only for the user's system and data, and for accessibility.

## What v1 does

Version 1 covers browsing, light editing, tagging, and export. Details and acceptance criteria are in the [functional spec](functional-spec.md).

- **Library:** add folders and the app works on them in place. Browse a folder tree and a thumbnail grid, and open photos in a full-size viewer. New, changed, and deleted files are picked up automatically.
- **Formats:** JPEG, HEIC/HEIF (the iPhone default), and PNG.
- **Editing:** crop with aspect-ratio presets, straighten, rotate and flip, exposure, contrast, color temperature and tint, saturation, red-eye removal, and one-click enhance. Edits can be copied between photos or saved as named presets.
- **Effects:** black and white, sepia, and vignette.
- **Non-destructive by design:** undo and redo that survive restarts, revert to original at any time, and reopening an edited photo shows the previous settings so they can be adjusted.
- **Organizing:** tags with subtags, star ratings, and filtering by tag, rating, and folder.
- **Export:** save edited copies to a chosen folder, singly or as a selection, with size and quality options. Original metadata is kept, with an option to remove location data.
- **Cleanup:** remove the app's files from one folder or from everything, with a preview of what will be deleted.

## Tags and subtags

Tags work like the tags in the old Delicious bookmarking service, with subtags added: each tag is a single string, with slashes marking levels, parsed at runtime and shown as a tree.

- A photo tagged `family/kids` belongs only to that tag. In the tree it shows as `kids` nested under `family`, and it does not appear under the separate tags `family` and `kids`.
- A photo can have any number of tags. Tags are stored in the photo's edit file, so they travel with the photo.
- When adding a tag, the user sees the list of existing tags and can click one; typing filters the list down as they go. This avoids near-duplicates like `Family` and `family`.
- No renaming or merging: tags are only added to or removed from individual photos, so the app never has to update every photo that uses one. A tag exists only while at least one photo uses it.
- On export, tags can optionally be written into the exported copy's keyword metadata, so other photo apps can read them. Never into the original.

This fits photos well: a photo can be in one folder on disk but belong to many tags, which is what albums were in Picasa.

## Where data lives

**Edit files.** `IMG_1234.jpg` gets `IMG_1234.jpg.seura.json` beside it. Keeping the original extension in the name stops `IMG_1234.jpg` and `IMG_1234.heic` from colliding. An edit file is created only once a photo has edits, tags, or a rating. Edit files always live beside their photos, so photos in read-only folders can be viewed and exported but not edited or tagged. The format is specified in the [seura-photos/edit-format](https://github.com/seura-photos/edit-format) repository.

**Settings and presets.** A visible folder at `Documents/Seura Photos`, which the app always checks first, so nothing is written to hidden locations like AppData or Application Support.

**Cache.** Thumbnails and a search index, so large folders open instantly. Default location `Documents/Seura Photos/Cache`, movable by the user, documented as safe to delete because everything in it is rebuilt from the photos and edit files. Expected size is roughly 500 MB per 10,000 photos, with thumbnails sharp on high-density (2×) screens. Whether the index is plain files or SQLite is decided in the technical design by measuring against a large test library; either way it is a rebuildable cache, never the source of truth. Documents is often synced to OneDrive or iCloud, so the app detects that and suggests a non-synced cache location rather than uploading hundreds of megabytes of thumbnails.

**API keys** (when AI arrives). Stored in the operating system's credential store (Windows Credential Manager, macOS Keychain), never in a plain file, and removed on cleanup.

**Cleanup.** A built-in command removes edit files for a folder or everywhere, and can clear the cache and settings. It previews what will be deleted, warns that edits will be lost, and offers to export edited versions first.

## AI assistance (first update after v1)

AI is optional, off by default, and uses the user's own API key; there is no Seura backend. It ships in the first update after v1; v1's design leaves room for it.

- **Providers:** OpenRouter, Anthropic, OpenAI, and Azure OpenAI. Local models (Ollama or LM Studio) are worth adding later for privacy and zero cost.
- **Suggest edits:** the model looks at a downscaled copy of the photo and returns proposed edits in the edit format. The user previews before and after, then accepts, adjusts, or discards. Because a suggestion is just a set of edit operations, it is as non-destructive and adjustable as a manual edit.
- **Suggest tags:** proposes tags from the user's existing tag tree.
- **Privacy:** a photo is sent only when the user asks for a suggestion. The app says what will be sent and to which provider, and strips location and other metadata first.
- **Guided setup:** helping a non-technical user get an API key and pick a model, likely after the first AI release.

## Not in v1, and non-goals

**Never:** cloud sync or accounts; importing or copying photos into a managed library; competing with Lightroom or Photoshop (layers, masks, advanced retouching).

**Not in v1, candidates for later:**

- RAW camera files, video, and spot removal
- Face recognition and natural-language search
- Importing edits from Apple `.AAE` files and old Picasa `.picasa.ini` files
- Slideshows, printing, and sharing to online services
- A plugin system (the edit format already allows plugin operations)
- Linux

## The project beyond the app

Seura Photos is meant to be a complete public project, not only an application. It has three purposes, all of them intended:

1. **A practical tool.** Something the owner uses himself, and that others will use too.
2. **An experience of building an application from scratch,** done properly at every stage: specification, design, implementation, testing, release, and support. The project and its record also serve as a showcase of that work.
3. **A public home.** A website that explains what Seura Photos is, describes its features, tells people why they might want it, and lets them download it. That is marketing in the plain sense of the word, and it is part of the plan, not an afterthought.

The website will include, at least:

- What Seura Photos is and who it's for, with screenshots.
- Features, and the reasons behind them (originals never touched, open edit format, no account or subscription).
- Downloads for Windows and macOS, linked to GitHub Releases.
- A user guide, including help topics such as installing Microsoft's HEIF and HEVC extensions on Windows.
- A page about the edit format, linking to its specification.
- The project story: how it was designed and built, as the showcase.

Not built yet. The milestone plan schedules the website and the materials around the app alongside the app itself, so they're ready by the first public release.

## Distribution and license

Free and open source, with no subscriptions, accounts, or paid tiers.

- **License:** Apache 2.0 for the app. The edit format specification is published separately under CC BY 4.0, with its schema and tests under Apache 2.0, so anyone can implement it.
- **Library compatibility:** HEIC decoding libraries such as libheif are LGPL, which permits use from a permissively licensed app when linked dynamically. Each dependency's license is confirmed in the technical design.
- **Releases:** installers for Windows and macOS published as GitHub Releases in the seura-photos organization, with built-in, signature-verified update checks.
- **Code signing:** required so the target users don't see security warnings. Apple's developer program costs about $99 per year; Windows options vary, including signing programs for open-source projects. To be priced before the first public release.

## Decisions log

| Topic | Decision |
| --- | --- |
| Name | Seura Photos. Edit files use the `.seura.json` suffix. Domains seuraphotos.com and seuraphotos.app; GitHub organization `seura-photos`. USPTO search (2026-10-17): one result, SÉURA (reg. 3495772, live), in class 20 (furniture, mirrors, frames), not software; judged low risk. "Seurat" in classes 9 and 42: live registrations belong to one company making industrial 3D printers and their software; judged low risk. "Sura": five live marks containing the word (university research, massage devices, dog training, an online drama series, a medical practice), none for software or photography; judged no conflict. WIPO Global Brand Database: the same US SÉURA mark, plus one expired Japanese registration; no conflict. Trademark clearance complete for a free open-source release; revisit with an attorney if the project becomes commercial. |
| Settings location | `Documents/Seura Photos`, with no hidden files. |
| API keys | Operating system credential store. |
| Retouching | Red-eye removal in v1; spot removal later. |
| Star ratings | In v1. |
| AI timing | First update after v1, once editing is solid. |
| Copy and paste edits, presets | Both in v1. |
| Read-only folders | View and export only. Edit files always live beside the originals, never elsewhere. |
| License | Apache 2.0. |
| Minimum OS versions | Windows 10 22H2+ and macOS 13+, with tiered test coverage; see the technical design, section 12. |
| Scope of the project | A complete public project: the app, a website that explains and distributes it, and the project's own record as a showcase. |
| GitHub organizations | One organization per app: `seura-photos` holds this app, its edit format, and its website. If more Seura apps follow, each gets its own organization, and anything they share goes in a separate shared organization (for example `seura` or `seura-shared`). Reserve the plain `seura` name if it's available. |
