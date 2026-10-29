//! The photo shown in the Viewer (BRW-5), and its neighbors in the grid's
//! order, decoded ahead so moving to the next one is quick (PRF-5).
//!
//! Each photo is decoded to fit the Viewer's area at the screen's density
//! (DSP-1), in the working space, and served by the `seura://` protocol as
//! `view-<id>` (technical design, section 6). At most the shown photo and
//! its two neighbors are kept. Zoomed in, the part on screen is cut from the
//! photo decoded at full size, which is kept while that photo is shown, and
//! served as `view-region`. Cloud-only photos are read only when the user
//! opens one, which downloads it, and never decoded ahead (LIB-9).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};

use seura_edit::{EditFile, edit_file_path};
use seura_image::{Region, Upright};
use seura_render::Settings;

use crate::editor::encode_linear;

/// Why a photo wasn't shown: the Viewer moved on before it was decoded.
pub const MOVED_ON: &str = "the Viewer moved on";

/// A decoded image's long edge is kept within this, whatever the window.
const MAX_LONG_EDGE: u32 = 8192;

/// A photo to show or decode ahead, as the index knows it.
#[derive(Clone, Debug)]
pub struct Wanted {
    pub id: u32,
    pub path: PathBuf,
}

/// The shown photo, decoded.
#[derive(Clone, Debug, PartialEq)]
pub struct Shown {
    /// The decoded size, fetched from `seura://…/view-<id>`.
    pub width: u32,
    pub height: u32,
    /// The photo's own upright size, for zooming.
    pub full_width: u32,
    pub full_height: u32,
    /// Its edits, from its edit file; none if that can't be read.
    pub settings: Settings,
}

#[derive(Default)]
pub struct Viewer {
    inner: Arc<Inner>,
    /// The shown photo at full size, while it's zoomed into.
    full: Mutex<Option<(u32, Arc<Upright>)>>,
    /// The last part cut from it, for the protocol.
    region: Mutex<Option<Arc<Vec<u8>>>>,
}

#[derive(Default)]
struct Inner {
    cache: Mutex<Cache>,
    /// Signalled whenever a decode finishes.
    decoded: Condvar,
}

#[derive(Default)]
struct Cache {
    /// The Viewer's area in device pixels; a new size means new decodes.
    area: (u32, u32),
    slots: HashMap<u32, Slot>,
}

#[derive(Clone)]
enum Slot {
    Decoding,
    Ready(Decoded),
    Failed(String),
}

#[derive(Clone)]
struct Decoded {
    size: (u32, u32),
    full: (u32, u32),
    /// What the protocol serves.
    bytes: Arc<Vec<u8>>,
}

impl Viewer {
    /// Shows `photo`, decoded to fit `area`, and starts decoding
    /// `neighbors` in the background; anything else is let go. Waits if
    /// `photo` is already being decoded ahead, and gives up if the Viewer
    /// moves on meanwhile, rather than reading it again.
    pub fn show(
        &self,
        photo: &Wanted,
        neighbors: &[Wanted],
        area: (u32, u32),
    ) -> Result<Shown, String> {
        let mut full = self.full.lock().expect("viewer lock");
        if full.as_ref().is_some_and(|(id, _)| *id != photo.id) {
            *full = None;
        }
        drop(full);

        let mut cache = self.inner.cache.lock().expect("viewer lock");
        if cache.area != area {
            cache.area = area;
            cache.slots.clear();
        }
        cache
            .slots
            .retain(|id, _| *id == photo.id || neighbors.iter().any(|neighbor| neighbor.id == *id));
        for neighbor in neighbors {
            if let std::collections::hash_map::Entry::Vacant(slot) = cache.slots.entry(neighbor.id)
            {
                slot.insert(Slot::Decoding);
                let inner = Arc::clone(&self.inner);
                let neighbor = neighbor.clone();
                std::thread::spawn(move || inner.decode(&neighbor, area));
            }
        }

        // Asking for a photo that failed tries it again, rather than repeating
        // the failure: a download the user cancelled, or a drive that's back,
        // may well work now (Try Again; LIB-9). Neighbors that failed are
        // tried again when they're shown.
        if matches!(cache.slots.get(&photo.id), Some(Slot::Failed(_))) {
            cache.slots.remove(&photo.id);
        }
        let mut waited = false;
        let decoded = loop {
            match cache.slots.get(&photo.id).cloned() {
                Some(Slot::Ready(decoded)) => break decoded,
                Some(Slot::Failed(error)) => return Err(error),
                Some(Slot::Decoding) => {
                    cache = self.inner.decoded.wait(cache).expect("viewer lock");
                    waited = true;
                }
                None if waited => return Err(MOVED_ON.into()),
                None => {
                    cache.slots.insert(photo.id, Slot::Decoding);
                    drop(cache);
                    self.inner.decode(photo, area);
                    cache = self.inner.cache.lock().expect("viewer lock");
                }
            }
        };
        drop(cache);
        Ok(Shown {
            width: decoded.size.0,
            height: decoded.size.1,
            full_width: decoded.full.0,
            full_height: decoded.full.1,
            settings: edits(photo),
        })
    }

