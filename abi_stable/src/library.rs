/*!
Traits and types related to loading an abi_stable dynamic library,
as well as functions/modules within.
*/

use std::{
    fmt::{self, Display},
    io,
    marker::PhantomData,
    mem,
    path::{Path,PathBuf},
    sync::atomic,
};

use core_extensions::prelude::*;

use libloading::{
    Library as LibLoadingLibrary,
    Symbol as LLSymbol,
};

use abi_stable_derive_lib::{
    mangled_root_module_loader_name,
};



use crate::{
    abi_stability::{
        AbiInfoWrapper,
        AbiInfo,
        stable_abi_trait::SharedStableAbi,
    },
    globals::{self,Globals},
    marker_type::ErasedObject,
    version::{ParseVersionError, VersionNumber, VersionStrings},
    utils::{transmute_reference},
    std_types::{RVec,RBoxError},
};


/// A handle to any dynamically loaded library,
/// not necessarily ones that export abi_stable compatible modules.
pub struct RawLibrary {
    path:PathBuf,
    library: LibLoadingLibrary,
}


/// What naming convention to expect when loading a library from a directory.
#[derive(Debug,Copy,Clone,PartialEq,Eq,Ord,PartialOrd,Hash)]
pub enum LibrarySuffix{
    /// Loads a dynamic library at `<folder>/<base_name>.extension`
    NoSuffix,
    
    /// Loads a dynamic library at `<folder>/<base_name>-<pointer_size>.<extension>`
    Suffix,
}


impl RawLibrary {
    /// Gets the full path a library would be loaded from,
    pub fn path_in_directory(
        directory: &Path,
        base_name: &str,
        suffix:LibrarySuffix,
    )->PathBuf{
        let formatted:String;

        let (prefix,extension) = match (cfg!(windows), cfg!(target_os="macos")) {
            (false, false) => ("lib","so"),
            (false, true) => ("lib","dylib"),
            (true, false) => ("","dll"),
            _ => unreachable!("system is both windows and mac"),
        };
        
        let is_64_bits =
            cfg!(any(x86_64, powerpc64, aarch64)) || ::std::mem::size_of::<usize>() == 8;
        let bits = if is_64_bits { "64" } else { "32" };

        let maybe_suffixed_name=match suffix {
            LibrarySuffix::Suffix=>{
                formatted=format!("{}-{}", base_name, bits);
                &*formatted
            }
            LibrarySuffix::NoSuffix=>{
                base_name
            }
        };

        let name=format!("{}{}.{}",prefix, maybe_suffixed_name, extension);
        directory.join(name)
    }

    /// Loads the dynamic library at the `full_path` path.
    pub fn load_at(full_path:&Path) -> Result<Self,LibraryError> {
        match LibLoadingLibrary::new(&full_path) {
            Ok(library)=>Ok(Self { path:full_path.to_owned(), library }),
            Err(io)=>Err(LibraryError::OpenError{ path:full_path.to_owned(), io }),
        }
    }

    /// Gets access to a static/function declared by the library.
    ///
    /// # Safety
    ///
    /// Passing a `T` of a type different than the compiled library declared is
    /// undefined behavior.
    ///
    ///
    ///
    unsafe fn get<T>(
        &self, 
        symbol_name: &[u8]
    ) -> Result<LLSymbol<'_,T>,LibraryError> 
    {
        match self.library.get::<T>(symbol_name) {
            Ok(symbol)=>Ok(symbol),
            Err(io)=>{
                let symbol=symbol_name.to_owned();
                Err(LibraryError::GetSymbolError{ 
                    library:self.path.clone(),
                    symbol, 
                    io 
                })
            }
        }
    }
}

//////////////////////////////////////////////////////////////////////

