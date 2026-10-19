//! The Seura edit format: parsing, validation against the schema, canonical
//! order, exact preservation of unknown operations and fields, history, and
//! presets (technical design, section 7).
//!
//! This crate must not depend on any other Seura crate, so it can be published
//! as the format's reference library (decision 0009).
//!
//! - [`EditDocument`] is a file's contents: the JSON as read, with a typed
//!   view of what this version understands. Edits change only what they're
//!   about, so everything else is saved back exactly as found.
//! - [`EditFile`] loads and saves one file safely: atomic writes, and never
//!   overwriting a file it can't read or one that changed underneath it.

mod document;
mod file;
mod operations;
mod validate;

pub use document::{
    Action, Edit, EditDocument, EditError, HISTORY_LIMIT, Kind, Operation, PhotoIdentity,
    ReadError, Step, clean_tags,
};
pub use file::{EditFile, LoadError, SaveError, Writer, edit_file_path, write_atomically};
pub use operations::{
    Amount, CORE_VERSION, CoreOpId, CoreOperation, Crop, Exposure, Orient, RedEye, RedEyeRegion,
    Stage, Straighten, Strength, Vignette, WhiteBalance,
};