    /// The bytes for `view-<id>`, if that photo is decoded.
    pub fn pixels(&self, id: u32) -> Option<Arc<Vec<u8>>> {
        let cache = self.inner.cache.lock().expect("viewer lock");
        match cache.slots.get(&id) {
            Some(Slot::Ready(decoded)) => Some(Arc::clone(&decoded.bytes)),
            _ => None,
        }
    }

    /// Cuts `region` (in the photo's own pixels) from `photo` at full size,
    /// scaled to `width` × `height`, never larger than the region itself
    /// (the Viewer enlarges it), and keeps it for `view-region`. Returns its
    /// size. The first part of a photo decodes it at full size.
    pub fn region(
        &self,
        photo: &Wanted,
        region: Region,
        width: u32,
        height: u32,
    ) -> Result<(u32, u32), String> {
        let kept = self
            .full
            .lock()
            .expect("viewer lock")
            .as_ref()
            .filter(|(id, _)| *id == photo.id)
            .map(|(_, full)| Arc::clone(full));
        let full = match kept {
            Some(full) => full,
            None => {
                let full = Arc::new(
                    seura_image::decode_upright(&photo.path).map_err(|error| error.to_string())?,
                );
                *self.full.lock().expect("viewer lock") = Some((photo.id, Arc::clone(&full)));
                full
            }
        };
        let width = width.clamp(1, region.width.max(1)).min(MAX_LONG_EDGE);
        let height = height.clamp(1, region.height.max(1)).min(MAX_LONG_EDGE);
        let image = full.region_linear(region, width, height);
        let size = (image.width, image.height);
        *self.region.lock().expect("viewer lock") = Some(Arc::new(encode_linear(&image)));
        Ok(size)
    }

    /// The bytes for `view-region`: the last part cut.
    pub fn region_pixels(&self) -> Option<Arc<Vec<u8>>> {
        self.region.lock().expect("viewer lock").clone()
    }

    /// The Viewer closed: its photos are let go.
    pub fn close(&self) {
        self.inner.cache.lock().expect("viewer lock").slots.clear();
        *self.full.lock().expect("viewer lock") = None;
        *self.region.lock().expect("viewer lock") = None;
    }
}

