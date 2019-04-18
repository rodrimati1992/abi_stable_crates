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
    library::{Library,LibraryError, LibraryTrait, ModuleTrait},
    version::VersionStrings,
    type_level::bools::*,
    erased_types::{InterfaceType,DeserializeInterfaceType},
    ZeroSized, VirtualWrapper,
    std_types::{RBox, RStr, RString,RVec,RArc, RSlice,RCow,RBoxError,RResult},
};




///////////////////////////////////////////////


/// The `InterfaceType` describing which traits are implemented by TOStateBox.
#[repr(C)]
#[derive(StableAbi)]
pub struct TOState;

/// The state passed to most functions in the TextOpsMod module.
pub type TOStateBox = VirtualWrapper<RBox<ZeroSized<TOState>>>;

// This macro is used to emulate default associated types.
// Look for the docs of InterfaceType to see 
// which other associated types you can define.
impl_InterfaceType!{
    impl InterfaceType for TOState {
        type Debug = True;
        type Serialize = True;
        type Deserialize = True;
        type PartialEq = True;
    }
}


impl DeserializeInterfaceType for TOState {
    type Deserialized = TOStateBox;

    fn deserialize_impl(s: RStr<'_>) -> Result<Self::Deserialized, RBoxError> {
        (MODULES.get().unwrap().text_operations.deserialize_state)(s).into_result()
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


// Used for showing that abi checking works
#[cfg(not(feature="different_abi"))]
pub type ThirdParam=();

/// Used for showing that abi checking works
#[cfg(feature="different_abi")]
pub type ThirdParam=usize;

/// The root module of the `text_operations` dynamic library.
/// With all the functions related to processing text.
///
/// To construct the entire tree of modules,including this one,
/// call `load_library_in(some_directory_path)`.
///
/// To construct this module by itself,
/// call <TextOpsMod as ModuleTrait>::load_from_library_in(some_directory_path)
#[repr(C)]
#[derive(StableAbi)] 
pub struct TextOpsMod {
    /// Constructs TOStateBox,state that is passed to other functions in this module.
    pub new: extern "C" fn() -> TOStateBox,
    
    /// The implementation for how TOStateBox is going to be deserialized.
    pub deserialize_state: extern "C" fn(RStr<'_>) -> RResult<TOStateBox, RBoxError>,
    
    /// Reverses the order of the lines.
    pub reverse_lines: extern "C" fn(&mut TOStateBox,RStr<'_>,ThirdParam) -> RString,
    
    /// Removes the `param.words` words from the `param.string` string.
    pub remove_words_cow: 
        for<'a>extern "C" fn(&mut TOStateBox,param:RemoveWords<'a,RCow<'a,RStr<'a>>>) -> RString,
    
    /// Removes the `param.words` words from the `param.string` string.
    pub remove_words_str: extern "C" fn(&mut TOStateBox,param:RemoveWords<RStr>) -> RString,
    
    /// Removes the `param.words` words from the `param.string` string.
    pub remove_words_string: extern "C" fn(&mut TOStateBox,param:RemoveWords<RString>) -> RString,

    /// Gets the ammount (in bytes) of text that was processed
    pub get_processed_bytes: extern "C" fn(&TOStateBox) -> u64,
}


/// An example sub-module in the text_operations dynamic library.
///
/// To construct the entire tree of modules,including this one,
/// call `load_library_in(some_directory_path)`.
///
/// To construct this module by itself,
/// call <HelloWorldMod as ModuleTrait>::load_from_library_in(some_directory_path)
#[repr(C)]
#[derive(StableAbi)] 
pub struct HelloWorldMod {
    pub greeter:extern "C" fn(RStr<'_>),
    pub for_tests:extern "C" fn()->ForTests,
}


/// A late-initialized reference to the global `Modules` instance,
///
/// Call `load_library_in` before calling `MODULES.get()` to get a `Some(_)` value back.
///
pub static MODULES:LazyStaticRef<Modules>=LazyStaticRef::new();

impl LibraryTrait for TextOpsMod {
    fn raw_library_ref()->&'static LazyStaticRef<Library>{
        static RAW_LIB:LazyStaticRef<Library>=LazyStaticRef::new();
        &RAW_LIB
    }

    const BASE_NAME: &'static str = "text_operations";
    const NAME: &'static str = "text_operations";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
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


/// All of the modules in the text_operations dynamic library.
/// 
/// To construct this,call `load_library_in(some_directory_path)`.
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
/// If it loads them once,this will continue returning the same reference.
pub fn load_library_in(directory:&Path) -> Result<&'static Modules,LibraryError> {
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
