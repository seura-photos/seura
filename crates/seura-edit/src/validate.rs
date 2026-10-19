//! Validation: the published JSON Schema, plus the rules JSON Schema can't
//! express, mirroring `tools/check.py` in the edit-format repository.

use std::sync::LazyLock;

use jsonschema::{Draft, Validator};
use serde_json::Value;

use crate::operations::order_key;

/// The schemas, from the `seura-photos/edit-format` submodule. Publishing
/// this crate on its own will need vendored copies.
const SCHEMA_1_0: &str = include_str!("../../../third_party/edit-format/schema/1.0.json");
const READER_1_X: &str = include_str!("../../../third_party/edit-format/schema/1.x-reader.json");

/// Format 1.0 exactly as published, for files whose version is "1.0".
static STRICT: LazyLock<Validator> = LazyLock::new(|| build(SCHEMA_1_0));

/// The reader schema, for files from a newer minor version (1.1 and later):
/// checks every field 1.0 defines and accepts the rest (edit format:
/// "Reading a newer minor version").
static READER: LazyLock<Validator> = LazyLock::new(|| build(READER_1_X));

fn build(schema: &str) -> Validator {
    let schema: Value =
        serde_json::from_str(schema).expect("the edit format schemas are valid JSON");
    jsonschema::options()
        .with_draft(Draft::Draft202012)
        .should_validate_formats(true)
        .build(&schema)
        .expect("the edit format schema compiles")
}

/// Every reason `doc` isn't a valid edit file; empty if it is.
/// `newer_minor` selects the reader schema, for format 1.1 and later.
pub(crate) fn problems(doc: &Value, newer_minor: bool) -> Vec<String> {
    let validator = if newer_minor { &*READER } else { &*STRICT };
    let mut problems: Vec<String> = validator
        .iter_errors(doc)
        .map(|error| {
            let path = error.instance_path().to_string();
            let path = if path.is_empty() {
                "(root)".to_string()
            } else {
                path
            };
            format!("{path}: {error}")
        })
        .collect();
    problems.extend(rule_problems(doc));
    problems
}

/// Whether a history record has been undone and not redone.
pub(crate) fn is_undone(record: &Value) -> bool {
    record.get("undone") == Some(&Value::Bool(true))
}

/// The rules the schema can't express (edit format: "Operations", "Tags and
/// rating", "History").
fn rule_problems(doc: &Value) -> Vec<String> {
    let mut problems = Vec::new();

    if let Some(operations) = doc.get("operations").and_then(Value::as_array) {
        let ids: Vec<&str> = operations
            .iter()
            .filter_map(|op| op.get("id").and_then(Value::as_str))
            .collect();
        let mut duplicates: Vec<&str> = ids
            .iter()
            .copied()
            .filter(|id| ids.iter().filter(|other| *other == id).count() > 1)
            .collect();
        duplicates.sort_unstable();
        duplicates.dedup();
        if !duplicates.is_empty() {
            problems.push(format!(
                "operation types appear more than once: {}",
                duplicates.join(", ")
            ));
        }

        // Operations whose place is unknown (core ids from a newer version)
        // are skipped; the rest must be in order.
        let keys: Vec<_> = operations.iter().filter_map(order_key).collect();
        if let Some(position) = keys.windows(2).position(|pair| pair[1] < pair[0]) {
            problems.push(format!(
                "operations are not in canonical render order (item {})",
                position + 2
            ));
        }
    }

    // Undone records always come last (edit format: "History").
    if let Some(history) = doc.get("history").and_then(Value::as_array) {
        let undone: Vec<bool> = history.iter().map(is_undone).collect();
        if undone.windows(2).any(|pair| pair[0] && !pair[1]) {
            problems.push(
                "undone history records must all come after the records that aren't undone".into(),
            );
        }
    }

    if let Some(tags) = doc.get("tags").and_then(Value::as_array) {
        let lowered: Vec<String> = tags
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_lowercase)
            .collect();
        let mut clashes: Vec<&String> = lowered
            .iter()
            .filter(|tag| lowered.iter().filter(|other| other == tag).count() > 1)
            .collect();
        clashes.sort_unstable();
        clashes.dedup();
        if !clashes.is_empty() {
            let clashes: Vec<&str> = clashes.into_iter().map(String::as_str).collect();
            problems.push(format!(
                "tags differ only in letter case: {}",
                clashes.join(", ")
            ));
        }
    }

    problems
}
