#![allow(clippy::missing_const_for_fn)]

use super::{lib_header::AbiHeader, root_mod_trait::RootModule};

use crate::{
    sabi_types::{ParseVersionError, VersionNumber, VersionStrings},
    std_types::{RBoxError, RResult, RVec},
};

use std::{
    fmt::{self, Display},
    path::PathBuf,
};

#[allow(unused_imports)]
use core_extensions::SelfOps;

/// All the possible errors that could happen when loading a library,
/// or a module.
#[derive(Debug)]
pub enum LibraryError {
    /// When a library can't be loaded, because it doesn't exist.
    OpenError {
        /// The path to the library
        path: PathBuf,
        /// The cause of the error
        err: Box<libloading::Error>,
    },
    /// When a function/static does not exist.
    GetSymbolError {
        /// The path to the library
        library: PathBuf,
        /// The name of the function/static.Does not have to be utf-8.
        symbol: Vec<u8>,
        /// The cause of the error
        err: Box<libloading::Error>,
    },
    /// The version string could not be parsed into a version number.
    ParseVersionError(ParseVersionError),
    /// The version numbers of the library was incompatible.
    IncompatibleVersionNumber {
        ///
        library_name: &'static str,
        ///
        expected_version: VersionNumber,
        ///
        actual_version: VersionNumber,
    },
    /// Error returned by the root module
    RootModule {
        /// The error returned by the `#[export_root_module]` function.
        err: RootModuleError,
        ///
        module_name: &'static str,
        ///
        version: VersionStrings,
    },
    /// The abi is incompatible.
    /// The error is opaque,since the error always comes from the main binary
    /// (dynamic libraries can be loaded from other dynamic libraries).
    AbiInstability(RBoxError),
    /// The type used to check that this is a compatible abi_stable
    /// is not the same.
    InvalidAbiHeader(AbiHeader),
    /// When Rust changes how it implements the C abi,
    InvalidCAbi {
        ///
        expected: RBoxError,
        ///
        found: RBoxError,
    },
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
            LibraryError::OpenError { path, err } => writeln!(
                f,
                "Could not open library at:\n\t{}\nbecause:\n\t{}",
                path.display(),
                err
            ),
            LibraryError::GetSymbolError {
                library,
                symbol,
                err,
            } => writeln!(
                f,
                "Could load symbol:\n\t{}\nin library:\n\t{}\nbecause:\n\t{}",
                String::from_utf8_lossy(symbol),
                library.display(),
                err
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
            LibraryError::RootModule {
                err,
                module_name,
                version,
            } => {
                writeln!(
                    f,
                    "An error ocurred while loading this library:\t\n{}",
                    module_name
                )?;
                writeln!(f, "version:\n\t{}", version)?;
                f.write_str("the error:\n\n")?;
                fmt::Display::fmt(err, f)
            }
            LibraryError::AbiInstability(x) => fmt::Display::fmt(x, f),
            LibraryError::InvalidAbiHeader(found) => write!(
                f,
                "The abi of the library was:\n{:#?}\n\
                 When this library expected:\n{:#?}",
                found,
                AbiHeader::VALUE,
            ),
            LibraryError::InvalidCAbi { expected, found } => {
                write! {
                    f,
                    "The C abi of the library is different than expected:\n\
                     While running tests on the library:\n\
                         Found:\n        {found}\n\
                         Expected:\n        {expected}\n\
                    ",
                    found=found,
                    expected=expected,
                }
            }
            LibraryError::Many(list) => {
                for e in list {
                    Display::fmt(e, f)?;
                }
                Ok(())
            }
        }?;
        f.write_str("\n")?;
        Ok(())
    }
}

impl ::std::error::Error for LibraryError {}

//////////////////////////////////////////////////////////////////////

/// The errors that a `#[export_root_module]` function can return.
#[repr(u8)]
#[derive(Debug, StableAbi)]
pub enum RootModuleError {
    /// When the root loader function returned an error normally
    Returned(RBoxError),
    /// When the root loader function panicked
    Unwound,
}

