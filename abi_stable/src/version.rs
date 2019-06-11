/*!
Types representing the version number of a library.
*/

use core_extensions::prelude::*;

use std::{
    error,
    fmt::{self, Display},
    num::ParseIntError,
};

use crate::std_types::StaticStr;

/// The `<major>.<minor>.<patch>` version of a library,
///
/// # Post 1.0 major version
///
/// Major versions are mutually incompatible for both users and implementors.
///
/// Minor allow users to have a version less than or equal to that of the implementor,
/// and disallows implementors from making changes that would break
/// any previous minor release (with the same major number).
///
/// Patch cannot change the api/abi of the library at all,fixes only.
///
/// # Pre 1.0 version
///
/// Minor versions are mutually incompatible for both users and implementors.
///
/// Patch cannot change the api/abi of the library at all,fixes only.
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[repr(C)]
pub struct VersionStrings {
    pub version: StaticStr,
}

/// The parsed (`<major>.<minor>.<patch>`) version number of a library.
///
/// # Post 1.0 major version
///
/// Major versions are mutually incompatible for both users and implementors.
///
/// Minor allow users to have a version less than or equal to that of the implementor,
/// and disallows implementors from making changes that would break
/// any previous minor release (with the same major number).
///
/// Patch cannot change the api/abi of the library at all,fixes only.
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[repr(C)]
pub struct VersionNumber {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl VersionStrings {
    pub const fn new(version:&'static str)->Self{
        Self{version:StaticStr::new(version)}
    }
    pub fn parsed(self) -> Result<VersionNumber, ParseVersionError> {
        VersionNumber::new(self)
    }
}

impl VersionNumber {
    pub fn new(vn: VersionStrings) -> Result<Self, ParseVersionError> {
        let mut iter=vn.version.splitn(3,'.');

        VersionNumber {
            major: iter.next()
                .unwrap_or("")
                .parse()
                .map_err(|x| ParseVersionError::new(vn, "major", x))?,
            minor: iter.next()
                .unwrap_or("")
                .parse()
                .map_err(|x| ParseVersionError::new(vn, "minor", x))?,
            patch: iter.next()
                .unwrap_or("")
                .split_while(|x| '0' <= x && x <= '9')
                .find(|x| x.key)
                .map_or("0", |x| x.str)
                .parse()
                .map_err(|x| ParseVersionError::new(vn, "patch", x))?,
        }
        .piped(Ok)
    }

    /// Whether the `self` version number is compatible with the
    /// library_implementor version number.
    ///
    /// This uses the same semver rules as cargo:
    ///
    /// - For 0.y.z ,y is interpreted as a major version,
    ///     z is interpreted as the minor version,
    ///
    /// - For x.y.z ,x>=1,y is interpreted as a minor version.
    ///
    /// - Libraries are compatible so long as they are the same
    ///     major version with a minor_version >=`self`.
    pub fn is_compatible(self, library_implementor: VersionNumber) -> bool {
        if self.major == 0 && library_implementor.major == 0 {
            self.minor == library_implementor.minor && self.patch <= library_implementor.patch
        } else {
            self.major == library_implementor.major && self.minor <= library_implementor.minor
        }
    }
}

impl fmt::Display for VersionNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl fmt::Display for VersionStrings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.version,f)
    }
}

////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////


/// Instantiates a `abi_stable::version::VersionStrings` with the 
/// major.minor.patch version of the library where it is invoked.
#[macro_export]
macro_rules! package_version_strings {
    () => {{
        use $crate::std_types::StaticStr;
        $crate::version::VersionStrings::new(env!("CARGO_PKG_VERSION"))
    }};
}

////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

/// When the `VersionStrings` could not be converted into a `VersionNumber`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseVersionError {
    version_strings: VersionStrings,
    which_field: &'static str,
    parse_error: ParseIntError,
}

impl ParseVersionError {
    fn new(
        version_strings: VersionStrings,
        which_field: &'static str,
        parse_error: ParseIntError,
    ) -> Self {
        Self {
            version_strings,
            which_field,
            parse_error,
        }
    }

    pub fn version_strings(&self) -> VersionStrings {
        self.version_strings
    }
}

impl Display for ParseVersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "\nInvalid version string:'{}'\nerror at the {} field:{}",
            self.version_strings, self.which_field, self.parse_error,
        )
    }
}

impl error::Error for ParseVersionError {}
