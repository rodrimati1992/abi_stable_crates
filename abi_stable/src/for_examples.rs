use crate::{
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