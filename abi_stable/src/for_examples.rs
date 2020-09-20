//! Types used in documentation examples.

use crate::{
    library::RootModule,
    sabi_types::VersionStrings,
    std_types::{ROption, RStr},
    StableAbi,
};


/// This type is used in prefix type examples.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = "Module_Ref", prefix_fields = "Module_Prefix")))]
pub struct Module{
    pub first: ROption<usize>,
    // The `#[sabi(last_prefix_field)]` attribute here means that this is 
    // the last field in this struct that was defined in the 
    // first compatible version of the library,
    // requiring new fields to always be added after it.
    // Moving this attribute is a breaking change, it can only be done in a 
    // major version bump..
    #[sabi(last_prefix_field)]
    pub second: RStr<'static>,
    pub third: usize,
}


impl RootModule for Module_Ref {
    crate::declare_root_module_statics!{Module_Ref}
    const BASE_NAME: &'static str = "example_root_module";
    const NAME: &'static str = "example_root_module";
    const VERSION_STRINGS: VersionStrings = crate::package_version_strings!();
}

