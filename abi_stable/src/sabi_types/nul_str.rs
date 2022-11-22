//! A nul-terminated string,which is just a pointer to the string data,
//! it doesn't know the length of the string.

#[cfg(test)]
mod tests;

use crate::std_types::RStr;

use const_panic::{concat_assert, concat_panic};

use std::{
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
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
/// - the string must be nul terminated
/// - the string must not be mutated while this is alive
/// (the same semantics as `&` references)
///
/// # Example
///
/// ### Passing to extern function
///
/// You can pass `NulStr` to C functions expecting a nul-terminated string.
///
/// ```rust
/// use abi_stable::sabi_types::NulStr;
///
/// extern "C" {
///     // the signature in the C side is `uint64_t add_digits(const char*)`
///     fn add_digits(_: NulStr<'_>) -> u64;
/// }
/// # #[export_name = "add_digits"]
/// # pub extern "C" fn add_digits___(str: NulStr<'_>) -> u64 {
/// #    str.to_str().bytes()
/// #    .filter_map(|x|{
/// #        match x {
/// #            b'0'..=b'9' => Some(u64::from(x - b'0')),
/// #            _ => None,
/// #        }
/// #    })
/// #    .sum()
/// # }
///
/// # fn main() {
/// const FOO: NulStr<'_> = NulStr::from_str("1.2.3\0");
/// const BAR: NulStr<'_> = NulStr::from_str("12|34\0");
/// const QUX: NulStr<'_> = NulStr::from_str("123_abcd_45\0");
///
/// assert_eq!(unsafe { add_digits(FOO) }, 6);
/// assert_eq!(unsafe { add_digits(BAR) }, 10);
/// assert_eq!(unsafe { add_digits(QUX) }, 15);
/// # }
/// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::sabi_types::NulStr;
    ///
    /// const FOO: NulStr<'_> = NulStr::from_str("foo\0");
    /// // `NulStr`s can be compared with `str`s
    /// assert_eq!(FOO, "foo");
    ///
    /// const BAR: NulStr<'_> = NulStr::from_str("bar\0");
    /// assert_eq!(BAR, "bar");
    ///
    /// const HEWWO: NulStr<'_> = NulStr::from_str("Hello, world!\0");
    /// assert_eq!(HEWWO, "Hello, world!");
    ///
    /// const TRUNCATED: NulStr<'_> = NulStr::from_str("baz\0world!\0");
    /// assert_eq!(TRUNCATED, "baz");
    ///
    /// ```
    pub const fn from_str(str: &'a str) -> Self {
        let this = Self {
            ptr: crate::utils::ref_as_nonnull(str).cast::<u8>(),
            _marker: PhantomData,
        };

        let last_byte = str.as_bytes()[str.len() - 1] as usize;
        concat_assert! {
            last_byte == 0,
            "expected a nul terminator, found:",
            last_byte,
        };
        this
    }

    /// Constructs an NulStr from a string slice.
    ///
    /// # Errors
    ///
    /// This returns a [`NulStrError::NoNulTerminator`] when the string does not end
    /// with `'\0'`.
    ///
    /// This returns a [`NulStrError::InnerNul`] when the string contains a
    /// `'\0'` before the `'\0'` terminator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::sabi_types::{NulStr, NulStrError};
    ///
    /// // `NulStr`s can be compared with `str`s
    /// assert_eq!(NulStr::try_from_str("hello\0").unwrap(), "hello");
    ///
    /// assert_eq!(
    ///     NulStr::try_from_str("hello\0world\0"),
    ///     Err(NulStrError::InnerNul { pos: 5 }),
    /// );
    ///
    /// ```
    ///
    /// [`NulStrError::InnerNul`]: enum.NulStrError.html#variant.InnerNul
    /// [`NulStrError::NoNulTerminator`]: enum.NulStrError.html#variant.NoNulTerminator
    pub const fn try_from_str(string: &'a str) -> Result<Self, NulStrError> {
        let mut i = 0;
        let mut bytes = string.as_bytes();

        bytes = match bytes {
            [rem @ .., 0] => rem,
            _ => return Err(NulStrError::NoNulTerminator),
        };

        while let [b, ref rem @ ..] = *bytes {
            if b == 0 {
                return Err(NulStrError::InnerNul { pos: i });
            }
            i += 1;
            bytes = rem;
        }

        unsafe { Ok(NulStr::from_ptr(string.as_ptr())) }
    }

    #[doc(hidden)]
    #[track_caller]
    pub const fn __try_from_str_unwrapping(s: &'a str) -> Self {
        match Self::try_from_str(s) {
            Ok(x) => x,
            Err(NulStrError::InnerNul { pos }) => {
                concat_panic!("encountered inner nul byte at position: ", pos)
            }
            Err(NulStrError::NoNulTerminator) => concat_panic!("found no nul-terminator"),
        }
    }

    /// Constructs an NulStr from a pointer.
    ///
    /// # Safety
    ///
    /// [The same as the type-level safety docs](#safety)
    ///
    /// # Correctness
    ///
    /// If the string contains interior nuls,
    /// the first nul will be considered the string terminator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::sabi_types::NulStr;
    ///
    /// const FOO: NulStr<'_> = unsafe { NulStr::from_ptr("foo\0".as_ptr()) };
    /// assert_eq!(FOO, "foo");
    ///
    /// const BAR: NulStr<'_> = unsafe { NulStr::from_ptr("bar\0".as_ptr()) };
    /// assert_eq!(BAR, "bar");
    ///
    /// const HEWWO: NulStr<'_> = unsafe { NulStr::from_ptr("Hello, world!\0".as_ptr()) };
    /// assert_eq!(HEWWO, "Hello, world!");
    ///
    /// const TRUNCATED: NulStr<'_> = unsafe { NulStr::from_ptr("baz\0world!\0".as_ptr()) };
    /// assert_eq!(TRUNCATED, "baz");
    ///
    /// ```
    pub const unsafe fn from_ptr(ptr: *const u8) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr as *mut u8) },
            _marker: PhantomData,
        }
    }

    /// Gets a pointer to the start of this nul-terminated string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::sabi_types::NulStr;
    ///
    /// let foo_str = "foo\0";
    /// let foo = NulStr::from_str(foo_str);
    /// assert_eq!(foo.as_ptr(), foo_str.as_ptr());
    ///
    /// ```
    pub const fn as_ptr(self) -> *const u8 {
        self.ptr.as_ptr()
    }

    /// Converts this `NulStr<'a>` to a `&'a str`,including the nul byte.
    ///
    /// # Performance
    ///
    /// This conversion requires traversing through the entire string to
    /// find the nul byte.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::sabi_types::NulStr;
    ///
    /// const FOO: NulStr<'_> = NulStr::from_str("foo bar\0");
    /// let foo: &str = FOO.to_str_with_nul();
    /// assert_eq!(&foo[..3], "foo");
    /// assert_eq!(&foo[4..], "bar\0");
    ///
    /// ```
    pub fn to_str_with_nul(&self) -> &'a str {
        unsafe {
            let bytes = std::ffi::CStr::from_ptr(self.ptr.as_ptr() as *const _).to_bytes_with_nul();
            std::str::from_utf8_unchecked(bytes)
        }
    }

    /// Computes the length of the string, NOT including the nul terminator.
    #[cfg(feature = "rust_1_64")]
    const fn compute_length(self) -> usize {
        let start: *const u8 = self.ptr.as_ptr();
        let mut ptr = start;
        let mut len = 0;
        unsafe {
            while *ptr != 0 {
                ptr = ptr.offset(1);
                len += 1;
            }
            len
        }
    }

    /// Converts this `NulStr<'a>` to a `&'a str`,including the nul byte.
    ///
    /// # Performance
    ///
    /// To make this function const-callable,
    /// this uses a potentially less efficient approach than
    /// [`to_str_with_nul`](Self::to_str_with_nul).
    ///
    /// This conversion requires traversing through the entire string to
    /// find the nul byte.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::sabi_types::NulStr;
    ///
    /// const FOO: NulStr<'_> = NulStr::from_str("foo bar\0");
    /// const FOO_S: &str = FOO.const_to_str_with_nul();
    /// assert_eq!(&FOO_S[..3], "foo");
    /// assert_eq!(&FOO_S[4..], "bar\0");
    ///
    /// ```
    #[cfg(feature = "rust_1_64")]
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "rust_1_64")))]
    pub const fn const_to_str_with_nul(&self) -> &'a str {
        unsafe {
            let len = self.compute_length();
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.as_ptr(), len + 1))
        }
    }

    /// Converts this `NulStr<'a>` to a `RStr<'a>`,including the nul byte.
    ///
    /// # Performance
    ///
    /// This conversion requires traversing through the entire string to
    /// find the nul byte.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::sabi_types::NulStr;
    /// use abi_stable::std_types::RStr;
    ///
    /// const BAZ: NulStr<'_> = NulStr::from_str("baz qux\0");
    /// let baz: RStr<'_> = BAZ.to_rstr_with_nul();
    /// assert_eq!(&baz[..3], "baz");
    /// assert_eq!(&baz[4..], "qux\0");
    ///
    /// ```
    pub fn to_rstr_with_nul(&self) -> RStr<'a> {
        self.to_str_with_nul().into()
    }

    /// Converts this `NulStr<'a>` to a `&'a str`,not including the nul byte.
    ///
    /// # Performance
    ///
    /// This conversion requires traversing through the entire string to
    /// find the nul byte.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::sabi_types::NulStr;
    ///
    /// const FOO: NulStr<'_> = NulStr::from_str("foo bar\0");
    /// let foo: &str = FOO.to_str();
    /// assert_eq!(&foo[..3], "foo");
    /// assert_eq!(&foo[4..], "bar");
    ///
    /// ```
    pub fn to_str(self) -> &'a str {
        unsafe {
            let bytes = std::ffi::CStr::from_ptr(self.ptr.as_ptr() as *const _).to_bytes();
            std::str::from_utf8_unchecked(bytes)
        }
    }

    /// Converts this `NulStr<'a>` to a `&'a str`,not including the nul byte.
    ///
    /// # Performance
    ///
    /// To make this function const-callable,
    /// this uses a potentially less efficient approach than [`to_str`](Self::to_str).
    ///
    /// This conversion requires traversing through the entire string to
    /// find the nul byte.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::sabi_types::NulStr;
    ///
    /// const FOO: NulStr<'_> = NulStr::from_str("foo bar\0");
    /// const FOO_S: &str = FOO.const_to_str();
    /// assert_eq!(&FOO_S[..3], "foo");
    /// assert_eq!(&FOO_S[4..], "bar");
    ///
    /// ```
    #[cfg(feature = "rust_1_64")]
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "rust_1_64")))]
    pub const fn const_to_str(self) -> &'a str {
        unsafe {
            let len = self.compute_length();
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.as_ptr(), len))
        }
    }

    /// Converts this `NulStr<'a>` to a `RStr<'a>`,not including the nul byte.
    ///
    /// # Performance
    ///
    /// This conversion requires traversing through the entire string to
    /// find the nul byte.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::sabi_types::NulStr;
    /// use abi_stable::std_types::RStr;
    ///
    /// const BAZ: NulStr<'_> = NulStr::from_str("baz qux\0");
    /// let baz: RStr<'_> = BAZ.to_rstr();
    /// assert_eq!(&baz[..3], "baz");
    /// assert_eq!(&baz[4..], "qux");
    ///
    /// ```
    pub fn to_rstr(self) -> RStr<'a> {
        self.to_str().into()
    }
}

