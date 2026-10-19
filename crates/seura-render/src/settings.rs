//! The settings a render uses, taken from an edit file.

use seura_edit::{CoreOperation, EditDocument};

/// The operations' parameters, neutral by default. Only exposure so far
/// (M1); the others arrive with the Editor in M4.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Settings {
    /// `seura.exposure`'s `ev`, in stops.
    pub exposure_ev: f32,
}

impl Settings {
    /// Disabled operations are skipped, as are ones Seura can't render
    /// (edit format: "Operations").
    pub fn from_document(document: &EditDocument) -> Settings {
        let mut settings = Settings::default();
        for operation in document.operations() {
            if !operation.enabled {
                continue;
            }
            if let Some(CoreOperation::Exposure(exposure)) = operation.core {
                settings.exposure_ev = exposure.ev as f32;
            }
        }
        settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(operations: &str) -> EditDocument {
        let json = format!(
            r#"{{"format": "seura-edit", "formatVersion": "1.0",
                "photo": {{"file": "a.jpg", "size": 1}}, "operations": {operations}}}"#
        );
        EditDocument::parse(json.as_bytes()).unwrap()
    }

    #[test]
    fn exposure_comes_from_the_file() {
        let doc = document(r#"[{"id": "seura.exposure", "version": 1, "params": {"ev": 0.5}}]"#);
        assert_eq!(Settings::from_document(&doc).exposure_ev, 0.5);
    }

    #[test]
    fn a_disabled_operation_is_skipped() {
        let doc = document(
            r#"[{"id": "seura.exposure", "version": 1, "enabled": false, "params": {"ev": 0.5}}]"#,
        );
        assert_eq!(Settings::from_document(&doc), Settings::default());
    }
}
