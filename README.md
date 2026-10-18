# Seura Photos

A free, open-source photo app for Windows and macOS, in the spirit of Picasa.

- **Works on your folders in place.** No importing, no managed library, no account.
- **Never touches your originals.** Every edit is stored in a small, readable file beside the photo, so you can change or undo it at any time.
- **Simple editing.** Crop, straighten, rotate, exposure, contrast, color, red-eye, and a few classic effects like black and white, sepia, and vignette.
- **Tags, not albums.** Organize with tags and subtags, and rate with stars.
- **Open format.** Edits are stored in the [Seura edit format](https://github.com/seura-photos/edit-format), an open specification any app can implement.

> **Status:** in design. There is no app to download yet. The documents below describe what's being built.

## Documents

- [Product brief](docs/product-brief.md): what Seura Photos is, who it's for, and the principles behind it.
- [Functional spec](docs/functional-spec.md): what version 1 does, with testable requirements.
- [Decision records](docs/decisions/): the major technical decisions and why they were made.
- [Edit format specification](https://github.com/seura-photos/edit-format): the file format for edits, tags, and ratings.

Still to come: UI design, technical design, and the milestone plan.

## Technology

[Tauri](https://tauri.app) with a Rust core and a TypeScript web UI, with editing rendered on the GPU. See [decision 0001](docs/decisions/0001-tauri-and-rust.md).

## License

[Apache License 2.0](LICENSE).
