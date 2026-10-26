//! An edit file's contents: the JSON exactly as read, with a typed view of the
//! parts Seura understands (technical design, section 7).
//!
//! Edits change only the specific entries they're about, in place, so every
//! unknown operation, field, and extension is saved back exactly as found
//! (edit format: "Preserving the unknown"; CLAUDE.md rule 4).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use unicode_normalization::UnicodeNormalization;

use crate::operations::{CORE_VERSION, CoreOpId, CoreOperation, order_key};
use crate::validate::{self, is_undone};

/// Seura keeps the last 50 history records (EDT-9).
pub const HISTORY_LIMIT: usize = 50;

const SCHEMA_URL: &str = "https://seuraphotos.app/schema/edit/1.0.json";

/// Top-level fields in the order Seura writes them. New fields are inserted
/// after the last present field that comes before them; unknown fields keep
/// their place.
const FIELD_ORDER: [&str; 13] = [
    "$schema",
    "format",
    "formatVersion",
    "kind",
    "name",
    "writer",
    "modified",
    "photo",
    "operations",
    "tags",
    "rating",
    "history",
    "extensions",
];

/// Why bytes couldn't be read as an edit file. A file that fails to read
/// must never be overwritten (CLAUDE.md rule 4).
#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("not valid UTF-8 JSON: {0}")]
    Syntax(String),
    #[error("not a Seura edit file")]
    NotSeuraEdit,
    #[error("format version {0} is not supported")]
    UnsupportedVersion(String),
    #[error("invalid edit file: {}", .0.join("; "))]
    Invalid(Vec<String>),
}

/// Why an edit couldn't be applied. The document is unchanged when this is returned.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum EditError {
    #[error("{0} is stored in a version this app can't edit")]
    UnsupportedOperation(String),
    #[error("presets have no tags, rating, or history")]
    NotForPresets,
    #[error("the result would be invalid: {}", .0.join("; "))]
    Invalid(Vec<String>),
}

/// Whether the file describes one photo or is a preset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Photo,
    Preset,
}

/// The photo an edit file belongs to (edit format: "Photo identity").
///
/// The fingerprint and dimensions need the photo's contents, so they may be
/// absent, for example for a cloud-only photo that was tagged without being
/// downloaded (LIB-9). [`EditDocument::fill_in_photo`] adds them later.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhotoIdentity {
    /// The photo's file name, without a path.
    pub file: String,
    pub size: u64,
    /// SHA-256 of the photo file, lowercase hex.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// Pixel dimensions after the photo's own orientation metadata; both or neither.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
}

/// One operation entry, as the typed view sees it.
#[derive(Clone, Debug, PartialEq)]
pub struct Operation {
    pub id: String,
    pub version: u64,
    pub enabled: bool,
    /// Set when the entry is a core operation this crate can render and edit.
    /// Other entries are kept unchanged and skipped when rendering.
    pub core: Option<CoreOperation>,
    /// For non-core operations: the user must be warned prominently when it
    /// can't be rendered.
    pub required: bool,
}

/// One user action: a label for menus and history, and what it changes.
#[derive(Clone, Debug, PartialEq)]
pub struct Action {
    pub label: String,
    pub edits: Vec<Edit>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    /// Set a core operation; neutral settings remove it.
    Set(CoreOperation),
    Remove(CoreOpId),
    /// Replace the tags. They're cleaned up first (edit format: "Tags and rating").
    Tags(Vec<String>),
    /// 0 to 5; 0 removes the rating.
    Rating(u8),
}

/// The result of [`EditDocument::undo`] or [`EditDocument::redo`].
#[derive(Clone, Debug, PartialEq)]
pub enum Step {
    /// The action with this label was undone or redone.
    Done(String),
    /// Nothing to undo or redo.
    Nothing,
    /// The file was changed without updating its history (or the record holds
    /// a change this version doesn't understand), so the record failed the
    /// consistency check and nothing was applied. After a failed undo, that
    /// record and all older ones were discarded; after a failed redo, all
    /// undone records were (edit format: "History").
    Discarded,
}

#[derive(Clone, Debug)]
pub struct EditDocument {
    root: Map<String, Value>,
    newer_minor: bool,
}

