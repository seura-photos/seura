//! The in-memory index (decision 0010): one record per photo, plus maps by
//! folder, tag, and name for browsing, filtering, and search.
//!
//! The index is a cache. The photos and their edit files are the source of
//! truth, and a missing or damaged snapshot only means a full re-scan.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PhotoId(pub(crate) u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FolderId(pub(crate) u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TagId(pub(crate) u32);

/// Ids as plain numbers, for the UI. An id that doesn't exist just finds
/// nothing: every lookup checks.
macro_rules! numeric_id {
    ($($id:ident),*) => {$(
        impl $id {
            pub fn new(value: u32) -> $id {
                $id(value)
            }

            pub fn value(self) -> u32 {
                self.0
            }
        }
    )*};
}
numeric_id!(PhotoId, FolderId, TagId);

/// A file's modification time, to the nanosecond, as seconds and
/// nanoseconds since 1970 (negative seconds before).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileTime {
    pub secs: i64,
    pub nanos: u32,
}

impl From<SystemTime> for FileTime {
    fn from(time: SystemTime) -> FileTime {
        match time.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(after) => FileTime {
                secs: after.as_secs() as i64,
                nanos: after.subsec_nanos(),
            },
            Err(before) => {
                let before = before.duration();
                let (secs, nanos) = match before.subsec_nanos() {
                    0 => (-(before.as_secs() as i64), 0),
                    n => (-(before.as_secs() as i64) - 1, 1_000_000_000 - n),
                };
                FileTime { secs, nanos }
            }
        }
    }
}

/// A photo's edit file, as last read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditFileState {
    #[default]
    None,
    /// Read successfully; `edited` if it holds any operations.
    Read { edited: bool },
    /// Present but unreadable: the photo is shown unedited with a warning,
    /// and the file is never overwritten (FIL-4).
    Unreadable,
}

/// One photo. About 100 bytes plus its name and tags; see the benchmarks.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Photo {
    pub folder: FolderId,
    pub name: Box<str>,
    pub size: u64,
    pub modified: FileTime,
    /// SHA-256 of the file, computed lazily, valid for `size` and `modified`.
    pub fingerprint: Option<[u8; 32]>,
    /// Pixel dimensions after orientation, once decoded (slice 4).
    pub dimensions: Option<(u32, u32)>,
    /// Date taken, seconds since 1970, from EXIF once read (slice 4).
    pub taken: Option<i64>,
    pub edit_file: EditFileState,
    /// The edit file's size and modification time when last read, so it's
    /// re-read only when it changes.
    pub edit_file_stamp: Option<(u64, FileTime)>,
    pub rating: u8,
    pub tags: Box<[TagId]>,
    /// The file's contents are in the cloud, not on disk.
    pub placeholder: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Folder {
    pub path: PathBuf,
    pub parent: Option<FolderId>,
    /// Seura can't add or change files here, so its photos can't be tagged
    /// or rated (TAG-7). Found again by every scan, so not in the snapshot.
    #[serde(skip)]
    pub read_only: bool,
}

/// What adding a library folder did (LIB-1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AddFolder {
    Added,
    /// The folder is already in the library, itself or inside this one.
    AlreadyIncluded(PathBuf),
    /// The folder was added and replaces these library folders inside it.
    Absorbed(Vec<PathBuf>),
}

/// Where to look (BRW-1, TAG-5, FND-2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// The folder's own photos, not its subfolders.
    Folder(FolderId),
    /// Photos with exactly this tag.
    Tag(TagId),
    All,
}

/// Narrowing what a scope shows (FND-1).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filter {
    pub min_rating: u8,
    /// A photo must have all of these.
    pub tags: Vec<TagId>,
    pub edited: Option<bool>,
    /// Inclusive, seconds since 1970. A photo whose date taken isn't known
    /// (cloud-only, not decoded yet, or none recorded) is placed by its
    /// file date instead, as it sorts (BRW-2, LIB-9).
    pub taken: Option<(i64, i64)>,
}

#[derive(Debug, Default)]
pub struct Index {
    pub(crate) roots: Vec<PathBuf>,
    pub(crate) folders: Vec<Option<Folder>>,
    pub(crate) photos: Vec<Option<Photo>>,
    pub(crate) tags: Vec<Box<str>>,
    // Derived, rebuilt after loading a snapshot.
    folder_ids: HashMap<PathBuf, FolderId>,
    by_folder: Vec<Vec<PhotoId>>,
    by_name: HashMap<(FolderId, Box<str>), PhotoId>,
    by_tag: Vec<Vec<PhotoId>>,
    tag_ids: HashMap<Box<str>, TagId>,
    /// Names and tags by [`seura_edit::tag_key`], for search (FND-2).
    search_names: Vec<Box<str>>,
    search_tags: Vec<Box<str>>,
    free_photos: Vec<PhotoId>,
}

