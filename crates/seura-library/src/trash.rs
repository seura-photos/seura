//! Moving files to the system Recycle Bin or Trash: how Seura removes an edit
//! file with nothing left in it (FIL-1), and later photos the user deletes
//! (BRW-9). Never a permanent deletion (CLAUDE.md rule 5).

use std::io;
use std::path::Path;

/// Moves a file to the Recycle Bin or Trash, from where the user can put it
/// back.
pub fn move_to_trash(path: &Path) -> io::Result<()> {
    #[allow(unused_mut, reason = "only macOS changes the context")]
    let mut context = trash::TrashContext::default();
    #[cfg(target_os = "macos")]
    {
        use trash::macos::{DeleteMethod, TrashContextExtMacos};
        // The crate's default scripts Finder, which makes macOS ask the user
        // to let Seura control Finder; the file manager needs no permission.
        context.set_delete_method(DeleteMethod::NsFileManager);
    }
    context.delete(path).map_err(io::Error::other)
}
