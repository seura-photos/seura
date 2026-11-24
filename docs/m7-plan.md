# M7 work plan: Release readiness

Status: draft · Last updated: 2026-11-24

The working plan for milestone M7 from the [milestone plan](milestone-plan.md): everything between "works for me" and "ready for strangers". It breaks the milestone into slices, one pull request each (or a few, lettered), in order. Drafted by Claude Code from the milestone plan, the [functional spec](functional-spec.md), the [user flows](ux/user-flows.md) and [wireframes](ux/wireframes.md), the [technical design](technical-design.md), the decision records, and the open issues under the "M7" milestone. The questions it couldn't settle are at the end. The design session answered the design questions (#270, with the website's update manifests in website #15); questions 1, 2 and 10 are Nevada's and still open.

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
- **New ways to remove files get the strictest tests.** Clean up is the first feature since Delete that removes the user's data. Its tests enforce rule 5: edit files, settings and presets go to the Trash or Recycle Bin, and an original photo is never touched (CLAUDE.md, rules 1 and 5). The one direct deletion is Seura's own cache, when the user clears it (decision 0018), and the tests allow it inside the cache folder only. Where a drive has no Trash, Clean up leaves an edit file as it is, as Delete does (decision 0017).
- **The updater writes somewhere new.** An update downloads its installer to the system's temporary folder and deletes it after installing (decision 0013, SET-4). `scripts/app_data_audit.py` checks that nothing else is left.
- **Audits produce issues, not essays.** Each audit is a checklist run on both machines. Every failure becomes a fix in the same slice if it's small, or an issue under "M7" if it isn't.
- **Strangers are the new test.** The beta is the first time someone other than Nevada runs Seura on purpose. Text a tester will read, from the installer to the release notes, is written for someone who has never heard of Seura.

## Slices

The order puts what depends on Nevada's purchases and on design answers early, so they can run in the background, and the beta last.

### 0. Carried in (#156)

