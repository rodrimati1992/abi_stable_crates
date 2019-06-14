use std::{
    borrow::{Cow,Borrow},
    fmt::{self, Display},
    ops::{Deref, Index},
    str,
};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::std_types::{RSlice, RString};

/// Ffi-safe equivalent of `&'a str`
#[repr(C)]
#[derive(Copy, Clone, StableAbi)]
pub struct RStr<'a> {
    inner: RSlice<'a, u8>,
}

impl<'a> RStr<'a> {
    pub const EMPTY: Self = RStr {
        inner: RSlice::EMPTY,
    };
}

impl RStr<'static> {
    #[doc(hidden)]
    pub const fn _private_from_raw_parts(ptr_: *const u8, len: usize) -> Self {
        unsafe { Self::from_raw_parts(ptr_, len) }
    }
}

impl<'a> RStr<'a> {
    /// Constructs an empty `RStr<'a>`.
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
    /// - ptr_ points to valid memory,
    ///
    /// - `ptr_ .. ptr+len` range is accessible memory,and is valid utf-8.
    ///
    /// - the data ptr_ points to must be valid for the lifetime of this `RStr<'a>`
    #[inline]
    pub const unsafe fn from_raw_parts(ptr_: *const u8, len: usize) -> Self {
        Self {
            inner: RSlice::from_raw_parts(ptr_, len),
        }
    }

    /// Converts `&'a str` to a `RStr<'a>`.
    #[inline]
    pub fn from_str(s: &'a str) -> Self {
        unsafe {
            Self {
                inner: s.as_bytes().into(),
            }
        }
    }

    /// For slicing `RStr`s.
    ///
    /// This is an inherent method instead of an implementation of the
    /// ::std::ops::Index trait because it does not return a reference.
    pub fn slice<I>(&self, i: I) -> RStr<'a>
    where
        str: Index<I, Output = str>,
    {
        self.as_str().index(i).into()
    }

    /// Accesses the underlying byte slice.
    #[inline]
    pub fn as_rslice(&self) -> RSlice<'a, u8> {
        self.inner
    }

    /// Casts this `RStr<'a>` to a `&'a str`.
    #[inline]
    pub fn as_str(&self) -> &'a str {
        unsafe { str::from_utf8_unchecked(self.inner.as_slice()) }
    }

    /// Gets the length(in bytes) of this `RStr<'a>`.
    #[inline]
    pub const fn len(&self) -> usize {
        self.inner.len()
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

////////////////////////////////

impl<'a> Into<Cow<'a, str>> for RStr<'a> {
    fn into(self) -> Cow<'a, str> {
        self.as_str().into()
    }
}

impl_into_rust_repr! {
    impl['a] Into<&'a str> for RStr<'a> {
        fn(this){
            this.as_str()
        }
    }
}

impl<'a> Into<String> for RStr<'a> {
    fn into(self) -> String {
        self.as_str().into()
    }
}

impl<'a> Into<RString> for RStr<'a> {
    fn into(self) -> RString {
        self.as_str().into()
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

impl<'a> Borrow<str> for RStr<'a>{
    fn borrow(&self)->&str{
        self
    }
}

impl AsRef<str> for RStr<'_>{
    fn as_ref(&self)->&str{
        self
    }
}

impl AsRef<[u8]> for RStr<'_>{
    fn as_ref(&self)->&[u8]{
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
    mod=slice_impls
    new_type=RStr['a][],
    original_type=Str,
}

////////////////////////////////////////////////////

//#[cfg(test)]
#[cfg(all(test,not(feature="only_new_tests")))]
mod test {
    use super::*;

    #[test]
    fn from_to_str() {
        let a = "what the hell";
        let b = RStr::from_str(a);

        assert_eq!(a, &*b);
    }
}
