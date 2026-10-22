//! Decoding (JPEG, PNG, HEIC), color management, orientation, thumbnails,
//! and metadata (technical design, section 5).
//!
//! Thumbnails are made in 8-bit sRGB. Editing and export work in the edit
//! format's working space: linear light with Display P3 primaries, in
//! floating point ([`decode_linear`]).

mod decode;
mod encode;
mod exif;
mod heif;
mod raster;
mod system_thumbnail;
mod thumbnail;

pub use decode::{DecodeError, Decoded, MAX_SIDE, Reduced, decode, decode_linear, decode_reduced};
pub use encode::{JPEG_QUALITY, encode_jpeg};
pub use raster::{LinearImage, Rgba8, decode_transfer};
pub use system_thumbnail::system_thumbnail;
pub use thumbnail::{THUMBNAIL_SIZE, fit, thumbnail};
