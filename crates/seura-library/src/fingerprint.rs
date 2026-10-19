//! Fingerprints: SHA-256 of a photo's bytes, computed lazily and cached in
//! the index with the size and modification time they were computed for
//! (technical design, section 4; FIL-6, FIL-7).

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::index::{FileTime, Index, PhotoId};

/// Hashes a file, reading it once, opened read-only.
pub fn fingerprint_file(path: &Path) -> io::Result<[u8; 32]> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 256 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().into())
}

/// Lowercase hex, as edit files store it.
pub fn to_hex(fingerprint: &[u8; 32]) -> String {
    fingerprint
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl Index {
    /// The photo's fingerprint: cached if the file hasn't changed since it
    /// was computed, otherwise computed now and cached.
    pub fn fingerprint(&mut self, id: PhotoId) -> io::Result<[u8; 32]> {
        let path = self
            .path(id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such photo"))?;
        let before = std::fs::metadata(&path)?;
        let stamp = (before.len(), FileTime::from(before.modified()?));
        let photo = self.photo(id).expect("the path came from the photo");
        if let Some(fingerprint) = photo.fingerprint
            && (photo.size, photo.modified) == stamp
        {
            return Ok(fingerprint);
        }

        let fingerprint = fingerprint_file(&path)?;
        // Cache it only if the file didn't change while it was being read.
        let after = std::fs::metadata(&path)?;
        if (after.len(), FileTime::from(after.modified()?)) == stamp
            && let Some(photo) = self.photo_mut(id)
        {
            photo.size = stamp.0;
            photo.modified = stamp.1;
            photo.fingerprint = Some(fingerprint);
        }
        Ok(fingerprint)
    }
}
