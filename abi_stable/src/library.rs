use std::{
    fmt::{self, Display},
    io::Error as LibLoadingError,
    mem,
    path::Path,
};

use core_extensions::prelude::*;

use libloading::Library as LibLoadingLibrary;

pub use libloading::Result as LibLoadingResult;

use crate::{
    abi_stability::{SharedStableAbi,StableAbi,AbiInfoWrapper,abi_checking::{check_abi_stability,AbiInstabilityErrors}},
    version::{InvalidVersionString, VersionNumber, VersionStrings},
    StaticStr
};

#[derive(Copy, Clone)]
pub struct Library {
    library: &'static LibLoadingLibrary,
}

impl Library {
    /// Loads a dynamic library at `<parent_folder>/<base_name>-<pointer_size>.<extension>`
    pub fn load_suffixed(
        parent_folder: &Path,
        base_name: &str,
    ) -> libloading::Result<&'static Self> {
        let is_64_bits =
            cfg!(any(x86_64, powerpc64, aarch64)) || ::std::mem::size_of::<usize>() == 8;
        let bits = if is_64_bits { "64" } else { "32" };

        Self::load(parent_folder, &format!("{}-{}", base_name, bits))
    }

    /// Loads a dynamic library at `<parent_folder>/<base_name>`
    pub fn load(parent_folder: &Path, base_name: &str) -> libloading::Result<&'static Self> {
        let extension = match (cfg!(windows), cfg!(mac)) {
            (false, false) => "so.1",
            (false, true) => "dylib",
            (true, false) => "dll",
            _ => unreachable!("system is both windows and mac"),
        };
        let path_to_lib = parent_folder.join(format!("{}.{}", base_name, extension));

        let lib = LibLoadingLibrary::new(&path_to_lib)?;
        lib.piped(Box::new)
            .piped(Box::leak)
            .piped(|x| Self { library: &*x })
            .piped(Box::new)
            .piped(|x| &*Box::leak(x))
            .piped(Ok)
    }

    pub fn library(&self) -> &'static LibLoadingLibrary {
        self.library
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
    pub unsafe fn get<T>(&self, symbol: &[u8]) -> libloading::Result<&'static T> {
        self.library
            .get::<T>(symbol)
            .map(|x| unsafe { mem::transmute::<&T, &'static T>(&*x) })
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
    pub unsafe fn get_copy<T>(&self, symbol: &[u8]) -> libloading::Result<T>
    where
        T: Copy + 'static,
    {
        self.get::<T>(symbol).map(|x| *x)
    }

    /// Gets access to a function declared by the library.
    ///
    /// # Safety
    ///
    /// Passing a `T` of a type different than the compiled library declared is
    /// undefined behavior.
    ///
    ///
    ///
    #[inline]
    pub unsafe fn get_fn<T: Copy>(&self, symbol: &[u8]) -> libloading::Result<T>
    where
        T: Copy + 'static,
    {
        self.get_copy(symbol)
    }
}

//////////////////////////////////////////////////////////////////////

/// A wrapper for abi_stable::library::Library which makes sure that
/// the library that is being loaded is compatible with the `interface crate`.
///
/// # Safety
///
/// Do note that these checks are not exhaustive,eg:abi compatible changes that break the api
/// won't be rejected by these checks.
#[must_use = "call `.into_inner()` to obtain the validated library."]
pub struct ASLibrary(&'static Library);

impl ASLibrary {
    pub fn into_inner(self) -> &'static Library {
        self.0
    }
}

//////////////////////////////////////////////////////////////////////


/// Which string to prefix/suffix to the names of each function of the library.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum NameSpace {
    /// Prefix this string to every library function name,
    /// with the `<prefix>_<function_name>` format.
    Prefix(&'static str),
    /// Suffix this string to every library function name
    /// with the `<function_name>_<suffix>` format.
    Suffix(&'static str),
}

impl NameSpace {
    pub fn apply(self, function_name: &str) -> String {
        let s = match self {
            NameSpace::Prefix(s) => s,
            NameSpace::Suffix(s) => s,
        };
        assert!(!s.is_empty());
        match self {
            NameSpace::Prefix(s) => [s, "_", function_name].concat(),
            NameSpace::Suffix(s) => [function_name, "_", s].concat(),
        }
    }
}



//////////////////////////////////////////////////////////////////////

pub trait LibraryTrait: Sized + SharedStableAbi {
    fn new(path: &Path) -> Result<&'static Self, LibraryError> {
        let lib = Library::load_suffixed(path, Self::BASE_NAME).unwrap();

        let vn = Self::VERSION_NUMBER;

        let version_string_fn: extern "C" fn() -> VersionStrings =
            unsafe { lib.get_fn(vn.ident.as_bytes())? };

        let user_version = (vn.version_string)().piped(VersionNumber::new)?;
        let library_version = version_string_fn().piped(VersionNumber::new)?;

        if user_version.major != library_version.major || user_version.minor > library_version.minor
        {
            return Err(LibraryError::IncompatibleVersionNumber {
                library_name: Self::NAME,
                user_version,
                library_version,
            });
        }

        let library_getter:extern fn()->WithLayout<&'static Self>=unsafe{
            lib.get_fn(Self::LOADER_FN.as_bytes())?
        };

        let items=library_getter();

        items.check_layout()?
            .initialization()
    }

    /// Defines behavior that happens once the library is loaded.
    fn initialization(self:&'static Self) -> Result<&'static Self, LibraryError> {
        Ok(self)
    }


    const BASE_NAME: &'static str;

    /// The name of the library used in error messages.
    const NAME: &'static str = Self::BASE_NAME;

    /// The name of the function which gets `&'static Self`.
    ///
    /// The function signature must be:
    ///
    /// extern fn()->WithLayout<&'static Self>
    const LOADER_FN:&'static str;

    // /// Which string to prefix/suffix to the names of each function of the library.
    // const NAMESPACE:NameSpace;

    /// An `extern function` defined in the interface crate
    /// which just returns its version number.
    ///
    /// The value for this constant must be
    /// `version_string_const!( some_function_name )` .
    const VERSION_NUMBER: &'static VersionStringsFnIdent;
}

//////////////////////////////////////////////////////////////////////


mod with_layout{
    use super::*;

    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(inside_abi_stable_crate)]
    pub struct WithLayout<T> {
        magic_number:usize,

        layout:&'static AbiInfoWrapper,
        value:T,
    }


    impl<T> WithLayout<T>{
        pub fn new(value:T)->Self
        where T:StableAbi
        {
            Self{
                magic_number:MAGIC_NUMBER,
                layout:T::ABI_INFO,
                value,
            }
        }

        pub fn check_layout(self)->Result<T,LibraryError>
        where T:StableAbi
        {
            if self.magic_number!=MAGIC_NUMBER {
                return Err(LibraryError::InvalidMagicNumber(self.magic_number));
            }
            check_abi_stability(T::ABI_INFO,self.layout)?;
            Ok(self.value)
        }
    }

}

