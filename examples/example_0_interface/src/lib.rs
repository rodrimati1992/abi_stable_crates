/*!
This is an example `interface crate`,
where all publically available modules(structs of function pointers) and types are declared,

To load the library and the modules together,use the `load_library` function,
which will load the dynamic library from a directory(folder),
and then all the modules inside of the library.

*/
use std::path::Path;

use abi_stable::{
    StableAbi,
    impl_InterfaceType,
    package_version_strings,
    lazy_static_ref::LazyStaticRef,
    library::{Library,LibraryError, RootModule},
    version::VersionStrings,
    type_level::bools::*,
    erased_types::{InterfaceType,DeserializeOwnedInterface,DeserializeBorrowedInterface},
    DynTrait,
    std_types::{RBox, RStr, RString,RVec,RArc, RSlice,RCow,RBoxError,RResult},
};




///////////////////////////////////////////////


/// An `InterfaceType` describing which traits are implemented by TOStateBox.
#[repr(C)]
#[derive(StableAbi)]
pub struct TOState;

/// The state passed to most functions in the TextOpsMod module.
pub type TOStateBox = DynTrait<'static,RBox<()>,TOState>;

// This macro is used to emulate default associated types.
// Look for the docs of InterfaceType to see 
// which other associated types you can define.
impl_InterfaceType!{
    impl InterfaceType for TOState {
        type Send=False;
        type Debug = True;
        type Serialize = True;
        type Deserialize = True;
        type PartialEq = True;
    }
}


impl DeserializeOwnedInterface<'static> for TOState {
    type Deserialized = TOStateBox;

    fn deserialize_impl(s: RStr<'_>) -> Result<Self::Deserialized, RBoxError> {
        MODULES.get().unwrap().deserializers().deserialize_state()(s).into_result()
    }
}


///////////////////////////////////////////////////////////////////////////////


/// An `InterfaceType` describing which traits are implemented by TOCommandBox.
#[repr(C)]
#[derive(StableAbi)]
pub struct TOCommand;

/// A de/serializable opaque command enum,used in the TextOpsMod::run_command function.
pub type TOCommandBox<'borr> = DynTrait<'borr,RBox<()>,TOCommand>;


impl_InterfaceType!{
    impl InterfaceType for TOCommand {
        type Debug = True;
        type Serialize = True;
        type Deserialize = True;
        type PartialEq = True;
    }
}


impl<'borr> DeserializeOwnedInterface<'borr> for TOCommand {
    type Deserialized = TOCommandBox<'borr>;

    fn deserialize_impl(s: RStr<'_>) -> Result<Self::Deserialized, RBoxError> {
        MODULES.get().unwrap().deserializers().deserialize_command()(s).into_result()
    }
}

impl<'borr> DeserializeBorrowedInterface<'borr> for TOCommand {
    type Deserialized = TOCommandBox<'borr>;

    fn deserialize_impl(s: RStr<'borr>) -> Result<Self::Deserialized, RBoxError> {
        MODULES.get().unwrap().deserializers().deserialize_command_borrowing()(s).into_result()
    }
}


///////////////////////////////////////////////////////////////////////////////


/// An `InterfaceType` describing which traits are implemented by TOReturnValueArc.
#[repr(C)]
#[derive(StableAbi)]
pub struct TOReturnValue;

/// A de/serializable opaque command enum,returned by the TextOpsMod::run_command function.
pub type TOReturnValueArc = DynTrait<'static,RArc<()>,TOReturnValue>;


impl_InterfaceType!{
    impl InterfaceType for TOReturnValue {
        type Debug = True;
        type Serialize = True;
        type Deserialize = True;
        type PartialEq = True;
    }
}


impl DeserializeOwnedInterface<'static> for TOReturnValue {
    type Deserialized = TOReturnValueArc;

    fn deserialize_impl(s: RStr<'_>) -> Result<Self::Deserialized, RBoxError> {
        MODULES.get().unwrap().deserializers().deserialize_return_value()(s).into_result()
    }
}


///////////////////////////////////////////////////////////////////////////////



/// The parameters for every `TextOpsMod.remove_words_*` function.
#[repr(C)]
#[derive(StableAbi)] 
pub struct RemoveWords<'a,S:'a>{
    /// The string we're processing.
    pub string:RStr<'a>,
    /// The words that will be removed from self.string.
    pub words:RSlice<'a,S>,
}


///////////////////////////////////////////////////////////////////////////////

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


///////////////////////////////////////////////////////////////////////////////


/// The root module of the `text_operations` dynamic library.
/// With all the functions/modules related to processing text.
///
/// To construct this module,
/// call <TextOpsMod_Prefix as ModuleTrait>::load_from_library_in(some_directory_path)
#[repr(C)]
#[derive(StableAbi)] 
#[sabi(kind(Prefix(prefix_struct="TextOpsMod_Prefix")))]
//#[sabi(debug_print)]
#[sabi(missing_field(panic))]
pub struct TextOpsMod {
    /// Constructs TOStateBox,state that is passed to other functions in this module.
    pub new: extern "C" fn() -> TOStateBox,
    
