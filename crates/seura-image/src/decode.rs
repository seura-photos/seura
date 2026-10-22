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
use crate::thumbnail::{Region, fit, resize_region};

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

/// A photo decoded for its thumbnail: scaled to fit `size` × `size`, with
/// the full photo's dimensions and date taken.
#[derive(Clone, Debug)]
pub struct Reduced {
    /// Upright, in sRGB, at most `size` on its long edge.
    pub image: Rgba8,
    /// The full photo's upright dimensions.
    pub width: u32,
    pub height: u32,
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

/// Decodes a photo for its thumbnail (PRF-3). JPEGs are decoded at a half,
/// quarter, or eighth of their size, the smallest still at least `size` on
/// the long edge, which is several times faster than decoding them whole.
/// A JPEG that way can't read is decoded whole, as other formats are.
pub fn decode_reduced(path: &Path, size: u32) -> Result<Reduced, DecodeError> {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    let source = if name.ends_with(".jpg") || name.ends_with(".jpeg") {
        let bytes = fs::read(path)?;
        // A decoder bug on a damaged file mustn't stop the thumbnail worker.
        match std::panic::catch_unwind(|| decode_jpeg_scaled(&bytes, size)) {
            Ok(Some(scaled)) => scaled,
            _ => full_size(decode_jpeg(&bytes)?),
        }
    } else {
        full_size(decode_source(path)?)
    };
    let (source, width, height) = source;
    Ok(Reduced {
        image: fit(&source.image.from_profile(source.icc.as_deref()), size),
        width,
        height,
        taken: source.taken,
    })
}

/// A decoded photo and its dimensions, which are the image's own.
fn full_size(source: Source) -> (Source, u32, u32) {
    let (width, height) = (source.image.width, source.image.height);
    (source, width, height)
}

/// A JPEG decoded at reduced size, with its full upright dimensions; `None`
/// if it isn't 8-bit gray or RGB, or can't be read this way.
fn decode_jpeg_scaled(bytes: &[u8], size: u32) -> Option<(Source, u32, u32)> {
    use jpeg_decoder::{Decoder, PixelFormat};

    let mut decoder = Decoder::new(bytes);
    decoder.read_info().ok()?;
    let full = decoder.info()?;
    let (width, height) = (u32::from(full.width), u32::from(full.height));
    if width > MAX_SIDE || height > MAX_SIDE {
        return None;
    }
    let requested = u16::try_from(size).ok()?;
    decoder.scale(requested, requested).ok()?;
    let pixels = decoder.decode().ok()?;
    let info = decoder.info()?;
    let pixels: Vec<u8> = match info.pixel_format {
        PixelFormat::RGB24 => pixels
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|[r, g, b]| [*r, *g, *b, 255])
            .collect(),
        PixelFormat::L8 => pixels.iter().flat_map(|&v| [v, v, v, 255]).collect(),
        PixelFormat::L16 | PixelFormat::CMYK32 => return None,
    };
    let (scaled_width, scaled_height) = (u32::from(info.width), u32::from(info.height));
    if pixels.len() != (scaled_width * scaled_height * 4) as usize {
        return None;
    }
    let exif = decoder.exif_data().map(exif::read).unwrap_or(ExifInfo {
        orientation: 1,
        taken: None,
    });
    // Orientations 5 to 8 turn the photo a quarter turn.
    let (width, height) = if exif.orientation >= 5 {
        (height, width)
    } else {
        (width, height)
    };
    let source = Source {
        image: Rgba8::new(scaled_width, scaled_height, pixels).orient(exif.orientation),
        icc: decoder.icc_profile(),
        taken: exif.taken,
    };
    Some((source, width, height))
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

/// A photo decoded at full size, upright, before color conversion: what the
/// Viewer keeps while zoomed in, so each part it shows is cut from it.
pub struct Upright {
    image: Rgba8,
    icc: Option<Vec<u8>>,
}

/// Decodes a photo at full size, with orientation applied, for cutting
/// parts from ([`Upright::region_linear`]).
pub fn decode_upright(path: &Path) -> Result<Upright, DecodeError> {
    let source = decode_source(path)?;
    Ok(Upright {
        image: source.image,
        icc: source.icc,
    })
}

impl Upright {
    pub fn width(&self) -> u32 {
        self.image.width
    }

    pub fn height(&self) -> u32 {
        self.image.height
    }

    /// The part `region` of the photo (kept within it), scaled to `width` ×
    /// `height`, in the working space, as [`decode_linear`] gives it.
    pub fn region_linear(&self, region: Region, width: u32, height: u32) -> LinearImage {
        let x = region.x.min(self.image.width.saturating_sub(1));
        let y = region.y.min(self.image.height.saturating_sub(1));
        let region = Region {
            x,
            y,
            width: region.width.clamp(1, self.image.width - x),
            height: region.height.clamp(1, self.image.height - y),
        };
        resize_region(&self.image, region, width.max(1), height.max(1))
            .to_linear_p3(self.icc.as_deref())
    }
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
