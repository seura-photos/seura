//! The Seura edit format: parsing, validation against the schema, canonical
//! order, exact preservation of unknown operations and fields, history, and
//! presets (technical design, section 7).
//!
//! This crate must not depend on any other Seura crate, so it can be published
//! as the format's reference library (decision 0009).
