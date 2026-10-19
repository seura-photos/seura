//! Thumbnails from the operating system, for photos that are only in the
//! cloud (LIB-9). The system and the sync service keep small previews of
//! cloud-only files, so a thumbnail can be shown without downloading the
//! photo. Never used for files on disk: those are decoded by Seura.

use std::path::Path;

use crate::raster::Rgba8;

/// The system's thumbnail for a file, at least `size` pixels on its long
/// edge where available, or `None` if there isn't one without reading the
/// file (never an icon in its place).
pub fn system_thumbnail(path: &Path, size: u32) -> Option<Rgba8> {
    platform::thumbnail(path, size)
}

#[cfg(target_os = "windows")]
mod platform {
    #![allow(unsafe_code, reason = "the Windows shell and GDI are C and COM APIs")]

    use std::path::Path;

    use windows::Win32::Foundation::SIZE;
    use windows::Win32::Graphics::Gdi::{
        BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC,
        DeleteObject, GetDIBits, GetObjectW, HBITMAP, HGDIOBJ,
    };
    use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
    use windows::Win32::UI::Shell::{
        IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF, SIIGBF_BIGGERSIZEOK,
        SIIGBF_THUMBNAILONLY,
    };
    use windows::core::HSTRING;

    use crate::raster::Rgba8;

    pub fn thumbnail(path: &Path, size: u32) -> Option<Rgba8> {
        unsafe {
            // A single-threaded apartment: in a multithreaded one, the shell
            // returns no thumbnail for cloud-only files (checked on Nevada's
            // OneDrive camera roll). WIC works in either.
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let factory: IShellItemImageFactory =
                SHCreateItemFromParsingName(&HSTRING::from(path), None).ok()?;
            let side = i32::try_from(size).ok()?;
            // A thumbnail only: never the file type's icon instead.
            let flags = SIIGBF(SIIGBF_THUMBNAILONLY.0 | SIIGBF_BIGGERSIZEOK.0);
            let bitmap = factory.GetImage(SIZE { cx: side, cy: side }, flags).ok()?;
            let image = pixels(bitmap);
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            image
        }
    }

    /// Copies a bitmap's pixels out as top-down RGBA.
    unsafe fn pixels(bitmap: HBITMAP) -> Option<Rgba8> {
        let mut info = BITMAP::default();
        let read = unsafe {
            GetObjectW(
                HGDIOBJ(bitmap.0),
                size_of::<BITMAP>() as i32,
                Some((&raw mut info).cast()),
            )
        };
        if read == 0 || info.bmWidth <= 0 || info.bmHeight == 0 {
            return None;
        }
        let (width, height) = (info.bmWidth, info.bmHeight.abs());
        let mut header = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // negative: top-down rows
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bgra = vec![0u8; width as usize * height as usize * 4];
        let copied = unsafe {
            let context = CreateCompatibleDC(None);
            let lines = GetDIBits(
                context,
                bitmap,
                0,
                height as u32,
                Some(bgra.as_mut_ptr().cast()),
                &mut header,
                DIB_RGB_COLORS,
            );
            let _ = DeleteDC(context);
            lines
        };
        if copied != height {
            return None;
        }
        // BGRA to RGBA. Photo thumbnails are opaque; the shell often leaves
        // alpha at zero.
        for pixel in bgra.as_chunks_mut::<4>().0 {
            pixel.swap(0, 2);
            pixel[3] = 255;
        }
        Some(Rgba8::new(width as u32, height as u32, bgra))
    }
}

/// macOS (Quick Look) follows; until then cloud-only photos show a placeholder.
#[cfg(not(target_os = "windows"))]
mod platform {
    use std::path::Path;

    use crate::raster::Rgba8;

    pub fn thumbnail(_path: &Path, _size: u32) -> Option<Rgba8> {
        None
    }
}
