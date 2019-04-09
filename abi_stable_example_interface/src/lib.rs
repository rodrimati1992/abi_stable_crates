use std::path::Path;

use abi_stable::{
    StableAbi,
    version_strings_const,
    lazy_static_ref::LazyStaticRef,
    library::{Library,LibraryError, LibraryTrait, ModuleTrait},
    version::VersionStrings,
    traits::{False, InterfaceType, True,DeserializeImplType},
    OpaqueType, VirtualWrapper,
    std_types::{RBox, RStr, RString, RSlice,RCow,RArc,RBoxError,RResult},
};




///////////////////////////////////////////////


/// An InterfaceType for the object we'll use to store the state of every text operation.
///
pub struct TOState;

pub type TOStateBox = VirtualWrapper<RBox<OpaqueType<TOState>>>;

impl InterfaceType for TOState {
    type Clone = False;
    type Default = False;
    type Display = False;
    type Debug = True;
    type Serialize = True;
    type Deserialize = True;
    type Eq = False;
    type PartialEq = False;
    type Ord = False;
    type PartialOrd = False;
    type Hash = False;
}

impl DeserializeImplType for TOState {
    type Deserialized = TOStateBox;

    fn deserialize_impl(s: RStr<'_>) -> Result<Self::Deserialized, RBoxError> {
        (TO_MOD.get().unwrap().deserialize_state)(s).into_result()
    }
}


///////////////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(StableAbi)] 
pub struct RemoveWords<'a,S:'a>
where 
    S: Clone + StableAbi,
{
    pub string:RStr<'a>,
    pub words:RSlice<'a,S>,
}


///////////////////////////////////////////////////////////////////////////////


// Used for showing that abi checking works
#[cfg(not(feature="different_abi"))]
pub type ThirdParam=();

/// Used for showing that abi checking works
#[cfg(feature="different_abi")]
pub type ThirdParam=usize;

#[repr(C)]
#[derive(StableAbi)] 
pub struct TOLib {
    pub new: extern "C" fn() -> TOStateBox,
    pub deserialize_state: extern "C" fn(RStr<'_>) -> RResult<TOStateBox, RBoxError>,
    pub reverse_lines: extern "C" fn(&mut TOStateBox,RStr<'_>,ThirdParam) -> RString,
    pub remove_words_cow: extern "C" fn(&mut TOStateBox,RemoveWords<RCow<str>>) -> RString,
    pub remove_words_str: extern "C" fn(&mut TOStateBox,RemoveWords<RStr>) -> RString,
    pub remove_words_string: extern "C" fn(&mut TOStateBox,RemoveWords<RString>) -> RString,
    pub get_processed_bytes: extern "C" fn(&TOStateBox) -> u64,
}

#[repr(C)]
#[derive(StableAbi)] 
pub struct HelloWorldSubMod {
    pub greeter:extern "C" fn(RStr<'_>),
}

static RAW_LIB:LazyStaticRef<Library>=LazyStaticRef::new();
static TO_MOD:LazyStaticRef<TOLib>=LazyStaticRef::new();
static HELLO_WORLD_MOD:LazyStaticRef<HelloWorldSubMod>=LazyStaticRef::new();

impl LibraryTrait for TOLib {
    fn raw_library_ref()->&'static LazyStaticRef<Library>{
        &RAW_LIB
    }

    const BASE_NAME: &'static str = "text_operations";
    const NAME: &'static str = "text_operations";
    const VERSION_STRINGS: VersionStrings =
        version_strings_const!(version_number_text_operations);
}

impl ModuleTrait for TOLib{
    type Library=Self;

    fn module_ref()->&'static LazyStaticRef<Self>{
        &TO_MOD
    }

    const LOADER_FN: &'static str = "get_library";
}

impl ModuleTrait for HelloWorldSubMod{
    type Library=TOLib;

    fn module_ref()->&'static LazyStaticRef<Self>{
        &HELLO_WORLD_MOD
    }

    const LOADER_FN: &'static str = "get_hello_world_mod";
}


///////////////////////////////////////////////////////////////////////////////


pub fn load_library(path:&Path) -> Result<&'static TOLib,LibraryError> {
    TOLib::load_library(path)
}
