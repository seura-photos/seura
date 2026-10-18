# 0007. An opinionated interface: one layout, ordered by frequency and process

Status: Accepted · 2026-10-17

## Context

Photo apps tend toward customizable workspaces: movable panels, toggles to show and hide tools, and sometimes separate beginner and advanced modes. The target users are former Picasa users who want something that just works, and the project is maintained by one person, so every option multiplies design, documentation, and testing.

## Decision

- **One layout per view, decided by the app.** No rearranging, hiding, or showing panels; no toolbar customization. The Info panel is always shown in the Library and Viewer; the Editor's tools replace it in the Editor.
- **No modes.** There is no separate beginner or advanced mode. Depth is handled by progressive disclosure that the app controls: details appear when they become relevant (an effect's strength slider appears once the effect is on).
- **Depth follows frequency.** Every action is assigned a tier with a maximum cost (see `docs/ux/user-flows.md`): the most frequent actions are one click or key from wherever the user is.
- **Order follows the process.** Find, sort, fix, organize, share. The Editor's tools are ordered the way a photo is usually fixed (Enhance, crop and straighten, light, color, effects), and a guided Review flow (REV-1 to REV-6) takes a new batch of photos through sorting and light fixing in one pass.
- **Few settings.** Settings cover only the user's system and data (folders, cache location, updates) and accessibility, which follows the operating system.

## Alternatives considered

- **Beginner and advanced modes.** Users often don't know which mode they're in or that the other exists; features must be designed and tested twice; and the advanced mode tends to collect everything that's hard to place.
- **Customizable panels and toolbars.** Flexible, but it moves design decisions onto users, makes support and documentation harder, and adds settings the target user doesn't want.

## Consequences

- New features must find a place in the existing order and tiers rather than adding a toggle or a mode. If a feature doesn't fit, that's a signal to reconsider the feature.
- Documentation and tutorials can describe exactly one way to do each task.
- Some experienced users will want more control; the answer is better defaults and well-chosen depth, not configuration.
