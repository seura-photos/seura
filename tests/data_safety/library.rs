//! A temporary photo library, and snapshots of every file in it.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use seura_edit::PhotoIdentity;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

/// Folders of stand-in photos. Their bytes are generated, not real images:
/// only the edit files are under test here, and the repository holds no
/// photos it can't redistribute.
pub struct Library {
    dir: TempDir,
}

const PHOTOS: [&str; 5] = [
    "Summer/IMG_0001.jpg",
    "Summer/IMG_0001.heic",
    "Summer/beach.png",
    "Summer/Day 2/IMG_0002.JPG",
    "Winter/snow.jpeg",
];

impl Library {
    pub fn new() -> Library {
        let library = Library {
            dir: tempfile::tempdir().expect("can create a temporary folder"),
        };
        for name in PHOTOS {
            library.add_photo(name);
        }
        library
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    /// Adds a read-only stand-in photo, so anything opening it for writing fails.
    pub fn add_photo(&self, name: &str) -> PathBuf {
        let path = self.root().join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        // A real signature, so the scanner accepts it, then filler.
        let lower = name.to_lowercase();
        let signature: &[u8] = if lower.ends_with(".png") {
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
        } else if lower.ends_with(".heic") {
            &[
                0, 0, 0, 0x18, b'f', b't', b'y', b'p', b'h', b'e', b'i', b'c',
            ]
        } else {
            &[0xFF, 0xD8, 0xFF, 0xE1]
        };
        let mut bytes = signature.to_vec();
        bytes.extend(name.bytes().cycle().take(64 * 1024));
        fs::write(&path, bytes).unwrap();
        set_read_only(&path, true);
        path
    }

    /// Makes the system refuse to save `edit_file` until the guard is
    /// dropped. Windows: the edit file is read-only, so replacing it fails
    /// after the temporary file is written. Elsewhere: the folder is
    /// read-only, so no file can be created in it.
    pub fn block_writes(&self, edit_file: &Path) -> WriteBlock {
        #[cfg(windows)]
        let target = edit_file.to_path_buf();
        #[cfg(not(windows))]
        let target = edit_file.parent().unwrap().to_path_buf();
        set_read_only(&target, true);
        WriteBlock(target)
    }

    /// Adds a real, decodable PNG, read-only, so thumbnails get written.
    pub fn add_real_photo(&self, name: &str) -> PathBuf {
        let path = self.root().join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut bytes = Vec::new();
        let mut encoder = png::Encoder::new(&mut bytes, 640, 480);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&[90; 640 * 480 * 3]).unwrap();
        writer.finish().unwrap();
        fs::write(&path, bytes).unwrap();
        set_read_only(&path, true);
        path
    }

    pub fn photos(&self) -> Vec<PathBuf> {
        PHOTOS.iter().map(|name| self.root().join(name)).collect()
    }

    pub fn is_photo(&self, relative: &str) -> bool {
        PHOTOS.contains(&relative)
    }

    /// The path relative to the library, with forward slashes.
    pub fn relative(&self, path: &Path) -> String {
        relative(self.root(), path)
    }

    pub fn identity(&self, photo: &Path) -> PhotoIdentity {
        let bytes = fs::read(photo).unwrap();
        PhotoIdentity {
            file: photo.file_name().unwrap().to_string_lossy().into_owned(),
            size: bytes.len() as u64,
            sha256: Some(sha256(&bytes)),
            width: Some(4032),
            height: Some(3024),
        }
    }
}

impl Drop for Library {
    /// Read-only files can't be deleted on Windows, so the temporary folder
    /// couldn't be cleaned up without this.
    fn drop(&mut self) {
        for path in files(self.root()) {
            set_read_only(&path, false);
        }
    }
}

pub struct WriteBlock(PathBuf);

impl Drop for WriteBlock {
    fn drop(&mut self) {
        set_read_only(&self.0, false);
    }
}

/// An example file from the edit-format repository.
pub fn format_example(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../third_party/edit-format/examples")
        .join(name);
    fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Every file in a folder: its contents' hash, size, modification time, and
/// read-only flag.
#[derive(Debug)]
pub struct Snapshot(BTreeMap<String, FileState>);

#[derive(Debug, PartialEq, Eq)]
pub struct FileState {
    sha256: String,
    size: u64,
    modified: SystemTime,
    read_only: bool,
}

impl Snapshot {
    pub fn of(root: &Path) -> Snapshot {
        Snapshot(
            files(root)
                .into_iter()
                .map(|path| {
                    let metadata = fs::metadata(&path).unwrap();
                    let state = FileState {
                        sha256: sha256(&fs::read(&path).unwrap()),
                        size: metadata.len(),
                        modified: metadata.modified().unwrap(),
                        read_only: metadata.permissions().readonly(),
                    };
                    (relative(root, &path), state)
                })
                .collect(),
        )
    }

    pub fn get(&self, relative: &str) -> Option<&FileState> {
        self.0.get(relative)
    }

    pub fn paths(&self) -> impl Iterator<Item = String> + '_ {
        self.0.keys().cloned()
    }

    pub fn added_since(&self, before: &Snapshot) -> Vec<String> {
        self.paths().filter(|p| !before.0.contains_key(p)).collect()
    }

    pub fn removed_since(&self, before: &Snapshot) -> Vec<String> {
        before.added_since(self)
    }

    pub fn changed_since(&self, before: &Snapshot) -> Vec<String> {
        self.0
            .iter()
            .filter(|(path, state)| before.0.get(*path).is_some_and(|old| old != *state))
            .map(|(path, _)| path.clone())
            .collect()
    }
}

fn files(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut folders = vec![root.to_path_buf()];
    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                folders.push(path);
            } else {
                found.push(path);
            }
        }
    }
    found
}

fn relative(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).expect("inside the library");
    relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn set_read_only(path: &Path, read_only: bool) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let writable = if path.is_dir() { 0o755 } else { 0o644 };
        permissions.set_mode(if read_only {
            writable & 0o555
        } else {
            writable
        });
    }
    #[cfg(not(unix))]
    permissions.set_readonly(read_only);
    fs::set_permissions(path, permissions).unwrap();
}
