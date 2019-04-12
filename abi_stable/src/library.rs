/*!
Traits and types related to loading an abi_stable dynamic library,
as well as functions/modules within.
*/

use std::{
    fmt::{self, Display},
    io,
    mem,
    path::{Path,PathBuf},
    sync::atomic,
};

use core_extensions::prelude::*;

use libloading::Library as LibLoadingLibrary;

use abi_stable_derive_lib::mangle_library_getter_ident;



use crate::{
    abi_stability::{
        abi_checking::{check_abi_stability, AbiInstabilityErrors},
        AbiInfoWrapper,  StableAbi,
    },
    lazy_static_ref::LazyStaticRef,
    version::{InvalidVersionString, VersionNumber, VersionStrings},
    utils::leak_value,
    std_types::RVec,
};


/// A handle to a loaded library.
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

        let (prefix,extension) = match (cfg!(windows), cfg!(mac)) {
            (false, false) => ("lib","so"),
            (false, true) => ("","dylib"),
            (true, false) => ("","dll"),
            _ => unreachable!("system is both windows and mac"),
        };
        
        let is_64_bits =
            cfg!(any(x86_64, powerpc64, aarch64)) || ::std::mem::size_of::<usize>() == 8;
        let bits = if is_64_bits { "64" } else { "32" };

        let maybe_suffixed_name=match suffix {
            LibrarySuffix::NoSuffix=>{
                formatted=format!("{}-{}", base_name, bits);
                &*formatted
            }
            LibrarySuffix::Suffix=>{
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

    /// Loads the dynamic library.
    /// 
    /// The full filename of the library is determined by `suffix`.
    pub fn load(
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
    pub unsafe fn get_static<T>(&self, symbol_name: &[u8]) -> Result<&'static T,LibraryError> 
    where T:'static
    {
        match self.library.get::<T>(symbol_name) {
            Ok(symbol)=>Ok(unsafe { 
                mem::transmute::<&T, &'static T>(&symbol)
            }),
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
/// (a struct of function pointers that implements ModuleTrait).
pub type LibraryGetterFn<T>=
    extern "C" fn() -> WithLayout<T>;

//////////////////////////////////////////////////////////////////////


/// Represents a dynamic library,which contains modules 
/// (structs loaded using functions exported in the `implemenetation crate`).
///
/// Generally it is the root module that implements this trait.
///
/// For an example of a type implementing this trait you can look 
/// for crates names `abi_stable_example*_interface` in this crates' repository .
pub trait LibraryTrait: Sized  {

    /// The late-initialized reference to the Library handle.
    fn raw_library_ref()->&'static LazyStaticRef<Library>;

    /// The base name of the dynamic library,which is the same on all platforms.
    /// This is generally the name of the `implementation crate`.
    const BASE_NAME: &'static str;

    /// The name of the library used in error messages.
    const NAME: &'static str = Self::BASE_NAME;

    /// The version number of this library.
    /// 
    /// Initialize this with ` package_version_strings!() `
    const VERSION_STRINGS: VersionStrings;
}

/// Represents a module
/// (struct of function pointers,loaded using functions exported in the `implemenetation crate`).
///
/// For an example of a type implementing this trait you can look 
/// for crates names `abi_stable_example*_interface` in this crates' repository .
pub trait ModuleTrait: 'static+Sized+ StableAbi{
    type Library:LibraryTrait;
    
    /// The name of the function which constructs this module.
    ///
    /// The function signature for the loader is:
    ///
    /// `extern "C" fn()->WithLayout<Self>`
    const LOADER_FN: &'static str;

    /// Returns the path the library would be loaded from.
    fn get_library_path(directory:&Path)-> PathBuf {
        let base_name=<Self::Library as LibraryTrait>::BASE_NAME;
        Library::get_library_path(directory, base_name,LibrarySuffix::Suffix)
    }

    /// Loads this module,
    /// first loading the dynamic library from the `directory` if it wasn't already loaded.
    fn load_from_library_in(directory: &Path) -> Result<&'static Self, LibraryError>{
        Self::Library::raw_library_ref()
            .try_init(||{
                let path=Self::get_library_path(directory);
                // println!("loading library at:\n\t{}\n",path.display());
                Library::load_at(&path) 
            })
            .and_then(Self::load_with)
    }

    /// Loads this module from the `raw_library`.
    fn load_with(raw_library:&'static Library)->Result<&'static Self,LibraryError>{
        let mangled=mangle_library_getter_ident(Self::LOADER_FN);

        let library_getter: &'static LibraryGetterFn<Self> =
            unsafe { raw_library.get_static::<LibraryGetterFn<Self>>(mangled.as_bytes())? };

        let items = library_getter();

        let user_version = <Self::Library as LibraryTrait>::VERSION_STRINGS
            .piped(VersionNumber::new)?;
        let library_version = items.version_strings().piped(VersionNumber::new)?;

        if user_version.major != library_version.major || 
            (user_version.major==0) && user_version.minor > library_version.minor
        {
            return Err(LibraryError::IncompatibleVersionNumber {
                library_name: Self::Library::NAME,
                user_version,
                library_version,
            });
        }

        items.check_layout()?.initialization()
    }

    /// Defines behavior that happens once the library is loaded.
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
        /// Constructs a WithLayout.
        pub fn from_ref(ref_:&'static T)->Self
        where
            T: ModuleTrait,
        {
            Self {
                magic_number: MAGIC_NUMBER,
                version_strings:T::Library::VERSION_STRINGS,
                layout: T::ABI_INFO,
                value:ref_,
            }
        }

        /// Constructs a WithLayout,leaking the value in the process.
        pub fn new(value:T) -> Self
        where
            T: ModuleTrait,
        {
            // println!("constructing a WithLayout");
                        
            value.piped(leak_value)
                .piped(Self::from_ref)
        }

        /// The version string of the library the module is being loaded from.
        pub fn version_strings(&self)->VersionStrings{
            self.version_strings
        }

        /// Checks that the layout of the `T` from the dynamic library is 
        /// compatible with the caller's .
        pub fn check_layout(self) -> Result<&'static T, LibraryError>
        where
            T: ModuleTrait,
        {
            if self.magic_number != MAGIC_NUMBER {
                return Err(LibraryError::InvalidMagicNumber(self.magic_number));
            }
            check_abi_stability(T::ABI_INFO, self.layout)?;
            
            atomic::compiler_fence(atomic::Ordering::SeqCst);
            
            Ok(self.value)
        }
    }

}

pub use self::with_layout::WithLayout;

// ABI major version 0
const MAGIC_NUMBER: usize = 0xAB1_57A_00;

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
    InvalidVersionString(InvalidVersionString),
    /// The version numbers of the library was incompatible.
    IncompatibleVersionNumber {
        library_name: &'static str,
        user_version: VersionNumber,
        library_version: VersionNumber,
    },
    /// The abi is incompatible.
    AbiInstability(AbiInstabilityErrors),
    /// The magic number used to check that this is a compatible abi_stable
    /// is not the same.
    InvalidMagicNumber(usize),
    /// There could have been 0 or more errors in the function.
    ///
    /// This is used in `abi_stable_example_interface` to collect all module loading errors.
    Many(RVec<Self>),
}

impl From<InvalidVersionString> for LibraryError {
    fn from(v: InvalidVersionString) -> LibraryError {
        LibraryError::InvalidVersionString(v)
    }
}

impl From<AbiInstabilityErrors> for LibraryError {
    fn from(v: AbiInstabilityErrors) -> Self {
        LibraryError::AbiInstability(v)
    }
}

impl Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            LibraryError::InvalidVersionString(x) => fmt::Display::fmt(x, f),
            LibraryError::IncompatibleVersionNumber {
                library_name,
                user_version,
                library_version,
            } => writeln!(
                f,
                "\n'{}' library version mismatch:\nuser:{}\nlibrary:{}",
                library_name, user_version, library_version,
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
        }
    }
}

impl ::std::error::Error for LibraryError {}