    /// An example module.
    pub hello_world:&'static HelloWorldMod_Prefix,

    #[sabi(last_prefix_field)]    
    pub deserializers:&'static DeserializerMod_Prefix,

    /// Reverses the order of the lines.
    pub reverse_lines: extern "C" fn(&mut TOStateBox,RStr<'_>) -> RString,
    
    /// Removes the `param.words` words from the `param.string` string.
    pub remove_words_cow: 
        for<'a>extern "C" fn(&mut TOStateBox,param:RemoveWords<'a,RCow<'a,RStr<'a>>>) -> RString,
    
    /// Removes the `param.words` words from the `param.string` string.
    pub remove_words_str: extern "C" fn(&mut TOStateBox,param:RemoveWords<RStr>) -> RString,
    
    /// Removes the `param.words` words from the `param.string` string.
    pub remove_words_string: extern "C" fn(&mut TOStateBox,param:RemoveWords<RString>) -> RString,

    /// Gets the ammount (in bytes) of text that was processed
    pub get_processed_bytes: extern "C" fn(&TOStateBox) -> u64,
 
    pub run_command: extern "C" fn(&mut TOStateBox,command:TOCommandBox<'_>)->TOReturnValueArc,

    /// An module used in prefix-type tests.
    pub prefix_types_tests:&'static PrefixTypeMod0_Prefix,
}


impl RootModule for TextOpsMod_Prefix {
    fn raw_library_ref()->&'static LazyStaticRef<Library>{
        static RAW_LIB:LazyStaticRef<Library>=LazyStaticRef::new();
        &RAW_LIB
    }

    const BASE_NAME: &'static str = "text_operations";
    const NAME: &'static str = "text_operations";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
    const LOADER_FN: &'static str = "get_library";
}


/// An example sub-module in the text_operations dynamic library.
#[repr(C)]
#[derive(StableAbi)] 
#[sabi(kind(Prefix(prefix_struct="HelloWorldMod_Prefix")))]
#[sabi(missing_field(panic))]
pub struct HelloWorldMod {
    #[sabi(last_prefix_field)]
    pub greeter:extern "C" fn(RStr<'_>),
    pub for_tests:extern "C" fn()->ForTests,
    
    #[cfg(feature="enable_field_a")]
    #[sabi(missing_field(option))]
    pub field_a:u32,
    
    #[cfg(feature="enable_field_b")]
    #[sabi(missing_field(option))]
    pub field_b:u32,
    
    #[cfg(feature="enable_field_c")]
    #[sabi(missing_field(option))]
    pub field_c:u32,
}

/// A module for all deserialization functions.
#[repr(C)]
#[derive(StableAbi)] 
#[sabi(kind(Prefix(prefix_struct="DeserializerMod_Prefix")))]
#[sabi(missing_field(panic))]
pub struct DeserializerMod {
    #[sabi(last_prefix_field)]
    /// The implementation for how TOStateBox is going to be deserialized.
    pub deserialize_state: extern "C" fn(RStr<'_>) -> RResult<TOStateBox, RBoxError>,

    /// The implementation for how TOCommandBox is going to be deserialized.
    pub deserialize_command: 
        for<'a> extern "C" fn(RStr<'a>) -> RResult<TOCommandBox<'static>, RBoxError>,
    
    /// The implementation for how TOCommandBox is going to be deserialized,
    /// borrowing from the input string.
    pub deserialize_command_borrowing: 
        for<'borr> extern "C" fn(RStr<'borr>) -> RResult<TOCommandBox<'borr>, RBoxError>,
    
    /// The implementation for how TOReturnValueArc is going to be deserialized.
    pub deserialize_return_value: extern "C" fn(RStr<'_>) -> RResult<TOReturnValueArc, RBoxError>,
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
    struct PrefixTypeMod0;
    prefix_struct="PrefixTypeMod0_Prefix";
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
    struct PrefixTypeMod1;
    prefix_struct="PrefixTypeMod1_Prefix";
    
    extra_fields=[
        pub field_b:u32,
        pub field_c:u32,
        #[sabi(missing_field(panic))]
        pub field_d:u32,
    ]
}


///////////////////////////////////////////////////////////////////////////////


/// A late-initialized reference to the global `TextOpsMod_Prefix` instance,
///
/// Call `load_library_in` before calling `MODULES.get()` to get a `Some(_)` value back.
///
pub static MODULES:LazyStaticRef<TextOpsMod_Prefix>=LazyStaticRef::new();


/// Loads all the modules of the library at the `directory`.
/// If it loads them once,this will continue returning the same reference.
pub fn load_library_in(directory:&Path) -> Result<&'static TextOpsMod_Prefix,LibraryError> {
    MODULES.try_init(||{
        TextOpsMod_Prefix::load_from_library_in(directory)
    })
}
