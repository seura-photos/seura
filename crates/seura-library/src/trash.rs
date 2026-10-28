//! Moving files to the system Recycle Bin or Trash: how Seura removes an edit
//! file with nothing left in it (FIL-1), and photos the user deletes
//! (BRW-9). Never a permanent deletion (CLAUDE.md rule 5).

use std::io;
use std::path::{Path, PathBuf};

use seura_edit::NoTrash;

use crate::platform::has_trash;

/// Moves a file to the Recycle Bin or Trash, from where the user can put it
/// back.
pub fn move_to_trash(path: &Path) -> io::Result<()> {
    move_all_to_trash(&[path.to_path_buf()])
}

/// Moves several files to the Recycle Bin or Trash in one operation, not
/// one each, which on Windows takes a fifth of a second apiece. If it fails
/// partway, some may have moved and some not. Nothing moves if any of them
/// is on a drive with no Recycle Bin, such as a USB stick: there Windows
/// would delete it for good, without asking, since the `trash` crate asks
/// for no prompts (#115).
pub fn move_all_to_trash(paths: &[PathBuf]) -> io::Result<()> {
    refuse_without_trash(paths, &has_trash)?;
    context()
        .delete_all(paths)
        .map_err(|error| io::Error::other(system_reason(error)))
}

/// The system's own words for why it didn't move a file, such as "you
/// don't have permission to access it", without the `trash` crate's
/// preface, so they can be shown to the user (#112).
fn system_reason(error: trash::Error) -> String {
    match error {
        trash::Error::Unknown { description } | trash::Error::Os { description, .. } => {
            let words = description
                .rsplit_once("failed: ")
                .map(|(_, words)| words)
                .or_else(|| description.strip_prefix("windows error: "))
                .unwrap_or(&description);
            words.trim().to_string()
        }
        other => other.to_string(),
    }
}

/// Refuses with [`NoTrash`], which lets an emptied edit file be deleted
/// directly instead (decision 0017).
fn refuse_without_trash(paths: &[PathBuf], has_trash: &dyn Fn(&Path) -> bool) -> io::Result<()> {
    match paths.iter().find(|path| !has_trash(path)) {
        Some(path) => Err(NoTrash::error(path)),
        None => Ok(()),
    }
}

fn context() -> trash::TrashContext {
    #[allow(unused_mut, reason = "only macOS changes the context")]
    let mut context = trash::TrashContext::default();
    #[cfg(target_os = "macos")]
    {
        use trash::macos::{DeleteMethod, TrashContextExtMacos};
        // The crate's default scripts Finder, which makes macOS ask the user
        // to let Seura control Finder; the file manager needs no permission.
        context.set_delete_method(DeleteMethod::NsFileManager);
    }
    context
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_reason_is_the_systems_own_words() {
        let mac = trash::Error::Unknown {
            description: concat!(
                r#"While deleting '"/Users/n/OneDrive/IMG_1.jpg"', `trashItemAtURL` failed: "#,
                "“IMG_1.jpg” couldn’t be moved to the trash because you don’t have ",
                "permission to access it.",
            )
            .into(),
        };
        assert_eq!(
            system_reason(mac),
            "“IMG_1.jpg” couldn’t be moved to the trash because you don’t have permission to access it."
        );
        let windows = trash::Error::Os {
            code: 5,
            description: "windows error: Access is denied. (0x80070005)".into(),
        };
        assert_eq!(system_reason(windows), "Access is denied. (0x80070005)");
        let plain = trash::Error::Unknown {
            description: "Some operations were aborted".into(),
        };
        assert_eq!(system_reason(plain), "Some operations were aborted");
    }

    #[test]
    fn nothing_goes_if_any_file_is_on_a_drive_with_no_recycle_bin() {
        let usb = Path::new(r"E:\IMG_2.jpg");
        let paths = [PathBuf::from(r"C:\IMG_1.jpg"), usb.to_path_buf()];
        let error = refuse_without_trash(&paths, &|path| path != usb).unwrap_err();
        assert!(NoTrash::is(&error));
        assert!(refuse_without_trash(&paths, &|_| true).is_ok());
    }
}
