//! This crate is where extra tests which don't belong in examples go.

use version_compatibility_interface::{RootMod, RootMod_Ref};

use abi_stable::{export_root_module, marker_type::NonOwningPhantom, prefix_type::PrefixTypeTrait};

///////////////////////////////////////////////////////////////////////////////////

#[export_root_module]
pub fn get_library() -> RootMod_Ref {
    RootMod {
        _marker: NonOwningPhantom::DEFAULT,
        abi_stable_version: abi_stable::ABI_STABLE_VERSION,
    }
    .leak_into_prefix()
}