impl Index {
    pub fn new() -> Index {
        Index::default()
    }

    /// Rebuilds an index from its stored parts.
    pub(crate) fn from_parts(
        roots: Vec<PathBuf>,
        folders: Vec<Option<Folder>>,
        photos: Vec<Option<Photo>>,
        tags: Vec<Box<str>>,
    ) -> Index {
        let mut index = Index {
            roots,
            ..Index::default()
        };
        for tag in tags {
            index.intern_tag(&tag);
        }
        for (id, folder) in folders.into_iter().enumerate() {
            if let Some(folder) = &folder {
                index
                    .folder_ids
                    .insert(folder.path.clone(), FolderId(id as u32));
            }
            index.folders.push(folder);
            index.by_folder.push(Vec::new());
        }
        for photo in photos.into_iter().flatten() {
            index.insert_photo(photo);
        }
        index
    }

    /// An empty index of these library folders, to be scanned.
    pub fn with_roots(roots: Vec<PathBuf>) -> Index {
        Index {
            roots,
            ..Index::default()
        }
    }

    /// The library folders.
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    /// Adds a library folder (LIB-1). Adding a folder already in the library,
    /// or inside one, does nothing; adding a parent of library folders
    /// replaces them. The path must exist.
    pub fn add_root(&mut self, path: &Path) -> std::io::Result<AddFolder> {
        let path = normalize(path)?;
        if let Some(root) = self.roots.iter().find(|root| path.starts_with(root)) {
            return Ok(AddFolder::AlreadyIncluded(root.clone()));
        }
        let (absorbed, kept): (Vec<_>, Vec<_>) = self
            .roots
            .drain(..)
            .partition(|root| root.starts_with(&path));
        self.roots = kept;
        self.roots.push(path);
        self.roots.sort();
        Ok(if absorbed.is_empty() {
            AddFolder::Added
        } else {
            AddFolder::Absorbed(absorbed)
        })
    }

    pub fn photo(&self, id: PhotoId) -> Option<&Photo> {
        self.photos.get(id.0 as usize)?.as_ref()
    }

    pub fn folder(&self, id: FolderId) -> Option<&Folder> {
        self.folders.get(id.0 as usize)?.as_ref()
    }

    pub fn folder_id(&self, path: &Path) -> Option<FolderId> {
        self.folder_ids.get(path).copied()
    }

    pub fn tag(&self, id: TagId) -> Option<&str> {
        self.tags.get(id.0 as usize).map(|tag| &**tag)
    }

    pub fn tag_id(&self, tag: &str) -> Option<TagId> {
        self.tag_ids.get(tag).copied()
    }

    /// The tag the library already has that `tag` matches, ignoring case
    /// and Unicode form (TAG-2), if some photo has it.
    pub fn matching_tag(&self, tag: &str) -> Option<&str> {
        let key = seura_edit::tag_key(tag);
        self.tags()
            .find(|(id, _, _)| self.search_tags[id.0 as usize].as_ref() == key)
            .map(|(_, tag, _)| tag)
    }

    /// How many photos have exactly this tag (TAG-5).
    pub fn tag_len(&self, id: TagId) -> usize {
        self.by_tag.get(id.0 as usize).map_or(0, Vec::len)
    }

    /// Every tag some photo has, with how many have it.
    pub fn tags(&self) -> impl Iterator<Item = (TagId, &str, usize)> {
        self.tags
            .iter()
            .enumerate()
            .map(|(id, tag)| (TagId(id as u32), &**tag, self.by_tag[id].len()))
            .filter(|(_, _, count)| *count > 0)
    }

    /// The full path of a photo.
    pub fn path(&self, id: PhotoId) -> Option<PathBuf> {
        let photo = self.photo(id)?;
        Some(self.folder(photo.folder)?.path.join(&*photo.name))
    }

    pub fn len(&self) -> usize {
        self.photos.len() - self.free_photos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn photo_ids(&self) -> impl Iterator<Item = PhotoId> + '_ {
        self.photos
            .iter()
            .enumerate()
            .filter(|(_, photo)| photo.is_some())
            .map(|(id, _)| PhotoId(id as u32))
    }

