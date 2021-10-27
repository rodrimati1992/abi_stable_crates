//! This crate is where extra tests which don't belong in examples go.

use testing_interface_1::{get_env_vars, ReturnWhat, TestingMod, TestingMod_Ref};

use abi_stable::{export_root_module, prefix_type::PrefixTypeTrait, std_types::RBoxError};

///////////////////////////////////////////////////////////////////////////////////

/// Exports the root module of this library.
///
/// LibHeader is used to check that the layout of `TextOpsMod` in this dynamic library
/// is compatible with the layout of it in the binary that loads this library.
#[export_root_module]
pub fn get_library() -> Result<TestingMod_Ref, RBoxError> {
    let envars = get_env_vars();

    match envars.return_what {
        ReturnWhat::Ok => {
            let ret = TestingMod { a: 5, b: 8, c: 13 }.leak_into_prefix();

            Ok(ret)
        }
        ReturnWhat::Error => Err(RBoxError::from_fmt("What the ....?")),
        ReturnWhat::Panic => {
            panic!()
        }
    }
}
