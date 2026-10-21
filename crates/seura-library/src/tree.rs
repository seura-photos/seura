//! The folder tree the sidebar shows (LIB-2), and removing a library folder
//! (LIB-5).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::index::{FolderId, Index};

/// One row of the folder tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FolderEntry {
    /// The folder in the index; `None` for a library folder with no photos
    /// yet, or one on a drive that has never been scanned.
    pub id: Option<FolderId>,
    /// The library folder's position in [`Index::roots`], for library
    /// folders; `None` for subfolders.
    pub root: Option<usize>,
    /// The folder above, for subfolders.
    pub parent: Option<FolderId>,
    pub path: PathBuf,
    /// Photos directly in this folder: the ones the grid shows (BRW-1).
    pub photos: usize,
    /// Photos here and in every subfolder.
    pub total: usize,
}

impl Index {
    /// The library folders, always, and every subfolder with photos in it
    /// or further down (LIB-2), each with its counts. Library folders come
    /// first, in order; subfolders follow in no particular order.
    pub fn folder_tree(&self) -> Vec<FolderEntry> {
        let mut totals: HashMap<FolderId, usize> = HashMap::new();
        for (id, _) in self.folders() {
            let own = self.folder_len(id);
            let mut current = Some(id);
            while let Some(folder) = current {
                *totals.entry(folder).or_default() += own;
                current = self.folder(folder).and_then(|folder| folder.parent);
            }
        }
        let root_ids: Vec<Option<FolderId>> = self
            .roots()
            .iter()
            .map(|root| self.folder_id(root))
            .collect();
        let mut entries: Vec<FolderEntry> = self
            .roots()
            .iter()
            .zip(&root_ids)
            .enumerate()
            .map(|(position, (path, id))| FolderEntry {
                id: *id,
                root: Some(position),
                parent: None,
                path: path.clone(),
                photos: id.map_or(0, |id| self.folder_len(id)),
                total: id.and_then(|id| totals.get(&id).copied()).unwrap_or(0),
            })
            .collect();
        entries.extend(
            self.folders()
                .filter(|(id, _)| !root_ids.contains(&Some(*id)))
                .map(|(id, folder)| FolderEntry {
                    id: Some(id),
                    root: None,
                    parent: folder.parent,
                    path: folder.path.clone(),
                    photos: self.folder_len(id),
                    total: totals.get(&id).copied().unwrap_or(0),
                }),
        );
        entries
    }

    /// Removes a library folder from the library (LIB-5): its photos and
    /// subfolders leave the index. Nothing on disk is touched. Returns the
    /// cache keys of the removed photos' thumbnails, so the caller can drop
    /// them from the cache.
    pub fn remove_root(&mut self, root: &Path) -> Vec<String> {
        let Some(position) = self.roots.iter().position(|known| known == root) else {
            return Vec::new();
        };
        self.roots.remove(position);
        let mut folders: Vec<(FolderId, PathBuf)> = self
            .folders()
            .filter(|(_, folder)| folder.path.starts_with(root))
            .map(|(id, folder)| (id, folder.path.clone()))
            .collect();
        let keys = folders
            .iter()
            .flat_map(|(id, _)| self.query(crate::Scope::Folder(*id), &crate::Filter::default()))
            .filter_map(|photo| self.thumbnail_job(photo))
            .map(|job| job.key)
            .collect();
        // Deepest first, as when a scan prunes folders.
        folders.sort_by_key(|(_, path)| std::cmp::Reverse(path.components().count()));
        for (id, _) in folders {
            self.remove_folder(id);
        }
        keys
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn jpeg() -> Vec<u8> {
        // Enough of a JPEG for the scanner's signature check.
        let mut bytes = vec![0xFF, 0xD8, 0xFF, 0xE0];
        bytes.extend([0; 60]);
        bytes
    }

    fn library(files: &[&str]) -> (tempfile::TempDir, PathBuf) {
        let folder = tempfile::tempdir().unwrap();
        let root = folder.path().join("Pictures");
        fs::create_dir_all(&root).unwrap();
        for file in files {
            let path = root.join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, jpeg()).unwrap();
        }
        (folder, root)
    }

    fn scanned(root: &Path) -> Index {
        let mut index = Index::new();
        index.add_root(root).unwrap();
        let root = index.roots()[0].clone();
        index.scan(&root);
        index
    }

    fn by_name(tree: &[FolderEntry], name: &str) -> FolderEntry {
        tree.iter()
            .find(|entry| entry.path.file_name().unwrap() == name)
            .unwrap_or_else(|| panic!("no folder {name} in {tree:?}"))
            .clone()
    }

    #[test]
    fn the_tree_counts_each_folder_and_everything_below_it() {
        let (_folder, root) = library(&[
            "a.jpg",
            "2026/Spring/b.jpg",
            "2026/Spring/c.jpg",
            "2026/Summer/d.jpg",
            "2026/Empty/notes.txt",
        ]);
        let index = scanned(&root);
        let tree = index.folder_tree();

        let pictures = by_name(&tree, "Pictures");
        assert_eq!(
            (pictures.root, pictures.photos, pictures.total),
            (Some(0), 1, 4)
        );
        let year = by_name(&tree, "2026");
        assert_eq!((year.photos, year.total), (0, 3));
        assert_eq!(year.parent, pictures.id);
        let spring = by_name(&tree, "Spring");
        assert_eq!(
            (spring.photos, spring.total, spring.parent),
            (2, 2, year.id)
        );
        // A folder with no photos anywhere inside isn't shown (LIB-2).
        assert!(!tree.iter().any(|entry| entry.path.ends_with("Empty")));
    }

    #[test]
    fn a_library_folder_is_listed_even_with_no_photos() {
        let (_folder, root) = library(&[]);
        let index = scanned(&root);
        let tree = index.folder_tree();
        assert_eq!(tree.len(), 1);
        assert_eq!((tree[0].root, tree[0].total), (Some(0), 0));
    }

    #[test]
    fn removing_a_library_folder_drops_it_and_everything_below_it() {
        let (_folder, root) = library(&["a.jpg", "2026/b.jpg"]);
        let mut index = scanned(&root);
        let root = index.roots()[0].clone();

        let keys = index.remove_root(&root);

        assert_eq!(keys.len(), 2);
        assert!(index.roots().is_empty());
        assert!(index.folder_tree().is_empty());
        assert!(index.is_empty());
        // The photos themselves are untouched (CLAUDE.md rule 1).
        assert!(root.join("a.jpg").is_file());
        assert!(root.join("2026/b.jpg").is_file());
    }
}
