//! The read-only `seura://` protocol: thumbnails for photos in the index,
//! and nothing else (decision 0009; technical design, section 10).
//!
//! `seura://localhost/thumb-<id>` (macOS) or `http://seura.localhost/thumb-<id>`
//! (Windows). A thumbnail that isn't made yet is a 404; the UI asks again
//! when the `ThumbnailReady` event arrives.

use std::borrow::Cow;

use seura_library::PhotoId;
use tauri::http::{Request, Response, StatusCode, header};
use tauri::{AppHandle, Manager, Runtime};

use crate::library::Library;

pub fn respond<R: Runtime>(
    app: &AppHandle<R>,
    request: &Request<Vec<u8>>,
) -> Response<Cow<'static, [u8]>> {
    thumbnail(app, request.uri().path()).unwrap_or_else(|| {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Cow::Borrowed(&[][..]))
            .expect("a static response builds")
    })
}

fn thumbnail<R: Runtime>(app: &AppHandle<R>, path: &str) -> Option<Response<Cow<'static, [u8]>>> {
    let id: u32 = path.strip_prefix("/thumb-")?.parse().ok()?;
    let library = app.try_state::<Library>()?;
    // Only photos in the index: never an arbitrary path.
    let job = library.index.read().ok()?.thumbnail_job(PhotoId::new(id))?;
    let bytes = std::fs::read(library.thumbnails.path(&job.key)).ok()?;
    Response::builder()
        .header(header::CONTENT_TYPE, "image/webp")
        .body(Cow::Owned(bytes))
        .ok()
}
