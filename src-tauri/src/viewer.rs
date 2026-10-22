//! The photo shown in the Viewer (BRW-5), and its neighbors in the grid's
//! order, decoded ahead so moving to the next one is quick (PRF-5).
//!
//! Each photo is decoded to fit the Viewer's area at the screen's density
//! (DSP-1), in the working space, and served by the `seura://` protocol as
//! `view-<id>` (technical design, section 6). At most the shown photo and
//! its two neighbors are kept. Cloud-only photos are never read here: that
//! would download them (LIB-9).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};

use seura_edit::{EditFile, edit_file_path};
use seura_render::Settings;

use crate::editor::encode_linear;

/// A photo's long edge is kept within this, whatever the window.
const MAX_LONG_EDGE: u32 = 8192;

/// A photo to show or decode ahead, as the index knows it.
#[derive(Clone, Debug)]
pub struct Wanted {
    pub id: u32,
    pub path: PathBuf,
    /// Upright pixel dimensions, if the index knows them.
    pub dimensions: Option<(u32, u32)>,
}

/// The shown photo, decoded.
#[derive(Clone, Debug, PartialEq)]
pub struct Shown {
    /// The decoded size, fetched from `seura://…/view-<id>`.
    pub width: u32,
    pub height: u32,
    /// Its edits, from its edit file; none if that can't be read.
    pub exposure_ev: f64,
}

#[derive(Default)]
pub struct Viewer {
    inner: Arc<Inner>,
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
    /// Width, height, and the bytes the protocol serves.
    Ready(u32, u32, Arc<Vec<u8>>),
    Failed(String),
}

impl Viewer {
    /// Shows `photo`, decoded to fit `area`, and starts decoding
    /// `neighbors` in the background; anything else is let go. Waits if
    /// `photo` is already being decoded ahead.
    pub fn show(
        &self,
        photo: &Wanted,
        neighbors: &[Wanted],
        area: (u32, u32),
    ) -> Result<Shown, String> {
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

        let (width, height) = loop {
            match cache.slots.get(&photo.id).cloned() {
                Some(Slot::Ready(width, height, _)) => break (width, height),
                Some(Slot::Failed(error)) => return Err(error),
                Some(Slot::Decoding) => {
                    cache = self.inner.decoded.wait(cache).expect("viewer lock");
                }
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
            width,
            height,
            exposure_ev: f64::from(edits(photo).exposure_ev),
        })
    }

    /// The bytes for `view-<id>`, if that photo is decoded.
    pub fn pixels(&self, id: u32) -> Option<Arc<Vec<u8>>> {
        let cache = self.inner.cache.lock().expect("viewer lock");
        match cache.slots.get(&id) {
            Some(Slot::Ready(_, _, bytes)) => Some(Arc::clone(bytes)),
            _ => None,
        }
    }

    /// The Viewer closed: its photos are let go.
    pub fn close(&self) {
        self.inner.cache.lock().expect("viewer lock").slots.clear();
    }
}

impl Inner {
    /// Decodes a photo and keeps it, if it's still wanted for this area
    /// when it's done. Only reads the photo (CLAUDE.md rule 1).
    fn decode(&self, photo: &Wanted, area: (u32, u32)) {
        let long_edge = long_edge(photo.dimensions, area);
        let slot = match seura_image::decode_linear(&photo.path, Some(long_edge)) {
            Ok(image) => Slot::Ready(image.width, image.height, Arc::new(encode_linear(&image))),
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

/// The long edge at which a photo fills `area` without being enlarged. A
/// photo whose size isn't known yet is decoded to the area's long edge.
fn long_edge(dimensions: Option<(u32, u32)>, area: (u32, u32)) -> u32 {
    let (area_width, area_height) = (area.0.max(1), area.1.max(1));
    let edge = match dimensions {
        Some((width, height)) if width > 0 && height > 0 => {
            let scale = (f64::from(area_width) / f64::from(width))
                .min(f64::from(area_height) / f64::from(height))
                .min(1.0);
            (f64::from(width.max(height)) * scale).round() as u32
        }
        _ => area_width.max(area_height),
    };
    edge.clamp(1, MAX_LONG_EDGE)
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

    fn wanted(folder: &Path, id: u32, dimensions: Option<(u32, u32)>) -> Wanted {
        Wanted {
            id,
            path: folder.join(format!("{id}.jpg")),
            dimensions,
        }
    }

    #[test]
    fn a_photo_is_decoded_to_fit_the_area_without_enlarging() {
        assert_eq!(long_edge(Some((4000, 3000)), (2000, 2000)), 2000);
        // A portrait photo in a wide area fits its height.
        assert_eq!(long_edge(Some((3000, 4000)), (2000, 1000)), 1000);
        assert_eq!(long_edge(Some((800, 600)), (2000, 2000)), 800);
        assert_eq!(long_edge(None, (1600, 900)), 1600);
        assert_eq!(
            long_edge(Some((40_000, 30_000)), (20_000, 20_000)),
            MAX_LONG_EDGE
        );
    }

    #[test]
    fn neighbors_are_decoded_ahead_and_others_let_go() {
        let folder = tempfile::tempdir().unwrap();
        for id in 1..=4 {
            jpeg(&folder.path().join(format!("{id}.jpg")), 400, 300);
        }
        let [one, two, three, four] =
            [1, 2, 3, 4].map(|id| wanted(folder.path(), id, Some((400, 300))));
        let viewer = Viewer::default();

        let shown = viewer
            .show(&two, &[one.clone(), three.clone()], (200, 200))
            .unwrap();
        assert_eq!((shown.width, shown.height), (200, 150));
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
        let one = wanted(folder.path(), 1, None);
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
                .show(&wanted(folder.path(), 1, None), &[], (100, 100))
                .is_err()
        );
    }
}
