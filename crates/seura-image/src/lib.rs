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

pub use decode::{
    DecodeError, Decoded, Linear, MAX_SIDE, Reduced, Upright, decode, decode_linear,
    decode_reduced, decode_upright,
};
pub use encode::{JPEG_QUALITY, encode_jpeg};
pub use exif::{Details, details};
pub use raster::{LinearImage, Rgba8, decode_transfer};
pub use system_thumbnail::system_thumbnail;
pub use thumbnail::{Region, THUMBNAIL_SIZE, fit, thumbnail};

/// The long edge of the image highlights and shadows build their luminance
/// guide from: the photo scaled to fit it, or as it is if it's smaller
/// (technical design, "Neighborhood operations").
pub const GUIDE_LONG_EDGE: u32 = 1024;
