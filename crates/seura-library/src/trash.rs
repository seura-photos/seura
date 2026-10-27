//! Moving files to the system Recycle Bin or Trash: how Seura removes an edit
//! file with nothing left in it (FIL-1), and photos the user deletes
//! (BRW-9). Never a permanent deletion (CLAUDE.md rule 5).

use std::io;
use std::path::{Path, PathBuf};

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
    context().delete_all(paths).map_err(io::Error::other)
}

fn refuse_without_trash(paths: &[PathBuf], has_trash: &dyn Fn(&Path) -> bool) -> io::Result<()> {
    match paths.iter().find(|path| !has_trash(path)) {
        Some(path) => Err(io::Error::other(format!(
            "{} is on a drive with no Recycle Bin, so it's left where it is",
            path.display()
        ))),
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
    fn nothing_goes_if_any_file_is_on_a_drive_with_no_recycle_bin() {
        let usb = Path::new(r"E:\IMG_2.jpg");
        let paths = [PathBuf::from(r"C:\IMG_1.jpg"), usb.to_path_buf()];
        let error = refuse_without_trash(&paths, &|path| path != usb).unwrap_err();
        assert!(error.to_string().contains("no Recycle Bin"));
        assert!(refuse_without_trash(&paths, &|_| true).is_ok());
    }
}
