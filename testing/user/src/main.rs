use std::{
    path::{Path,PathBuf},
    io,
    sync::Arc,
};


use core_extensions::SelfOps;

use abi_stable::{
    std_types::{RString,RVec,RArc,RBox},
    library::RootModule,
};

use testing_interface_0::{TestingMod,PrefixTypeMod0,PrefixTypeMod1};



/// Returns the path the library will be loaded from.
fn compute_library_path()->io::Result<PathBuf>{
    let debug_dir  ="../../target/debug/"  .as_ref_::<Path>().into_(PathBuf::T);
    let release_dir="../../target/release/".as_ref_::<Path>().into_(PathBuf::T);

    let debug_path  =TestingMod::get_library_path(&debug_dir);
    let release_path=TestingMod::get_library_path(&release_dir);

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


/// For transmuting a reference.
unsafe fn transmute_reference<T,U>(ref_:&T)->&U{
    &*(ref_ as *const T as *const U)
}


fn main()-> io::Result<()> {
    let library_path=compute_library_path().unwrap();
    let mods=TestingMod::load_from_directory(&library_path)
        .unwrap_or_else(|e| panic!("{}", e) );
    
    run_dynamic_library_tests(mods);

    Ok(())
}


/// This tests that a type coming from a dynamic library 
/// cannot be converted back to its std-library equivalent
/// while reusing the heap allocation.
///
/// The reason why they can't reuse the heap allocation is because they might
/// be using a different global allocator that this binary is using.
///
/// There is no way that I am aware to check at compile-time what allocator
/// the type is using,so this is the best I can do while staying safe.
pub fn run_dynamic_library_tests(mods:&'static TestingMod){
    {
        let hw=mods.prefix_types_tests();
        let hw=unsafe{
            // This only works because I know that both structs have the same alignment,
            // if either struct alignment changed that conversion would be unsound.
            transmute_reference::<PrefixTypeMod0,PrefixTypeMod1>(hw)
        };

        assert_eq!(hw.field_a(),123);
        assert_eq!(hw.field_b(),None);
        assert_eq!(hw.field_c(),None);
        let res=::std::panic::catch_unwind(||{
            let _=hw.field_d();
        });
        assert!(res.is_err(),"value:{:#?}",res);
    }

    let val=mods.for_tests()();
    {
        let arc_std=val.arc.piped(RArc::into_arc);
        assert_eq!(Arc::strong_count(&arc_std),1);
        assert_ne!(
            (&*arc_std) as *const _ as usize,
            val.arc_address
        );
    }
    {
        let box_std=val.box_.piped(RBox::into_box);
        assert_ne!(
            (&*box_std) as *const _ as usize,
            val.box_address
        );
    }
    {
        let vec_std=val.vec_.piped(RVec::into_vec);
        assert_ne!(
            vec_std.as_ptr() as usize,
            val.vec_address
        );
    }
    {
        let string_std=val.string.piped(RString::into_string);
        assert_ne!(
            string_std.as_ptr() as usize,
            val.string_address
        );
    }
    
    println!();
    println!(".-------------------------.");
    println!("|     tests succeeded!    |");
    println!("'-------------------------'");
}

