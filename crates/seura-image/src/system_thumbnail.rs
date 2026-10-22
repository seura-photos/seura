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

#[cfg(target_os = "macos")]
mod platform {
    #![allow(
        unsafe_code,
        reason = "Quick Look and Core Graphics are Objective-C and C APIs"
    )]

    use std::path::Path;
    use std::sync::mpsc;
    use std::time::Duration;

    use block2::RcBlock;
    use objc2::AllocAnyThread;
    use objc2::rc::Retained;
    use objc2_core_foundation::{CGFloat, CGPoint, CGRect, CGSize};
    use objc2_core_graphics::{
        CGBitmapContextCreate, CGColorSpace, CGContext, CGImage, CGImageAlphaInfo,
        kCGColorSpaceSRGB,
    };
    use objc2_foundation::{NSError, NSString, NSURL};
    use objc2_quick_look_thumbnailing::{
        QLThumbnailGenerationRequest, QLThumbnailGenerationRequestRepresentationTypes,
        QLThumbnailGenerator, QLThumbnailRepresentation, QLThumbnailRepresentationType,
    };

    use crate::raster::Rgba8;

    /// How long to wait for Quick Look before showing the placeholder.
    const WAIT: Duration = Duration::from_secs(10);

    /// Quick Look's thumbnail, which for a cloud-only file comes from the
    /// sync service's File Provider, as Finder shows it.
    pub fn thumbnail(path: &Path, size: u32) -> Option<Rgba8> {
        let url = NSURL::fileURLWithPath(&NSString::from_str(path.to_str()?));
        let side = CGFloat::from(size as f32);
        // A thumbnail only: never the file type's icon instead.
        let request = unsafe {
            QLThumbnailGenerationRequest::initWithFileAtURL_size_scale_representationTypes(
                QLThumbnailGenerationRequest::alloc(),
                &url,
                CGSize {
                    width: side,
                    height: side,
                },
                1.0,
                QLThumbnailGenerationRequestRepresentationTypes::Thumbnail,
            )
        };
        let (sender, receiver) = mpsc::channel();
        let handler = RcBlock::new(
            move |representation: *mut QLThumbnailRepresentation, _error: *mut NSError| {
                // Quick Look owns the representation for the handler's call.
                let pixels = unsafe { representation.as_ref() }
                    .filter(|representation| unsafe {
                        representation.r#type() == QLThumbnailRepresentationType::Thumbnail
                    })
                    .and_then(|representation| {
                        let image = unsafe { representation.CGImage() };
                        // The binding wraps a Core Graphics image as an
                        // Objective-C one; it's borrowed while `image` holds it.
                        pixels(unsafe { &*Retained::as_ptr(&image) })
                    });
                let _ = sender.send(pixels);
            },
        );
        let generator = unsafe { QLThumbnailGenerator::sharedGenerator() };
        unsafe {
            generator.generateBestRepresentationForRequest_completionHandler(&request, &handler)
        };
        match receiver.recv_timeout(WAIT) {
            Ok(pixels) => pixels,
            Err(_) => {
                unsafe { generator.cancelRequest(&request) };
                None
            }
        }
    }

    /// Draws an image into an sRGB bitmap, as top-down RGBA.
    fn pixels(image: &CGImage) -> Option<Rgba8> {
        let width = CGImage::width(Some(image));
        let height = CGImage::height(Some(image));
        if width == 0 || height == 0 {
            return None;
        }
        let srgb = CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB }))?;
        let mut pixels = vec![0u8; width * height * 4];
        {
            let context: Option<_> = unsafe {
                CGBitmapContextCreate(
                    pixels.as_mut_ptr().cast(),
                    width,
                    height,
                    8,
                    width * 4,
                    Some(&srgb),
                    CGImageAlphaInfo::PremultipliedLast.0,
                )
            };
            let bounds = CGRect {
                origin: CGPoint { x: 0.0, y: 0.0 },
                size: CGSize {
                    width: width as CGFloat,
                    height: height as CGFloat,
                },
            };
            let context = context?;
            CGContext::draw_image(Some(&context), bounds, Some(image));
            // The context writes into `pixels`; it's dropped here, before they're used.
        }
        // Photo thumbnails are opaque.
        for pixel in pixels.as_chunks_mut::<4>().0 {
            pixel[3] = 255;
        }
        Some(Rgba8::new(width as u32, height as u32, pixels))
    }
}

/// Other platforms aren't supported; cloud-only photos show a placeholder.
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod platform {
    use std::path::Path;

    use crate::raster::Rgba8;

    pub fn thumbnail(_path: &Path, _size: u32) -> Option<Rgba8> {
        None
    }
}