impl Inner {
    /// Decodes a photo and keeps it, if it's still wanted for this area
    /// when it's done. Only reads the photo (CLAUDE.md rule 1).
    fn decode(&self, photo: &Wanted, area: (u32, u32)) {
        let slot = match seura_image::decode_upright(&photo.path) {
            Ok(upright) => {
                let full = (upright.width(), upright.height());
                let size = fitted(full, area);
                let whole = Region {
                    x: 0,
                    y: 0,
                    width: full.0,
                    height: full.1,
                };
                let image = upright.region_linear(whole, size.0, size.1);
                Slot::Ready(Decoded {
                    size,
                    full,
                    bytes: Arc::new(encode_linear(&image)),
                })
            }
            Err(error) => Slot::Failed(error.to_string()),
        };
        let mut cache = self.cache.lock().expect("viewer lock");
        if cache.area == area
            && let Some(wanted) = cache.slots.get_mut(&photo.id)
            && matches!(wanted, Slot::Decoding)
        {
            *wanted = slot;
        }
        drop(cache);
        self.decoded.notify_all();
    }
}

/// The size at which a photo of `full` size fills `area` without being
/// enlarged, keeping its shape.
fn fitted(full: (u32, u32), area: (u32, u32)) -> (u32, u32) {
    let (width, height) = (f64::from(full.0.max(1)), f64::from(full.1.max(1)));
    let scale = (f64::from(area.0.max(1)) / width)
        .min(f64::from(area.1.max(1)) / height)
        .min(f64::from(MAX_LONG_EDGE) / width.max(height))
        .min(1.0);
    let size = |side: f64| ((side * scale).round() as u32).max(1);
    (size(width), size(height))
}

