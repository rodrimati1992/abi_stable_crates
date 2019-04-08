use std::{
    borrow::Cow,
    fmt::{self, Display},
    ops::Deref,
    str,
};

use core_extensions::prelude::*;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::std_types::{RSlice, RString};

/// Type used to represent a Vec<u8> in any language.
///
/// This allows sharing a Vec<u8> between different versions of Rust,
/// even ones with a different allocator

#[repr(C)]
#[derive(Copy, Clone, StableAbi)]
#[sabi(inside_abi_stable_crate)]
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
    pub const fn empty() -> Self {
        Self::EMPTY
    }

    pub const unsafe fn from_raw_parts(ptr_: *const u8, len: usize) -> Self {
        Self {
            inner: RSlice::from_raw_parts(ptr_, len),
        }
    }

    pub fn from_str(s: &'a str) -> Self {
        unsafe {
            Self {
                inner: s.as_bytes().into(),
            }
        }
    }

    pub fn map_slice<F>(self, f: F) -> RStr<'a>
    where
        F: FnOnce(&'a str) -> &'a str,
    {
        self.as_str().piped(f).into()
    }

    #[inline]
    pub fn as_rslice(&self) -> RSlice<'a, u8> {
        self.inner
    }

    pub fn as_str(&self) -> &'a str {
        unsafe { str::from_utf8_unchecked(self.inner.as_slice()) }
    }

    #[inline]
    pub fn len(&self) -> usize {
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

impl<'a> AsRef<str> for RStr<'a>{
    fn as_ref(&self)->&str{
        self
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_to_str() {
        let a = "what the hell";
        let b = RStr::from_str(a);

        assert_eq!(a, &*b);
    }
}