impl EditDocument {
    /// Reads and validates an edit file or preset.
    pub fn parse(bytes: &[u8]) -> Result<EditDocument, ReadError> {
        // A byte-order mark isn't allowed, but some editors add one; reading
        // past it is harmless, and it's never written back.
        let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
        let value: Value =
            serde_json::from_slice(bytes).map_err(|e| ReadError::Syntax(e.to_string()))?;
        let Value::Object(root) = value else {
            return Err(ReadError::NotSeuraEdit);
        };
        if root.get("format").and_then(Value::as_str) != Some("seura-edit") {
            return Err(ReadError::NotSeuraEdit);
        }
        let version = root
            .get("formatVersion")
            .and_then(Value::as_str)
            .unwrap_or("");
        let newer_minor = match parse_version(version) {
            // A malformed version is checked against 1.0, which reports it.
            Some((1, 0)) | None => false,
            Some((1, _)) => true,
            // A newer major version may change meaning: never write it.
            Some(_) => return Err(ReadError::UnsupportedVersion(version.to_string())),
        };
        let document = EditDocument { root, newer_minor };
        let problems = document.problems();
        if problems.is_empty() {
            Ok(document)
        } else {
            Err(ReadError::Invalid(problems))
        }
    }

    /// A new, unedited document for a photo, in format 1.0.
    pub fn new_for_photo(photo: &PhotoIdentity) -> EditDocument {
        let mut root = Map::new();
        root.insert("$schema".into(), json!(SCHEMA_URL));
        root.insert("format".into(), json!("seura-edit"));
        root.insert("formatVersion".into(), json!("1.0"));
        root.insert(
            "photo".into(),
            serde_json::to_value(photo).expect("photo serializes"),
        );
        EditDocument {
            root,
            newer_minor: false,
        }
    }

