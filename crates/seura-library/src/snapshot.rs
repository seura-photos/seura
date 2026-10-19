//! The index snapshot: the whole index in one file in the cache folder,
//! loaded at launch before anything else so the grid appears at once (PRF-1),
//! written atomically (decision 0010; technical design, section 4).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::index::{Folder, Index, Photo};

/// The snapshot's file name in the cache folder.
pub const SNAPSHOT_FILE: &str = "index.bin";

/// Explains the cache folder to anyone who opens it (technical design, section 4).
const README_FILE: &str = "README.txt";
const README: &str = include_str!("../resources/cache-readme.txt");

/// Identifies the file, then its layout. A snapshot written by another
/// layout version is ignored, which only means a full re-scan.
const MAGIC: &[u8; 8] = b"SEURAIDX";
const LAYOUT_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct Stored {
    roots: Vec<PathBuf>,
    folders: Vec<Option<Folder>>,
    photos: Vec<Photo>,
    tags: Vec<Box<str>>,
}

impl Index {
    /// Writes the snapshot to `cache/index.bin`, creating the cache folder and
    /// its README if needed. Nothing is written anywhere else.
    pub fn save_snapshot(&self, cache: &Path) -> io::Result<()> {
        fs::create_dir_all(cache)?;
        let readme = cache.join(README_FILE);
        if !readme.exists() {
            seura_edit::write_atomically(&readme, README.as_bytes())?;
        }
        seura_edit::write_atomically(&cache.join(SNAPSHOT_FILE), &self.to_snapshot())
    }

    /// Loads the snapshot from `cache/index.bin`. `None` if there isn't one,
    /// or it's damaged or from another layout version: the caller then scans
    /// from scratch.
    pub fn load_snapshot(cache: &Path) -> Option<Index> {
        Index::from_snapshot(&fs::read(cache.join(SNAPSHOT_FILE)).ok()?)
    }

    /// The snapshot's bytes. Removed photos leave no gaps.
    pub fn to_snapshot(&self) -> Vec<u8> {
        let stored = Stored {
            roots: self.roots.clone(),
            folders: self.folders.clone(),
            photos: self.photos.iter().flatten().cloned().collect(),
            tags: self.tags.clone(),
        };
        let mut bytes = MAGIC.to_vec();
        bytes.extend(LAYOUT_VERSION.to_le_bytes());
        postcard::to_extend(&stored, bytes).expect("the index always serializes")
    }

    pub fn from_snapshot(bytes: &[u8]) -> Option<Index> {
        let rest = bytes.strip_prefix(MAGIC)?;
        let (version, rest) = rest.split_first_chunk::<4>()?;
        if u32::from_le_bytes(*version) != LAYOUT_VERSION {
            return None;
        }
        let stored: Stored = postcard::from_bytes(rest).ok()?;
        // A damaged file could still decode; reject references that don't exist.
        let folders = stored.folders.len();
        let tags = stored.tags.len();
        let consistent = stored.photos.iter().all(|photo| {
            stored
                .folders
                .get(photo.folder.0 as usize)
                .is_some_and(Option::is_some)
                && photo.tags.iter().all(|tag| (tag.0 as usize) < tags)
        }) && stored.folders.iter().flatten().all(|folder| {
            folder
                .parent
                .is_none_or(|parent| (parent.0 as usize) < folders)
        });
        consistent.then(|| {
            Index::from_parts(
                stored.roots,
                stored.folders,
                stored.photos.into_iter().map(Some).collect(),
                stored.tags,
            )
        })
    }
}
