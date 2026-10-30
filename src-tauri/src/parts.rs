//! The sharp part of a zoomed-in photo, for the Viewer (BRW-5) and the
//! Editor (EDT-1): cut from the photo decoded at full size, which is kept
//! while that photo is zoomed into, and served by the `seura://` protocol
//! (`view-region` and `preview-region`). Only reads the photo (CLAUDE.md
//! rule 1).

use std::path::Path;
use std::sync::{Arc, Mutex};

use seura_image::{Region, Upright};

use crate::editor::encode_linear;

/// A part's long edge is kept within this, whatever the window.
const MAX_LONG_EDGE: u32 = 8192;

#[derive(Default)]
pub struct Parts {
    /// The photo at full size, by id, once a part of it has been cut.
    full: Mutex<Option<(u32, Arc<Upright>)>>,
    /// The last part cut, for the protocol.
    last: Mutex<Option<Arc<Vec<u8>>>>,
}

impl Parts {
    /// Cuts `region` (in the photo's own pixels) from photo `id`, at `path`,
    /// scaled to `width` × `height`, never larger than the region itself
    /// (the screen enlarges it), and keeps it for the protocol. Returns its
    /// size. The first part of a photo decodes it at full size.
    pub fn cut(
        &self,
        id: u32,
        path: &Path,
        region: Region,
        width: u32,
        height: u32,
    ) -> Result<(u32, u32), String> {
        let kept = self
            .full
            .lock()
            .expect("parts lock")
            .as_ref()
            .filter(|(kept, _)| *kept == id)
            .map(|(_, full)| Arc::clone(full));
        let full = match kept {
            Some(full) => full,
            None => {
                let full =
                    Arc::new(seura_image::decode_upright(path).map_err(|error| error.to_string())?);
                *self.full.lock().expect("parts lock") = Some((id, Arc::clone(&full)));
                full
            }
        };
        let width = width.clamp(1, region.width.max(1)).min(MAX_LONG_EDGE);
        let height = height.clamp(1, region.height.max(1)).min(MAX_LONG_EDGE);
        let image = full.region_linear(region, width, height);
        let size = (image.width, image.height);
        *self.last.lock().expect("parts lock") = Some(Arc::new(encode_linear(&image)));
        Ok(size)
    }

    /// The bytes of the last part cut.
    pub fn pixels(&self) -> Option<Arc<Vec<u8>>> {
        self.last.lock().expect("parts lock").clone()
    }

    /// Lets the full-size photo go unless it's photo `id`.
    pub fn keep_only(&self, id: u32) {
        let mut full = self.full.lock().expect("parts lock");
        if full.as_ref().is_some_and(|(kept, _)| *kept != id) {
            *full = None;
        }
    }

    /// Which photo is kept at full size, if any.
    #[cfg(test)]
    pub fn kept(&self) -> Option<u32> {
        self.full
            .lock()
            .expect("parts lock")
            .as_ref()
            .map(|(id, _)| *id)
    }

    /// Lets everything go.
    pub fn clear(&self) {
        *self.full.lock().expect("parts lock") = None;
        *self.last.lock().expect("parts lock") = None;
    }
}
