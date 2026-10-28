# Seura Photos — Milestone Plan

Status: approved · Last updated: 2026-10-18

How Seura Photos gets from design to a public 1.0 release. This covers the whole project: the app, its visual design and brand, the website, and the project record that serves as a showcase. Requirement IDs refer to the [functional spec](functional-spec.md).

There are no dates. This is a one-person project built alongside other commitments, so the plan fixes the order and what "done" means, not the pace.

## Principles

- **Risk first, then usefulness.** The first milestone proves the hard technical pieces on both platforms. After that, every milestone leaves the owner with an app he can actually use for more of his own photos than before.
- **Read before write.** The app only reads until the data-safety tests are in place. Writing edit files comes after browsing works, and deletion comes after writing works.
- **Process order.** Features arrive roughly in the order of the user's process: find, sort, fix, organize, share. The Review flow comes once the actions it combines exist.
- **Every milestone ends the same way:** its exit criteria met, CI green on Windows and macOS, a numbered pre-release build, and a short write-up for the project record. From 0.2, these builds are offered on the website as early builds anyone can try, with no promise of support; the beta, with testers and a checklist, is still M7 (decision 0016).

## Tracks

Four tracks run side by side. The app track sets the order; the others attach to it.

| Track | What it produces |
| --- | --- |
| App | The application, milestone by milestone. |
| Brand and visual design | Colors, type, icons, app icon, logo: a small brand kit used by both the app and the website. |
| Website | `seura-photos/seura-photos.github.io`, published on GitHub Pages at seuraphotos.app. |
| Project record | A write-up at the end of each milestone, kept in `docs/journal/`, later the basis for the website's project story. |

## Before code: decisions only the owner can make

These are not app work, but they block parts of it.

- **Trademark search** for "Seura" in the relevant classes (software, photography). Before the placeholder website goes up, since the site markets the name.
- **Domains:** register seuraphotos.app and seuraphotos.com. Done: both registered with Cloudflare.
- **Code signing:** price Apple's developer program and a Windows signing option. Needed before the beta, not before.

## M1. Walking skeleton

Proves every risky piece on both platforms, as described in the [technical design](technical-design.md), section 13: the scaffold and CI; folder scan, index snapshot, and thumbnails in a virtualized grid; decoding JPEG, PNG, and iPhone HEIC with correct orientation and color; one edit operation (exposure) with a WebGPU preview, the WebGL2 fallback, autosave to a valid edit file, and wgpu export; the data-safety test suite; and the index benchmarks for decision 0010.

**Done when:** all six items work on the owner's Mac and Windows PC, the WebGL2 path passes in CI on an older macOS image, and the index decision is confirmed or replaced by a new record.

