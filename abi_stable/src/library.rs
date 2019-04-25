/*!
Traits and types related to loading an abi_stable dynamic library,
as well as functions/modules within.
*/

use std::{
    fmt::{self, Display},
    io,
    path::{Path,PathBuf},
    sync::atomic,
};

use core_extensions::prelude::*;

use libloading::{
    Library as LibLoadingLibrary,
    Symbol as LLSymbol,
};

use abi_stable_derive_lib::{
    mangle_library_getter_ident,
    mangle_initialize_globals_with_ident,
};



use crate::{
    abi_stability::{
        AbiInfoWrapper,
        stable_abi_trait::SharedStableAbi,
    },
    globals::{self,InitializeGlobalsWithFn},
    lazy_static_ref::LazyStaticRef,
    prefix_type::PrefixTypeTrait,
    version::{ParseVersionError, VersionNumber, VersionStrings},
    utils::leak_value,
    std_types::{RVec,RBoxError},
};


/// A handle to any dynamically loaded library,
/// not necessarily ones that export abi_stable compatible modules.
#[derive(Copy, Clone)]
pub struct Library {
    path:&'static Path,
    library: &'static LibLoadingLibrary,
}


/// What naming convention to expect when loading a library from a directory.
#[derive(Debug,Copy,Clone,PartialEq,Eq,Ord,PartialOrd,Hash)]
pub enum LibrarySuffix{
    /// Loads a dynamic library at `<folder>/<base_name>.extension`
    NoSuffix,
    
    /// Loads a dynamic library at `<folder>/<base_name>-<pointer_size>.<extension>`
    Suffix,
}


impl Library {
    /// Gets the full path a library would be loaded from,
    pub fn get_library_path(
        folder: &Path,
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
        folder.join(name)
    }

    /// Loads the dynamic library at the `full_path` path.
    pub fn load_at(full_path:&Path) -> Result<&'static Self,LibraryError> {
        LibLoadingLibrary::new(full_path)
            .map_err(|io|{
                LibraryError::OpenError{ path:full_path.to_owned(), io }
            })?
            .piped(leak_value)
            .piped(|library| Self { path:leak_value(full_path.to_owned()), library })
            .piped(leak_value)
            .piped(Ok)
    }

    /// Loads the dynamic library from the `folder`.
    /// 
    /// The full filename of the library is determined by `suffix`.
    pub fn load_in(
        folder: &Path,
        base_name: &str,
        suffix:LibrarySuffix,
    ) -> Result<&'static Self,LibraryError> {
        let path=Self::get_library_path(folder,base_name,suffix);
        Self::load_at(&path)
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
    unsafe fn get_static<T>(
        &self, 
        symbol_name: &[u8]
    ) -> Result<LLSymbol<'static,T>,LibraryError> 
    where T:'static
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

/// A type alias for a function that exports a module 
/// (a struct of function pointers that implements RootModule).
pub type LibraryGetterFn<T>=
    extern "C" fn() -> WithLayout<T>;

//////////////////////////////////////////////////////////////////////


/// The root module of a dynamic library,
/// which may contain other modules,function pointers,and static references.
///
/// For an example of a type implementing this trait you can look 
/// for the `example/example_*_interface` crates  in this crates' repository .
pub trait RootModule: Sized+SharedStableAbi  {

    /// The late-initialized reference to the Library handle.
    fn raw_library_ref()->&'static LazyStaticRef<Library>;

    /// The base name of the dynamic library,which is the same on all platforms.
    /// This is generally the name of the `implementation crate`.
    const BASE_NAME: &'static str;

    /// The name of the library used in error messages.
    const NAME: &'static str;

    /// The version number of this library.
    /// 
    /// Initialize this with ` package_version_strings!() `
    const VERSION_STRINGS: VersionStrings;

    /// The name of the function which constructs this module.
    ///
    /// The function signature for the loader is:
    ///
    /// `extern "C" fn()->WithLayout<Self>`
    const LOADER_FN: &'static str;

    /// Returns the path the library would be loaded from.
    fn get_library_path(directory:&Path)-> PathBuf {
        let base_name=Self::BASE_NAME;
        Library::get_library_path(directory, base_name,LibrarySuffix::NoSuffix)
    }

