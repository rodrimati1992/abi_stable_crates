//! Types representing the version number of a library.

use core_extensions::{SelfOps, StringExt};

use std::{
    error,
    fmt::{self, Display},
    num::ParseIntError,
};

use crate::std_types::RStr;

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
///
/// # Example
///
/// ```
/// use abi_stable::sabi_types::VersionStrings;
///
/// let v1_0_0 = VersionStrings::new("1.0.0").parsed().unwrap();
/// let v1_0_5 = VersionStrings::new("1.0.5").parsed().unwrap();
/// let v1_1_0 = VersionStrings::new("1.1.0").parsed().unwrap();
/// let v2_0_0 = VersionStrings::new("1.0.5").parsed().unwrap();
///
/// assert!(v1_0_0.is_compatible(v1_0_5), "'{}' '{}'", v1_0_0, v1_0_5);
/// assert!(v1_0_5.is_compatible(v1_1_0), "'{}' '{}'", v1_0_5, v1_1_0);
/// assert!(!v1_1_0.is_compatible(v2_0_0), "'{}' '{}'", v1_1_0, v2_0_0);
///
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[repr(transparent)]
pub struct VersionStrings {
    /// The `major.minor.patch` version string
    pub version: RStr<'static>,
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
///
/// # Example
///
/// ```
/// use abi_stable::sabi_types::VersionNumber;
///
/// let v0_1_0 = VersionNumber {
///     major: 0,
///     minor: 1,
///     patch: 0,
/// };
/// let v0_1_5 = VersionNumber {
///     major: 0,
///     minor: 1,
///     patch: 5,
/// };
/// let v0_1_8 = VersionNumber {
///     major: 0,
///     minor: 1,
///     patch: 8,
/// };
/// let v0_2_0 = VersionNumber {
///     major: 0,
///     minor: 2,
///     patch: 0,
/// };
///
/// assert!(v0_1_0.is_compatible(v0_1_5), "'{}' '{}'", v0_1_0, v0_1_5);
/// assert!(v0_1_5.is_compatible(v0_1_8), "'{}' '{}'", v0_1_5, v0_1_8);
/// assert!(!v0_1_8.is_compatible(v0_2_0), "'{}' '{}'", v0_1_8, v0_2_0);
///
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[repr(C)]
pub struct VersionNumber {
    ///
    pub major: u32,
    ///
    pub minor: u32,
    ///
    pub patch: u32,
}

impl VersionStrings {
    /// Constructs a VersionStrings from a string with the
    /// "major.minor.patch" format,where each one is a valid number.
    ///
    /// This does not check whether the string is correctly formatted,
    /// that check is done inside `VersionStrings::parsed`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::VersionStrings;
    ///
    /// static VERSION: VersionStrings = VersionStrings::new("0.1.2");
    ///
    /// ```
    pub const fn new(version: &'static str) -> Self {
        Self {
            version: RStr::from_str(version),
        }
    }

    /// Attempts to convert a `VersionStrings` into a `VersionNumber`
    ///
    /// # Errors
    ///
    /// This returns a `ParseVersionError` if the string is not correctly formatted.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::{VersionNumber, VersionStrings};
    ///
    /// static VERSION: VersionStrings = VersionStrings::new("0.1.2");
    ///
    /// assert_eq!(
    ///     VERSION.parsed(),
    ///     Ok(VersionNumber {
    ///         major: 0,
    ///         minor: 1,
    ///         patch: 2
    ///     })
    /// );
    ///
    /// let err_version = VersionStrings::new("0.a.2.b");
    /// assert!(err_version.parsed().is_err());
    ///
    /// ```
    pub fn parsed(self) -> Result<VersionNumber, ParseVersionError> {
        VersionNumber::new(self)
    }
}

impl VersionNumber {
    /// Attempts to convert a `VersionStrings` into a `VersionNumber`
    ///
    /// # Errors
    ///
    /// This returns a `ParseVersionError` if the string is not correctly formatted.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::{VersionNumber, VersionStrings};
    ///
    /// static VERSION: VersionStrings = VersionStrings::new("10.5.20");
    ///
    /// assert_eq!(
    ///     VersionNumber::new(VERSION),
    ///     Ok(VersionNumber {
    ///         major: 10,
    ///         minor: 5,
    ///         patch: 20
    ///     })
    /// );
    ///
    /// let err_version = VersionStrings::new("not a version number");
    /// assert!(VersionNumber::new(err_version).is_err());
    ///
    /// ```
    pub fn new(vn: VersionStrings) -> Result<Self, ParseVersionError> {
        let mut iter = vn.version.splitn(3, '.');

        VersionNumber {
            major: iter
                .next()
                .unwrap_or("")
                .parse()
                .map_err(|x| ParseVersionError::new(vn, "major", x))?,
            minor: iter
                .next()
                .unwrap_or("")
                .parse()
                .map_err(|x| ParseVersionError::new(vn, "minor", x))?,
            patch: iter
                .next()
                .unwrap_or("")
                .split_while(|x| ('0'..='9').contains(&x))
                .find(|x| x.key)
                .map_or("0", |x| x.str)
                .parse()
                .map_err(|x| ParseVersionError::new(vn, "patch", x))?,
        }
        .piped(Ok)
    }