**Status:** complete, 2026-10-19, with the hands-on Editor checks moved to integration testing by the owner's decision; see the [M1 work plan's status](m1-plan.md#status-at-the-end-of-m1).

The UI is deliberately unstyled. The CLAUDE.md file gains real build, test, and lint commands. The slices are in the [M1 work plan](m1-plan.md).

## Brand and visual design (during M1)

Runs while M1 is being built, because M1 needs no visual design and M2 needs all of it.

- Color tokens for light and dark mode meeting WCAG AA (ACC-3), type scale, spacing, and the icon set (vector, DSP-1).
- App icon and logo, in the sizes each platform and the website need.
- The wireframe canvas updated to the visual design, for the Library, Viewer, Editor, and Review.

**Done when:** the brand kit is in the repo and the owner has approved the styled screens.

## M2. Browse

Seura replaces the owner's photo viewer. Read-only: no edit files written yet beyond M1's test path.

- Library: LIB-1 to LIB-8, with the first-run welcome (SET-1) and settings storage (SET-3).
- Browsing: BRW-1 to BRW-8, photo info (BRW-6), correct display (BRW-7).
- Display rules DSP-1 to DSP-4 and keyboard access for everything built so far (ACC-1).
- The visual design applied throughout.
- Performance targets for scanning and browsing measured on the reference hardware.

**Done when:** the owner can point Seura at his real photo library and browse it daily on both machines.

**Status:** complete, 2026-10-26, with build 0.1.1 as the owner's photo viewer on both machines; see the [M2 work plan's status](m2-plan.md#status).

## M3. Organize

The first milestone that writes user data in earnest.

- Edit files: FIL-1 to FIL-7, including unknown-content preservation and renamed-photo matching.
- Tags and ratings: TAG-1 to TAG-7, including the anchored tag picker.
- Rating as a quick action (part of BRW-10).
- Undo and redo in the Library for tags and ratings: BRW-11.
- Finding: FND-1 to FND-3.
- Delete to Trash: BRW-9.

**Done when:** the owner is tagging and rating his own library, the edit files pass the edit format's `check.py`, and the data-safety suite covers every new write path.

**Status:** complete, 2026-10-27, with build 0.2.1 as the owner's tool for tagging and rating on both machines; see the [M3 work plan's status](m3-plan.md#status).

## M4. Fix

The full editor.

- EDT-1 to EDT-12: crop, straighten, rotate and flip, adjustments, effects, red-eye, undo across sessions, revert, autosave, and Enhance.
- Rotate and Enhance as quick actions in the Library and Viewer, including on hover (BRW-10).
- Each operation implemented once as a WGSL shader (decision 0004), with a reference-image test showing preview and export match.

**Done when:** the owner fixes his own photos in Seura instead of another app. The edit format is proven by a full implementation, so the format can be frozen at 1.0 (see M7).

## M5. Reuse and share

- Copy and paste edits, and presets: PRE-1 to PRE-5.
- Export: EXP-1 to EXP-6, including sRGB conversion and metadata handling.

**Done when:** the owner exports real photos for sharing, and the exports look right in other apps on both platforms.

## M6. Review

- The Review flow: REV-1 to REV-6, combining rating, rotate, Enhance, tagging, marking for deletion, and editing.
- The new-photos banner after a scan.

**Done when:** the owner runs his newly added photos through Review end to end.

## M7. Release readiness

Everything between "works for me" and "ready for strangers."

- Cleanup: CLN-1 to CLN-5. Settings: SET-2. Updates: SET-4, with signed update packages.
- Audits against the spec: accessibility (ACC-1 to ACC-5), interaction rules (INT-1 to INT-5), display (DSP-1 to DSP-4), and every performance target.
- Signed installers for Windows and macOS; third-party license notices.
- The user guide, including installing Microsoft's HEIF and HEVC extensions on Windows, and how Seura works with cloud-synced folders, with a brief summary of what each service offers (for example, Google Photos has no desktop sync, so Android users may prefer OneDrive or Dropbox camera backup; iCloud Photos on a Mac isn't a folder).
- The edit format released as 1.0 in the `edit-format` repository; from then on, changes follow its versioning rules.
- **Beta:** a short beta with volunteers, especially on Windows 10 and macOS 13 to 15, following a written test checklist. If nobody tests macOS 13 to 15, the macOS minimum is raised to Tahoe 26 with a new decision record.

**Done when:** the beta finds no data-safety problems and no blocking bugs, and the checklist passes on every supported OS.

## Website

- **Placeholder (any time after the trademark search and domains):** the name, a one-line description, and "coming soon," so the name and domain are visibly in use.
- **Full site (during M7):** what Seura Photos is and who it's for, with screenshots; features and the reasons behind them; downloads linked to GitHub Releases; the user guide; a page about the edit format; and the project story, drawn from the project record.

**Done when:** the full site is reviewed and ready to go live with the release.

## M8. Version 1.0

- Release 1.0 on GitHub Releases, with release notes.
- The website goes live with download links.
- An announcement, written for the owner's own channels.
- A closing write-up for the project record: what was built, what was learned, what comes next.

## After 1.0

The first update adds AI assistance (bring your own key), as the [product brief](product-brief.md) describes. Spot removal and the other "later" items in the functional spec follow, each planned when its time comes.

## Working within a milestone

- Each milestone is broken into GitHub issues, one per requirement or small slice, labeled with the requirement IDs.
- Changes are small and reviewable, each with tests, referencing requirement IDs in commit messages.
- Pre-release builds are numbered 0.1 (M2) through 0.6 (M7's beta), so the owner can run and compare them. From 0.2 they're public early builds (decision 0016).
- A change that conflicts with the spec, the technical design, or a decision record stops and gets raised rather than worked around.
