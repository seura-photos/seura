//! The read-only `seura://` protocol: thumbnails for photos in the index,
//! the open photo's preview, and nothing else (decision 0009; technical
//! design, section 10).
//!
//! `seura://localhost/<path>` (macOS) or `http://seura.localhost/<path>`
//! (Windows), where the path is:
//!
//! - `thumb-<id>`: a thumbnail. One that isn't made yet is a 404; the UI asks
//!   again when the `ThumbnailReady` event arrives.
//! - `preview`: the photo open in the Editor, in the working space (see
//!   `editor::encode_linear`).
//! - `check-image`: the render check's reference image, in the same form.
//! - `view-<id>`: a photo decoded for the Viewer, in the same form (see
//!   `viewer`). Only photos the Viewer has decoded.
//! - `view-region`: the last part of the shown photo cut for zooming in, in
//!   the same form.

use std::borrow::Cow;

use seura_library::PhotoId;
use tauri::http::{Request, Response, StatusCode, header};
use tauri::{AppHandle, Manager, Runtime};

use crate::editor::{Editor, check_image_bytes};
use crate::library::Library;
use crate::viewer::Viewer;

pub fn respond<R: Runtime>(
    app: &AppHandle<R>,
    request: &Request<Vec<u8>>,
) -> Response<Cow<'static, [u8]>> {
    let path = request.uri().path();
    let response = match path {
        "/preview" => app
            .try_state::<Editor>()
            .and_then(|editor| editor.preview())
            .and_then(|bytes| pixels(bytes.as_ref().clone())),
        "/check-image" => pixels(check_image_bytes()),
        "/view-region" => app
            .try_state::<Viewer>()
            .and_then(|viewer| viewer.region_pixels())
            .and_then(|bytes| pixels(bytes.as_ref().clone())),
        _ if path.starts_with("/view-") => viewed(app, path),
        _ => thumbnail(app, path),
    };
    response.unwrap_or_else(|| {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Cow::Borrowed(&[][..]))
            .expect("a static response builds")
    })
}

/// Working-space pixels, fetched rather than shown as an image: the page's
/// origin differs from the protocol's, so the response says it may read it.
fn pixels(bytes: Vec<u8>) -> Option<Response<Cow<'static, [u8]>>> {
    Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(Cow::Owned(bytes))
        .ok()
}

fn viewed<R: Runtime>(app: &AppHandle<R>, path: &str) -> Option<Response<Cow<'static, [u8]>>> {
    let id: u32 = path.strip_prefix("/view-")?.parse().ok()?;
    let bytes = app.try_state::<Viewer>()?.pixels(id)?;
    pixels(bytes.as_ref().clone())
}

fn thumbnail<R: Runtime>(app: &AppHandle<R>, path: &str) -> Option<Response<Cow<'static, [u8]>>> {
    let id: u32 = path.strip_prefix("/thumb-")?.parse().ok()?;
    let library = app.try_state::<Library>()?;
    // Only photos in the index: never an arbitrary path.
    let job = library.index.read().ok()?.thumbnail_job(PhotoId::new(id))?;
    let bytes = std::fs::read(library.thumbnail_path(&job.key)).ok()?;
    Response::builder()
        .header(header::CONTENT_TYPE, "image/webp")
        .body(Cow::Owned(bytes))
        .ok()
}
