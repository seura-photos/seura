//! HEIC on Windows: WIC when Microsoft's HEIF and HEVC extensions are
//! installed, libheif (with libde265) otherwise (decision 0011).

#![allow(unsafe_code, reason = "WIC is a COM API and libheif is loaded by hand")]

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};
use windows::Win32::Foundation::GENERIC_READ;
use windows::Win32::Graphics::Imaging::{
    CLSID_WICImagingFactory, GUID_WICPixelFormat32bppRGBA, IWICImagingFactory,
    WICBitmapDitherTypeNone, WICBitmapPaletteTypeCustom, WICDecodeMetadataCacheOnDemand,
};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
};
use windows::Win32::System::LibraryLoader::{LOAD_WITH_ALTERED_SEARCH_PATH, LoadLibraryExW};
use windows::core::HSTRING;

use super::{HeifDecoder, Pixels};
use crate::decode::{DecodeError, MAX_SIDE};
use crate::raster::Rgba8;

pub(super) fn decode(path: &Path, decoder: HeifDecoder) -> Result<Pixels, DecodeError> {
    match decoder {
        HeifDecoder::System => decode_with_wic(path),
        HeifDecoder::Libheif => decode_with_libheif(path),
        // Microsoft's extensions missing or failing means libheif.
        HeifDecoder::Automatic => decode_with_wic(path).or_else(|_| decode_with_libheif(path)),
    }
}

/// WIC decodes the pixels and applies the container's rotation and
/// mirroring itself. (Checked against libheif with test files made by
/// Apple's ImageIO; a claim elsewhere that it doesn't was wrong.)
fn decode_with_wic(path: &Path) -> Result<Pixels, DecodeError> {
    let unavailable =
        |error: windows::core::Error| DecodeError::HeifUnavailable(format!("WIC: {error}"));
    unsafe {
        // Each worker thread joins the multithreaded apartment once and stays.
        // A thread already in another apartment can still use WIC.
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)
                .map_err(unavailable)?;
        // Opened for reading only (CLAUDE.md rule 1).
        let decoder = factory
            .CreateDecoderFromFilename(
                &HSTRING::from(path),
                None,
                GENERIC_READ,
                WICDecodeMetadataCacheOnDemand,
            )
            .map_err(unavailable)?;
        let frame = decoder.GetFrame(0).map_err(unavailable)?;
        let (mut width, mut height) = (0, 0);
        frame
            .GetSize(&mut width, &mut height)
            .map_err(unavailable)?;
        if width > MAX_SIDE || height > MAX_SIDE {
            return Err(DecodeError::TooLarge(width, height));
        }
        let converter = factory.CreateFormatConverter().map_err(unavailable)?;
        converter
            .Initialize(
                &frame,
                &GUID_WICPixelFormat32bppRGBA,
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeCustom,
            )
            .map_err(unavailable)?;
        let mut pixels = vec![0; width as usize * height as usize * 4];
        converter
            .CopyPixels(std::ptr::null(), width * 4, &mut pixels)
            .map_err(unavailable)?;
        Ok(Pixels {
            image: Rgba8::new(width, height, pixels),
            transformed: true,
            color_managed: false,
        })
    }
}

/// libheif decodes the pixels and applies the container's rotation itself.
fn decode_with_libheif(path: &Path) -> Result<Pixels, DecodeError> {
    load_libheif()?;
    let failed = |error: libheif_rs::HeifError| DecodeError::Damaged(format!("libheif: {error}"));
    let name = path
        .to_str()
        .ok_or_else(|| DecodeError::Damaged("file name isn't valid Unicode".into()))?;
    let context = HeifContext::read_from_file(name).map_err(failed)?;
    let handle = context.primary_image_handle().map_err(failed)?;
    if handle.width() > MAX_SIDE || handle.height() > MAX_SIDE {
        return Err(DecodeError::TooLarge(handle.width(), handle.height()));
    }
    let image = LibHeif::new()
        .decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), None)
        .map_err(failed)?;
    let plane = image
        .planes()
        .interleaved
        .ok_or_else(|| DecodeError::Damaged("libheif returned no RGBA plane".into()))?;
    let row = plane.width as usize * 4;
    let pixels: Vec<u8> = plane
        .data
        .chunks(plane.stride)
        .take(plane.height as usize)
        .flat_map(|line| &line[..row])
        .copied()
        .collect();
    Ok(Pixels {
        image: Rgba8::new(plane.width, plane.height, pixels),
        transformed: true,
        color_managed: false,
    })
}

/// Loads `heif.dll` (and through it `libde265.dll`) from a known folder.
/// It's delay-loaded (see `build.rs`), so the app starts without it and
/// only HEIC decoding needs it. Looked for beside the app, where the
/// installer puts it, then in vcpkg's build output, for development.
fn load_libheif() -> Result<(), DecodeError> {
    static LOADED: OnceLock<Result<(), String>> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let folders = libheif_folders();
            for folder in &folders {
                let dll = folder.join("heif.dll");
                if dll.is_file()
                    && unsafe {
                        LoadLibraryExW(
                            &HSTRING::from(dll.as_path()),
                            None,
                            LOAD_WITH_ALTERED_SEARCH_PATH,
                        )
                    }
                    .is_ok()
                {
                    return Ok(());
                }
            }
            Err(format!("heif.dll wasn't found in {folders:?}"))
        })
        .clone()
        .map_err(DecodeError::HeifUnavailable)
}

fn libheif_folders() -> Vec<PathBuf> {
    let mut folders = Vec::new();
    if let Some(folder) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    {
        folders.push(folder);
    }
    if let Some(root) = std::env::var_os("VCPKG_ROOT") {
        folders.push(
            PathBuf::from(root)
                .join("installed")
                .join("x64-windows")
                .join("bin"),
        );
    }
    folders
}