/// The path a library is loaded from.
#[derive(Debug,Copy,Clone,PartialEq,Eq,Ord,PartialOrd,Hash)]
pub enum LibraryPath<'a>{
    FullPath(&'a Path),
    Directory(&'a Path),
}


//////////////////////////////////////////////////////////////////////


/// The root module of a dynamic library,
/// which may contain other modules,function pointers,and static references.
///
/// For an example of a type implementing this trait you can look 
/// for the `example/example_*_interface` crates  in this crates' repository .
pub trait RootModule: Sized+SharedStableAbi  {

    /// The base name of the dynamic library,which is the same on all platforms.
    /// This is generally the name of the `implementation crate`.
    const BASE_NAME: &'static str;

    /// The name of the library used in error messages.
    const NAME: &'static str;

    /// The version number of this library.
    /// 
    /// Initialize this with ` package_version_strings!() `
    const VERSION_STRINGS: VersionStrings;

    const CONSTANTS:RootModuleConsts<Self>=RootModuleConsts{
        base_name:Self::BASE_NAME,
        name:Self::NAME,
        version_strings:Self::VERSION_STRINGS,
        abi_info:<&Self>::S_ABI_INFO,
        _priv:PhantomData,
    };

    /// Returns the path the library would be loaded from.
    fn get_library_path(where_:LibraryPath<'_>)-> PathBuf {
        match where_ {
            LibraryPath::Directory(directory)=>{
                let base_name=Self::BASE_NAME;
                RawLibrary::path_in_directory(directory, base_name,LibrarySuffix::NoSuffix)
            }
            LibraryPath::FullPath(full_path)=>  
                full_path.to_owned(),
        }
    }

    /// Loads this module from the path specified by `where_`,
    /// first loading the dynamic library if it wasn't already loaded.
    fn load_from_library(where_:LibraryPath<'_>) -> Result<&'static Self, LibraryError>{
        load_raw_library::<Self>(where_)
            .and_then(load_root_mod_with_raw_lib)
    }

    /// Returns the layout of the root module of the library at the specified path.
    fn layout_of_library(where_:LibraryPath<'_>)->Result<&'static AbiInfo,LibraryError>{
        let raw_lib=load_raw_library::<Self>(where_)?;

        let library_getter=unsafe{ with_layout_from_raw_library(&raw_lib)? };

        let layout=library_getter.layout()?;

        // Important,If I don't leak the library after sucessfully loading the root module
        // it would cause any use of the module to be a use after free.
        mem::forget(raw_lib);

        Ok(layout)

    }

    /// Defines behavior that happens once the module is loaded.
    ///
    /// The default implementation does nothing.
    fn initialization(self: &'static Self) -> Result<&'static Self, LibraryError> {
        Ok(self)
    }
}


/// Loads this module from the `raw_library`.
fn load_root_mod_with_raw_lib<M>(
    raw_library:RawLibrary
)->Result<&'static M,LibraryError>
where
    M:RootModule
{

    let items = unsafe{ with_layout_from_raw_library(&raw_library)? };

    let globals=globals::initialized_globals();
    
    // This has to run before anything else.    
    items.initialize_library_globals(globals);

    let expected_version = M::VERSION_STRINGS
        .piped(VersionNumber::new)?;
    let actual_version = items.version_strings().piped(VersionNumber::new)?;

    if expected_version.major != actual_version.major || 
        (expected_version.major==0) && expected_version.minor > actual_version.minor
    {
        return Err(LibraryError::IncompatibleVersionNumber {
            library_name: M::NAME,
            expected_version,
            actual_version,
        });
    }

    let root_mod=items.check_layout::<M>()?
        .initialization()?;

    // Important,If I don't leak the library after sucessfully loading the root module
    // it would cause any use of the module to be a use after free.
    mem::forget(raw_library);

    Ok(root_mod)
}


/// Loads the raw library at `where_`
fn load_raw_library<M>(where_:LibraryPath<'_>) -> Result<RawLibrary, LibraryError>
where
    M:RootModule
{
    let path=M::get_library_path(where_);
    RawLibrary::load_at(&path)
}


///
///
/// # Safety
///
/// The WithLayout is implicitly tied to the lifetime of the library,
/// so it will be invalidated if the library is dropped. 
unsafe fn with_layout_from_raw_library(
    raw_library:&RawLibrary
)->Result< WithLayout , LibraryError>
{
    unsafe{
        let mut mangled=mangled_root_module_loader_name();
        mangled.push('\0');
        let library_getter=
            raw_library.get::<&'static WithLayout>(mangled.as_bytes())?;

        Ok(**library_getter)
    }
}


pub fn with_layout_from_path(path:&Path)->Result< WithLayout , LibraryError> {
    let raw_lib=RawLibrary::load_at(path)?;

    let library_getter=unsafe{ with_layout_from_raw_library(&raw_lib)? };

    mem::forget(raw_lib);

    Ok(library_getter)

}


//////////////////////////////////////////////////////////////////////

/// Encapsulates all the important constants of `RootModule` for `M`,
/// used mostly to construct a `WithLayout` with `WithLayout::from_constructor`.
#[derive(Copy,Clone)]
pub struct RootModuleConsts<M>{
    pub base_name: &'static str,
    pub name: &'static str,
    pub version_strings: VersionStrings,
    pub abi_info: &'static AbiInfoWrapper,
    _priv:PhantomData<extern fn()->M>,
}


//////////////////////////////////////////////////////////////////////

/// Newtype wrapper for functions which construct constants.
///
/// Declared to pass a function pointers to const fn.
#[repr(C)]
#[derive(StableAbi)]
pub struct Constructor<T>(pub extern fn()->T);

impl<T> Copy for Constructor<T>{}

impl<T> Clone for Constructor<T>{
    fn clone(&self)->Self{
        *self
    }
}

//////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(StableAbi,Copy,Clone)]
struct InitGlobalsWith(pub extern fn(&'static Globals));

const INIT_GLOBALS_WITH:InitGlobalsWith=
    InitGlobalsWith(crate::globals::initialize_globals_with);

//////////////////////////////////////////////////////////////////////

mod with_layout {
    use super::*;

    /// Used to check the layout of modules returned by module-loading functions
    /// exported by dynamic libraries.
    #[repr(C)]
    #[derive(StableAbi,Copy,Clone)]
    pub struct WithLayout {
        magic_number: usize,

        version_strings:VersionStrings,
        layout: &'static AbiInfoWrapper,
        init_globals_with:InitGlobalsWith,
        module:ConstructorOrValue<&'static ErasedObject>
    }

    impl WithLayout {
        /// Constructs a WithLayout from the root module loader.
        pub const unsafe fn from_constructor<M>(
            constructor:Constructor<&'static ErasedObject>,
            constants:RootModuleConsts<M>,
        )->Self
        {
            Self {
                magic_number: MAGIC_NUMBER,
                version_strings:constants.version_strings,
                layout: constants.abi_info,
                init_globals_with: INIT_GLOBALS_WITH,
                module:ConstructorOrValue::Constructor(constructor),
            }
        }

        /// Constructs a WithLayout from the module.
        pub fn from_module<T>(value:&'static T)->Self
        where
            T: RootModule,
        {
            let value=unsafe{ transmute_reference::<T,ErasedObject>(value) };
            Self {
                magic_number: MAGIC_NUMBER,
                version_strings:T::VERSION_STRINGS,
                layout: <&T>::S_ABI_INFO,
                init_globals_with: INIT_GLOBALS_WITH,
                module:ConstructorOrValue::Value(value),
            }
        }

        fn check_abi(&self)->Result<(), LibraryError> {
            if self.magic_number == MAGIC_NUMBER {
                Ok(())
            }else{
                Err(LibraryError::InvalidMagicNumber(self.magic_number))
            }
        }

        /// The version string of the library the module is being loaded from.
        pub fn version_strings(&self)->VersionStrings{
            self.version_strings
        }

        /// Gets the layout of the root module.
        ///
        /// # Errors 
        ///
        /// This returns a LibraryError if the abi is incompatible.
        pub fn layout(&self)->Result<&'static AbiInfo , LibraryError>{
            self.check_abi()?;

            Ok(self.layout.get())
        }

        pub fn initialize_library_globals(&self,globals:&'static Globals){
            (self.init_globals_with.0)(globals);
        }

        /// Checks that the layout of the `T` from the dynamic library is 
        /// compatible with the caller's .
        pub fn check_layout<T>(mut self) -> Result<&'static T, LibraryError>
        where
            T: RootModule,
        {
            self.check_abi()?;

            // Using this instead of
            // crate::abi_stability::abi_checking::check_abi_stability
            // so that if this is called in a dynamic-library that loads 
            // another dynamic-library,
            // it uses the layout checker of the executable,
            // ensuring a globally unique view of the layout of types.
            //
            // This might also reduce the code in the library,
            // because it doesn't have to compile the layout checker for every library.
            (globals::initialized_globals().layout_checking)
                (<&T>::S_ABI_INFO, self.layout)
                .into_result()
                .map_err(LibraryError::AbiInstability)?;
            
            atomic::compiler_fence(atomic::Ordering::SeqCst);
            
            let ret=unsafe{ 
                transmute_reference::<ErasedObject,T>(self.module.get())
            };
            Ok(ret)
        }
    }
}


pub use self::with_layout::WithLayout;


// ABI version 0.3
// Format:
// ABI_(A for pre-1.0 version number ,B for major version number)_(version number)
const MAGIC_NUMBER: usize = 0xAB1_A_0003;

//////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(StableAbi,Copy,Clone)]
enum ConstructorOrValue<T>{
    Constructor(Constructor<T>),
    Value(T)
}

impl<T:Copy> ConstructorOrValue<T>{
    fn get(&mut self)->T{
        match *self {
            ConstructorOrValue::Value(v)=>v,
            ConstructorOrValue::Constructor(func)=>{
                let v=(func.0)();
                *self=ConstructorOrValue::Value(v);
                v
            },
        }
    }
}

//////////////////////////////////////////////////////////////////////


/// All the possible errors that could happen when loading a library,
/// or a module.
#[derive(Debug)]
pub enum LibraryError {
    /// When a library can't be loaded, because it doesn't exist.
    OpenError{
        path:PathBuf,
        io:io::Error,
    },
    /// When a function/static does not exist.
    GetSymbolError{
        library:PathBuf,
        /// The name of the function/static.Does not have to be utf-8.
        symbol:Vec<u8>,
        io:io::Error,
    },
    /// The version string could not be parsed into a version number.
    ParseVersionError(ParseVersionError),
    /// The version numbers of the library was incompatible.
    IncompatibleVersionNumber {
        library_name: &'static str,
        expected_version: VersionNumber,
        actual_version: VersionNumber,
    },
    /// The abi is incompatible.
    /// The error is opaque,since the error always comes from the main binary
    /// (dynamic libraries can be loaded from other dynamic libraries),
    /// and no approach for extensible enums is settled on yet.
    AbiInstability(RBoxError),
    /// The magic number used to check that this is a compatible abi_stable
    /// is not the same.
    InvalidMagicNumber(usize),
    /// There could have been 0 or more errors in the function.
    Many(RVec<Self>),
}

impl From<ParseVersionError> for LibraryError {
    fn from(v: ParseVersionError) -> LibraryError {
        LibraryError::ParseVersionError(v)
    }
}

impl Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\n")?;
        match self {
            LibraryError::OpenError{path,io} => writeln!(
                f,
                "Could not open library at:\n\t{}\nbecause:\n\t{}",
                path.display(),io
            ),
            LibraryError::GetSymbolError{library,symbol,io} => writeln!(
                f,
                "Could load symbol:\n\t{}\nin library:\n\t{}\nbecause:\n\t{}",
                String::from_utf8_lossy(symbol),
                library.display(),
                io
            ),
            LibraryError::ParseVersionError(x) => fmt::Display::fmt(x, f),
            LibraryError::IncompatibleVersionNumber {
                library_name,
                expected_version,
                actual_version,
            } => writeln!(
                f,
                "\n'{}' library version mismatch:\nuser:{}\nlibrary:{}",
                library_name, expected_version, actual_version,
            ),
            LibraryError::AbiInstability(x) => fmt::Display::fmt(x, f),
            LibraryError::InvalidMagicNumber(found) => write!(
                f,
                "magic number used to load a library was {},when this library expected {}",
                found, MAGIC_NUMBER,
            ),
            LibraryError::Many(list)=>{
                for e in list {
                    Display::fmt(e,f)?;
                }
                Ok(())
            }
        }?;
        f.write_str("\n")?;
        Ok(())
    }
}

impl ::std::error::Error for LibraryError {}
