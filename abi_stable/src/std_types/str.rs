//! Contains an ffi-safe equivalent of `&'a str`.

use std::{
    borrow::{Borrow, Cow},
    fmt::{self, Display},
    ops::{Deref, Index},
    str,
};

#[allow(unused_imports)]
use core_extensions::SelfOps;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::std_types::{RSlice, RString};

/// Ffi-safe equivalent of `&'a str`
///
/// # Example
///
/// This defines a function that returns the first word in a string.
///
/// ```
/// use abi_stable::{sabi_extern_fn, std_types::RStr};
///
/// #[sabi_extern_fn]
/// fn first_word(phrase: RStr<'_>) -> RStr<'_> {
///     match phrase.as_str().split_whitespace().next() {
///         Some(x) => x.into(),
///         None => "".into(),
///     }
/// }
///
///
/// ```
#[repr(C)]
#[derive(Copy, Clone, StableAbi)]
pub struct RStr<'a> {
    inner: RSlice<'a, u8>,
}

impl<'a> RStr<'a> {
    /// An empty `RStr`.
    pub const EMPTY: Self = RStr {
        inner: RSlice::EMPTY,
    };
}

impl<'a> RStr<'a> {
    /// Constructs an empty `RStr<'a>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RStr;
    ///
    /// const STR: RStr<'static> = RStr::empty();
    ///
    /// assert_eq!(STR, RStr::from(""));
    ///
    /// ```
    #[inline]
    pub const fn empty() -> Self {
        Self::EMPTY
    }

    /// Constructs an `RStr<'a>` from a pointer to the first byte,
    /// and a length.
    ///
    /// # Safety
    ///
    /// Callers must ensure that:
    ///
    /// - `ptr_` points to valid memory,
    ///
    /// - `ptr_ .. ptr+len` range is accessible memory, and is valid utf-8.
    ///
    /// - The data that `ptr_` points to must be valid for the lifetime of this `RStr<'a>`
    ///
    /// # Examples
    ///
    /// This function unsafely converts a `&str` to an `RStr<'_>`,
    /// equivalent to doing `RStr::from`.
    ///
    /// ```
    /// use abi_stable::std_types::RStr;
    ///
    /// fn convert(slice_: &str) -> RStr<'_> {
    ///     unsafe { RStr::from_raw_parts(slice_.as_ptr(), slice_.len()) }
    /// }
    ///
    /// ```
    #[inline]
    pub const unsafe fn from_raw_parts(ptr_: *const u8, len: usize) -> Self {
        Self {
            inner: unsafe { RSlice::from_raw_parts(ptr_, len) },
        }
    }

    /// Converts `&'a str` to a `RStr<'a>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RStr;
    ///
    /// assert_eq!(RStr::from_str("").as_str(), "");
    /// assert_eq!(RStr::from_str("Hello").as_str(), "Hello");
    /// assert_eq!(RStr::from_str("World").as_str(), "World");
    ///
    /// ```
    pub const fn from_str(s: &'a str) -> Self {
        unsafe { Self::from_raw_parts(s.as_ptr(), s.len()) }
    }

    /// For slicing `RStr`s.
    ///
    /// This is an inherent method instead of an implementation of the
    /// `std::ops::Index` trait because it does not return a reference.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RStr;
    ///
    /// let str = RStr::from("What is that.");
    ///
    /// assert_eq!(str.slice(..), str);
    /// assert_eq!(str.slice(..4), RStr::from("What"));
    /// assert_eq!(str.slice(4..), RStr::from(" is that."));
    /// assert_eq!(str.slice(4..7), RStr::from(" is"));
    ///
    /// ```
    pub fn slice<I>(&self, i: I) -> RStr<'a>
    where
        str: Index<I, Output = str>,
    {
        self.as_str().index(i).into()
    }

