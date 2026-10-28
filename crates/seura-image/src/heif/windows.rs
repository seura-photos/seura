//! HEIC on Windows: WIC, with Microsoft's HEIF and HEVC extensions from the
//! Microsoft Store (decision 0014).

#![allow(unsafe_code, reason = "WIC and Media Foundation are COM APIs")]

use std::path::Path;

use windows::Win32::Foundation::GENERIC_READ;
use windows::Win32::Graphics::Imaging::{
    CLSID_WICImagingFactory, GUID_ContainerFormatHeif, GUID_WICPixelFormat32bppRGBA,
    IWICImagingFactory, WICBitmapDitherTypeNone, WICBitmapPaletteTypeCustom,
    WICDecodeMetadataCacheOnDemand,
};
use windows::Win32::Media::MediaFoundation::{
    IMFActivate, MFMediaType_Video, MFT_CATEGORY_VIDEO_DECODER, MFT_ENUM_FLAG_ALL,
    MFT_REGISTER_TYPE_INFO, MFTEnumEx, MFVideoFormat_HEVC,
};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree,
};
use windows::core::HSTRING;

use super::Pixels;
use crate::decode::{DecodeError, MAX_SIDE};
use crate::raster::Rgba8;

/// WIC decodes the pixels and applies the container's rotation and
/// mirroring itself. (Checked with test files made by Apple's ImageIO.)
pub(super) fn decode(path: &Path) -> Result<Pixels, DecodeError> {
    unsafe {
        // Each worker thread joins a single-threaded apartment once and stays
        // (the system thumbnails need one; see system_thumbnail.rs). A thread
        // already in another apartment can still use WIC.
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)
                .map_err(|error| DecodeError::HeifUnavailable(format!("WIC: {error}")))?;
        // A failure is the photo's fault only if both extensions are there.
        let failed = |error: windows::core::Error| match missing_extension(&factory) {
            Some(missing) => DecodeError::HeifUnavailable(missing.into()),
            None => DecodeError::Damaged(format!("WIC: {error}")),
        };
        // Opened for reading only (CLAUDE.md rule 1).
        let decoder = factory
            .CreateDecoderFromFilename(
                &HSTRING::from(path),
                None,
                GENERIC_READ,
                WICDecodeMetadataCacheOnDemand,
            )
            .map_err(failed)?;
        let frame = decoder.GetFrame(0).map_err(failed)?;
        let (mut width, mut height) = (0, 0);
        frame.GetSize(&mut width, &mut height).map_err(failed)?;
        if width > MAX_SIDE || height > MAX_SIDE {
            return Err(DecodeError::TooLarge(width, height));
        }
        let converter = factory.CreateFormatConverter().map_err(failed)?;
        converter
            .Initialize(
                &frame,
                &GUID_WICPixelFormat32bppRGBA,
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeCustom,
            )
            .map_err(failed)?;
        let mut pixels = vec![0; width as usize * height as usize * 4];
        converter
            .CopyPixels(std::ptr::null(), width * 4, &mut pixels)
            .map_err(failed)?;
        Ok(Pixels {
            image: Rgba8::new(width, height, pixels),
            transformed: true,
            icc: None,
        })
    }
}

/// Which of Microsoft's extensions is missing, if either: HEIF Image
/// Extensions (WIC's HEIF decoder) or HEVC Video Extensions (the video
/// decoder that HEIF decoder uses for the pixels).
fn missing_extension(factory: &IWICImagingFactory) -> Option<&'static str> {
    if unsafe { factory.CreateDecoder(&GUID_ContainerFormatHeif, std::ptr::null()) }.is_err() {
        return Some("Microsoft's HEIF Image Extensions aren't installed");
    }
    if !hevc_decoder_installed() {
        return Some("Microsoft's HEVC Video Extensions aren't installed");
    }
    None
}

fn hevc_decoder_installed() -> bool {
    let input = MFT_REGISTER_TYPE_INFO {
        guidMajorType: MFMediaType_Video,
        guidSubtype: MFVideoFormat_HEVC,
    };
    let mut found: *mut Option<IMFActivate> = std::ptr::null_mut();
    let mut count = 0;
    unsafe {
        if MFTEnumEx(
            MFT_CATEGORY_VIDEO_DECODER,
            MFT_ENUM_FLAG_ALL,
            Some(&input),
            None,
            &mut found,
            &mut count,
        )
        .is_err()
        {
            return false;
        }
        if !found.is_null() {
            // Releases each activation object, then the array itself.
            for i in 0..count as usize {
                found.add(i).drop_in_place();
            }
            CoTaskMemFree(Some(found as *const _));
        }
    }
    count > 0
}

#[cfg(test)]
mod tests {
    /// Seura's development PC has both extensions; CI's runners may not.
    #[test]
    #[ignore = "needs Microsoft's HEIF and HEVC extensions"]
    fn both_extensions_are_found_where_installed() {
        unsafe {
            let _ = super::CoInitializeEx(None, super::COINIT_APARTMENTTHREADED);
            let factory: super::IWICImagingFactory = super::CoCreateInstance(
                &super::CLSID_WICImagingFactory,
                None,
                super::CLSCTX_INPROC_SERVER,
            )
            .unwrap();
            assert_eq!(super::missing_extension(&factory), None);
        }
    }
}