impl<'a> PartialEq<NulStr<'a>> for &str {
    fn eq(&self, other: &NulStr<'a>) -> bool {
        self.as_ptr() == other.as_ptr() || *self == other.to_str()
    }
}

impl<'a> PartialEq<&str> for NulStr<'a> {
    fn eq(&self, other: &&str) -> bool {
        self.as_ptr() == other.as_ptr() || self.to_str() == *other
    }
}

impl<'a> PartialEq<NulStr<'a>> for str {
    fn eq(&self, other: &NulStr<'a>) -> bool {
        self.as_ptr() == other.as_ptr() || self == other.to_str()
    }
}

impl<'a> PartialEq<str> for NulStr<'a> {
    fn eq(&self, other: &str) -> bool {
        self.as_ptr() == other.as_ptr() || self.to_str() == other
    }
}

impl<'a> PartialEq for NulStr<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr || self.to_str() == other.to_str()
    }
}

impl<'a> Eq for NulStr<'a> {}

impl<'a> PartialOrd for NulStr<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for NulStr<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.ptr == other.ptr {
            Ordering::Equal
        } else {
            self.to_str().cmp(other.to_str())
        }
    }
}

impl Default for NulStr<'_> {
    fn default() -> Self {
        NulStr::EMPTY
    }
}

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

/// Error from trying to convert a `&str` to a [`NulStr`]
///
/// [`NulStr`]: ./struct.NulStr.html
#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum NulStrError {
    /// When the string has a `'\0'` before the end.
    InnerNul {
        /// the position of the first '\0' character.
        pos: usize,
    },
    /// When the string doesn't end with `'\0'`
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
