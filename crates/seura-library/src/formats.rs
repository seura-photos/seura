//! Which files are photos: JPEG, HEIC/HEIF, and PNG, recognized by
//! extension and confirmed by their first bytes (LIB-8). Everything else,
//! including a Live Photo's paired video, is ignored silently.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Jpeg,
    Png,
    Heif,
}

impl Format {
    /// The format a file name's extension claims, if it's one Seura shows.
    pub fn from_name(name: &str) -> Option<Format> {
        let (_, extension) = name.rsplit_once('.')?;
        match extension.to_ascii_lowercase().as_str() {
            "jpg" | "jpeg" => Some(Format::Jpeg),
            "png" => Some(Format::Png),
            "heic" | "heif" => Some(Format::Heif),
            _ => None,
        }
    }

    /// Whether a file's first bytes are this format's signature.
    pub fn matches(self, head: &[u8]) -> bool {
        match self {
            Format::Jpeg => head.starts_with(&[0xFF, 0xD8, 0xFF]),
            Format::Png => head.starts_with(b"\x89PNG\r\n\x1a\n"),
            // An ISO base media file: a `ftyp` box whose major brand is HEIF.
            Format::Heif => {
                head.len() >= 12
                    && &head[4..8] == b"ftyp"
                    && HEIF_BRANDS.iter().any(|brand| &head[8..12] == *brand)
            }
        }
    }
}

const HEIF_BRANDS: [&[u8; 4]; 8] = [
    b"heic", b"heix", b"hevc", b"hevx", b"heim", b"heis", b"mif1", b"msf1",
];

/// Reads enough of a file to check its signature. Opens it read-only.
pub(crate) fn read_head(path: &Path) -> io::Result<Vec<u8>> {
    let mut head = Vec::with_capacity(12);
    File::open(path)?.take(12).read_to_end(&mut head)?;
    Ok(head)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extensions_are_matched_without_regard_to_case() {
        assert_eq!(Format::from_name("IMG_0001.JPG"), Some(Format::Jpeg));
        assert_eq!(Format::from_name("a.jpeg"), Some(Format::Jpeg));
        assert_eq!(Format::from_name("a.HEIC"), Some(Format::Heif));
        assert_eq!(Format::from_name("a.png"), Some(Format::Png));
        // A Live Photo's video, an edit file, and files with no extension.
        assert_eq!(Format::from_name("IMG_0001.MOV"), None);
        assert_eq!(Format::from_name("IMG_0001.jpg.seura.json"), None);
        assert_eq!(Format::from_name("jpg"), None);
    }

    #[test]
    fn signatures_are_checked() {
        assert!(Format::Jpeg.matches(&[0xFF, 0xD8, 0xFF, 0xE1]));
        assert!(!Format::Jpeg.matches(b"\x89PNG\r\n\x1a\n"));
        assert!(Format::Png.matches(b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"));
        assert!(Format::Heif.matches(b"\0\0\0\x18ftypheic\0\0\0\0"));
        assert!(!Format::Heif.matches(b"\0\0\0\x18ftypqt  \0\0\0\0"));
        assert!(!Format::Heif.matches(b"\0\0"));
    }
}