    /// Accesses the underlying byte slice.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RSlice, RStr};
    ///
    /// let str = RStr::from("What is that.");
    /// let bytes = RSlice::from("What is that.".as_bytes());
    ///
    /// assert_eq!(str.as_rslice(), bytes);
    ///
    /// ```
    #[inline]
    pub const fn as_rslice(&self) -> RSlice<'a, u8> {
        self.inner
    }

    conditionally_const! {
        feature = "rust_1_64"
        /// Casts this `RStr<'a>` to a `&'a str`.
        ///
        ;
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::std_types::RStr;
        ///
        /// let str = "What is that.";
        /// assert_eq!(RStr::from(str).as_str(), str);
        ///
        /// ```
        #[inline]
        pub fn as_str(&self) -> &'a str {
            unsafe { str::from_utf8_unchecked(self.inner.as_slice()) }
        }
    }

    /// Gets a raw pointer to the start of the string slice.
    pub const fn as_ptr(&self) -> *const u8 {
        self.inner.as_ptr()
    }

    /// Gets the length(in bytes) of this `RStr<'a>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RStr;
    ///
    /// assert_eq!(RStr::from("").len(), 0);
    /// assert_eq!(RStr::from("a").len(), 1);
    /// assert_eq!(RStr::from("What").len(), 4);
    ///
    /// ```
    #[inline]
    pub const fn len(&self) -> usize {
        self.inner.len()
    }

    /// Queries whether this RStr is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RStr;
    ///
    /// assert_eq!(RStr::from("").is_empty(), true);
    /// assert_eq!(RStr::from("a").is_empty(), false);
    /// assert_eq!(RStr::from("What").is_empty(), false);
    ///
    /// ```
    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

unsafe impl<'a> Send for RStr<'a> {}
unsafe impl<'a> Sync for RStr<'a> {}

impl<'a> Default for RStr<'a> {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl<'a> Deref for RStr<'a> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

deref_coerced_impl_cmp_traits! {
    RStr<'_>;
    coerce_to = str,
    [
        String,
        str,
        &str,
        std::borrow::Cow<'_, str>,
        crate::std_types::RCowStr<'_>,
    ]
}

////////////////////////////////

impl<'a> From<RStr<'a>> for Cow<'a, str> {
    fn from(this: RStr<'a>) -> Cow<'a, str> {
        this.as_str().into()
    }
}

impl_into_rust_repr! {
    impl['a] Into<&'a str> for RStr<'a> {
        fn(this){
            this.as_str()
        }
    }
}

impl From<RStr<'_>> for String {
    fn from(this: RStr<'_>) -> String {
        this.as_str().into()
    }
}

impl From<RStr<'_>> for RString {
    fn from(this: RStr<'_>) -> RString {
        this.as_str().into()
    }
}

impl_from_rust_repr! {
    impl['a] From<&'a str> for RStr<'a> {
        fn(this){
            RStr {
                inner: this.as_bytes().into(),
            }
        }
    }
}

////////////////////////////////

impl<'a> Borrow<str> for RStr<'a> {
    fn borrow(&self) -> &str {
        self
    }
}

impl AsRef<str> for RStr<'_> {
    fn as_ref(&self) -> &str {
        self
    }
}

impl AsRef<[u8]> for RStr<'_> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

////////////////////////////////

impl Display for RStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl<'de> Deserialize<'de> for RStr<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <&'de str as Deserialize<'de>>::deserialize(deserializer).map(Self::from)
    }
}

impl<'a> Serialize for RStr<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

#[allow(dead_code)]
type Str<'a> = &'a str;

shared_impls! {
    mod = slice_impls
    new_type = RStr['a][],
    original_type = Str,
}

////////////////////////////////////////////////////

//#[cfg(test)]
#[cfg(all(test, not(feature = "only_new_tests")))]
mod test {
    use super::*;

    #[test]
    fn from_to_str() {
        const RS: RStr<'_> = RStr::from_str("foo bar");

        let string = "what the hell";
        let rstr = RStr::from_str(string);

        assert_eq!(rstr, string);
        assert_eq!(RS, "foo bar");
    }

    #[cfg(feature = "rust_1_64")]
    #[test]
    fn const_as_str() {
        const RS: RStr<'_> = RStr::from_str("Hello, world!");
        const S: &str = RS.as_str();

        assert_eq!(S, "Hello, world!");
    }
}
