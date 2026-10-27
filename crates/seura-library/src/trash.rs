//! Moving files to the system Recycle Bin or Trash: how Seura removes an edit
//! file with nothing left in it (FIL-1), and photos the user deletes
//! (BRW-9). Never a permanent deletion (CLAUDE.md rule 5).

use std::io;
use std::path::{Path, PathBuf};

/// Moves a file to the Recycle Bin or Trash, from where the user can put it
/// back.
pub fn move_to_trash(path: &Path) -> io::Result<()> {
    context().delete(path).map_err(io::Error::other)
}

/// Moves several files to the Recycle Bin or Trash in one operation, not
/// one each, which on Windows takes a fifth of a second apiece. If it fails
/// partway, some may have moved and some not.
pub fn move_all_to_trash(paths: &[PathBuf]) -> io::Result<()> {
    context().delete_all(paths).map_err(io::Error::other)
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
