//! The grid's order (BRW-2): one order for the whole app, chosen in the
//! settings.

use std::cmp::Ordering;

use crate::index::{Index, PhotoId};
use crate::settings::{Sort, SortBy};

impl Index {
    /// `ids` in the given order. Date taken falls back to the file's date
    /// for photos without one, or whose date isn't known yet (BRW-2). Ties
    /// are broken by name, so the order is always the same.
    pub fn sorted(&self, mut ids: Vec<PhotoId>, sort: Sort) -> Vec<PhotoId> {
        ids.sort_by(|a, b| {
            let (Some(a), Some(b)) = (self.photo(*a), self.photo(*b)) else {
                return Ordering::Equal;
            };
            let by_name = natural(&a.name, &b.name);
            let order = match sort.by {
                SortBy::DateTaken => a
                    .taken
                    .unwrap_or(a.modified.secs)
                    .cmp(&b.taken.unwrap_or(b.modified.secs)),
                SortBy::DateModified => {
                    (a.modified.secs, a.modified.nanos).cmp(&(b.modified.secs, b.modified.nanos))
                }
                SortBy::FileName => Ordering::Equal,
            }
            .then(by_name);
            if sort.ascending {
                order
            } else {
                order.reverse()
            }
        });
        ids
    }
}

/// Names as people read them: letters without regard to case, and runs of
/// digits as numbers, so `IMG_9` comes before `IMG_10`, as in Explorer and
/// Finder.
pub fn natural(a: &str, b: &str) -> Ordering {
    let mut a = a.chars().peekable();
    let mut b = b.chars().peekable();
    // Letter case only decides between names that are otherwise the same.
    let mut by_case = Ordering::Equal;
    loop {
        match (a.peek().copied(), b.peek().copied()) {
            (None, None) => return by_case,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let (x, y) = (digits(&mut a), digits(&mut b));
                let (x_value, y_value) = (x.trim_start_matches('0'), y.trim_start_matches('0'));
                let order = x_value
                    .len()
                    .cmp(&y_value.len())
                    .then_with(|| x_value.cmp(y_value))
                    .then_with(|| x.len().cmp(&y.len()));
                if order != Ordering::Equal {
                    return order;
                }
            }
            (Some(x), Some(y)) => {
                let order = x.to_lowercase().cmp(y.to_lowercase());
                if order != Ordering::Equal {
                    return order;
                }
                if by_case == Ordering::Equal {
                    by_case = x.cmp(&y);
                }
                a.next();
                b.next();
            }
        }
    }
}

/// The run of digits at the start of `chars`, taken from it.
fn digits(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut run = String::new();
    while let Some(c) = chars.peek().copied().filter(char::is_ascii_digit) {
        run.push(c);
        chars.next();
    }
    run
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::{Filter, Scope};

    #[test]
    fn names_sort_as_people_read_them() {
        let mut names = vec![
            "IMG_10.jpg",
            "img_9.jpg",
            "IMG_100.jpg",
            "Beach.png",
            "IMG_09b.jpg",
        ];
        names.sort_by(|a, b| natural(a, b));
        assert_eq!(
            names,
            [
                "Beach.png",
                "img_9.jpg",
                "IMG_09b.jpg",
                "IMG_10.jpg",
                "IMG_100.jpg"
            ]
        );
    }

    fn jpeg() -> Vec<u8> {
        let mut bytes = vec![0xFF, 0xD8, 0xFF, 0xE0];
        bytes.extend([0; 60]);
        bytes
    }

    #[test]
    fn photos_sort_by_date_taken_name_or_date_modified() {
        let folder = tempfile::tempdir().unwrap();
        let root = folder.path().join("Pictures");
        fs::create_dir_all(&root).unwrap();
        for name in ["b.jpg", "a.jpg", "c.jpg"] {
            fs::write(root.join(name), jpeg()).unwrap();
        }
        let mut index = Index::new();
        index.add_root(&root).unwrap();
        let root = index.roots()[0].clone();
        index.scan(&root);
        let folder = index.folder_id(&root).unwrap();
        let ids = index.query(Scope::Folder(folder), &Filter::default());
        let id = |name: &str| {
            *ids.iter()
                .find(|id| &*index.photo(**id).unwrap().name == name)
                .unwrap()
        };
        let (a, b, c) = (id("a.jpg"), id("b.jpg"), id("c.jpg"));
        // b was taken first; c's date isn't known, so its file date counts.
        for (photo, taken, modified) in [(a, Some(300), 50), (b, Some(100), 60), (c, None, 200)] {
            let photo = index.photo_mut(photo).unwrap();
            photo.taken = taken;
            photo.modified.secs = modified;
        }
        let names = |order: Vec<PhotoId>| -> Vec<String> {
            order
                .iter()
                .map(|id| index.photo(*id).unwrap().name.to_string())
                .collect()
        };
        let sort = |by, ascending| Sort { by, ascending };

        assert_eq!(
            names(index.sorted(ids.clone(), sort(SortBy::DateTaken, true))),
            ["b.jpg", "c.jpg", "a.jpg"]
        );
        assert_eq!(
            names(index.sorted(ids.clone(), sort(SortBy::DateTaken, false))),
            ["a.jpg", "c.jpg", "b.jpg"]
        );
        assert_eq!(
            names(index.sorted(ids.clone(), sort(SortBy::FileName, true))),
            ["a.jpg", "b.jpg", "c.jpg"]
        );
        assert_eq!(
            names(index.sorted(ids.clone(), sort(SortBy::DateModified, true))),
            ["a.jpg", "b.jpg", "c.jpg"]
        );
        assert_eq!(
            names(index.sorted(ids, sort(SortBy::DateModified, false))),
            ["c.jpg", "b.jpg", "a.jpg"]
        );
    }
}