impl RootModuleError {
    /// Reallocates the error using the current global allocator,
    /// to ensure that there is no pointer into the dynamic library.
    pub fn reallocate(&mut self) {
        match self {
            Self::Returned(e) => {
                *e = e.to_formatted_error();
            }
            Self::Unwound => {}
        }
    }

    /// Converts this `RootModuleError` into a `LibraryError`,
    /// with metadata about the module that failed to load.
    pub fn into_library_error<M: RootModule>(self) -> LibraryError {
        LibraryError::RootModule {
            err: self,
            module_name: M::NAME,
            version: M::VERSION_STRINGS,
        }
    }
}

impl Display for RootModuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\n")?;
        match self {
            Self::Returned(e) => Display::fmt(e, f)?,
            Self::Unwound => f.write_str("the root module loader panicked")?,
        }
        f.write_str("\n")?;
        Ok(())
    }
}

impl ::std::error::Error for RootModuleError {}

//////////////////////////////////////////////////////////////////////

/// For converting the return value of a `#[export_root_module]` function
/// to a `Result<_, RootModuleError>`.
pub trait IntoRootModuleResult {
    /// The module that is loaded in the success case.
    type Module: RootModule;

    /// Performs the conversion
    fn into_root_module_result(self) -> Result<Self::Module, RootModuleError>;
}

impl<M: RootModule> IntoRootModuleResult for Result<M, RBoxError> {
    type Module = M;

    fn into_root_module_result(self) -> Result<M, RootModuleError> {
        self.map_err(RootModuleError::Returned)
    }
}

impl<M: RootModule> IntoRootModuleResult for RResult<M, RBoxError> {
    type Module = M;

    fn into_root_module_result(self) -> Result<M, RootModuleError> {
        self.into_result().map_err(RootModuleError::Returned)
    }
}

impl<M: RootModule> IntoRootModuleResult for M {
    type Module = M;

    fn into_root_module_result(self) -> Result<M, RootModuleError> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        for_examples::{Module, Module_Ref},
        prefix_type::WithMetadata,
        std_types::{RBox, RErr, ROk, RSome},
    };

    use std::fmt::Error as FmtError;

    const MOD_WM: &WithMetadata<Module> = &WithMetadata::new(Module {
        first: RSome(5),
        second: rstr!(""),
        third: 13,
    });

    // `const PREFIX` can have different address every time it's used,
    // to fix that I made it a static
    static PREFIX: Module_Ref = Module_Ref(MOD_WM.static_as_prefix());

    #[test]
    fn into_root_module_result_test() {
        type Res = Result<Module_Ref, RBoxError>;
        type RRes = RResult<Module_Ref, RBoxError>;

        {
            assert_eq!(
                PREFIX.into_root_module_result().unwrap().0.to_raw_ptr() as usize,
                PREFIX.0.to_raw_ptr() as usize,
            );
        }

        fn test_case(
            ok: Result<Module_Ref, RootModuleError>,
            err: Result<Module_Ref, RootModuleError>,
        ) {
            assert_eq!(
                ok.unwrap().0.to_raw_ptr() as usize,
                PREFIX.0.to_raw_ptr() as usize
            );

            let downcasted = match err.err().unwrap() {
                RootModuleError::Returned(x) => x.downcast::<FmtError>().unwrap(),
                RootModuleError::Unwound => unreachable!(),
            };
            assert_eq!(downcasted, RBox::new(FmtError));
        }

        // From Result
        {
            let ok: Res = Ok(PREFIX);
            let err: Res = Err(RBoxError::new(FmtError));

            test_case(ok.into_root_module_result(), err.into_root_module_result());
        }

        // From RResult
        {
            let ok: RRes = ROk(PREFIX);
            let err: RRes = RErr(RBoxError::new(FmtError));

            test_case(ok.into_root_module_result(), err.into_root_module_result());
        }
    }
}
