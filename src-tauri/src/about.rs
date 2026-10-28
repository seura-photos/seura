//! About Seura Photos (SET-5): the version, a line saying it's an early
//! build while below 1.0, and links to the website, the project, and the
//! license. On Windows the UI shows it in a dialog from the toolbar's ⋯
//! button; on macOS it's the system's About panel, from the app menu
//! (ACC-4).

use serde::Deserialize;
use specta::Type;

/// The app's version, from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// A link in About. The UI names one of these rather than passing an
/// address, so it can't open any other.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub enum Link {
    Website,
    Project,
    License,
}

impl Link {
    #[cfg(any(target_os = "macos", test))]
    pub const ALL: [Link; 3] = [Link::Website, Link::Project, Link::License];

    pub fn url(self) -> &'static str {
        match self {
            Link::Website => "https://seuraphotos.app",
            Link::Project => "https://github.com/seura-photos/seura",
            Link::License => "https://github.com/seura-photos/seura/blob/main/LICENSE",
        }
    }
}

/// What the system's About panel shows besides the name and icon, which it
/// takes from the app bundle. The panel shows its credits as plain text, so
/// the links are there as addresses, not to click.
#[cfg(target_os = "macos")]
pub fn metadata() -> tauri::menu::AboutMetadata<'static> {
    use crate::messages::t;

    let mut credits = Vec::new();
    // An early build, before the beta: any version below 1.0.
    if VERSION.starts_with("0.") {
        credits.push(t("about-early-build"));
        credits.push(String::new());
    }
    credits.extend(Link::ALL.map(|link| link.url().trim_start_matches("https://").to_string()));
    tauri::menu::AboutMetadata {
        version: Some(VERSION.to_string()),
        credits: Some(credits.join("\n")),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_link_is_secure_and_seuras_own() {
        for link in Link::ALL {
            let url = link.url();
            assert!(
                url == "https://seuraphotos.app"
                    || url.starts_with("https://github.com/seura-photos/seura"),
                "{url}"
            );
        }
    }
}
