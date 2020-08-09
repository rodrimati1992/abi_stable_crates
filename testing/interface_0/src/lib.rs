/*!

This is an example `interface crate`,
where all publically available modules(structs of function pointers) and types are declared,

This crate is where extra tests which don't belong in examples go.

To load the library and the modules together,
call `<TestingMod_Ref as RootModule>::load_from_directory`,
which will load the dynamic library from a directory(folder),
and then all the modules inside of the library.

*/


use abi_stable::{
    StableAbi,
    package_version_strings,
    library::RootModule,
    sabi_types::VersionStrings,
    std_types::{RBox, RStr, RString,RVec,RArc},
};




impl RootModule for TestingMod_Ref {
    abi_stable::declare_root_module_statics!{TestingMod_Ref}

    const BASE_NAME: &'static str = "testing";
    const NAME: &'static str = "testing";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}


#[repr(C)]
#[derive(StableAbi)] 
#[sabi(kind(Prefix(prefix_ref="TestingMod_Ref")))]
#[sabi(missing_field(panic))]
pub struct TestingMod {
    #[sabi(last_prefix_field)]
    pub greeter:extern "C" fn(RStr<'_>),
    pub for_tests:extern "C" fn()->ForTests,

    /// An module used in prefix-type tests.
    pub prefix_types_tests: PrefixTypeMod0_Ref,
}


/// This type is used in tests between the interface and user crates.
#[repr(C)]
#[derive(StableAbi)] 
pub struct ForTests{
    pub arc:RArc<RString>,
    pub arc_address:usize,

    pub box_:RBox<u32>,
    pub box_address:usize,
    
    pub vec_:RVec<RStr<'static>>,
    pub vec_address:usize,
    
    pub string:RString,
    pub string_address:usize,
}


// Macro used to make sure that PrefixTypeMod0_Ref and PrefixTypeMod1 
// are changed in lockstep.
macro_rules! declare_PrefixTypeMod {
    (
        $(#[$attr:meta])*
        struct $struct_ident:ident;
        prefix_ref=$prefix:literal ;
    
        $(extra_fields=[ $($extra_fields:tt)* ])?
    ) => (
        $(#[$attr])*
        #[repr(C)]
        #[derive(StableAbi)] 
        #[sabi(kind(Prefix(prefix_ref=$prefix)))]
        #[sabi(missing_field(option))]
        pub struct $struct_ident {
            #[sabi(last_prefix_field)]
            pub field_a:u32,
            $($($extra_fields)*)?
        }
    )
}


declare_PrefixTypeMod!{
    struct PrefixTypeMod0;
    prefix_ref="PrefixTypeMod0_Ref";
}

declare_PrefixTypeMod!{
    /**
    This is unsafely converted from PrefixTypeMod0_Ref in tests to check that 
    `prefix.field_a()==some_integer`,
    `prefix.field_b()==None`,
    `prefix.field_c()==None`.

    This only works because I know that both structs have the same alignment,
    if either struct alignment changed that conversion would be unsound.
    */
    struct PrefixTypeMod1;
    prefix_ref="PrefixTypeMod1_Ref";
    
    extra_fields=[
        pub field_b:u32,
        pub field_c:u32,
        #[sabi(missing_field(panic))]
        pub field_d:u32,
    ]
}

