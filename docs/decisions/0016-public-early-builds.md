# 0016. Early builds are public from 0.2

Status: Accepted · 2026-10-27

## Context

The milestone plan made each milestone's numbered pre-release build "for the owner's own use", with a public beta, with volunteer testers and a written checklist, in M7. Build 0.1.1 was published as a GitHub pre-release, where anyone who looks can find it, but nothing pointed to it (#76).

By the end of M3, two things changed:

- **The Mac build opens without Terminal.** It's ad-hoc signed (#71), so the first launch needs only System Settings → Privacy & Security → Open Anyway. Windows SmartScreen warns, then runs it.
- **Build 0.2 can tag and rate,** not just browse, so it shows what Seura is for.

The project is built in the open, with a devlog. Readers can follow the decisions but had no way to try the result.

## Decision

From build 0.2, each milestone's pre-release build is offered on the website (seuraphotos.app) in a short "Try an early build" section, linking to the GitHub release.

- **What the website says:** that it's an early build, not the beta; that it isn't signed for distribution, and what each system shows the first time; that there are no automatic updates until M7; what it can and can't do yet; and that it never changes the user's photos, and where it keeps its own files.
- **Install steps stay in each release's notes,** where they're correct for that build. The website links to them.
- **Release notes** begin: "An early build of Seura Photos, before the beta: some features are still missing, and there are no automatic updates yet. It never changes your photos." They don't describe what the build can do, so they never go out of date.
- **No promise of support.** Problems can be reported on GitHub, but early builds aren't the beta: no checklist, no recruited testers.

## Rejected

- **Keeping builds unannounced until the beta (M7):** safe, but readers of a public devlog couldn't try anything for months.
- **Announcing from 0.1:** it needed a Terminal command to open on the Mac, and could only browse.

## Consequences

- The milestone plan's wording changes (each milestone's build is public from 0.2).
- Data safety matters to strangers from 0.2, not only to the owner. The rules in CLAUDE.md and the data-safety tests already apply to every build.
- The release workflow's notes change (Claude Code).
