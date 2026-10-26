//! Text the core shows itself, such as menu labels, kept outside the code
//! in a Project Fluent catalog, the same format as the UI's (ACC-5).
//! English only in v1.

use std::borrow::Cow;
use std::sync::LazyLock;

pub use fluent_bundle::FluentValue;
use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};

static BUNDLE: LazyLock<FluentBundle<FluentResource>> = LazyLock::new(|| {
    let resource = FluentResource::try_new(include_str!("../messages/en.ftl").to_string())
        .expect("messages/en.ftl parses");
    let mut bundle =
        FluentBundle::new_concurrent(vec!["en".parse().expect("a valid language tag")]);
    // Bidirectional isolation marks are for right-to-left languages, which
    // come after v1; without them, text stays plain in menus and tests.
    bundle.set_use_isolating(false);
    #[cfg(any(target_os = "macos", test))]
    if crate::locale::pseudo() {
        bundle.set_transform(Some(|text: &str| {
            Cow::Owned(crate::locale::pseudo_translate(text))
        }));
    }
    bundle
        .add_resource(resource)
        .expect("messages/en.ftl has no duplicate messages");
    bundle
});

/// The text for message `id`. A missing message is a bug, caught by the
/// test below for every id the core uses.
#[cfg_attr(
    not(any(target_os = "macos", test)),
    expect(dead_code, reason = "only the macOS menu uses text so far")
)]
pub fn t(id: &str) -> String {
    t_with(id, &[])
}

/// The text for message `id`, with its variables. Numbers are passed as
/// numbers, so plurals work.
#[cfg_attr(
    not(any(target_os = "macos", test)),
    expect(dead_code, reason = "only the macOS menu uses text so far")
)]
pub fn t_with(id: &str, values: &[(&str, FluentValue)]) -> String {
    let message = BUNDLE
        .get_message(id)
        .and_then(|message| message.value())
        .unwrap_or_else(|| panic!("no message {id}"));
    let args: FluentArgs = values.iter().cloned().collect();
    let mut errors = Vec::new();
    match BUNDLE.format_pattern(message, Some(&args), &mut errors) {
        Cow::Borrowed(text) => text.to_string(),
        Cow::Owned(text) => text,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_message_the_core_uses_exists() {
        let sources = [include_str!("menu.rs")];
        let ids = sources
            .iter()
            .flat_map(|source| {
                let plain = source.split("t(\"").skip(1);
                plain.chain(source.split("t_with(\"").skip(1))
            })
            .map(|rest| rest.split('"').next().unwrap());
        let mut count = 0;
        for id in ids {
            assert!(super::BUNDLE.has_message(id), "messages/en.ftl lacks {id}");
            count += 1;
        }
        assert!(count > 10, "the menu's messages were found");
    }

    #[test]
    fn the_edit_menu_says_what_undo_will_undo() {
        use super::FluentValue;
        let label = |kind: &str, tag: &str, count: u32| {
            super::t_with(
                "menu-undo-action",
                &[
                    ("kind", FluentValue::from(kind)),
                    ("tag", FluentValue::from(tag)),
                    ("count", FluentValue::from(count)),
                ],
            )
        };
        assert_eq!(label("rating", "", 5), "Undo Rating (5 photos)");
        assert_eq!(label("rating", "", 1), "Undo Rating (1 photo)");
        assert_eq!(
            label("addTag", "beach", 3),
            "Undo Add Tag “beach” (3 photos)"
        );
    }

    #[test]
    fn messages_are_plain_text() {
        assert_eq!(super::t("menu-add-folder"), "Add Folder…");
    }
}
