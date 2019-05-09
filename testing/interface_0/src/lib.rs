/*!

This is an example `interface crate`,
where all publically available modules(structs of function pointers) and types are declared,

This crate is where extra tests which don't belong in examples go.

To load the library and the modules together,use the `load_library` function,
which will load the dynamic library from a directory(folder),
and then all the modules inside of the library.

*/
use std::path::Path;

use abi_stable::{
    StableAbi,
    package_version_strings,
    lazy_static_ref::LazyStaticRef,
    library::{Library,LibraryError, RootModule},
    version::VersionStrings,
    std_types::{RBox, RStr, RString,RVec,RArc},
};




impl RootModule for TestingMod {
    fn raw_library_ref()->&'static LazyStaticRef<Library>{
        static RAW_LIB:LazyStaticRef<Library>=LazyStaticRef::new();
        &RAW_LIB
    }

    const BASE_NAME: &'static str = "testing";
    const NAME: &'static str = "testing";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
    const LOADER_FN: &'static str = "get_library";
}


#[repr(C)]
#[derive(StableAbi)] 
#[sabi(kind(Prefix(prefix_struct="TestingMod")))]
#[sabi(missing_field(panic))]
pub struct TestingModVal {
    #[sabi(last_prefix_field)]
    pub greeter:extern "C" fn(RStr<'_>),
    pub for_tests:extern "C" fn()->ForTests,

    /// An module used in prefix-type tests.
    pub prefix_types_tests:&'static PrefixTypeMod0,
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


// Macro used to make sure that PrefixTypeMod0 and PrefixTypeMod1 
// are changed in lockstep.
macro_rules! declare_PrefixTypeMod {
    (
        $(#[$attr:meta])*
        struct $struct_ident:ident;
        prefix_struct=$prefix:literal ;
    
        $(extra_fields=[ $($extra_fields:tt)* ])?
    ) => (
        $(#[$attr])*
        #[repr(C)]
        #[derive(StableAbi)] 
        #[sabi(kind(Prefix(prefix_struct=$prefix)))]
        #[sabi(missing_field(option))]
        pub struct $struct_ident {
            #[sabi(last_prefix_field)]
            pub field_a:u32,
            $($($extra_fields)*)?
        }
    )
}


declare_PrefixTypeMod!{
    struct PrefixTypeMod0Val;
    prefix_struct="PrefixTypeMod0";
}

declare_PrefixTypeMod!{
    /**
    This is unsafely converted from PrefixTypeMod0 in tests to check that 
    `prefix.field_a()==some_integer`,
    `prefix.field_b()==None`,
    `prefix.field_c()==None`.

    This only works because I know that both structs have the same alignment,
    if either struct alignment changed that conversion would be unsound.
    */
    struct PrefixTypeMod1Val;
    prefix_struct="PrefixTypeMod1";
    
    extra_fields=[
        pub field_b:u32,
        pub field_c:u32,
        #[sabi(missing_field(panic))]
        pub field_d:u32,
    ]
}


///////////////////////////////////////////////////////////////////////////////


/// A late-initialized reference to the global `TestingMod` instance,
///
/// Call `load_library_in` before calling `MODULES.get()` to get a `Some(_)` value back.
///
pub static MODULES:LazyStaticRef<TestingMod>=LazyStaticRef::new();


/// Loads all the modules of the library at the `directory`.
/// If it loads them once,this will continue returning the same reference.
pub fn load_library_in(directory:&Path) -> Result<&'static TestingMod,LibraryError> {
    MODULES.try_init(||{
        TestingMod::load_from_library_in(directory)
    })
}