pub use self::with_layout::WithLayout;

// ABI major version 0
const MAGIC_NUMBER:usize=0xAB1_57A_00;


//////////////////////////////////////////////////////////////////////

pub struct VersionStringsFnIdent {
    pub version_string: extern "C" fn() -> VersionStrings,
    pub ident: StaticStr,
}

#[macro_export]
macro_rules! version_string_const {
    ( $function_name:ident ) => {{
        use $crate::{StaticStr, library::VersionStringsFnIdent, version::VersionStrings};
        #[no_mangle]
        pub extern "C" fn $function_name() -> VersionStrings {
            VersionStrings {
                major: StaticStr::new(env!("CARGO_PKG_VERSION_MAJOR")),
                minor: StaticStr::new(env!("CARGO_PKG_VERSION_MINOR")),
                patch: StaticStr::new(env!("CARGO_PKG_VERSION_PATCH")),
            }
        }
        &VersionStringsFnIdent {
            ident: StaticStr::new(stringify!($function_name)),
            version_string: $function_name,
        }
    }};
}

//////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum LibraryError {
    LibLoading(LibLoadingError),
    InvalidVersionString(InvalidVersionString),
    IncompatibleVersionNumber {
        library_name: &'static str,
        user_version: VersionNumber,
        library_version: VersionNumber,
    },
    AbiInstability(AbiInstabilityErrors),
    InvalidMagicNumber(usize),
}

impl From<LibLoadingError> for LibraryError {
    fn from(v: LibLoadingError) -> LibraryError {
        LibraryError::LibLoading(v)
    }
}

impl From<InvalidVersionString> for LibraryError {
    fn from(v: InvalidVersionString) -> LibraryError {
        LibraryError::InvalidVersionString(v)
    }
}

impl From<AbiInstabilityErrors> for LibraryError {
    fn from(v:AbiInstabilityErrors)->Self{
        LibraryError::AbiInstability(v)
    }
}

impl Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibraryError::LibLoading(x) => fmt::Display::fmt(x, f),
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
            LibraryError::AbiInstability(x)=>{
                fmt::Display::fmt(x,f)
            }
            LibraryError::InvalidMagicNumber(found)=>{
                write!(
                    f,
                    "magic number used to load a library was {},when this library expected {}",
                    found,
                    MAGIC_NUMBER,
                )
            }
        }
    }
}

impl ::std::error::Error for LibraryError {}
