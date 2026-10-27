# Seura Photos — Visual Design

Status: approved · Last updated: 2026-10-27

The look of the app, its icon, and its brand. The layouts themselves are in the [wireframes](ux/wireframes.md); this document covers color, type, icons, and the mark. The values live in [`design/tokens.json`](../design/tokens.json), which the app and the website both read, so they can't drift apart.

## Principles

- **Photos first.** The interface is pure neutral gray, with no warm or cool tint, so it never shifts how a photo's colors look. Color in the interface means something: the accent marks the main action, the selection, and focus.
- **One accent.** Seura blue, a deep ultramarine. Changing it later means changing one token.
- **Native and quiet.** The system font, simple line icons, light borders rather than shadows.
- **The personality lives in the brand.** The colorful dots belong to the icon, logo, website, and installer, never to the working interface.

## Color

Two sets of tokens, light and dark; the app follows the operating system's setting (ACC-3) and has no appearance option of its own (SET-2). Every text color meets WCAG AA contrast against the surfaces it's used on.

| Token | Light | Dark | Used for |
| --- | --- | --- | --- |
| photo | `#F7F7F7` | `#1A1A1A` | Behind photos: grid, viewer, editor, review |
| panel | `#F0F0F0` | `#222222` | Header, sidebar, info and edit panels |
| surface | `#FFFFFF` | `#2E2E2E` | Buttons, fields, chips, cards |
| border | `#D6D6D6` | `#3A3A3A` | Dividers and control outlines |
| text | `#1B1B1B` | `#ECECEC` | Primary text |
| textSecondary | `#5E5E5E` | `#A0A0A0` | Labels, counts, metadata |
| accent | `#3352CC` | `#3D5BE0` | Main action fill, selection outline, focus ring (white text on it) |
| accentText | `#3352CC` | `#8EA6FF` | Links and quiet buttons |
| accentSubtle | `#E3E8FA` | `#26304F` | Selected rows, banners, effects that are on |
| danger | `#B3261E` | `#FF8A80` | Unreadable edit files, Trash confirmations |
| rating | `#9A6700` | `#E3B341` | Filled stars |
| ratingOff | `#7A7A7A` | `#808080` | Empty stars, drawn as outlines (3:1 or more on the panel and surface) |
| ratingDisabled | `#C9C9C9` | `#4D4D4D` | Stars in a read-only folder; disabled controls are exempt from contrast, and a line of text says why (TAG-7) |

Markers drawn over photos (edited, rating, hover tools) use a dark translucent pill with white content, which reads on any photo in either mode.

**Brand dots**, for the mark, website, and installer only: blue `#3352CC`, sky `#7FA3E8`, green `#3E9A6A`, gold `#E9B949`, orange `#E8823A`, rose `#D9606A`.

## Components

Details the tokens alone don't settle, decided while building M3.

- **Selected row** (folder and tag trees): the accentSubtle fill, plus a 3 px bar on the left edge and the text and icon in accentText. The fill alone is about 1.1:1 against the panel, below the 3:1 a state needs (WCAG 1.4.11). The bar uses accentText, not accent: in dark mode accent is only 2.3:1 against the fill, accentText 5.6:1.
- **Stars:** filled in rating, empty as outlines in ratingOff, so empty and filled differ by shape as well as color; disabled in ratingDisabled.
- **Notice bar** (problems with files, INT-6): accentSubtle with an info icon for information; with a warning icon for a caution that needs nothing done, such as a photo changed by another program (FIL-7); dangerSubtle with a warning icon only when the user needs to act, such as an unreadable edit file with Start over (FIL-4). 8 px corners, at the top of the photo area.
- **Message** (the last action's result, INT-6): text on surface with a border and a soft shadow (`0 6px 20px rgba(0,0,0,0.18)`), 8 px corners, near the bottom of the photo area, centered, up to 480 px wide.
- **Sidebar divider:** the 1 px border at rest; 2 px in accentText on hover, while dragging, and when focused, with the focus ring (wireframes, "Resizable sidebar"). Widths are the sidebar, sidebarMin, and sidebarMax tokens.

## Type

- **App:** the operating system's font: Segoe UI Variable on Windows, San Francisco on macOS. It looks native, renders crisply at any scaling, and follows the system text size. Base size 13 px; view titles 17 px semibold; metadata 12 px; section labels 11 px semibold capitals.
- **Brand:** [Figtree](https://github.com/erikdkennedy/figtree) for the wordmark, website, and installer. Licensed under the SIL Open Font License, compatible with decision 0012.

## Icons

Line icons on a 24-pixel grid with a 1.75 stroke and rounded ends, from [Lucide](https://lucide.dev) (ISC license, compatible with decision 0012), as inline SVG so they stay crisp at any density (DSP-1). Every icon-only button has a text label for screen readers and a tooltip with its shortcut.

## The mark

A sun of warm dots setting behind a horizon of cool ones: a photograph, painted the way Seurat painted, which is where the name comes from. Chosen from three directions (the others were an S in dots and a lens of dots).

| File | Use |
| --- | --- |
| [`design/brand/app-icon.svg`](../design/brand/app-icon.svg) | App icon at 48 px and above, installer, website |
| [`design/brand/app-icon-small.svg`](../design/brand/app-icon-small.svg) | App icon at 32 px and below: taskbar, window title, file lists |
| [`design/brand/seura-mark.svg`](../design/brand/seura-mark.svg) | The mark without its background, beside the wordmark |
| [`design/brand/seura-mark-small.svg`](../design/brand/seura-mark-small.svg) | The same, at small sizes |

The small version uses fewer, larger dots so it stays recognizable at 16 px. Platform icon files (`.ico`, `.icns`) are generated from these SVGs during the build (M1).

**Wordmark:** the mark, then "Seura" in Figtree bold and "Photos" in Figtree regular, in the secondary text color. In the app's header, the small mark and "Seura" alone.

## Where to see it

The design canvas has a brand board and the Library (light and dark), Viewer, Editor, and Review screens styled with these tokens. Images of them are in [`docs/ux/mockups/`](ux/mockups/). Each styled screen has a light and dark switch. The remaining screens are styled during M2, from these tokens, as each is built.