    /// The photos in a scope that pass the filter, in index order.
    pub fn query(&self, scope: Scope, filter: &Filter) -> Vec<PhotoId> {
        let candidates: Box<dyn Iterator<Item = PhotoId> + '_> = match scope {
            Scope::Folder(folder) => match self.by_folder.get(folder.0 as usize) {
                Some(ids) => Box::new(ids.iter().copied()),
                None => Box::new(std::iter::empty()),
            },
            Scope::Tag(tag) => match self.by_tag.get(tag.0 as usize) {
                Some(ids) => Box::new(ids.iter().copied()),
                None => Box::new(std::iter::empty()),
            },
            Scope::All => Box::new(self.photo_ids()),
        };
        candidates.filter(|id| self.passes(*id, filter)).collect()
    }

    /// Photos anywhere in the library whose file name or any tag contains
    /// `text`, ignoring case and Unicode form (TAG-2), that pass the filter
    /// (FND-2).
    pub fn search(&self, text: &str, filter: &Filter) -> Vec<PhotoId> {
        let text = seura_edit::tag_key(text.trim());
        if text.is_empty() {
            return self.query(Scope::All, filter);
        }
        let matching_tags: Vec<bool> = self
            .search_tags
            .iter()
            .map(|tag| tag.contains(&text))
            .collect();
        self.photo_ids()
            .filter(|id| {
                let photo = self.photo(*id).expect("listed ids exist");
                self.search_names[id.0 as usize].contains(&text)
                    || photo.tags.iter().any(|tag| matching_tags[tag.0 as usize])
            })
            .filter(|id| self.passes(*id, filter))
            .collect()
    }

    fn passes(&self, id: PhotoId, filter: &Filter) -> bool {
        let Some(photo) = self.photo(id) else {
            return false;
        };
        photo.rating >= filter.min_rating
            && filter.tags.iter().all(|tag| photo.tags.contains(tag))
            && filter.edited.is_none_or(|edited| {
                (photo.edit_file == EditFileState::Read { edited: true }) == edited
            })
            && filter.taken.is_none_or(|(from, to)| {
                (from..=to).contains(&photo.taken.unwrap_or(photo.modified.secs))
            })
    }

    /// What's needed to make the photo's thumbnail.
    pub fn thumbnail_job(&self, id: PhotoId) -> Option<crate::thumbnails::Job> {
        let photo = self.photo(id)?;
        let path = self.path(id)?;
        let edited = photo.edit_file == EditFileState::Read { edited: true };
        Some(crate::thumbnails::Job {
            id,
            key: crate::thumbnails::ThumbnailCache::key(
                &path,
                photo.size,
                photo.modified,
                photo.edit_file_stamp.filter(|_| edited),
            ),
            path,
            placeholder: photo.placeholder,
            edited,
        })
    }

    /// Records what decoding the photo learned: its upright dimensions and
    /// date taken.
    pub fn record_details(&mut self, id: PhotoId, made: &crate::thumbnails::Made) {
        if let Some(photo) = self.photo_mut(id) {
            photo.dimensions = Some((made.width, made.height));
            if made.taken.is_some() {
                photo.taken = made.taken;
            }
        }
    }

    /// The photo with this name in this folder.
    pub(crate) fn find(&self, folder: FolderId, name: &str) -> Option<PhotoId> {
        self.by_name.get(&(folder, Box::from(name))).copied()
    }

    /// Every folder in the index.
    pub fn folders(&self) -> impl Iterator<Item = (FolderId, &Folder)> {
        self.folders
            .iter()
            .enumerate()
            .filter_map(|(id, folder)| Some((FolderId(id as u32), folder.as_ref()?)))
    }

    /// How many photos are directly in a folder.
    pub fn folder_len(&self, id: FolderId) -> usize {
        self.by_folder.get(id.0 as usize).map_or(0, Vec::len)
    }

    pub(crate) fn photo_mut(&mut self, id: PhotoId) -> Option<&mut Photo> {
        self.photos.get_mut(id.0 as usize)?.as_mut()
    }

    pub(crate) fn ensure_folder(&mut self, path: &Path, parent: Option<FolderId>) -> FolderId {
        if let Some(id) = self.folder_ids.get(path) {
            return *id;
        }
        let id = FolderId(self.folders.len() as u32);
        self.folders.push(Some(Folder {
            path: path.to_path_buf(),
            parent,
            read_only: false,
        }));
        self.by_folder.push(Vec::new());
        self.folder_ids.insert(path.to_path_buf(), id);
        id
    }

    /// Records whether a folder is read-only (TAG-7).
    pub(crate) fn set_read_only(&mut self, id: FolderId, read_only: bool) {
        if let Some(Some(folder)) = self.folders.get_mut(id.0 as usize) {
            folder.read_only = read_only;
        }
    }

    /// Records what a photo's edit file now holds, as read by a scan or
    /// written by a change, with the size and time it was read or written at.
    pub(crate) fn set_edit_file(
        &mut self,
        id: PhotoId,
        state: EditFileState,
        stamp: Option<(u64, FileTime)>,
        rating: u8,
        tags: &[String],
    ) {
        let tags: Box<[_]> = tags.iter().map(|tag| self.intern_tag(tag)).collect();
        let Some(mut photo) = self.photo(id).cloned() else {
            return;
        };
        photo.edit_file = state;
        photo.edit_file_stamp = stamp;
        photo.rating = rating;
        photo.tags = tags;
        self.replace_photo(id, photo);
    }

    pub(crate) fn remove_folder(&mut self, id: FolderId) {
        for photo in std::mem::take(&mut self.by_folder[id.0 as usize]) {
            self.remove_photo(photo);
        }
        if let Some(folder) = self.folders[id.0 as usize].take() {
            self.folder_ids.remove(&folder.path);
        }
    }

    pub(crate) fn intern_tag(&mut self, tag: &str) -> TagId {
        if let Some(id) = self.tag_ids.get(tag) {
            return *id;
        }
        let id = TagId(self.tags.len() as u32);
        self.tags.push(tag.into());
        self.tag_ids.insert(tag.into(), id);
        self.by_tag.push(Vec::new());
        self.search_tags.push(seura_edit::tag_key(tag).into());
        id
    }

    pub(crate) fn insert_photo(&mut self, photo: Photo) -> PhotoId {
        let id = match self.free_photos.pop() {
            Some(id) => id,
            None => {
                self.photos.push(None);
                self.search_names.push(Box::from(""));
                PhotoId(self.photos.len() as u32 - 1)
            }
        };
        self.link(id, &photo);
        self.photos[id.0 as usize] = Some(photo);
        id
    }

    /// Replaces a photo's record, keeping its id.
    pub(crate) fn replace_photo(&mut self, id: PhotoId, photo: Photo) {
        if let Some(old) = self.photos[id.0 as usize].take() {
            self.unlink(id, &old);
        }
        self.link(id, &photo);
        self.photos[id.0 as usize] = Some(photo);
    }

    pub(crate) fn remove_photo(&mut self, id: PhotoId) {
        if let Some(old) = self.photos[id.0 as usize].take() {
            self.unlink(id, &old);
            self.free_photos.push(id);
        }
    }

    fn link(&mut self, id: PhotoId, photo: &Photo) {
        self.by_folder[photo.folder.0 as usize].push(id);
        self.by_name.insert((photo.folder, photo.name.clone()), id);
        for tag in &photo.tags {
            self.by_tag[tag.0 as usize].push(id);
        }
        self.search_names[id.0 as usize] = seura_edit::tag_key(&photo.name).into();
    }

    fn unlink(&mut self, id: PhotoId, photo: &Photo) {
        self.by_folder[photo.folder.0 as usize].retain(|other| *other != id);
        self.by_name.remove(&(photo.folder, photo.name.clone()));
        for tag in &photo.tags {
            self.by_tag[tag.0 as usize].retain(|other| *other != id);
        }
    }
}