    /// The document as written to disk: UTF-8 JSON, two-space indent, final
    /// newline. Fails rather than produce an invalid file.
    pub fn to_bytes(&self) -> Result<Vec<u8>, EditError> {
        let problems = self.problems();
        if !problems.is_empty() {
            return Err(EditError::Invalid(problems));
        }
        let mut bytes =
            serde_json::to_vec_pretty(&self.root).expect("a JSON object always serializes");
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// The whole document, as JSON.
    pub fn json(&self) -> &Map<String, Value> {
        &self.root
    }

    pub fn kind(&self) -> Kind {
        match self.root.get("kind").and_then(Value::as_str) {
            Some("preset") => Kind::Preset,
            _ => Kind::Photo,
        }
    }

    pub fn photo(&self) -> Option<PhotoIdentity> {
        serde_json::from_value(self.root.get("photo")?.clone()).ok()
    }

    pub fn operations(&self) -> Vec<Operation> {
        self.entries()
            .map(|entries| entries.iter().map(read_operation).collect())
            .unwrap_or_default()
    }

    /// The core operation `id`, if present and editable.
    pub fn core(&self, id: CoreOpId) -> Option<CoreOperation> {
        self.operations()
            .into_iter()
            .find(|op| op.id == id.id())
            .and_then(|op| op.core)
    }

    pub fn tags(&self) -> Vec<String> {
        self.root
            .get("tags")
            .and_then(Value::as_array)
            .map(|tags| {
                tags.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn rating(&self) -> u8 {
        self.root
            .get("rating")
            .and_then(Value::as_u64)
            .map_or(0, |rating| rating.min(5) as u8)
    }

    /// Whether the file holds nothing worth keeping: no operations (known or
    /// not), tags, or rating, no extensions, and no field Seura doesn't know,
    /// such as another app's or a newer version's. Such a file goes to the
    /// Trash rather than stay empty (FIL-1; edit format: "Rules for readers
    /// and writers"). The history alone isn't kept: it only records changes
    /// to what's now gone.
    pub fn is_empty(&self) -> bool {
        const KNOWN_PHOTO_FIELDS: [&str; 5] = ["file", "size", "sha256", "width", "height"];
        let known_fields = self.root.keys().all(|key| {
            FIELD_ORDER.contains(&key.as_str())
                && !matches!(
                    key.as_str(),
                    "operations" | "tags" | "rating" | "extensions"
                )
        });
        let known_photo = self
            .root
            .get("photo")
            .and_then(Value::as_object)
            .is_none_or(|photo| {
                photo
                    .keys()
                    .all(|key| KNOWN_PHOTO_FIELDS.contains(&key.as_str()))
            });
        self.kind() == Kind::Photo && known_fields && known_photo
    }

    /// Fills in the photo's fingerprint and dimensions where they're missing,
    /// once the photo has been read (edit format: "Photo identity"). Values
    /// already there are kept: a different fingerprint means the photo
    /// changed (FIL-7), which is for the caller to report. Not a user action,
    /// so not recorded in the history. Returns whether anything was added.
    pub fn fill_in_photo(
        &mut self,
        sha256: &str,
        width: u32,
        height: u32,
    ) -> Result<bool, EditError> {
        let before = self.root.clone();
        let Some(photo) = self.root.get_mut("photo").and_then(Value::as_object_mut) else {
            return Ok(false);
        };
        let mut added = false;
        if !photo.contains_key("sha256") {
            let after_size = photo
                .keys()
                .position(|k| k == "size")
                .map_or(photo.len(), |i| i + 1);
            photo.shift_insert(after_size, "sha256".into(), json!(sha256));
            added = true;
        }
        if !photo.contains_key("width") && !photo.contains_key("height") {
            photo.insert("width".into(), json!(width));
            photo.insert("height".into(), json!(height));
            added = true;
        }
        if let Err(error) = self.check() {
            self.root = before;
            return Err(error);
        }
        Ok(added)
    }

    /// The number of history records in the file, undone or not.
    pub fn history_len(&self) -> usize {
        self.history().map_or(0, Vec::len)
    }

    /// How many actions can be undone.
    pub fn undo_steps(&self) -> usize {
        self.history_len() - self.redo_steps()
    }

    /// How many undone actions can be redone, including after a restart.
    pub fn redo_steps(&self) -> usize {
        self.history()
            .map_or(0, |history| history.iter().filter(|r| is_undone(r)).count())
    }

    /// The change [`EditDocument::undo`] would undo next: the last history
    /// record that isn't undone. For checking that it's still the one an
    /// action wrote (BRW-11).
    pub fn last_change(&self) -> Option<Value> {
        let history = self.history()?;
        history
            .iter()
            .rev()
            .find(|r| !is_undone(r))
            .map(without_undone)
    }

    /// The change [`EditDocument::redo`] would redo next: the first undone
    /// history record, without its `undone` mark.
    pub fn next_redo(&self) -> Option<Value> {
        self.history()?
            .iter()
            .find(|r| is_undone(r))
            .map(without_undone)
    }

    /// Applies one user action and records it in the history. Returns whether
    /// anything changed. On error, nothing changes.
    pub fn apply(&mut self, action: &Action, at: OffsetDateTime) -> Result<bool, EditError> {
        let before = self.root.clone();
        match self.try_apply(action, at) {
            Ok(changed) => Ok(changed),
            Err(error) => {
                self.root = before;
                Err(error)
            }
        }
    }

    fn try_apply(&mut self, action: &Action, at: OffsetDateTime) -> Result<bool, EditError> {
        let mut changes = Vec::new();
        for edit in &action.edits {
            if let Some(change) = self.apply_edit(edit)? {
                changes.push(change);
            }
        }
        if changes.is_empty() {
            return Ok(false);
        }
        if self.kind() == Kind::Photo {
            let record = json!({ "at": timestamp(at), "label": action.label, "changes": changes });
            let history = self.history_mut();
            // A new change discards every undone record.
            history.retain(|record| !is_undone(record));
            history.push(record);
            if history.len() > HISTORY_LIMIT {
                let excess = history.len() - HISTORY_LIMIT;
                history.drain(..excess);
            }
        }
        self.check()?;
        Ok(true)
    }

    /// Applies one edit and returns its history change, or `None` if it
    /// changed nothing.
    fn apply_edit(&mut self, edit: &Edit) -> Result<Option<Value>, EditError> {
        match edit {
            Edit::Set(op) if op.is_neutral() => self.apply_edit(&Edit::Remove(op.id())),
            Edit::Set(op) => {
                let id = op.id();
                let to = op.to_params();
                let from = self.editable_params(id)?;
                if from.as_ref().is_some_and(|from| same(from, &to)) {
                    return Ok(None);
                }
                self.set_params(id.id(), Some(&to));
                Ok(Some(json!({ "op": id.id(), "from": from, "to": to })))
            }
            Edit::Remove(id) => {
                let Some(from) = self.editable_params(*id)? else {
                    return Ok(None);
                };
                self.set_params(id.id(), None);
                Ok(Some(json!({ "op": id.id(), "from": from, "to": null })))
            }
            Edit::Tags(tags) => {
                if self.kind() == Kind::Preset {
                    return Err(EditError::NotForPresets);
                }
                let to = clean_tags(tags);
                let from = self.tags();
                if from == to {
                    return Ok(None);
                }
                self.set_tags(&to);
                Ok(Some(json!({ "field": "tags", "from": from, "to": to })))
            }
            Edit::Rating(rating) => {
                if self.kind() == Kind::Preset {
                    return Err(EditError::NotForPresets);
                }
                let from = self.rating();
                if from == *rating {
                    return Ok(None);
                }
                self.set_rating(*rating);
                Ok(Some(
                    json!({ "field": "rating", "from": from, "to": rating }),
                ))
            }
        }
    }

    /// Undoes the last record that isn't undone, after checking the file
    /// still matches it, and marks it `"undone": true` so it can be redone,
    /// even after a restart.
    pub fn undo(&mut self) -> Result<Step, EditError> {
        let Some(index) = self
            .history()
            .and_then(|history| history.iter().rposition(|r| !is_undone(r)))
        else {
            return Ok(Step::Nothing);
        };
        let record = self.history_mut()[index].clone();
        let before = self.root.clone();
        if !self.matches(&record, "to") || !self.restore(&record, "from") {
            self.root = before;
            // That record and all older ones are discarded.
            self.history_mut().drain(..=index);
            self.remove_history_if_empty();
            return Ok(Step::Discarded);
        }
        mark_undone(&mut self.history_mut()[index], true);
        if let Err(error) = self.check() {
            self.root = before;
            return Err(error);
        }
        Ok(Step::Done(label(&record)))
    }

    /// Redoes the first undone record, after checking the file still matches
    /// its starting point.
    pub fn redo(&mut self) -> Result<Step, EditError> {
        let Some(index) = self
            .history()
            .and_then(|history| history.iter().position(is_undone))
        else {
            return Ok(Step::Nothing);
        };
        let record = self.history_mut()[index].clone();
        let before = self.root.clone();
        if !self.matches(&record, "from") || !self.restore(&record, "to") {
            self.root = before;
            // All undone records are discarded.
            self.history_mut().retain(|record| !is_undone(record));
            self.remove_history_if_empty();
            return Ok(Step::Discarded);
        }
        mark_undone(&mut self.history_mut()[index], false);
        if let Err(error) = self.check() {
            self.root = before;
            return Err(error);
        }
        Ok(Step::Done(label(&record)))
    }

    fn remove_history_if_empty(&mut self) {
        if self.history_len() == 0 {
            self.root.remove("history");
        }
    }

    /// Updates `writer` and `modified` for a save (edit format: "Rules for
    /// readers and writers"). Other fields in `writer` are kept.
    pub(crate) fn stamp(&mut self, app: &str, version: &str, at: OffsetDateTime) {
        if !self.root.get("writer").is_some_and(Value::is_object) {
            self.insert_field("writer", json!({}));
        }
        if let Some(writer) = self.root.get_mut("writer").and_then(Value::as_object_mut) {
            writer.insert("app".into(), json!(app));
            writer.insert("version".into(), json!(version));
        }
        self.insert_field("modified", json!(timestamp(at)));
    }

    fn problems(&self) -> Vec<String> {
        validate::problems(&Value::Object(self.root.clone()), self.newer_minor)
    }

    fn check(&self) -> Result<(), EditError> {
        let problems = self.problems();
        if problems.is_empty() {
            Ok(())
        } else {
            Err(EditError::Invalid(problems))
        }
    }

    fn entries(&self) -> Option<&Vec<Value>> {
        self.root.get("operations").and_then(Value::as_array)
    }

    fn history(&self) -> Option<&Vec<Value>> {
        self.root.get("history").and_then(Value::as_array)
    }

    fn history_mut(&mut self) -> &mut Vec<Value> {
        if !self.root.get("history").is_some_and(Value::is_array) {
            self.insert_field("history", json!([]));
        }
        self.root
            .get_mut("history")
            .and_then(Value::as_array_mut)
            .expect("history was just made an array")
    }

    /// The current params of core operation `id`, or `None` if absent. An
    /// entry this crate can't edit (another version, unknown params) is an error,
    /// so it's never replaced by a downgrade.
    fn editable_params(&self, id: CoreOpId) -> Result<Option<Value>, EditError> {
        let Some(entry) = self.entry(id.id()) else {
            return Ok(None);
        };
        match read_operation(entry).core {
            Some(_) => Ok(entry.get("params").cloned()),
            None => Err(EditError::UnsupportedOperation(id.id().to_string())),
        }
    }

    fn entry(&self, id: &str) -> Option<&Value> {
        self.entries()?
            .iter()
            .find(|entry| entry.get("id").and_then(Value::as_str) == Some(id))
    }

    /// Sets an operation's params in place, inserts a new core entry at its
    /// canonical position, or removes the entry (`None`). Other entries and
    /// the entry's other fields (such as `enabled`) are untouched.
    fn set_params(&mut self, id: &str, params: Option<&Value>) {
        if !self.root.get("operations").is_some_and(Value::is_array) {
            self.insert_field("operations", json!([]));
        }
        let entries = self
            .root
            .get_mut("operations")
            .and_then(Value::as_array_mut)
            .expect("operations was just made an array");
        let position = entries
            .iter()
            .position(|entry| entry.get("id").and_then(Value::as_str) == Some(id));
        match (position, params) {
            (Some(index), Some(params)) => entries[index]["params"] = params.clone(),
            (Some(index), None) => {
                entries.remove(index);
            }
            (None, Some(params)) => {
                let entry = json!({ "id": id, "version": CORE_VERSION, "params": params });
                let key = order_key(&entry);
                let index = entries
                    .iter()
                    .position(|other| order_key(other).is_some_and(|other| Some(other) > key))
                    .unwrap_or(entries.len());
                entries.insert(index, entry);
            }
            (None, None) => {}
        }
        if entries.is_empty() {
            self.root.remove("operations");
        }
    }

    fn set_tags(&mut self, tags: &[String]) {
        if tags.is_empty() {
            self.root.remove("tags");
        } else {
            self.insert_field("tags", json!(tags));
        }
    }

    fn set_rating(&mut self, rating: u8) {
        // Zero means unrated, and is omitted.
        if rating == 0 {
            self.root.remove("rating");
        } else {
            self.insert_field("rating", json!(rating));
        }
    }

    /// Sets a top-level field, keeping its place if present, otherwise
    /// inserting it after the last field that precedes it in [`FIELD_ORDER`].
    fn insert_field(&mut self, key: &str, value: Value) {
        if let Some(existing) = self.root.get_mut(key) {
            *existing = value;
            return;
        }
        let rank = FIELD_ORDER.iter().position(|k| *k == key);
        let index = self
            .root
            .keys()
            .enumerate()
            .filter(|(_, k)| {
                let other = FIELD_ORDER.iter().position(|f| f == k);
                matches!((other, rank), (Some(other), Some(rank)) if other < rank)
            })
            .map(|(index, _)| index + 1)
            .max()
            .unwrap_or(0);
        self.root.shift_insert(index, key.to_string(), value);
    }

    /// Whether the current state equals each change's `side` ("from" or "to").
    fn matches(&self, record: &Value, side: &str) -> bool {
        let Some(changes) = record.get("changes").and_then(Value::as_array) else {
            return false;
        };
        changes.iter().all(|change| {
            let Some(expected) = change.get(side) else {
                return false;
            };
            if let Some(op) = change.get("op").and_then(Value::as_str) {
                let current = self.entry(op).and_then(|entry| entry.get("params"));
                return match current {
                    Some(current) => same(current, expected),
                    None => expected.is_null(),
                };
            }
            match change.get("field").and_then(Value::as_str) {
                Some("tags") => same(&json!(self.tags()), expected),
                Some("rating") => same(&json!(self.rating()), expected),
                _ => false,
            }
        })
    }

    /// Applies each change's `side`. Returns false if a change can't be
    /// applied: an unknown kind of change, or recreating an operation this
    /// crate can't describe (only core operations can be recreated).
    fn restore(&mut self, record: &Value, side: &str) -> bool {
        let Some(changes) = record.get("changes").and_then(Value::as_array).cloned() else {
            return false;
        };
        for change in changes {
            let Some(target) = change.get(side) else {
                return false;
            };
            if let Some(op) = change.get("op").and_then(Value::as_str) {
                let exists = self.entry(op).is_some();
                match (target.is_null(), exists) {
                    (true, _) => self.set_params(op, None),
                    (false, true) => self.set_params(op, Some(target)),
                    (false, false) if CoreOpId::from_id(op).is_some() => {
                        self.set_params(op, Some(target))
                    }
                    (false, false) => return false,
                }
                continue;
            }
            match (change.get("field").and_then(Value::as_str), target) {
                (Some("tags"), Value::Array(tags)) => {
                    let tags: Vec<String> = tags
                        .iter()
                        .filter_map(Value::as_str)
                        .map(String::from)
                        .collect();
                    self.set_tags(&tags);
                }
                (Some("rating"), rating) => match rating.as_u64() {
                    Some(rating) if rating <= 5 => self.set_rating(rating as u8),
                    _ => return false,
                },
                _ => return false,
            }
        }
        true
    }
}

fn read_operation(entry: &Value) -> Operation {
    let id = entry
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let version = entry.get("version").and_then(Value::as_u64).unwrap_or(0);
    let core = CoreOpId::from_id(&id).and_then(|core| {
        CoreOperation::from_params(core, version, entry.get("params").unwrap_or(&Value::Null))
    });
    Operation {
        enabled: entry
            .get("enabled")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        required: entry
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        id,
        version,
        core,
    }
}

/// Sets or clears a record's `undone` flag, placed after its label as in
/// the spec's examples.
fn mark_undone(record: &mut Value, undone: bool) {
    let Some(record) = record.as_object_mut() else {
        return;
    };
    if !undone {
        record.shift_remove("undone");
        return;
    }
    let after = ["label", "at"]
        .iter()
        .find_map(|key| record.keys().position(|k| k == key))
        .map_or(0, |index| index + 1);
    record.shift_insert(after, "undone".into(), Value::Bool(true));
}

/// A history record as written, without its `undone` mark.
fn without_undone(record: &Value) -> Value {
    let mut record = record.clone();
    if let Some(record) = record.as_object_mut() {
        record.shift_remove("undone");
    }
    record
}

fn label(record: &Value) -> String {
    record
        .get("label")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// `"major.minor"`, without leading zeros, as `tools/check.py` reads it.
fn parse_version(version: &str) -> Option<(u32, u32)> {
    let (major, minor) = version.split_once('.')?;
    let number = |s: &str| -> Option<u32> {
        let canonical = s == "0" || (!s.is_empty() && !s.starts_with('0'));
        if canonical && s.bytes().all(|b| b.is_ascii_digit()) {
            s.parse().ok()
        } else {
            None
        }
    };
    Some((number(major)?, number(minor)?))
}

/// UTC, ISO 8601, whole seconds: `2026-10-17T19:40:12Z`.
fn timestamp(at: OffsetDateTime) -> String {
    at.to_offset(time::UtcOffset::UTC)
        .replace_nanosecond(0)
        .expect("0 is a valid nanosecond")
        .format(&Rfc3339)
        .expect("RFC 3339 formats any UTC time")
}

/// What two tags are compared by (TAG-2): the text normalized (NFC) and case
/// folded with Unicode's full folding, so matching works in every language,
/// not just A to Z: `Ärzte` matches `ärzte`, and `STRASSE` matches `straße`.
pub fn tag_key(tag: &str) -> String {
    let normalized: String = tag.nfc().collect();
    caseless::default_case_fold_str(&normalized).nfc().collect()
}

/// Cleans tags as writers must: no leading or trailing spaces or slashes, no
/// empty levels, and no two tags differing only in letter case, compared by
/// [`tag_key`] (the first wins, in the form it was typed). Stored in NFC.
pub fn clean_tags(tags: &[String]) -> Vec<String> {
    let mut cleaned: Vec<String> = Vec::new();
    for tag in tags {
        let levels: Vec<&str> = tag
            .trim()
            .split('/')
            .filter(|level| !level.trim().is_empty())
            .collect();
        let tag: String = levels.join("/").trim().nfc().collect();
        if tag.is_empty() {
            continue;
        }
        let key = tag_key(&tag);
        if !cleaned.iter().any(|other| tag_key(other) == key) {
            cleaned.push(tag);
        }
    }
    cleaned
}

/// JSON equality that treats numbers by value, so `35`, `35.0`, and `3.5e1`
/// are the same setting.
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => match (a.as_i64(), b.as_i64()) {
            (Some(a), Some(b)) => a == b,
            _ => a.as_f64() == b.as_f64(),
        },
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same(a, b))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len() && a.iter().all(|(k, v)| b.get(k).is_some_and(|w| same(v, w)))
        }
        _ => a == b,
    }
}
