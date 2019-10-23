use std::{
    path::{Path,PathBuf},
    io,
};

use abi_stable::{
    library::{AbiHeader,RootModule,LibraryError,abi_header_from_path},
};

use core_extensions::SelfOps;

use version_compatibility_interface::RootMod;



/// Returns the path the library will be loaded from.
fn compute_library_dir()->io::Result<PathBuf>{
    let debug_dir  ="../../../target/debug/"  .as_ref_::<Path>().into_(PathBuf::T);
    let release_dir="../../../target/release/".as_ref_::<Path>().into_(PathBuf::T);

    let debug_path  =RootMod::get_library_path(&debug_dir);
    let release_path=RootMod::get_library_path(&release_dir);

    match (debug_path.exists(),release_path.exists()) {
        (false,false)=>debug_dir,
        (true,false)=>debug_dir,
        (false,true)=>release_dir,
        (true,true)=>{
            if debug_path.metadata()?.modified()? < release_path.metadata()?.modified()? {
                release_dir
            }else{
                debug_dir
            }
        }
    }.piped(Ok)
}


fn main()-> io::Result<()> {
    let library_dir=compute_library_dir().unwrap();

    (||->Result<(),LibraryError>{
        let header=abi_header_from_path(&RootMod::get_library_path(&library_dir))?;

        println!("Executable's AbiHeader {:?}", AbiHeader::VALUE);
        println!("Executable's abi_stable version {:?}", abi_stable::ABI_STABLE_VERSION);

        println!();

        if header.is_valid() {
            let lib_header=header.upgrade()?;
            
            println!("Loaded AbiHeader {:?}", header);

            unsafe{
                let root=lib_header.init_root_module_with_unchecked_layout::<RootMod>()?;
                println!("Loaded abi_stable version {:?}", root.abi_stable_version());
            }


            lib_header.check_layout::<RootMod>()?;
        }
        Ok(())
    })().unwrap_or_else(|e| panic!("{}", e) );

    Ok(())
}