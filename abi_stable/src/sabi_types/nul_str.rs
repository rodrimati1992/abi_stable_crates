//! A nul-terminated string,which is just a pointer to the string data,
//! it doesn't know the length of the string.

use crate::{std_types::RStr, utils::ref_as_nonnull};

use std::{
    cmp::{Eq, PartialEq},
    fmt::{self, Debug, Display},
    marker::PhantomData,
    ptr::NonNull,
};

/// A utf8 nul-terminated immutable borrowed string.
///
/// For the purpose of passing `NulStr`s to C,
/// this has the same ABI as a `std::ptr::NonNull<u8>`,
/// and an `Option<NulStr<'_>>` has the same ABI as `*const u8`.
///
/// # Safety
///
/// `NulStr` has these safety requirement:
/// - the string must be valid to read for the `'a` lifetime
/// - the string must be utf8 encoded
/// - the string must be nul terminated and not contain interior nul bytes
/// - the string must not be mutated while this is alive
/// (the same semantics as `&` references)
#[repr(transparent)]
#[derive(Copy, Clone, StableAbi)]
pub struct NulStr<'a> {
    ptr: NonNull<u8>,
    _marker: PhantomData<&'a u8>,
}

unsafe impl Sync for NulStr<'_> {}
unsafe impl Send for NulStr<'_> {}

impl NulStr<'static> {
    /// An empty string.
    pub const EMPTY: Self = NulStr::from_str("\0");
}

impl<'a> NulStr<'a> {
    /// Constructs an `NulStr` from a string slice.
    ///
    /// # Correctness
    ///
    /// If the string contains interior nuls,
    /// the first nul will be considered the string terminator.
    ///
    /// # Panics
    ///
    /// This panics when the string does not end with `'\0'`.
    pub const fn from_str(str: &'a str) -> Self {
        let this = unsafe {
            Self {
                ptr: NonNull::new_unchecked(str.as_ptr() as *mut u8),
                _marker: PhantomData,
            }
        };

        let last_byte = str.as_bytes()[str.len() - 1] as usize;
        [this /* expected a nul terminator */][last_byte]
    }

    const_fn_if_1_46! {
        /// Constructs an NulStr from a string slice.
        ///
        /// # Cosntness
        ///
        /// This is a `const fn` if the `"rust_1_46"` feature is enabled,
        /// otherwise it is a non-`const` function.
        ///
        /// # Errors
        ///
        /// This returns a `NulStrError::NoNulTerminator` when the string does not end
        /// with `'\0'`.
        ///
        /// This returns a `NulStrError::InnerNul` when the string contains a
        /// `'\0'` before the `'\0'` terminator.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::sabi_types::{NulStr, NulStrError};
        ///
        /// assert_eq!(NulStr::try_from_str("hello\0").unwrap().to_str(), "hello");
        ///
        /// assert_eq!(
        ///     NulStr::try_from_str("hello\0world\0"),
        ///     Err(NulStrError::InnerNul{pos: 5}),
        /// );
        ///
        /// ```
        pub fn try_from_str(string: &'a str) -> Result<Self, NulStrError> {
            let mut i = 0;
            let mut bytes = string.as_bytes();
            let len = string.len();

            bytes = match bytes {
                [rem @ .., 0] => rem,
                _ => return Err(NulStrError::NoNulTerminator),
            };

            while let [b, ref rem @ ..] = *bytes {
                if b == 0 {
                    return Err(NulStrError::InnerNul {pos : i});
                }
                i += 1;
                bytes = rem;
            }

            unsafe{
                Ok(NulStr::from_str(string))
            }
        }
    }

    #[doc(hidden)]
    #[cfg(feature = "rust_1_46")]
    pub const fn __try_from_str_unwrapping(s: &'a str) -> Self {
        match Self::try_from_str(s) {
            Ok(x) => x,
            Err(NulStrError::InnerNul { pos }) => [/* encountered nul byte at `pos` */][pos],
            Err(_) => [/* unreachable */][s.len()],
        }
    }

    /// Constructs an NulStr from a pointer.
    ///
    /// # Safety
    ///
    /// [The same as the type-level safety docs](#safety)
    pub const unsafe fn from_ptr(ptr: *const u8) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr as *mut u8),
            _marker: PhantomData,
        }
    }

    /// Gets a pointer to the start of this nul-terminated string.
    pub const fn as_ptr(self) -> *const std::os::raw::c_char {
        self.ptr.as_ptr() as _
    }

    /// Converts this `NulStr<'a>` to a `&'a str`,including the nul byte.
    ///
    /// # Performance
    ///
    /// This conversion requires traversing through the entire string to
    /// find the nul byte.
    pub fn to_str_with_nul(&self) -> &'a str {
        unsafe {
            let bytes = std::ffi::CStr::from_ptr(self.ptr.as_ptr() as *const _).to_bytes_with_nul();
            std::str::from_utf8_unchecked(bytes)
        }
    }

    /// Converts this `NulStr<'a>` to a `RStr<'a>`,including the nul byte.
    ///
    /// # Performance
    ///
    /// This conversion requires traversing through the entire string to
    /// find the nul byte.
    pub fn to_rstr_with_nul(&self) -> RStr<'a> {
        self.to_str_with_nul().into()
    }

    /// Converts this `NulStr<'a>` to a `&'a str`,not including the nul byte.
    ///
    /// # Performance
    ///
    /// This conversion requires traversing through the entire string to
    /// find the nul byte.
    pub fn to_str(self) -> &'a str {
        unsafe {
            let bytes = std::ffi::CStr::from_ptr(self.ptr.as_ptr() as *const _).to_bytes();
            std::str::from_utf8_unchecked(bytes)
        }
    }

    /// Converts this `NulStr<'a>` to a `RStr<'a>`,not including the nul byte.
    ///
    /// # Performance
    ///
    /// This conversion requires traversing through the entire string to
    /// find the nul byte.
    pub fn to_rstr(self) -> RStr<'a> {
        self.to_str().into()
    }
}

impl<'a> PartialEq for NulStr<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr || self.to_str() == other.to_str()
    }
}

impl<'a> Eq for NulStr<'a> {}

impl Display for NulStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.to_str(), f)
    }
}

impl Debug for NulStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.to_str(), f)
    }
}

#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum NulStrError {
    InnerNul { pos: usize },
    NoNulTerminator,
}

impl Display for NulStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::InnerNul { pos } => {
                write!(f, "there is an internal nul at the {} byte offset", pos)
            }
            Self::NoNulTerminator => f.write_str("there is no nul terminator in the string"),
        }
    }
}

impl std::error::Error for NulStrError {}
