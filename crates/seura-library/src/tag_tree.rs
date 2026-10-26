//! The tag tree the sidebar shows (TAG-1, TAG-5): `family/kids` as `kids`
//! under `family`. A slash only arranges tags for display; each full tag is
//! a tag of its own, so `family` and `family/kids` are unrelated.

use std::collections::BTreeMap;

use crate::index::Index;

/// One node of the tag tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagEntry {
    /// The full tag: `family/kids`.
    pub tag: String,
    /// The last level: `kids`.
    pub name: String,
    /// The node above, for nested tags: `family`.
    pub parent: Option<String>,
    /// Photos with exactly this tag. `None` for a node no photo has exactly,
    /// such as `family` when only `family/kids` is used: it shows no count,
    /// not 0, and not a total of the tags beneath it (TAG-5).
    pub photos: Option<usize>,
}

impl Index {
    /// Every tag some photo has, and the nodes above each, in natural order
    /// by the full tag. A tag no photo uses is left out (TAG-4).
    pub fn tag_tree(&self) -> Vec<TagEntry> {
        let mut nodes: BTreeMap<String, Option<usize>> = BTreeMap::new();
        for (_, tag, count) in self.tags() {
            let mut above = tag;
            while let Some((parent, _)) = above.rsplit_once('/') {
                nodes.entry(parent.to_string()).or_insert(None);
                above = parent;
            }
            nodes.insert(tag.to_string(), Some(count));
        }
        let mut entries: Vec<TagEntry> = nodes
            .into_iter()
            .map(|(tag, photos)| {
                let (parent, name) = match tag.rsplit_once('/') {
                    Some((parent, name)) => (Some(parent.to_string()), name.to_string()),
                    None => (None, tag.clone()),
                };
                TagEntry {
                    tag,
                    name,
                    parent,
                    photos,
                }
            })
            .collect();
        entries.sort_by(|a, b| crate::natural(&a.tag, &b.tag));
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{EditFileState, FileTime, Photo};

    fn tagged(index: &mut Index, name: &str, tags: &[&str]) {
        let folder = index.ensure_folder(std::path::Path::new("/Pictures"), None);
        let tags = tags.iter().map(|tag| index.intern_tag(tag)).collect();
        index.insert_photo(Photo {
            folder,
            name: name.into(),
            size: 1,
            modified: FileTime::default(),
            fingerprint: None,
            dimensions: None,
            taken: None,
            edit_file: EditFileState::Read { edited: false },
            edit_file_stamp: None,
            rating: 0,
            tags,
            placeholder: false,
        });
    }

    #[test]
    fn nested_tags_sit_under_nodes_that_count_only_their_own_photos() {
        let mut index = Index::new();
        tagged(&mut index, "1.jpg", &["family/kids", "vacation/2026/beach"]);
        tagged(&mut index, "2.jpg", &["family/kids", "vacation"]);
        // Interned but no longer used by any photo.
        index.intern_tag("gone");

        let tree = index.tag_tree();
        let rows: Vec<(&str, &str, Option<&str>, Option<usize>)> = tree
            .iter()
            .map(|e| {
                (
                    e.tag.as_str(),
                    e.name.as_str(),
                    e.parent.as_deref(),
                    e.photos,
                )
            })
            .collect();
        assert_eq!(
            rows,
            [
                ("family", "family", None, None),
                ("family/kids", "kids", Some("family"), Some(2)),
                ("vacation", "vacation", None, Some(1)),
                ("vacation/2026", "2026", Some("vacation"), None),
                (
                    "vacation/2026/beach",
                    "beach",
                    Some("vacation/2026"),
                    Some(1)
                ),
            ]
        );
    }
}
