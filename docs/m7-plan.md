# M7 work plan: Release readiness

Status: draft · Last updated: 2026-11-12

The working plan for milestone M7 from the [milestone plan](milestone-plan.md): everything between "works for me" and "ready for strangers". It breaks the milestone into slices, one pull request each (or a few, lettered), in order. Drafted by Claude Code from the milestone plan, the [functional spec](functional-spec.md), the [user flows](ux/user-flows.md) and [wireframes](ux/wireframes.md), the [technical design](technical-design.md), the decision records, and the open issues under the "M7" milestone. The questions it couldn't settle are at the end, each with a suggestion.

**Scope** (milestone plan, M7):
- Cleanup (CLN-1 to CLN-5), Settings (SET-2), and Updates (SET-4) with signed update packages.
- Audits against the spec: accessibility (ACC-1 to ACC-7), interaction rules (INT-1 to INT-6), display (DSP-1 to DSP-4), and every performance target.
- Signed installers for Windows and macOS, and third-party license notices (SET-5).
- The user guide.
- The edit format released as 1.0.
- A short beta with volunteers, following a written checklist, as build 0.6.

**Done when** (milestone plan): the beta finds no data-safety problems and no blocking bugs, and the checklist passes on every supported OS.

**Also carried in:**
- Node 26 (#156), allowed since 2026-11-11.
- Google Drive and Dropbox on both platforms (#162).
- Decoding colors in full, the rest of #188.
- Next-photo speed on Windows (#190), for the performance audit.
- Deleting OneDrive photos on the Mac, retested with Developer ID signing (#112).
- Finder's Put Back for edit files (#254).
- The Windows install location (#70, a design question), which the signed installer needs settled.

**Not in M7:** HDR display (#60); AI assistance and spot removal, which come after 1.0; the website's full site, which the design sessions build during M7 (Claude Code supplies screenshots on request).

## How to work

As in M6 ([M6 work plan](m6-plan.md), "How to work"):
- One slice per branch and pull request, branched from `main` and named `m7/<slice>`. Every pull request's base is `main`, and Nevada merges.
- Dependencies at least two weeks old, checked against decision 0012.
- Each slice has an issue under the "M7" milestone, and its pull request closes it.
- Screens are styled from the tokens and checked against the mockups, with screenshots in light and dark mode in the pull request.
- New text goes in the message catalogs (ACC-5).
- Journal everything.

New in M7:

- **Secrets stay with Nevada.** Signing certificates, the Apple account, and the updater's private key are created by Nevada and stored only as GitHub Actions secrets. Claude Code writes the workflow steps that use them, and never sees or handles their values. Turning signing on is an outward step that needs Nevada's go-ahead, like publishing.
- **New ways to remove files get the strictest tests.** Clean up is the first feature since Delete that removes the user's data. Its tests enforce rule 5: everything goes to the Trash or Recycle Bin, nothing is deleted directly, and an original photo is never touched (CLAUDE.md, rules 1 and 5). Where a drive has no Trash, Clean up leaves the file as it is, as Delete does (decision 0017).
- **The updater writes somewhere new.** An update downloads an installer before running it. Where that file goes is checked with `scripts/app_data_audit.py` and recorded in decision 0013 if it's outside the places rule 2 allows (question 7).
- **Audits produce issues, not essays.** Each audit is a checklist run on both machines. Every failure becomes a fix in the same slice if it's small, or an issue under "M7" if it isn't.
- **Strangers are the new test.** The beta is the first time someone other than Nevada runs Seura on purpose. Text a tester will read, from the installer to the release notes, is written for someone who has never heard of Seura.

## Slices

The order puts what depends on Nevada's purchases and on design answers early, so they can run in the background, and the beta last.

### 0. Carried in (#156)

- **Node 26** (#156): `.nvmrc`, CI and the docs, to the newest Node 26 LTS release that's at least two weeks old.

### 1. Signed installers (#70, #112)

- **Windows:** the NSIS installer and the app signed in the release workflow, with the option Nevada chooses (question 1). The install location settled by #70 (question 2).
- **macOS:** the app signed with a Developer ID certificate, hardened runtime, and notarized. The disk image is signed and stapled, so it opens without Privacy & Security's Open Anyway.
- **#112 retested** with the Developer ID build, as decided on 2026-10-29: deleting a downloaded OneDrive photo on the Mac. If it still fails, the current refusal stays, and the user guide says so.
- **The release notes' install steps** lose the SmartScreen and Open Anyway instructions once they're no longer needed.
- **Done when:** a build from the release workflow installs on both machines without a warning, and #112 has a recorded result.

### 2. Updates (SET-4)

- **Checking:** with update checks on (the Welcome screen's checkbox, already saved as `checkForUpdates`), Seura checks at launch, at most once a day (question 6). The check is a single request to a fixed address. It sends nothing about the user or their library (rule 6).
- **Offering:** a new version shows as a message with its version and a link to its notes, and Install and Not now. Nothing installs without the user's say. Install downloads, verifies the update's signature with the public key built into the app, and restarts into the new version after saving any waiting edit (EDT-11).
- **Publishing:** the release workflow builds signed update packages with the Tauri updater key (technical design, section 10). The update manifest is published only when Nevada publishes a release, so a draft never reaches users.
- **The Content Security Policy** stays closed to everything but the update address, and the request is made from Rust.
- **Done when:** on both machines, a build of 0.6.0 offers a test 0.6.1, installs it on request and restarts into it with edits intact; a package with a bad signature is refused; and with checks off, no request is made.

### 3. Settings (SET-2)

- **Where:** Settings… in the app menu on macOS, as a window, and in ⋯ on Windows, as a dialog (user flows, ACC-4). Its layout comes from a design session (question 3).
- **What, and nothing else:**
  - Library folders: the list, with Add and Remove, using the existing commands.
  - The cache: its location and size, with Move… and Clear. Moving copies the cache, then removes the old one; clearing removes it. The settings file keeps the library, so nothing is forgotten (SET-3).
  - Seura's own folder: where it is, with Open folder.
  - Default export options: the ones the Export dialog already remembers.
  - Check for updates automatically.
- **The cache is Seura's to delete,** so clearing or moving it may remove files directly: they're thumbnails Seura made, rebuilt on demand, and nothing in them is the user's (decision 0017's test). The tests check that only the cache folder's own contents are removed, never the folder the user chose to hold it.
- **Done when:** on both machines, Nevada changes each setting, and it's in effect at once and after a restart; moving the cache to another drive and clearing it both leave a working Library.

### 4. Clean up (CLN-1 to CLN-4)

- **Where:** File → Clean up… on macOS, ⋯ → Clean up… on Windows, and Clean up this folder… in a folder's right-click menu (user flows, flow 10). The dialog's layout comes from a design session (question 4).
- **Scope** (CLN-1): one folder, with or without its subfolders; the whole library; or only orphaned edit files, those whose photo no longer exists.
- **What** (CLN-2): edit files; the cache; and, for a full reset, settings and presets too.
- **Preview** (CLN-3): before anything moves, the counts of what will go, and how many photos will lose their edits, with Export edited photos first, which opens the Export dialog on exactly those photos.
- **Recoverable** (CLN-4): files go to the Trash or Recycle Bin, never deleted directly, except the cache (slice 3). On a drive without a Trash, edit files are left as they are, and the result says how many. Unknown files beside photos are never touched: only `*.seura.json` files whose name matches a photo, or names no photo for orphans.
- **Tests:** a data-safety test for each scope, enforcing that photos are never touched, that everything removed goes through the Trash, and that a full reset doesn't remove anything outside Seura's folder and the library's edit files.
- **Done when:** on both machines, on scratch libraries, Nevada cleans up a folder, orphans and the whole library, finds everything in the Trash or Recycle Bin, puts some back, and sees their edits return.

### 5. Uninstalling (CLN-5, #231)

- **Windows:** the uninstaller offers what Clean up offers for the whole library, defaulting to leaving edit files in place (CLN-5). It also removes what decision 0013 says it should: the web view's data, which is in Seura's cache folder.
- **macOS:** apps dragged to Applications have no uninstaller (question 5).
- **Decision 0013** updated with what the M5 wrap-up found, the GPU shader caches (#231), once a design session has written it.
- **Done when:** on Windows, uninstalling with each choice leaves exactly what it says, checked with `scripts/app_data_audit.py`; and on the Mac, whatever question 5 settles is in the user guide.

### 6. License notices (SET-5)

- **Generated, not written by hand,** at build time, from Cargo's and npm's metadata, into one file bundled with the app. A check in CI fails if a dependency has no license text.
- **Shown** from About: a Third-party notices link on Windows, and in the About panel's credits on macOS (question 9).
- **Done when:** About opens the notices on both machines, and every dependency in `cargo deny list` and `npm run licenses` is in them.

### 7. Cloud services (#162, #254)

- **Google Drive on Windows** is treated as cloud-only throughout, as the technical design says, since Drive gives no per-file signal: browsing never downloads its photos, and its thumbnails come from the system where it has them. Its location comes from Drive for desktop's settings.
- **Google Drive and Dropbox on the Mac,** and Dropbox on Windows (which already behaves like OneDrive), tested against #162's list: scanning, opening, detection, Delete, the deletion notice, and offline. Each finding is fixed or written into the user guide's section on cloud folders.
- **#254, Finder's Put Back:** the user guide explains how to put back an edit file with its photo (question 11).
- **Done when:** each service on each platform has a recorded result in #162, and the behaviour the user guide will describe matches the app.

### 8. Fixes before the beta (#217, #188, #255, #258, #264)

Lettered pull requests, one per issue:
- **8a. #217:** on the Mac, quitting from the Dock or by logging out waits for a waiting save (EDT-11).
- **8b. #264:** an edit file without width and height gets them once Seura knows them, even if it has a checksum. Recording the size the Viewer and Review learn when they decode a photo too.
- **8c. #255:** a library folder saved through a symbolic link sees live changes, by watching the resolved path.
- **8d. #258:** if it shows up again in the performance audit, find why the Viewer once never drew a photo. If not, close it as not reproduced.
- **8e. #188:** decoding colors in full, as edit-format #16 recommends: unclipped wide-gamut colors and fuller PNG color descriptions. It's a SHOULD, so it can move after 1.0 if the audits take longer (question 12).
- **Done when:** each fix has its test, and Nevada has checked the ones with a hands-on part on the machine it's about.

### 9. Audits: accessibility, interaction and display (ACC, INT, DSP)

- **A checklist per view,** run on both machines, written into the pull request with each result:
  - **ACC-1 Keyboard:** every flow with the keyboard alone.
  - **ACC-2 Screen readers:** Narrator and VoiceOver: every control has a label, and thumbnails are announced by name, date and rating.
  - **ACC-3 Display:** WCAG AA contrast in both modes, the system text size at its largest, and reduced motion.
  - **ACC-4 Native pieces** and **ACC-5 Language:** no text outside the catalogs.
  - **ACC-6 Regional formats:** a German region on each machine.
  - **ACC-7 Room for translation:** `SEURA_LOCALE=pseudo` at the minimum window size.
  - **INT-1 to INT-6:** each action's tier and cost; one layout; process order; disclosure; pointer travel through Review, Edit and Export; messages and notices kept apart.
  - **DSP-1 to DSP-4:** crisp at 100%, 150% and 200% scaling; every view at 1280×720; the extra room at 2560×1440 going to photos; thumbnails sharp at 2× at the largest size.
- **Done when:** every item passes on both machines, or has an issue Nevada agreed can wait until after 1.0.

### 10. Performance audit (PRF-1 to PRF-7, #190)

- **Every target** measured with `scripts/measure.py` on both machines, on a release build with signing, updates and Settings in.
- **#190:** next photo on Windows back under the 75 ms aim, or the reason it can't be, recorded.
- **The app-data audit** over the same build, with an update installed and a clean-up run.
- **Done when:** every target is met on both machines, and the audit passes.

### 11. The edit format 1.0 (edit-format repository)

- **Released:** the spec's status from draft to released, a changelog, and the tag `v1.0.0` in `seura-photos/edit-format`. From then on, changes follow its versioning rules.
- **The schemas frozen:** `1.0.json` and `1.x-reader.json` on the website match the release, and never change again (website CLAUDE.md, "Site rules").
- **Seura's submodule** points at the tag.
- **Done when:** the tag is published with Nevada's go-ahead, and `check.py` from the tag passes every edit file in Nevada's library, read only.

### 12. The user guide

- **Its content** is listed in the milestone plan: installing Microsoft's HEIF and HEVC extensions on Windows; cloud-synced folders in a section of their own (what downloads a photo and what doesn't, freeing the space again, what doesn't work yet, and what each service offers); PNG color descriptions; export never replacing a file; and Explorer's missing location for PNGs. Slices 1, 5 and 7 add their findings.
- **Who writes it, and where it lives,** is question 8.
- **Done when:** the guide is published on the website with the beta, and Claude Code has checked every statement in it against the app.

### 13. The beta: build 0.6

- **The checklist,** drafted by Claude Code from the functional spec and the user flows, for Nevada to review: install, first run, browse, view, edit, tag, review, export, clean up, update, uninstall. Each step says what to do and what to expect, for someone who has never used Seura.
- **The build:** 0.6.0, signed, published as a pre-release with the steps above, and the website pointing testers at it.
- **Testers:** volunteers Nevada finds, especially on Windows 10 and macOS 13 to 15 (question 10). They report through GitHub issues, with a template that asks for the version (copied from About), the OS and the steps.
- **During the beta:** blocking bugs and data-safety problems are fixed in 0.6.x builds, delivered through the updater. Everything else is triaged with Nevada into before or after 1.0.
- **The macOS gate:** if nobody tests macOS 13 to 15, the minimum is raised to Tahoe 26, in a new decision record (milestone plan, technical design section 12).
- **Done when:** the milestone's "done when".

### 14. Wrap-up

- **Confirming every slice's "done when",** the final measurements, the devlog post, and the Status section. Each outward step needs Nevada's approval.

## Questions

Questions 1, 2 and 10 are Nevada's; the others are for a design session. Each has Claude Code's suggestion.

1. **Code signing (Nevada).** The product brief says signing is needed "so the target users don't see security warnings". It still needs pricing and a choice.
   - On the Mac: Apple's Developer Program, about $99 a year, enrolled as an individual (the certificate then shows Nevada's name) or as an organization (which needs a legal entity).
   - On Windows, the options include Microsoft's cloud signing service (a monthly fee, with identity checks), a signing program for open-source projects (free, with conditions), or a certificate from a certificate authority.
   - *Suggestion:* Apple's program as an individual; for Windows, Nevada prices the cloud service and the open-source program, and Claude Code checks each against the release workflow before choosing.
2. **The Windows install location (#70, Nevada or a design session).** Per user only, as now, in `%LOCALAPPDATA%\Programs`; Program Files for everyone; or a choice in the installer. *Suggestion:* per user only, as now. It needs no administrator rights, keeps updates silent, and every file Seura installs stays in the user's own profile. The user guide says where it is.
3. **The Settings screen's layout.** The wireframes don't have one yet. *Suggestion:* one page, no tabs, in SET-2's order: Library folders, Cache, Seura's folder, Export defaults, Updates. On macOS, a window that changes things at once, as Mac settings do; on Windows, the same in a dialog with a Close button.
4. **The Clean up dialog's layout.** The wireframes don't have one, and the user flow is three lines. *Suggestion:* scope at the top (this folder, with subfolders, the whole library, or orphaned edit files only), then what to remove as checkboxes, then the preview's counts, which update as choices change, then Export edited photos first, Cancel and Move to Trash. A full reset needs its own extra confirmation.
5. **CLN-5 on the Mac.** Mac apps have no uninstaller: users drag them to the Trash, and Seura can't run anything when that happens. *Suggestion:* Clean up's full reset is the Mac's way to remove Seura's data. The user guide says to run it before dragging the app to the Trash, and lists what's left otherwise. The spec gets a sentence saying so.
6. **How often to check for updates.** *Suggestion:* at launch, at most once a day, and never during a review or an export. Settings shows when it last checked, with a Check now button.
7. **Updates and pre-releases, and where the downloaded installer goes.**
   - GitHub's "latest release" address skips pre-releases, so beta testers on 0.6 wouldn't be offered 0.6.1.
   - *Suggestion:* the manifest lives at a fixed address on the website, `seuraphotos.app/updates/latest.json`, updated only when Nevada publishes a release, so pre-releases can be offered during the beta.
   - The updater downloads the installer to the system's temporary folder before running it. *Suggestion:* allowed, deleted after installing, and recorded in decision 0013.
8. **The user guide: who writes it, and where.** *Suggestion:* it's a page on the website, written in a design session, since the design sessions own the site and the documents. Claude Code supplies a fact sheet of how the app behaves, drawn from the journal and the code, and then checks the draft against the app.
9. **Where the license notices are shown.** *Suggestion:* a Third-party notices link in About on Windows, and the About panel's credits on macOS, both opening the generated file.
10. **The beta (Nevada).** How many testers, found where, and for how long. *Suggestion:* five to ten people, with at least one each on Windows 10, Windows 11, macOS 13 to 15 and macOS 26; two weeks; reports through GitHub issues. The beta ends early only for a data-safety problem.
11. **Finder's Put Back for edit files (#254).** The cause is unknown, and scripting Finder was set aside. *Suggestion:* a known issue for 1.0, explained in the user guide: put back the `.seura.json` file beside the photo too.
12. **Decoding colors in full (#188) before or after 1.0.** The edit format calls it a SHOULD, and photos look right today for sRGB and Display P3. *Suggestion:* in M7, as slice 8e, but it may move after 1.0 if the audits run long, with Nevada's agreement then.