    /// Loads this module from the library in the `directory` directory,
    /// first loading the dynamic library from the `directory` if it wasn't already loaded.
    fn load_from_library_in(directory: &Path) -> Result<&'static Self, LibraryError>{
        Self::raw_library_ref()
            .try_init(||{
                let path=Self::get_library_path(directory);
                // println!("loading library at:\n\t{}\n",path.display());
                Library::load_at(&path) 
            })
            .and_then(Self::load_with)
    }
    
    /// Loads this module from the library at the `full_path` path,
    /// first loading the dynamic library from the `directory` if it wasn't already loaded.
    fn load_from_library_at(full_path: &Path) -> Result<&'static Self, LibraryError>{
        Self::raw_library_ref()
            .try_init(|| Library::load_at(full_path)  )
            .and_then(Self::load_with)
    }

    /// Loads this module from the `raw_library`.
    fn load_with(raw_library:&'static Library)->Result<&'static Self,LibraryError>{

        let library_getter: LLSymbol<'static,LibraryGetterFn<Self>> =unsafe{
            let mut mangled=mangle_library_getter_ident(Self::LOADER_FN);
            mangled.push('\0');
            raw_library.get_static::<LibraryGetterFn<Self>>(mangled.as_bytes())?
        };
        

        let initialize_globals_with: LLSymbol<'static,InitializeGlobalsWithFn>=unsafe{
            let mut mangled=mangle_initialize_globals_with_ident(Self::LOADER_FN);
            mangled.push('\0');
            raw_library.get_static::<InitializeGlobalsWithFn>(mangled.as_bytes())?
        };
        

        let globals=globals::initialized_globals();
        

        // This has to run before anything else.
        initialize_globals_with(globals);
        
        
        let items = library_getter();
        
        
        let expected_version = Self::VERSION_STRINGS
            .piped(VersionNumber::new)?;
        let actual_version = items.version_strings().piped(VersionNumber::new)?;

        if expected_version.major != actual_version.major || 
            (expected_version.major==0) && expected_version.minor > actual_version.minor
        {
            return Err(LibraryError::IncompatibleVersionNumber {
                library_name: Self::NAME,
                expected_version,
                actual_version,
            });
        }

        items.check_layout()?
            .initialization()
    }

    /// Defines behavior that happens once the module is loaded.
    fn initialization(self: &'static Self) -> Result<&'static Self, LibraryError> {
        Ok(self)
    }
}


//////////////////////////////////////////////////////////////////////

mod with_layout {
    use super::*;

    /// Used to check the layout of modules returned by module-loading functions
    /// exported by dynamic libraries.
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(inside_abi_stable_crate)]
    pub struct WithLayout <T:'static>{
        magic_number: usize,

        version_strings:VersionStrings,
        layout: &'static AbiInfoWrapper,
        value: &'static T,
    }

    impl<T> WithLayout<T> {
        /// Constructs a WithLayout from the `Type_Prefix` struct of a type 
        /// deriving `StableAbi` with 
        /// `#[sabi(kind(Prefix(prefix_struct="Type_Prefix" )))]`.
        pub fn from_prefix(ref_:&'static T)->Self
        where
            T: RootModule,
        {
            Self {
                magic_number: MAGIC_NUMBER,
                version_strings:T::VERSION_STRINGS,
                layout: <&T>::S_ABI_INFO,
                value:ref_,
            }
        }

        /// Constructs a WithLayout from the 
        /// type deriving `StableAbi` with `#[sabi(kind(Prefix(..)))]`,
        /// leaking the value in the process.
        pub fn new<M>(value:M) -> Self
        where
            M:PrefixTypeTrait<Prefix=T>+'static,
            T: RootModule,
        {
            // println!("constructing a WithLayout");
                        
            value.leak_into_prefix()
                .piped(Self::from_prefix)
        }

        /// The version string of the library the module is being loaded from.
        pub fn version_strings(&self)->VersionStrings{
            self.version_strings
        }

        /// Checks that the layout of the `T` from the dynamic library is 
        /// compatible with the caller's .
        pub fn check_layout(self) -> Result<&'static T, LibraryError>
        where
            T: RootModule,
        {
            if self.magic_number != MAGIC_NUMBER {
                return Err(LibraryError::InvalidMagicNumber(self.magic_number));
            }

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
            
            Ok(self.value)
        }
    }

}

pub use self::with_layout::WithLayout;

// ABI version 0.2
// Format:
// ABI_(A for pre-1.0 version number ,B for major version number)_(version number)
const MAGIC_NUMBER: usize = 0xAB1_A_0002;

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
        library:&'static Path,
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
