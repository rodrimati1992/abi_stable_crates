/*!
This crate is where extra tests which don't belong in examples go.

*/

use testing_interface_1::{TestingMod,TestingMod_Ref, get_env_vars};

use abi_stable::{
    export_root_module,
    extern_fn_panic_handling, 
    prefix_type::PrefixTypeTrait,
};

///////////////////////////////////////////////////////////////////////////////////


/// Exports the root module of this library.
///
/// LibHeader is used to check that the layout of `TextOpsMod` in this dynamic library
/// is compatible with the layout of it in the binary that loads this library.
#[export_root_module]
pub fn get_library() -> TestingMod_Ref {
    let envars = get_env_vars();

    println!("lib: {:?}", envars);

    TestingMod{
        a: 5,
        b: 8,
        c: 13,
    }.leak_into_prefix()
}

