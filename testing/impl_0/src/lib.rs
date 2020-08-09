/*!
This crate is where extra tests which don't belong in examples go.

*/

use testing_interface_0::{
    TestingMod,TestingMod_Ref,ForTests,PrefixTypeMod0,
};

use abi_stable::{
    export_root_module,
    extern_fn_panic_handling, 
    prefix_type::PrefixTypeTrait,
    traits::{IntoReprC},
    std_types::{RStr,RBox,RVec,RArc, RString}, 
};
#[allow(unused_imports)]
use core_extensions::{SelfOps};

///////////////////////////////////////////////////////////////////////////////////


/// Exports the root module of this library.
///
/// LibHeader is used to check that the layout of `TextOpsMod` in this dynamic library
/// is compatible with the layout of it in the binary that loads this library.
#[export_root_module]
pub fn get_library() -> TestingMod_Ref {
    TestingMod{
        greeter,
        for_tests,
        prefix_types_tests:PrefixTypeMod0{
            field_a:123,
        }.leak_into_prefix(),
    }.leak_into_prefix()
}


pub extern "C" fn greeter(name:RStr<'_>){
    extern_fn_panic_handling!{
        println!("Hello, {}!", name);
    }
}

pub extern "C" fn for_tests()->ForTests{
    extern_fn_panic_handling!{
        let arc=RArc::new(RString::from("hello"));
        ::std::mem::forget(arc.clone());
        let box_=RBox::new(10);
        let vec_=RVec::from(vec!["world".into_c()]);
        let string=RString::from("what the foo.");
        ForTests{
            arc_address:(&*arc) as *const _ as usize,
            arc,

            box_address:(&*box_) as *const _ as usize,
            box_,
            
            vec_address:vec_.as_ptr() as usize,
            vec_,
            
            string_address:string.as_ptr() as usize,
            string,
        }
    }
}