    /// Whether the `self` version number is compatible with the
    /// `library_implementor` version number.
    ///
    /// This uses modified semver rules where:
    ///
    /// - For 0.y.z ,y is interpreted as a major version,
    ///     z is interpreted as the minor version,
    ///
    /// - For x.y.z ,x>=1,y is interpreted as a minor version.
    ///
    /// - Libraries are compatible so long as they are the same
    ///     major version with a minor_version >=`self`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::VersionNumber;
    ///
    /// let v0_1_0 = VersionNumber {
    ///     major: 0,
    ///     minor: 1,
    ///     patch: 0,
    /// };
    /// let v0_1_5 = VersionNumber {
    ///     major: 0,
    ///     minor: 1,
    ///     patch: 5,
    /// };
    /// let v0_1_8 = VersionNumber {
    ///     major: 0,
    ///     minor: 1,
    ///     patch: 8,
    /// };
    /// let v0_2_0 = VersionNumber {
    ///     major: 0,
    ///     minor: 2,
    ///     patch: 0,
    /// };
    ///
    /// assert!(v0_1_0.is_compatible(v0_1_5), "'{}' '{}'", v0_1_0, v0_1_5);
    /// assert!(v0_1_5.is_compatible(v0_1_8), "'{}' '{}'", v0_1_5, v0_1_8);
    /// assert!(!v0_1_8.is_compatible(v0_2_0), "'{}' '{}'", v0_1_8, v0_2_0);
    ///
    /// ```
    pub const fn is_compatible(self, library_implementor: VersionNumber) -> bool {
        if self.major == 0 && library_implementor.major == 0 {
            self.minor == library_implementor.minor && self.patch <= library_implementor.patch
        } else {
            self.major == library_implementor.major && self.minor <= library_implementor.minor
        }
    }
    /// Whether the `self` version number is compatible with the
    /// library version number.
    ///
    /// This uses the same semver rules as cargo:
    ///
    /// - For 0.y.z ,y is interpreted as a major version,
    ///     z is interpreted as the minor version,
    ///
    /// - For x.y.z ,x>=1,y is interpreted as a minor version.
    ///
    /// - Libraries are compatible so long as they are the same
    ///     major version irrespective of their minor version.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::VersionNumber;
    ///
    /// let v0_1_0 = VersionNumber {
    ///     major: 0,
    ///     minor: 1,
    ///     patch: 0,
    /// };
    /// let v0_1_5 = VersionNumber {
    ///     major: 0,
    ///     minor: 1,
    ///     patch: 5,
    /// };
    /// let v0_1_8 = VersionNumber {
    ///     major: 0,
    ///     minor: 1,
    ///     patch: 8,
    /// };
    /// let v0_2_0 = VersionNumber {
    ///     major: 0,
    ///     minor: 2,
    ///     patch: 0,
    /// };
    /// let v0_2_8 = VersionNumber {
    ///     major: 0,
    ///     minor: 2,
    ///     patch: 8,
    /// };
    /// let v1_0_0 = VersionNumber {
    ///     major: 1,
    ///     minor: 0,
    ///     patch: 0,
    /// };
    /// let v1_5_0 = VersionNumber {
    ///     major: 1,
    ///     minor: 5,
    ///     patch: 0,
    /// };
    /// let v2_0_0 = VersionNumber {
    ///     major: 2,
    ///     minor: 0,
    ///     patch: 0,
    /// };
    ///
    /// fn is_compat_assert(l: VersionNumber, r: VersionNumber, are_they_compat: bool) {
    ///     assert_eq!(l.is_loosely_compatible(r), are_they_compat);
    ///     assert_eq!(r.is_loosely_compatible(l), are_they_compat);
    /// }
    ///
    /// is_compat_assert(v0_1_0, v0_1_5, true);
    /// is_compat_assert(v0_1_5, v0_1_8, true);
    /// is_compat_assert(v1_0_0, v1_5_0, true);
    /// is_compat_assert(v0_1_8, v0_2_0, false);
    /// is_compat_assert(v0_2_8, v1_0_0, false);
    /// is_compat_assert(v2_0_0, v1_0_0, false);
    /// is_compat_assert(v2_0_0, v1_5_0, false);
    ///
    /// ```
    pub const fn is_loosely_compatible(self, library_implementor: VersionNumber) -> bool {
        if self.major == 0 && library_implementor.major == 0 {
            self.minor == library_implementor.minor
        } else {
            self.major == library_implementor.major
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
        fmt::Display::fmt(&self.version, f)
    }
}

////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

/// Instantiates a [`VersionStrings`] with the
/// major.minor.patch version of the library where it is invoked.
///
/// [`VersionStrings`]: ./sabi_types/version/struct.VersionStrings.html
#[macro_export]
macro_rules! package_version_strings {
    () => {{
        $crate::sabi_types::VersionStrings::new(env!("CARGO_PKG_VERSION"))
    }};
}

////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

/// When the `VersionStrings` could not be converted into a `VersionNumber`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseVersionError {
    version_strings: VersionStrings,
    which_field: &'static str,
    parse_error: ParseIntError,
}

impl ParseVersionError {
    const fn new(
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

    /// Gets back the `VersionStrings` that could not be parsed into a `VersionNumber`.
    pub const fn version_strings(&self) -> VersionStrings {
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