- **Node 26** (#156): `.nvmrc`, CI and the docs, to the newest Node 26 LTS release that's at least two weeks old.

### 1. Signed installers (#70, #112)

- **Windows:** the NSIS installer and the app signed in the release workflow through SignPath Foundation's program for open-source projects (question 1). Nevada applies to SignPath. Claude Code checks its conditions first, then writes the workflow steps: builds fully automated from CI, two-factor authentication on GitHub and SignPath, no closed-source parts, a code signing policy page, and SignPath's attribution line in the releases. The install location is settled by #70 (question 2).
- **macOS:** the app signed with a Developer ID certificate, hardened runtime, and notarized. The disk image is signed and stapled, so it opens without Privacy & Security's Open Anyway.
- **#112 retested** with the Developer ID build, as decided on 2026-10-29: deleting a downloaded OneDrive photo on the Mac. If it still fails, the current refusal stays, and the user guide says so.
- **The release notes' install steps** lose the SmartScreen and Open Anyway instructions once they're no longer needed.
- **Done when:** a build from the release workflow installs on both machines without a warning, and #112 has a recorded result.

### 2. Updates (SET-4)

- **Checking:** with update checks on (the Welcome screen's checkbox, already saved as `checkForUpdates`), Seura checks at launch, at most once a day, and when the user presses Check now in Settings (question 6). The check is a single request to a fixed address on the website. It sends nothing about the user or the computer, and no identifier (rule 6).
- **Two channels** (question 7): a release build reads `seuraphotos.app/updates/latest.json`, the newest release; a pre-release build (below 1.0, or with a pre-release suffix) reads `seuraphotos.app/updates/beta.json`, the newest release or pre-release. So 0.6 testers are offered 0.6.1 and then 1.0, and from 1.0 an app sees only releases.
- **Offering:** a new version is a quiet notice, never a dialog, with its version, a link to its notes, and Install and Not now. It waits while a review or an export is under way. Nothing installs without the user's say. Install downloads to the system's temporary folder, verifies the update's signature with the public key built into the app, and restarts into the new version after saving any waiting edit (EDT-11). The downloaded installer is deleted after installing, and one left by an interrupted update is deleted at the next launch (decision 0013), once rule 5 lists it (see the note after the questions).
- **Publishing:** the release workflow builds signed update packages with the Tauri updater key (technical design, section 10). The manifests change only when Nevada publishes a release, as a website pull request he merges (website CLAUDE.md), so a draft never reaches users.
- **The Content Security Policy** stays closed to everything but the update address, and the request is made from Rust.
- **Done when:** on both machines, a build of 0.6.0 offers a test 0.6.1 from `beta.json` and a release build doesn't; it installs on request and restarts into it with edits intact; a package with a bad signature is refused; with checks off, no request is made; and no installer is left in the temporary folder.

### 3. Settings (SET-2)

- **Where:** on macOS, a window from the app menu (Cmd+,), with no buttons; on Windows, a dialog with Close, from ⋯ → Settings… (Ctrl+,). One page, no tabs, and changes apply at once (SET-2, wireframes §9).
- **What, and nothing else,** in this order:
  - Library folders: the list with each folder's photo count, Add folder… and Remove, using the existing commands. Removing confirms that the folder's photos and edit files aren't touched.
  - The cache: its location and size, with Move… (with progress) and Clear (confirming that it's rebuilt automatically). The settings file keeps the library, so nothing is forgotten (SET-3).
  - Seura's own folder: where it is, with Open folder.
  - Export defaults: the same controls as the Export dialog, which already remembers them.
  - Updates: Check for updates automatically, "Last checked …", and Check now.
- **Clearing the cache deletes it directly** (decision 0018). Moving it copies it, then deletes the old copy directly, which Claude Code reads as the same decision. The tests check that only the cache folder's own contents are removed, never the folder the user chose to hold it.
- **Done when:** on both machines, Nevada changes each setting, and it's in effect at once and after a restart; moving the cache to another drive and clearing it both leave a working Library.

### 4. Clean up (CLN-1 to CLN-4)

- **Where:** a dialog from File → Clean up… on macOS, ⋯ → Clean up… on Windows, and a folder's right-click menu, which opens it with that folder chosen (user flows, flow 10; wireframes §10).
- **Scope** (CLN-1): This folder, with Include subfolders; The whole library; or Only edit files whose photos are gone.
- **What** (CLN-2): checkboxes, none checked at first: Edit files; The cache; Everything, a full reset that adds settings and presets. Choices that don't apply to the scope are disabled.
- **Preview** (CLN-3): updating as the choices change. It leads with how many photos will lose their edits, tags or ratings, then how many edit files and how big the cache. Export edited photos first… opens the Export dialog on exactly those photos.
- **Buttons:** Cancel, which has the focus, and Move to Recycle Bin (Move to Trash on the Mac), in the danger style and disabled until something is chosen.
- **A full reset** asks once more: "Seura will forget your library and settings and start again with the Welcome screen. Your photos aren't touched." Then it moves settings, presets and the chosen edit files to the Trash, deletes the cache, and restarts at Welcome.
- **Recoverable** (CLN-4): edit files, settings and presets go to the Trash or Recycle Bin. The cache is deleted directly (decision 0018). On a drive without a Trash, edit files are left as they are, and the result says how many. Unknown files beside photos are never touched: only `*.seura.json` files whose name matches a photo, or names no photo for orphans.
- **Tests:** a data-safety test for each scope, enforcing that photos are never touched, that everything removed except the cache goes through the Trash, and that a full reset doesn't remove anything outside Seura's folder, the cache and the library's edit files.
- **Done when:** on both machines, on scratch libraries, Nevada cleans up a folder, orphans and the whole library, finds everything in the Trash or Recycle Bin, puts some back, and sees their edits return.

### 5. Uninstalling (CLN-5, #231)

- **Windows:** the uninstaller offers what Clean up offers for the whole library, defaulting to leaving edit files in place (CLN-5). It also removes what decision 0013 says it should: the web view's data, which is in Seura's cache folder, and Seura's own `D3DSCache` folder if it can be identified (#231).
- **Identifying the `D3DSCache` folder,** which the design session asked Claude Code to look into: Direct3D names it after a hash, so the slice finds out whether the hash can be worked out from the installed program, or recognized from its contents, reliably enough to delete only Seura's. If not, it stays, and decision 0013 says so.
- **macOS:** apps dragged to the Trash have no uninstaller, so Clean up's full reset is the way to remove Seura's data (CLN-5). The user guide says to run it first, and lists what remains otherwise.
- **Done when:** on Windows, uninstalling with each choice leaves exactly what it says, checked with `scripts/app_data_audit.py`, and the `D3DSCache` question has a recorded answer.

### 6. License notices (SET-5)

- **Generated, not written by hand,** at build time, from Cargo's and npm's metadata, into one file bundled with the app. A check in CI fails if a dependency has no license text.
- **Shown** from About: a Third-party notices link in the Windows dialog, and the About panel's credits on macOS, both opening the generated list (SET-5).
- **Done when:** About opens the notices on both machines, and every dependency in `cargo deny list` and `npm run licenses` is in them.

### 7. Cloud services (#162, #254)

- **Google Drive on Windows** is treated as cloud-only throughout, as the technical design says, since Drive gives no per-file signal: browsing never downloads its photos, and its thumbnails come from the system where it has them. Its location comes from Drive for desktop's settings.
- **Google Drive and Dropbox on the Mac,** and Dropbox on Windows (which already behaves like OneDrive), tested against #162's list: scanning, opening, detection, Delete, the deletion notice, and offline. Each finding is fixed or written into the user guide's section on cloud folders.
- **#254, Finder's Put Back:** a known issue for 1.0. The user guide explains how to put back an edit file with its photo (question 11).
- **Done when:** each service on each platform has a recorded result in #162, and the behaviour the user guide will describe matches the app.

### 8. Fixes before the beta (#217, #188, #255, #258, #264)

Lettered pull requests, one per issue:
- **8a. #217:** on the Mac, quitting from the Dock or by logging out waits for a waiting save (EDT-11).
- **8b. #264:** an edit file without width and height gets them once Seura knows them, even if it has a checksum. Recording the size the Viewer and Review learn when they decode a photo too.
- **8c. #255:** a library folder saved through a symbolic link sees live changes, by watching the resolved path.
- **8d. #258:** if it shows up again in the performance audit, find why the Viewer once never drew a photo. If not, close it as not reproduced.
- **8e. #188:** decoding colors in full, as edit-format #16 recommends: unclipped wide-gamut colors and fuller PNG color descriptions. It's a SHOULD, so it can move after 1.0 if the audits take longer, but only with Nevada's agreement (question 12).
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
- **The fact sheet:** Claude Code writes down how the app behaves, from the journal and the code, including the known issues, for the design session to write from (question 8).
- **The page** is on the website, written in a design session. Claude Code then checks every statement in it against the app.
- **Done when:** the guide is published on the website with the beta, and Claude Code's check has found nothing left to correct.

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

Questions 1, 2 and 10 are Nevada's, each with Claude Code's suggestion; 1 and 10 are answered, and 2 is still open. The others were the design session's, answered on 2026-11-24 (#270); each answer is recorded below, and the slices above follow it.

1. **Code signing (Nevada).** The product brief says signing is needed "so the target users don't see security warnings". It still needs pricing and a choice.
   - On the Mac: Apple's Developer Program, about $99 a year, enrolled as an individual (the certificate then shows Nevada's name) or as an organization (which needs a legal entity).
   - On Windows, the options include Microsoft's cloud signing service (a monthly fee, with identity checks), a signing program for open-source projects (free, with conditions), or a certificate from a certificate authority.
   - *Suggestion:* Apple's program as an individual; for Windows, Nevada prices the cloud service and the open-source program, and Claude Code checks each against the release workflow before choosing.
   - **Nevada's answer (2026-11-24):** Apple's Developer Program, which Nevada is signing up for. For Windows, SignPath Foundation's free program for open-source projects, since cost matters for a project given away that may or may not find an audience, and a monthly subscription most of all. With it, the certificate names SignPath Foundation as the publisher, builds must come from CI, and releases carry SignPath's attribution line (slice 1).
2. **The Windows install location (#70, Nevada or a design session).** Per user only, as now, in `%LOCALAPPDATA%\Programs`; Program Files for everyone; or a choice in the installer. *Suggestion:* per user only, as now. It needs no administrator rights, keeps updates silent, and every file Seura installs stays in the user's own profile. The user guide says where it is.
3. **The Settings screen's layout.** As suggested: one page in SET-2's order, applying at once. On macOS, a window from the app menu (Cmd+,); on Windows, a dialog with Close, from ⋯ → Settings… (Ctrl+,). Added: removing a library folder confirms that its photos and edit files aren't touched; clearing the cache confirms that it's rebuilt automatically; Updates shows "Last checked …" and Check now. In the wireframes, §9.
4. **The Clean up dialog's layout.** As suggested, plus: nothing checked at first, with Move to Recycle Bin (Trash) disabled until something is chosen and Cancel holding the focus; choices that don't apply to the scope disabled; the preview leading with how many photos will lose edits, tags or ratings; and a full reset confirming once more, then restarting at Welcome. A folder's right-click menu opens it with that folder chosen. In the wireframes, §10. **A rule change came with it:** clearing the cache, from Settings or Clean up, deletes it directly rather than moving it to the Trash (decision 0018, CLN-4, CLAUDE.md rule 5). Edit files, settings and presets still go to the Trash.
5. **CLN-5 on the Mac.** As suggested, now in the spec.
6. **How often to check for updates.** At launch, at most once a day, plus Check now. An available update is a quiet notice, never a dialog, and never during a review or an export. The request carries nothing identifying: no identifier, and the version only if the address needs it.
7. **Updates and pre-releases, and where the downloaded installer goes.** Two manifests on the website, changed only when Nevada publishes a release, as a website pull request he merges (website #15): `/updates/latest.json`, the newest release, read by release builds; and `/updates/beta.json`, the newest release or pre-release, read by pre-release builds (below 1.0, or with a pre-release suffix). The installer goes to the system's temporary folder and is deleted after installing; one left over is deleted at the next launch (decision 0013).
8. **The user guide: who writes it, and where.** As suggested: a page on the website, written in a design session from Claude Code's fact sheet, which includes the known issues. Claude Code then checks the draft against the app.
9. **Where the license notices are shown.** As suggested.
10. **The beta (Nevada).** How many testers, found where, and for how long. *Suggestion:* five to ten people, with at least one each on Windows 10, Windows 11, macOS 13 to 15 and macOS 26; two weeks; reports through GitHub issues. The beta ends early only for a data-safety problem.
   - **Nevada's answer (2026-11-24):** agreed. Testers come first from LinkedIn, where Nevada has been posting about Seura, and other channels can follow. Interest is limited so far, and a signed installer should help.
11. **Finder's Put Back for edit files (#254).** As suggested: a known issue for 1.0, explained in the user guide.
12. **Decoding colors in full (#188) before or after 1.0.** As suggested: in M7, before the beta. Moving it after 1.0 needs Nevada's agreement.

**Raised by Claude Code with the answers, for a design session:** rule 5 lists every file Seura may delete directly, and tests enforce that list. Decision 0018 adds the cache to it, but the update installer isn't on it: deleting it after installing, and deleting one left over at the next launch, are both direct deletions. Rule 5 also says leftovers from an earlier operation go to the Trash. The installer is Seura's own download and nothing in it can be lost, so it seems to fit decision 0017's test, but it needs adding to the list (and to the tests' allowance) before slice 2 deletes it. Until then, slice 2 sends a leftover installer to the Trash.
