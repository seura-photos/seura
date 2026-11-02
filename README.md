<img src="design/brand/app-icon.svg" width="96" alt="">

# Seura Photos

A free, open-source photo app for Windows and macOS, in the spirit of Picasa.

- **Works on your folders in place.** No importing, no managed library, no account.
- **Never touches your originals.** Every edit is stored in a small, readable file beside the photo, so you can change or undo it at any time.
- **Simple editing.** Crop, straighten, rotate, brightness, contrast, highlights and shadows, color, red-eye, and a few classic effects like black and white, sepia, and vignette.
- **Tags, not albums.** Organize with tags and subtags, and rate with stars.
- **Open format.** Edits are stored in the [Seura edit format](https://github.com/seura-photos/edit-format), an open specification any app can implement.

> **Status:** in development. [Build 0.3](https://github.com/seura-photos/seura/releases) is an early build, not yet the beta: it browses, organizes, and edits, but export, presets, and the Review flow come later, so for now edits show only in Seura. It isn't signed for distribution yet; each release's notes say how to install it. Progress is written up in the [devlog](https://seuraphotos.app/devlog/).

## Documents

- [Product brief](docs/product-brief.md): what Seura Photos is, who it's for, and the principles behind it.
- [Functional spec](docs/functional-spec.md): what version 1 does, with testable requirements.
- [Decision records](docs/decisions/): the major technical decisions and why they were made.
- [Edit format specification](https://github.com/seura-photos/edit-format): the file format for edits, tags, and ratings.
- [User flows and screen map](docs/ux/user-flows.md): the screens of v1 and how users move between them.
- [Wireframes](docs/ux/wireframes.md): the approved layout of each screen.
- [Technical design](docs/technical-design.md): architecture, libraries, rendering, testing.
- [Visual design](docs/visual-design.md): color, type, icons, and the Seura mark.
- [Milestone plan](docs/milestone-plan.md): the order of work from first code to version 1.0.
- Work plans for each milestone, slice by slice: [M1](docs/m1-plan.md), [M2](docs/m2-plan.md), [M3](docs/m3-plan.md), [M4](docs/m4-plan.md).
- [Benchmarks](docs/benchmarks/): the performance targets, measured at the end of each milestone.
- [Journal](docs/journal/): short dated notes on decisions, surprises, and mistakes as the work goes.

## Technology

[Tauri](https://tauri.app) with a Rust core and a TypeScript web UI, with editing rendered on the GPU. See [decision 0001](docs/decisions/0001-tauri-and-rust.md).

To build and run it yourself, see [CLAUDE.md](CLAUDE.md#build-test-and-lint): it lists the tools, the commands, and the checks CI runs.

## License

[Apache License 2.0](LICENSE).
