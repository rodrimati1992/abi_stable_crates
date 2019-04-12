use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    iter::{FromIterator, FusedIterator},
    mem,
    ops::{Deref, Index, Range},
    str::{from_utf8, from_utf8_unchecked, Chars,FromStr, Utf8Error},
    string::FromUtf16Error,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[allow(unused_imports)]
use core_extensions::{prelude::*, SliceExt};

use crate::std_types::{RStr, RVec};

mod iters;

#[cfg(test)]
mod tests;

pub use self::iters::{Drain, IntoIter};

/// Ffi-safe equivalent of ::std::string::String
#[derive(Clone)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct RString {
    inner: RVec<u8>,
}

impl RString {
    pub fn new() -> Self {
        String::new().into()
    }

    pub fn with_capacity(cap:usize) -> Self {
        String::with_capacity(cap).into()
    }

    /// For slicing into `RStr`s.
    ///
    /// This is an inherent method instead of an implementation of the
    /// ::std::ops::Index trait because it does not return a reference.
    #[inline]
    pub fn slice<'a, I>(&'a self, i: I) -> RStr<'a>
    where
        str: Index<I, Output = str>,
    {
        (&self[i]).into()
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &*self
    }

    #[inline]
    pub fn as_rstr(&self) -> RStr<'_> {
        self.as_str().into()
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    #[inline]
    pub unsafe fn from_utf8_unchecked<V>(vec: RVec<u8>) -> Self
    where
        V: Into<RVec<u8>>,
    {
        RString { inner: vec.into() }
    }

    pub fn from_utf8<V>(vec: V) -> Result<Self, FromUtf8Error>
    where
        V: Into<RVec<u8>>,
    {
        let vec = vec.into();
        match from_utf8(&*vec) {
            Ok(..) => Ok(RString { inner: vec }),
            Err(e) => Err(FromUtf8Error {
                bytes: vec,
                error: e,
            }),
        }
    }

    pub fn from_utf16(s: &[u16]) -> Result<Self, FromUtf16Error> {
        String::from_utf16(s).map(From::from)
    }
    pub fn into_bytes(self) -> RVec<u8> {
        self.inner
    }
    pub fn into_string(self) -> String {
        unsafe { String::from_utf8_unchecked(self.inner.into_vec()) }
    }
    pub fn to_string(&self) -> String {
        unsafe { String::from_utf8_unchecked(self.inner.to_vec()) }
    }

    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional);
    }

    pub fn push(&mut self, ch: char) {
        match ch.len_utf8() {
            1 => self.inner.push(ch as u8),
            _ => self
                .inner
                .extend_from_copy_slice(ch.encode_utf8(&mut [0; 4]).as_bytes()),
        }
    }

    pub fn push_str(&mut self, s: &str) {
        self.inner.extend_from_copy_slice(s.as_bytes());
    }
}

impl Default for RString {
    fn default() -> Self {
        String::new().into()
    }
}

////////////////////

impl_into_rust_repr! {
    impl Into<String> for RString {
        fn(this){
            this.into_string()
        }
    }
}

impl<'a> Into<Cow<'a, str>> for RString {
    fn into(self) -> Cow<'a, str> {
        self.into_string().piped(Cow::Owned)
    }
}

impl<'a> From<&'a str> for RString {
    fn from(this: &'a str) -> Self {
        this.to_owned().into()
    }
}

impl_from_rust_repr! {
    impl From<String> for RString {
        fn(this){
            RString {
                inner: this.into_bytes().into(),
            }
        }
    }
}

impl<'a> From<Cow<'a, str>> for RString {
    fn from(this: Cow<'a, str>) -> Self {
        this.into_owned().into()
    }
}

////////////////////


impl FromStr for RString {
    type Err = <String as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<String>().map(RString::from)
    }
}


////////////////////


impl AsRef<str> for RString{
    fn as_ref(&self)->&str{
        self
    }
}


////////////////////


impl Deref for RString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { from_utf8_unchecked(self.inner.as_slice()) }
    }
}

impl Display for RString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl fmt::Write for RString {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }

    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.push(c);
        Ok(())
    }
}

shared_impls! {
    mod=string_impls
    new_type=RString[][],
    original_type=str,
}

impl<'de> Deserialize<'de> for RString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(From::from)
    }
}

impl Serialize for RString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

//////////////////////////////////////////////////////

impl RString {
    pub fn drain<I>(&mut self, index: I) -> Drain<'_>
    where
        str: Index<I, Output = str>,
    {
        let string = self as *mut _;
        let slic_ = &(*self)[index];
        let start = self.offset_of_slice(slic_);
        let end = start + slic_.len();
        Drain {
            string,
            removed: start..end,
            iter: slic_.chars(),
        }
    }
}

impl IntoIterator for RString {
    type Item = char;

    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        unsafe {
            // Make sure that the buffer is not deallocated as long as the iterator is accessible.
            let text = mem::transmute::<&str, &'static str>(&self);
            unsafe {
                IntoIter {
                    iter: text.chars(),
                    _buf: self,
                }
            }
        }
    }
}

impl FromIterator<char> for RString {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = char>,
    {
        iter.piped(String::from_iter).piped(Self::from)
    }
}

impl<'a> FromIterator<&'a char> for RString {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a char>,
    {
        iter.piped(String::from_iter).piped(Self::from)
    }
}

//////////////////////////////////////////////////////

/// Error that happens when attempting to convert an `RVec<u8>` into an RString.
#[derive(Debug)]
pub struct FromUtf8Error {
    bytes: RVec<u8>,
    error: Utf8Error,
}

impl FromUtf8Error{
    pub fn into_bytes(self)->RVec<u8>{
        self.bytes
    }
    pub fn error(&self)->Utf8Error{
        self.error
    }
}

impl fmt::Display for FromUtf8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.error, f)
    }
}
