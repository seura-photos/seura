//! Text the core shows itself, such as menu labels, kept outside the code
//! like the UI's (ACC-5). English only in v1.

use std::collections::HashMap;
use std::sync::LazyLock;

static EN: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../messages/en.json"))
        .expect("messages/en.json is a flat object of strings")
});

/// The text for `key`. A missing key is a bug, caught by the test below
/// for every key the core uses.
#[cfg_attr(
    not(target_os = "macos"),
    expect(dead_code, reason = "only the macOS menu uses text so far")
)]
pub fn t(key: &str) -> &'static str {
    EN.get(key)
        .map(String::as_str)
        .unwrap_or_else(|| panic!("no message for {key}"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_key_the_core_uses_has_text() {
        let sources = [include_str!("menu.rs")];
        let keys = sources
            .iter()
            .flat_map(|source| source.split("t(\"").skip(1))
            .map(|rest| rest.split('"').next().unwrap());
        for key in keys {
            assert!(super::EN.contains_key(key), "messages/en.json lacks {key}");
        }
    }
}
