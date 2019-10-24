/*!
This crate is where extra tests which don't belong in examples go.

*/

use version_compatibility_interface::{RootMod,RootModVal};

use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_types::VersionStrings,
};
use std::marker::PhantomData;

///////////////////////////////////////////////////////////////////////////////////

#[export_root_module]
pub fn get_library() -> &'static RootMod {
    RootModVal{
        _marker:PhantomData,
        abi_stable_version:abi_stable::ABI_STABLE_VERSION,
    }.leak_into_prefix()
}