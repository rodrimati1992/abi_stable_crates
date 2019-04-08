#[macro_use]
extern crate abi_stable;

use std::path::Path;

use abi_stable::{
    StableAbi,
    library::{LibraryError, LibraryTrait},
    version::VersionStrings,
    traits::{False, InterfaceType, True},
    OpaqueType, VirtualWrapper,
    std_types::{RBox, RStr, RString, RSlice,RCow,RArc},
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
    type Serialize = False;
    type Deserialize = False;
    type Eq = False;
    type PartialEq = False;
    type Ord = False;
    type PartialOrd = False;
    type Hash = False;
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


#[repr(C)]
#[derive(StableAbi)] 
pub struct TOLib {
    pub new: extern "C" fn() -> TOStateBox,
    pub reverse_lines: extern "C" fn(&mut TOStateBox,RStr<'_>) -> RString,
    pub remove_words_cow: extern "C" fn(&mut TOStateBox,RemoveWords<RCow<str>>) -> RString,
    pub remove_words_str: extern "C" fn(&mut TOStateBox,RemoveWords<RStr>) -> RString,
    pub remove_words_string: extern "C" fn(&mut TOStateBox,RemoveWords<RString>) -> RString,
    pub get_processed_bytes: extern "C" fn(&TOStateBox) -> u64,
}

impl LibraryTrait for TOLib {
    const LOADER_FN: &'static str = "get_library_text_operations";
    const BASE_NAME: &'static str = "text_operations";
    const NAME: &'static str = "text_operations";
    const VERSION_NUMBER: VersionStrings =
        version_number_const!(version_number_text_operations);
}


///////////////////////////////////////////////////////////////////////////////


pub fn load_library(path:&Path) -> Result<&'static TOLib,LibraryError> {
    TOLib::new(path)
}
