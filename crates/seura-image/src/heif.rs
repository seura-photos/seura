//! HEIC/HEIF decoding (decision 0011): the system's decoder where there is
//! one, libheif on Windows otherwise. The platform implementations follow.

use std::path::Path;

use crate::decode::{DecodeError, Decoded};

pub fn decode(_path: &Path) -> Result<Decoded, DecodeError> {
    Err(DecodeError::HeifUnavailable("not implemented yet".into()))
}
