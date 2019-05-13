use super::*;

#[derive(StableAbi)]
#[repr(C)]
#[sabi(inside_abi_stable_crate)]
#[sabi(kind(Prefix(prefix_struct="EmptyPrefixType")))]
#[sabi(missing_field(panic))]
pub struct EmptyPrefixTypeVal {
    #[sabi(last_prefix_field)]
    _empty:()
}


impl EmptyPrefixType{
    const VALUE:EmptyPrefixTypeVal=
        EmptyPrefixTypeVal{ _empty:() };

    // The VTABLE for this type in this executable/library
    pub const NEW: &'static WithMetadata<EmptyPrefixTypeVal> = 
        &WithMetadata::new(PrefixTypeTrait::METADATA,Self::VALUE);

}