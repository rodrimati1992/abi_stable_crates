//! Utilities for use while developing dynamic libraries.

use std::{
    io,
    path::{Path, PathBuf},
};

use crate::library::RootModule;

/// Returns the path in the target directory
/// to the last version of an implementation crate's dynamic library.
///
/// The path can be in either the "debug" or "release" subdirectories.
pub fn compute_library_path<M: RootModule>(target_path: &Path) -> io::Result<PathBuf> {
    let debug_dir = target_path.join("debug/");
    let release_dir = target_path.join("release/");

    let debug_path = M::get_library_path(&debug_dir);
    let release_path = M::get_library_path(&release_dir);

    Ok(match (debug_path.exists(), release_path.exists()) {
        (false, false) => debug_dir,
        (true, false) => debug_dir,
        (false, true) => release_dir,
        (true, true) => {
            if debug_path.metadata()?.modified()? < release_path.metadata()?.modified()? {
                release_dir
            } else {
                debug_dir
            }
        }
    })
}