/// A photo's edits, from its edit file. One that can't be read shows the
/// photo as it is; the grid already marks it (FIL-4).
fn edits(photo: &Wanted) -> Settings {
    match EditFile::load(&edit_file_path(&photo.path)) {
        Ok(Some(file)) => Settings::from_document(&file.document),
        _ => Settings::default(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;

    fn jpeg(path: &Path, width: u16, height: u16) {
        let mut bytes = Vec::new();
        jpeg_encoder::Encoder::new(&mut bytes, 90)
            .encode(
                &[90, 120, 150].repeat(usize::from(width) * usize::from(height)),
                width,
                height,
                jpeg_encoder::ColorType::Rgb,
            )
            .unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn wanted(folder: &Path, id: u32) -> Wanted {
        Wanted {
            id,
            path: folder.join(format!("{id}.jpg")),
        }
    }

    #[test]
    fn a_photo_is_decoded_to_fit_the_area_without_enlarging() {
        assert_eq!(fitted((4000, 3000), (2000, 2000)), (2000, 1500));
        // A portrait photo in a wide area fits its height.
        assert_eq!(fitted((3000, 4000), (2000, 1000)), (750, 1000));
        assert_eq!(fitted((800, 600), (2000, 2000)), (800, 600));
        assert_eq!(
            fitted((40_000, 30_000), (20_000, 20_000)),
            (MAX_LONG_EDGE, 6144)
        );
    }

    #[test]
    fn neighbors_are_decoded_ahead_and_others_let_go() {
        let folder = tempfile::tempdir().unwrap();
        for id in 1..=4 {
            jpeg(&folder.path().join(format!("{id}.jpg")), 400, 300);
        }
        let [one, two, three, four] = [1, 2, 3, 4].map(|id| wanted(folder.path(), id));
        let viewer = Viewer::default();

        let shown = viewer
            .show(&two, &[one.clone(), three.clone()], (200, 200))
            .unwrap();
        assert_eq!((shown.width, shown.height), (200, 150));
        assert_eq!((shown.full_width, shown.full_height), (400, 300));
        assert!(viewer.pixels(2).is_some());

        // Moving on finds 3 decoded, or waits for it; 1 is let go.
        let shown = viewer
            .show(&three, &[two.clone(), four.clone()], (200, 200))
            .unwrap();
        assert_eq!(shown.width, 200);
        assert!(viewer.pixels(1).is_none());
        // The protocol's bytes: width and height, then half floats.
        let bytes = viewer.pixels(3).unwrap();
        assert_eq!(&bytes[..8], &[200, 0, 0, 0, 150, 0, 0, 0]);
        assert_eq!(bytes.len(), 8 + 200 * 150 * 4 * 2);

        viewer.close();
        assert!(viewer.pixels(3).is_none());
    }

    #[test]
    fn a_new_area_means_a_new_decode() {
        let folder = tempfile::tempdir().unwrap();
        jpeg(&folder.path().join("1.jpg"), 400, 300);
        let one = wanted(folder.path(), 1);
        let viewer = Viewer::default();
        assert_eq!(viewer.show(&one, &[], (100, 100)).unwrap().width, 100);
        assert_eq!(viewer.show(&one, &[], (300, 300)).unwrap().width, 300);
    }

    #[test]
    fn a_photo_that_cant_be_decoded_is_an_error() {
        let folder = tempfile::tempdir().unwrap();
        fs::write(folder.path().join("1.jpg"), b"not a photo").unwrap();
        let viewer = Viewer::default();
        assert!(
            viewer
                .show(&wanted(folder.path(), 1), &[], (100, 100))
                .is_err()
        );
    }

    /// Try Again after a failure reads the photo again: found by Nevada,
    /// whose cancelled OneDrive download kept failing until the Viewer was
    /// closed (LIB-9).
    #[test]
    fn a_photo_that_failed_is_tried_again() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("1.jpg");
        fs::write(&path, b"not yet a photo").unwrap();
        let viewer = Viewer::default();
        let one = wanted(folder.path(), 1);
        assert!(viewer.show(&one, &[], (100, 100)).is_err());

        // The download finishes, or the drive comes back.
        jpeg(&path, 400, 300);

        assert_eq!(viewer.show(&one, &[], (100, 100)).unwrap().width, 100);
    }

    /// A request waiting for a photo being decoded ahead gives up when the
    /// Viewer moves on, rather than reading it itself: for a cloud-only
    /// photo, that read would download it again (LIB-9).
    #[test]
    fn waiting_for_a_photo_the_viewer_moved_on_from_gives_up() {
        let folder = tempfile::tempdir().unwrap();
        let viewer = Arc::new(Viewer::default());
        {
            let mut cache = viewer.inner.cache.lock().unwrap();
            cache.area = (100, 100);
            cache.slots.insert(1, Slot::Decoding);
        }
        let waiting = {
            let viewer = Arc::clone(&viewer);
            let one = wanted(folder.path(), 1);
            std::thread::spawn(move || viewer.show(&one, &[], (100, 100)))
        };
        // Let it start waiting, then move on: its slot goes.
        std::thread::sleep(std::time::Duration::from_millis(100));
        viewer.inner.cache.lock().unwrap().slots.clear();
        viewer.inner.decoded.notify_all();

        assert_eq!(waiting.join().unwrap(), Err(MOVED_ON.to_string()));
    }

    #[test]
    fn zoomed_in_a_part_is_cut_at_full_size_never_enlarged() {
        let folder = tempfile::tempdir().unwrap();
        jpeg(&folder.path().join("1.jpg"), 400, 300);
        let one = wanted(folder.path(), 1);
        let viewer = Viewer::default();
        viewer.show(&one, &[], (100, 100)).unwrap();
        let part = Region {
            x: 100,
            y: 50,
            width: 120,
            height: 80,
        };
        // At 100%: the part's own pixels.
        assert_eq!(viewer.region(&one, part, 120, 80).unwrap(), (120, 80));
        // At 50%: scaled down.
        assert_eq!(viewer.region(&one, part, 60, 40).unwrap(), (60, 40));
        // At 200%: still its own pixels; the Viewer enlarges them.
        assert_eq!(viewer.region(&one, part, 240, 160).unwrap(), (120, 80));
        let bytes = viewer.region_pixels().unwrap();
        assert_eq!(&bytes[..8], &[120, 0, 0, 0, 80, 0, 0, 0]);
        // Moving to another photo lets the full-size one go.
        jpeg(&folder.path().join("2.jpg"), 400, 300);
        viewer
            .show(&wanted(folder.path(), 2), &[], (100, 100))
            .unwrap();
        assert!(viewer.full.lock().unwrap().is_none());
    }
}
