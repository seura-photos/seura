//! Validation: the published JSON Schema, plus the rules JSON Schema can't
//! express, mirroring `tools/check.py` in the edit-format repository.

use std::sync::LazyLock;

use jsonschema::{Draft, Validator};
use serde_json::{Value, json};

use crate::operations::order_key;

/// The schema for format 1.0, from the `seura-photos/edit-format` submodule.
/// Publishing this crate on its own will need a vendored copy.
const SCHEMA_1_0: &str = include_str!("../../../third_party/edit-format/schema/1.0.json");

/// Format 1.0 exactly as published.
static STRICT: LazyLock<Validator> = LazyLock::new(|| build(strict_schema()));

/// For files from a newer minor version (1.1 and later), which may add
/// optional fields and core operations this version doesn't know. Everything
/// 1.0 defines is still checked; what it doesn't define is allowed, and kept.
static RELAXED: LazyLock<Validator> = LazyLock::new(|| build(relaxed_schema()));

fn strict_schema() -> Value {
    serde_json::from_str(SCHEMA_1_0).expect("the edit format schema is valid JSON")
}

fn relaxed_schema() -> Value {
    let mut schema = strict_schema();
    schema["properties"]["formatVersion"] = json!({ "type": "string", "pattern": "^1\\.[0-9]+$" });
    schema["additionalProperties"] = json!(true);
    let defs = &mut schema["$defs"];
    defs["photo"]["additionalProperties"] = json!(true);
    defs["operation"]["additionalProperties"] = json!(true);
    defs["historyRecord"]["additionalProperties"] = json!(true);
    // Core ids a newer version adds are allowed; their params are its business.
    if let Some(then) = defs["operation"]["then"]["properties"].as_object_mut() {
        then.remove("id");
    }
    schema
}

fn build(schema: Value) -> Validator {
    jsonschema::options()
        .with_draft(Draft::Draft202012)
        .should_validate_formats(true)
        .build(&schema)
        .expect("the edit format schema compiles")
}

/// Every reason `doc` isn't a valid edit file; empty if it is.
/// `newer_minor` selects the relaxed checks for format 1.1 and later.
pub(crate) fn problems(doc: &Value, newer_minor: bool) -> Vec<String> {
    let validator = if newer_minor { &*RELAXED } else { &*STRICT };
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

/// The rules the schema can't express (edit format: "Operations", "Tags and rating").
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
