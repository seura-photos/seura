//! Synthetic libraries for benchmarks and tests (decision 0010's benchmark
//! gate): realistic shapes, generated deterministically, no files on disk.

use std::path::PathBuf;

use crate::index::{EditFileState, FileTime, Index, Photo, TagId};

/// The folder holding exactly 1,000 photos, for PRF-2.
pub const LARGE_FOLDER: &str = "Phone backup";

/// An index of `count` photos shaped like a family library: dated folders of
/// about 200 photos (plus one of 1,000), names like `IMG_4821.JPG`, a few
/// hundred tags in a hierarchy with 40% of photos tagged, a third rated, and
/// 15% edited.
pub fn synthetic(count: usize) -> Index {
    let mut index = Index::new();
    let mut random = Random(0x5EED_5EED);
    let root = PathBuf::from(if cfg!(windows) {
        r"C:\Users\Someone\Pictures"
    } else {
        "/Users/someone/Pictures"
    });
    index.roots.push(root.clone());
    let root_id = index.ensure_folder(&root, None);

    let tags: Vec<TagId> = (0..300)
        .map(|n| {
            let group = ["family", "friends", "places", "events", "pets", "nature"][n % 6];
            index.intern_tag(&format!("{group}/topic {n}"))
        })
        .collect();

    let large = index.ensure_folder(&root.join(LARGE_FOLDER), Some(root_id));
    let mut folder = large;
    for n in 0..count {
        if n >= 1_000 && (n - 1_000) % 200 == 0 {
            let year = 2010 + (n / 7_000) % 16;
            let month = 1 + (n / 600) % 12;
            let path = root
                .join(format!("{year}"))
                .join(format!("{year}-{month:02} Folder {n}"));
            let year_folder = index.ensure_folder(path.parent().unwrap(), Some(root_id));
            folder = index.ensure_folder(&path, Some(year_folder));
        }
        let tagged = random.below(10) < 4;
        let photo_tags: Box<[TagId]> = if tagged {
            (0..1 + random.below(3))
                .map(|_| tags[random.below(tags.len())])
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect()
        } else {
            Box::new([])
        };
        let rating = if random.below(3) == 0 {
            1 + random.below(5) as u8
        } else {
            0
        };
        let edited = random.below(100) < 15;
        let has_edit_file = edited || tagged || rating > 0;
        let taken = 1_262_304_000 + random.below(500_000_000) as i64;
        let name = match random.below(3) {
            0 => format!("IMG_{:04}.JPG", n % 10_000),
            1 => format!("IMG_{:04}.HEIC", n % 10_000),
            _ => format!("PXL_2023{:04}_{:09}.jpg", n % 1231, n),
        };
        index.insert_photo(Photo {
            folder,
            name: name.into(),
            size: 1_500_000 + random.below(4_000_000) as u64,
            modified: FileTime {
                secs: taken,
                nanos: 0,
            },
            fingerprint: has_edit_file.then_some([n as u8; 32]),
            dimensions: Some((4032, 3024)),
            taken: Some(taken),
            edit_file: if has_edit_file {
                EditFileState::Read { edited }
            } else {
                EditFileState::None
            },
            edit_file_stamp: has_edit_file.then_some((
                2_000,
                FileTime {
                    secs: taken,
                    nanos: 0,
                },
            )),
            rating,
            tags: photo_tags,
            placeholder: false,
        });
    }
    index
}

/// A small deterministic generator (xorshift), so every run measures the
/// same library.
struct Random(u64);

impl Random {
    fn below(&mut self, limit: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % limit as u64) as usize
    }
}
