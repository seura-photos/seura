//! Decoding a photo file to upright pixels, with its dimensions and date
//! taken (LIB-8, BRW-7): 8-bit sRGB for thumbnails, or the working space
//! for editing and export. Photos are only read, never written.

use std::fs;
use std::io::Cursor;
use std::path::Path;

use zune_jpeg::JpegDecoder;
use zune_jpeg::zune_core::bytestream::ZCursor;
use zune_jpeg::zune_core::colorspace::ColorSpace;
use zune_jpeg::zune_core::options::DecoderOptions;

use crate::exif::{self, ExifInfo};
use crate::heif;
use crate::raster::{LinearImage, Rgba8};
use crate::thumbnail::fit;

/// Photos larger than this on either side aren't decoded (about 268
/// megapixels), so a damaged or hostile file can't exhaust memory.
pub const MAX_SIDE: u32 = 16_384;

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("couldn't read the photo: {0}")]
    Io(#[from] std::io::Error),
    #[error("not a photo Seura can show")]
    Unsupported,
    #[error("the photo is damaged: {0}")]
    Damaged(String),
    #[error("the photo is too large ({0}×{1})")]
    TooLarge(u32, u32),
    #[error("HEIC photos can't be decoded on this computer: {0}")]
    HeifUnavailable(String),
}

/// A decoded photo.
#[derive(Clone, Debug)]
pub struct Decoded {
    /// Upright, in sRGB.
    pub image: Rgba8,
    /// Seconds since 1970 on the photo's own clock, from EXIF.
    pub taken: Option<i64>,
}

/// Decoded pixels before color conversion: upright, 8-bit, in the color
/// space of the photo's profile (sRGB without one).
pub(crate) struct Source {
    pub image: Rgba8,
    pub icc: Option<Vec<u8>>,
    pub taken: Option<i64>,
}

/// Decodes a JPEG, PNG, or HEIC/HEIF file, with orientation applied and
/// its color profile converted to sRGB.
pub fn decode(path: &Path) -> Result<Decoded, DecodeError> {
    let source = decode_source(path)?;
    Ok(Decoded {
        image: source.image.from_profile(source.icc.as_deref()),
        taken: source.taken,
    })
}

/// Decodes a photo into the working space (linear light, Display P3), for
/// editing and export. With `long_edge`, it's first scaled down to fit, for
/// the preview (DSP-1).
pub fn decode_linear(path: &Path, long_edge: Option<u32>) -> Result<LinearImage, DecodeError> {
    let source = decode_source(path)?;
    let image = match long_edge {
        Some(size) => fit(&source.image, size),
        None => source.image,
    };
    Ok(image.to_linear_p3(source.icc.as_deref()))
}

fn decode_source(path: &Path) -> Result<Source, DecodeError> {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    let extension = name.rsplit('.').next().unwrap_or_default();
    match extension {
        "jpg" | "jpeg" => decode_jpeg(&fs::read(path)?),
        "png" => decode_png(&fs::read(path)?),
        "heic" | "heif" => heif::decode(path),
        _ => Err(DecodeError::Unsupported),
    }
}

pub(crate) fn decode_jpeg(bytes: &[u8]) -> Result<Source, DecodeError> {
    let options = DecoderOptions::default()
        .jpeg_set_out_colorspace(ColorSpace::RGBA)
        .set_max_width(MAX_SIDE as usize)
        .set_max_height(MAX_SIDE as usize);
    let mut decoder = JpegDecoder::new_with_options(ZCursor::new(bytes), options);
    let pixels = decoder.decode().map_err(|e| damaged(&e))?;
    let info = decoder
        .info()
        .ok_or_else(|| DecodeError::Damaged("no image header".into()))?;
    let exif = decoder
        .exif()
        .map(|raw| exif::read(raw))
        .unwrap_or(ExifInfo {
            orientation: 1,
            taken: None,
        });
    let icc = decoder.icc_profile();
    let image =
        Rgba8::new(u32::from(info.width), u32::from(info.height), pixels).orient(exif.orientation);
    Ok(Source {
        image,
        icc,
        taken: exif.taken,
    })
}

pub(crate) fn decode_png(bytes: &[u8]) -> Result<Source, DecodeError> {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(|e| damaged(&e))?;
    let (width, height) = (reader.info().width, reader.info().height);
    if width > MAX_SIDE || height > MAX_SIDE {
        return Err(DecodeError::TooLarge(width, height));
    }
    let icc = reader.info().icc_profile.as_ref().map(|icc| icc.to_vec());
    let exif = reader
        .info()
        .exif_metadata
        .as_ref()
        .map(|raw| exif::read(raw));
    let size = reader
        .output_buffer_size()
        .ok_or(DecodeError::TooLarge(width, height))?;
    let mut buffer = vec![0; size];
    let frame = reader.next_frame(&mut buffer).map_err(|e| damaged(&e))?;
    buffer.truncate(frame.buffer_size());
    let pixels = to_rgba(&buffer, frame.color_type)?;
    let exif = exif.unwrap_or(ExifInfo {
        orientation: 1,
        taken: None,
    });
    Ok(Source {
        image: Rgba8::new(width, height, pixels).orient(exif.orientation),
        icc,
        taken: exif.taken,
    })
}

/// 8-bit gray, gray-alpha, RGB, or RGBA to RGBA.
fn to_rgba(buffer: &[u8], color: png::ColorType) -> Result<Vec<u8>, DecodeError> {
    let pixels = match color {
        png::ColorType::Rgba => buffer.to_vec(),
        png::ColorType::Rgb => buffer
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|[r, g, b]| [*r, *g, *b, 255])
            .collect(),
        png::ColorType::Grayscale => buffer.iter().flat_map(|&v| [v, v, v, 255]).collect(),
        png::ColorType::GrayscaleAlpha => buffer
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|[v, a]| [*v, *v, *v, *a])
            .collect(),
        png::ColorType::Indexed => {
            return Err(DecodeError::Damaged("palette not expanded".into()));
        }
    };
    Ok(pixels)
}

fn damaged(error: &impl std::fmt::Display) -> DecodeError {
    DecodeError::Damaged(error.to_string())
}
