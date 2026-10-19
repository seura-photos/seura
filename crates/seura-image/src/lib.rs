//! Decoding (JPEG, PNG, HEIC), color management, orientation, thumbnails,
//! and metadata (technical design, section 5).
//!
//! Thumbnails are made in sRGB; the floating-point working space for
//! editing (linear light, Display P3 primaries) arrives with the render
//! pipeline in M1 slice 5.

mod decode;
mod exif;
mod heif;
mod raster;
mod thumbnail;

pub use decode::{DecodeError, Decoded, MAX_SIDE, decode};
pub use heif::{DECODER_SWITCH, HeifDecoder, decode_with as decode_heif_with};
pub use raster::Rgba8;
pub use thumbnail::{THUMBNAIL_SIZE, fit, thumbnail};