/// An absolute path with the folder's real name, comparable component by
/// component. Windows' `\\?\` prefix is dropped where it isn't needed.
pub(crate) fn normalize(path: &Path) -> std::io::Result<PathBuf> {
    let path = std::fs::canonicalize(path)?;
    #[cfg(windows)]
    {
        let text = path.to_string_lossy();
        if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
            return Ok(PathBuf::from(format!(r"\\{rest}")));
        }
        if let Some(rest) = text.strip_prefix(r"\\?\") {
            return Ok(PathBuf::from(rest));
        }
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_times_before_1970_are_ordered() {
        let before = SystemTime::UNIX_EPOCH - std::time::Duration::from_millis(1500);
        assert_eq!(
            FileTime::from(before),
            FileTime {
                secs: -2,
                nanos: 500_000_000
            }
        );
    }

    #[test]
    fn library_folders_follow_lib_1() {
        let dir = tempfile::tempdir().unwrap();
        let photos = dir.path().join("Pictures");
        let summer = photos.join("Summer");
        let winter = photos.join("Winter");
        for folder in [&summer, &winter] {
            std::fs::create_dir_all(folder).unwrap();
        }
        let mut index = Index::new();
        assert_eq!(index.add_root(&summer).unwrap(), AddFolder::Added);
        assert_eq!(index.add_root(&winter).unwrap(), AddFolder::Added);
        assert_eq!(
            index.add_root(&summer).unwrap(),
            AddFolder::AlreadyIncluded(normalize(&summer).unwrap())
        );
        match index.add_root(&photos).unwrap() {
            AddFolder::Absorbed(children) => assert_eq!(children.len(), 2),
            other => panic!("expected the parent to absorb its children, got {other:?}"),
        }
        assert_eq!(index.roots(), [normalize(&photos).unwrap()]);
        assert!(matches!(
            index.add_root(&summer).unwrap(),
            AddFolder::AlreadyIncluded(_)
        ));
    }
}
