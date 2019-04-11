use std::path::Path;

use abi_stable::{
    StableAbi,
    version_strings_const,
    lazy_static_ref::LazyStaticRef,
    library::{Library,LibraryError, LibraryTrait, ModuleTrait},
    version::VersionStrings,
    traits::{False, InterfaceType, True,DeserializeImplType},
    OpaqueType, VirtualWrapper,
    std_types::{RBox, RStr, RString,RVec, RSlice,RCow,RArc,RBoxError,RResult},
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
        (MODULES.get().unwrap().text_operations.deserialize_state)(s).into_result()
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
pub struct TextOpsMod {
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
pub struct HelloWorldMod {
    pub greeter:extern "C" fn(RStr<'_>),
}


pub static MODULES:LazyStaticRef<Modules>=LazyStaticRef::new();

impl LibraryTrait for TextOpsMod {
    fn raw_library_ref()->&'static LazyStaticRef<Library>{
        static RAW_LIB:LazyStaticRef<Library>=LazyStaticRef::new();
        &RAW_LIB
    }

    const BASE_NAME: &'static str = "text_operations";
    const NAME: &'static str = "text_operations";
    const VERSION_STRINGS: VersionStrings =
        version_strings_const!(version_number_text_operations);
}

impl ModuleTrait for TextOpsMod{
    type Library=Self;

    const LOADER_FN: &'static str = "get_library";
}

impl ModuleTrait for HelloWorldMod{
    type Library=TextOpsMod;

    const LOADER_FN: &'static str = "get_hello_world_mod";
}


///////////////////////////////////////////////////////////////////////////////


pub struct Modules{
    pub text_operations:&'static TextOpsMod,
    pub hello_world:&'static HelloWorldMod,
}


macro_rules! try_load_mod {
    ( $load_lib_expr:expr , $errors:ident ) => ({
        let res:Result<&'static _,abi_stable::library::LibraryError>=
            $load_lib_expr;

        let _: RVec<abi_stable::library::LibraryError>=
            $errors ;

        match res {
            Ok(x)=>Some(x),
            Err(e@LibraryError::OpenError{..})=>return Err(e),
            Err(e)=>{
                $errors.push(e);
                None
            },
        }
    })
}

/// Loads all the modules of the library at the `directory`.
/// If it loads them once,this will continue loading them.
pub fn load_library(directory:&Path) -> Result<&'static Modules,LibraryError> {
    MODULES.try_init(||{
        let mut errors=RVec::new();
        
        let text_operations=
            try_load_mod!( TextOpsMod::load_from_library_in(directory) , errors );
        let hello_world=
            try_load_mod!( HelloWorldMod::load_from_library_in(directory) , errors );

        if errors.is_empty() {
            let mods=Modules{
                text_operations:text_operations.unwrap(),
                hello_world:hello_world.unwrap(),
            };
            Ok(Box::leak(Box::new(mods)))
        }else{
            Err(LibraryError::Many(errors))
        }
    })
}
