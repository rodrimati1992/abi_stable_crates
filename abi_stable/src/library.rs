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

#[derive(Copy, Clone)]
pub struct Library {
    path:&'static Path,
    library: &'static LibLoadingLibrary,
}


#[derive(Debug,Copy,Clone,PartialEq,Eq,Ord,PartialOrd,Hash)]
pub enum LibrarySuffix{
    /// Loads a dynamic library at `<parent_folder>/<base_name>.extension`
    NoSuffix,
    
    /// Loads a dynamic library at `<parent_folder>/<base_name>-<pointer_size>.<extension>`
    Suffix,
}


impl Library {
    pub fn get_library_path(
        parent_folder: &Path,
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
        parent_folder.join(name)
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
    pub fn load(
        parent_folder: &Path,
        base_name: &str,
        suffix:LibrarySuffix,
    ) -> Result<&'static Self,LibraryError> {
        let path=Self::get_library_path(parent_folder,base_name,suffix);
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
                    path:self.path.clone(),
                    symbol, 
                    io 
                })
            }
        }
    }
}

//////////////////////////////////////////////////////////////////////

pub type LibraryGetterFn<T>=
    extern "C" fn() -> WithLayout<T>;

//////////////////////////////////////////////////////////////////////

pub trait LibraryTrait: Sized + StableAbi {
    fn raw_library_ref()->&'static LazyStaticRef<Library>;

    /// The base name of the dynamic library,which is the same on all platforms.
    /// This is generally the name of the `implementation crate`.
    const BASE_NAME: &'static str;

    /// The name of the library used in error messages.
    const NAME: &'static str = Self::BASE_NAME;

    /// The version number of this library.
    /// 
    /// 
    /// 
    const VERSION_STRINGS: VersionStrings;
}


pub trait ModuleTrait: 'static+Sized+ StableAbi{
    type Library:LibraryTrait;
    
    /// The name of the function which constructs this module.
    ///
    /// The function signature is:
    ///
    /// extern "C" fn()->WithLayout<Self>
    const LOADER_FN: &'static str;

    fn get_library_path(directory:&Path)-> PathBuf {
        let base_name=<Self::Library as LibraryTrait>::BASE_NAME;
        Library::get_library_path(directory, base_name,LibrarySuffix::Suffix)
    }

    /// Loads this module by first loading the dynamic library from the `directory`.
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
    /// Only returns the module if it was already loaded.
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
        pub fn new(value:T) -> Self
        where
            T: ModuleTrait,
        {
            // println!("constructing a WithLayout");
                        
            value.piped(leak_value)
                .piped(Self::from_ref)
        }

        pub fn version_strings(&self)->VersionStrings{
            self.version_strings
        }

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

#[macro_export]
macro_rules! version_strings_const {
    ( $function_name:ident ) => {{
        use $crate::{version::VersionStrings, std_types::StaticStr};
        VersionStrings {
            major: StaticStr::new(env!("CARGO_PKG_VERSION_MAJOR")),
            minor: StaticStr::new(env!("CARGO_PKG_VERSION_MINOR")),
            patch: StaticStr::new(env!("CARGO_PKG_VERSION_PATCH")),
        }
    }};
}

//////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum LibraryError {
    OpenError{
        path:PathBuf,
        io:io::Error,
    },
    GetSymbolError{
        path:&'static Path,
        symbol:Vec<u8>,
        io:io::Error,
    },
    
    InvalidVersionString(InvalidVersionString),
    IncompatibleVersionNumber {
        library_name: &'static str,
        user_version: VersionNumber,
        library_version: VersionNumber,
    },
    AbiInstability(AbiInstabilityErrors),
    InvalidMagicNumber(usize),
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
            LibraryError::GetSymbolError{path,symbol,io} => writeln!(
                f,
                "Could load symbol:\n\t{}\nin library:\n\t{}\nbecause:\n\t{}",
                String::from_utf8_lossy(symbol),
                path.display(),
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
