use std::{
    io,
    path::{Path, PathBuf},
};

use abi_stable::library::{abi_header_from_path, AbiHeader, LibraryError, RootModule};

use core_extensions::SelfOps;

use version_compatibility_interface::RootMod_Ref;

/// Returns the path the library will be loaded from.
fn compute_library_dir() -> io::Result<PathBuf> {
    let debug_dir = "../../../target/debug/"
        .as_ref_::<Path>()
        .into_::<PathBuf>();
    let release_dir = "../../../target/release/"
        .as_ref_::<Path>()
        .into_::<PathBuf>();

    let debug_path = RootMod_Ref::get_library_path(&debug_dir);
    let release_path = RootMod_Ref::get_library_path(&release_dir);

    match (debug_path.exists(), release_path.exists()) {
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
    }
    .piped(Ok)
}

fn main() -> io::Result<()> {
    let library_dir = compute_library_dir().unwrap();

    (|| -> Result<(), LibraryError> {
        let header = abi_header_from_path(&RootMod_Ref::get_library_path(&library_dir))?;

        println!("header: {:?}", header);
        println!();
        println!("Executable's AbiHeader {:?}", AbiHeader::VALUE);
        println!();
        println!(
            "Executable's abi_stable version {:?}",
            abi_stable::ABI_STABLE_VERSION
        );
        println!();

        if header.is_valid() {
            let lib_header = header.upgrade()?;

            unsafe {
                let root = lib_header.init_root_module_with_unchecked_layout::<RootMod_Ref>()?;
                println!("Loaded abi_stable version {:?}", root.abi_stable_version());
                println!();
            }

            lib_header.check_layout::<RootMod_Ref>()?;
            println!(
                "\
                The types in abi_stable on crates.io are compatible with those on \
                the \"local\" repository\
            "
            );
        } else {
            println!("The abi_stable on crates.io isn't semver compatible with this one");
        }
        Ok(())
    })()
    .unwrap_or_else(|e| panic!("{}", e));

    Ok(())
}
