//! Contains an ffi-safe equivalent of `std::string::String`.

use std::{
    borrow::{Borrow, Cow},
    fmt::{self, Display, Formatter},
    iter::{FromIterator, FusedIterator},
    marker::PhantomData,
    ops::{Deref, Index, Range},
    ptr,
    str::{from_utf8, Chars, FromStr, Utf8Error},
    string::FromUtf16Error,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[allow(unused_imports)]
use core_extensions::{SelfOps, SliceExt, StringExt};

use crate::std_types::{RStr, RVec};

mod iters;

#[cfg(test)]
// #[cfg(all(test, not(feature = "only_new_tests")))]
mod tests;

pub use self::iters::{Drain, IntoIter};

/// Ffi-safe equivalent of `std::string::String`.
///
/// # Example
///
/// This defines a function returning the last word of an `RString`.
///
/// ```
/// use abi_stable::{sabi_extern_fn, std_types::RString};
///
/// #[sabi_extern_fn]
/// fn first_word(phrase: RString) -> RString {
///     match phrase.split_whitespace().next_back() {
///         Some(x) => x.into(),
///         None => RString::new(),
///     }
/// }
///
///
/// ```
///
#[derive(Clone)]
#[repr(C)]
#[derive(StableAbi)]
pub struct RString {
    inner: RVec<u8>,
}

impl RString {
    /// Creates a new, empty RString.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// let str = RString::new();
    ///
    /// assert_eq!(&str[..], "");
    ///
    /// ```
    pub const fn new() -> Self {
        Self::NEW
    }

    const NEW: Self = Self { inner: RVec::new() };

    /// Creates a new,
    /// empty RString with the capacity for `cap` bytes without reallocating.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// let str = RString::with_capacity(10);
    ///
    /// assert_eq!(&str[..], "");
    /// assert_eq!(str.capacity(), 10);
    ///
    /// ```
    pub fn with_capacity(cap: usize) -> Self {
        String::with_capacity(cap).into()
    }

    /// For slicing into `RStr`s.
    ///
    /// This is an inherent method instead of an implementation of the
    /// `std::ops::Index` trait because it does not return a reference.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RStr, RString};
    ///
    /// let str = RString::from("What is that.");
    ///
    /// assert_eq!(str.slice(..), RStr::from("What is that."));
    /// assert_eq!(str.slice(..4), RStr::from("What"));
    /// assert_eq!(str.slice(4..), RStr::from(" is that."));
    /// assert_eq!(str.slice(4..7), RStr::from(" is"));
    ///
    /// ```
    #[inline]
    #[allow(clippy::needless_lifetimes)]
    pub fn slice<'a, I>(&'a self, i: I) -> RStr<'a>
    where
        str: Index<I, Output = str>,
    {
        (&self[i]).into()
    }

    conditionally_const! {
        feature = "rust_1_64"
        /// Creates a `&str` with access to all the characters of the `RString`.
        ///
        ;
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::std_types::RString;
        ///
        /// let str = "What is that.";
        /// assert_eq!(RString::from(str).as_str(), str);
        ///
        /// ```
        #[inline]
        pub fn as_str(&self) -> &str {
            unsafe { std::str::from_utf8_unchecked(self.inner.as_slice()) }
        }
    }

    /// Creates an `RStr<'_>` with access to all the characters of the `RString`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RStr, RString};
    ///
    /// let str = "What is that.";
    /// assert_eq!(RString::from(str).as_rstr(), RStr::from(str),);
    ///
    /// ```
    #[inline]
    pub const fn as_rstr(&self) -> RStr<'_> {
        unsafe { RStr::from_raw_parts(self.as_ptr(), self.len()) }
    }

    /// Returns the current length (in bytes) of the RString.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// assert_eq!(RString::from("").len(), 0);
    /// assert_eq!(RString::from("a").len(), 1);
    /// assert_eq!(RString::from("Regular").len(), 7);
    ///
    /// ```
    #[inline]
    pub const fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns whether the RString is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// assert_eq!(RString::from("").is_empty(), true);
    /// assert_eq!(RString::from("a").is_empty(), false);
    /// assert_eq!(RString::from("Regular").is_empty(), false);
    ///
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Gets a raw pointer to the start of this RString's buffer.
    pub const fn as_ptr(&self) -> *const u8 {
        self.inner.as_ptr()
    }

    /// Returns the current capacity (in bytes) of the RString.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// let mut str = RString::with_capacity(13);
    ///
    /// assert_eq!(str.capacity(), 13);
    ///
    /// str.push_str("What is that.");
    /// assert_eq!(str.capacity(), 13);
    ///
    /// str.push(' ');
    /// assert_ne!(str.capacity(), 13);
    ///
    /// ```
    #[inline]
    pub const fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// An unchecked conversion from a `RVec<u8>` to an `RString`.
    ///
    /// # Safety
    ///
    /// This has the same safety requirements as
    /// [`String::from_utf8_unchecked`
    /// ](https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8_unchecked).
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::std_types::{RString, RVec};
    ///
    /// let bytes = RVec::from("hello".as_bytes());
    ///
    /// unsafe {
    ///     assert_eq!(RString::from_utf8_unchecked(bytes).as_str(), "hello");
    /// }
    ///
    /// ```
    #[inline]
    pub const unsafe fn from_utf8_unchecked(vec: RVec<u8>) -> Self {
        RString { inner: vec }
    }

    /// Converts the `vec` vector of bytes to an `RString`.
    ///
    /// # Errors
    ///
    /// This returns a `Err(FromUtf8Error{..})` if `vec` is not valid utf-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::std_types::{RString, RVec};
    ///
    /// let bytes_ok = RVec::from("hello".as_bytes());
    /// let bytes_err = RVec::from(vec![255]);
    ///
    /// assert_eq!(
    ///     RString::from_utf8(bytes_ok).unwrap(),
    ///     RString::from("hello")
    /// );
    /// assert!(RString::from_utf8(bytes_err).is_err());
    ///
    /// ```
    pub fn from_utf8<V>(vec: V) -> Result<Self, FromUtf8Error>
    where
        V: Into<RVec<u8>>,
    {
        let vec = vec.into();
        match from_utf8(&vec) {
            Ok(..) => Ok(RString { inner: vec }),
            Err(e) => Err(FromUtf8Error {
                bytes: vec,
                error: e,
            }),
        }
    }

    /// Decodes a utf-16 encoded `&[u16]` to an `RString`.
    ///
    /// # Errors
    ///
    /// This returns a `Err(::std::string::FromUtf16Error{..})`
    /// if `vec` is not valid utf-8.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// let str = "What the 😈.";
    /// let str_utf16 = str.encode_utf16().collect::<Vec<u16>>();
    ///
    /// assert_eq!(RString::from_utf16(&str_utf16).unwrap(), RString::from(str),);
    /// ```
    pub fn from_utf16(s: &[u16]) -> Result<Self, FromUtf16Error> {
        String::from_utf16(s).map(From::from)
    }

    /// Cheap conversion of this `RString` to a `RVec<u8>`
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RString, RVec};
    ///
    /// let bytes = RVec::from("hello".as_bytes());
    /// let str = RString::from("hello");
    ///
    /// assert_eq!(str.into_bytes(), bytes);
    ///
    /// ```
    #[allow(clippy::missing_const_for_fn)]
    pub fn into_bytes(self) -> RVec<u8> {
        self.inner
    }

    /// Converts this `RString` to a `String`.
    ///
    /// # Allocation
    ///
    /// If this is invoked outside of the dynamic library/binary that created it,
    /// it will allocate a new `String` and move the data into it.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// let std_str = String::from("hello");
    /// let str = RString::from("hello");
    ///
    /// assert_eq!(str.into_string(), std_str);
    ///
    /// ```
    pub fn into_string(self) -> String {
        unsafe { String::from_utf8_unchecked(self.inner.into_vec()) }
    }
    /// Copies the `RString` into a `String`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// assert_eq!(RString::from("world").to_string(), String::from("world"));
    ///
    /// ```
    #[allow(clippy::inherent_to_string_shadow_display)]
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }

    /// Reserves `àdditional` additional capacity for any extra string data.
    /// This may reserve more than necessary for the additional capacity.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// let mut str = RString::new();
    ///
    /// str.reserve(10);
    /// assert!(str.capacity() >= 10);
    ///
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    /// Shrinks the capacity of the RString to match its length.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// let mut str = RString::with_capacity(100);
    /// str.push_str("nope");
    /// str.shrink_to_fit();
    /// assert_eq!(str.capacity(), 4);
    ///
    /// ```
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit()
    }

    /// Reserves `àdditional` additional capacity for any extra string data.
    ///
    /// Prefer using `reserve` for most situations.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// let mut str = RString::new();
    ///
    /// str.reserve_exact(10);
    /// assert_eq!(str.capacity(), 10);
    ///
    /// ```
    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional);
    }

    /// Appends `ch` at the end of this RString.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// let mut str = RString::new();
    ///
    /// str.push('O');
    /// str.push('O');
    /// str.push('P');
    ///
    /// assert_eq!(str.as_str(), "OOP");
    ///
    /// ```
    pub fn push(&mut self, ch: char) {
        match ch.len_utf8() {
            1 => self.inner.push(ch as u8),
            _ => self.push_str(ch.encode_utf8(&mut [0; 4])),
        }
    }

    /// Appends `str` at the end of this RString.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// let mut str = RString::new();
    ///
    /// str.push_str("green ");
    /// str.push_str("frog");
    ///
    /// assert_eq!(str.as_str(), "green frog");
    ///
    /// ```
    pub fn push_str(&mut self, str: &str) {
        self.inner.extend_from_copy_slice(str.as_bytes());
    }

    /// Removes the last character,
    /// returns `Some(_)` if this `RString` is not empty,
    /// otherwise returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RString, RVec};
    ///
    /// let mut str = RString::from("yep");
    ///
    /// assert_eq!(str.pop(), Some('p'));
    /// assert_eq!(str.pop(), Some('e'));
    /// assert_eq!(str.pop(), Some('y'));
    /// assert_eq!(str.pop(), None);
    ///
    /// ```
    pub fn pop(&mut self) -> Option<char> {
        // literal copy-paste of std, so if this is wrong std is wrong.

        let ch = self.chars().rev().next()?;
        let newlen = self.len() - ch.len_utf8();
        unsafe {
            self.inner.set_len(newlen);
        }
        Some(ch)
    }

    /// Removes and returns the character starting at the `idx` byte position,
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds or if it is not on a char boundary.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RString, RVec};
    ///
    /// let mut str = RString::from("Galileo");
    ///
    /// assert_eq!(str.remove(3), 'i');
    /// assert_eq!(str.as_str(), "Galleo");
    ///
    /// assert_eq!(str.remove(4), 'e');
    /// assert_eq!(str.as_str(), "Gallo");
    ///
    /// ```
    pub fn remove(&mut self, idx: usize) -> char {
        // literal copy-paste of std, so if this is wrong std is wrong.

        let ch = match self[idx..].chars().next() {
            Some(ch) => ch,
            None => panic!("cannot remove a char beyond the end of a string"),
        };

        let next = idx + ch.len_utf8();
        let len = self.len();
        unsafe {
            let ptr = self.inner.as_mut_ptr();
            ptr::copy(ptr.add(next), ptr.add(idx), len - next);
            self.inner.set_len(len - (next - idx));
        }
        ch
    }

    /// Insert the `ch` character at the `ìdx` byte position.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds or if it is not on a char boundary.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RString, RVec};
    ///
    /// let mut str = RString::from("Cap");
    ///
    /// str.insert(1, 'r');
    /// assert_eq!(str.as_str(), "Crap");
    ///
    /// str.insert(4, 'p');
    /// assert_eq!(str.as_str(), "Crapp");
    ///
    /// str.insert(5, 'y');
    /// assert_eq!(str.as_str(), "Crappy");
    ///
    /// ```
    pub fn insert(&mut self, idx: usize, ch: char) {
        let mut bits = [0; 4];
        let str_ = ch.encode_utf8(&mut bits);

        self.insert_str(idx, str_);
    }

    /// Insert the `string` at the `ìdx` byte position.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds or if it is not on a char boundary.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RString, RVec};
    ///
    /// let mut str = RString::from("rust");
    ///
    /// str.insert_str(0, "T");
    /// assert_eq!(str.as_str(), "Trust");
    ///
    /// str.insert_str(5, " the source");
    /// assert_eq!(str.as_str(), "Trust the source");
    ///
    /// str.insert_str(5, " the types in");
    /// assert_eq!(str.as_str(), "Trust the types in the source");
    ///
    /// ```
    pub fn insert_str(&mut self, idx: usize, string: &str) {
        // literal copy-paste of std, so if this is wrong std is wrong.

        assert!(self.is_char_boundary(idx));

        unsafe {
            self.insert_bytes(idx, string.as_bytes());
        }
    }

    unsafe fn insert_bytes(&mut self, idx: usize, bytes: &[u8]) {
        let len = self.len();
        let amt = bytes.len();
        self.inner.reserve(amt);

        let ptr = self.inner.as_mut_ptr();
        unsafe {
            ptr::copy(ptr.add(idx), ptr.add(idx + amt), len - idx);
            ptr::copy(bytes.as_ptr(), self.inner.as_mut_ptr().add(idx), amt);
            self.inner.set_len(len + amt);
        }
    }

    /// Retains only the characters that satisfy the `pred` predicate
    ///
    /// This means that a character will be removed if `pred(that_character)`
    /// returns false.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RString, RVec};
    ///
    /// {
    ///     let mut str = RString::from("There were 10 people.");
    ///     str.retain(|c| !c.is_numeric());
    ///     assert_eq!(str.as_str(), "There were  people.");
    /// }
    /// {
    ///     let mut str = RString::from("There were 10 people.");
    ///     str.retain(|c| !c.is_whitespace());
    ///     assert_eq!(str.as_str(), "Therewere10people.");
    /// }
    /// {
    ///     let mut str = RString::from("There were 10 people.");
    ///     str.retain(|c| c.is_numeric());
    ///     assert_eq!(str.as_str(), "10");
    /// }
    ///
    /// ```
    #[inline]
    pub fn retain<F>(&mut self, mut pred: F)
    where
        F: FnMut(char) -> bool,
    {
        let len = self.len();
        let mut del_bytes = 0;
        let mut idx = 0;

        unsafe {
            self.inner.set_len(0);
        }

        let start = self.inner.as_mut_ptr();

        while idx < len {
            let curr = unsafe { start.add(idx) };

            let ch = unsafe {
                RStr::from_raw_parts(curr, len - idx)
                    .chars()
                    .next()
                    .unwrap()
            };
            let ch_len = ch.len_utf8();

            if !pred(ch) {
                del_bytes += ch_len;
            } else if del_bytes > 0 {
                unsafe {
                    ptr::copy(curr, curr.sub(del_bytes), ch_len);
                }
            }

            // Point idx to the next char
            idx += ch_len;
        }

        unsafe {
            self.inner.set_len(len - del_bytes);
        }
    }

    /// Turns this into an empty RString, keeping the same allocated buffer.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RString, RVec};
    ///
    /// let mut str = RString::from("Nurse");
    ///
    /// assert_eq!(str.as_str(), "Nurse");
    ///
    /// str.clear();
    ///
    /// assert_eq!(str.as_str(), "");
    ///
    /// ```
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

/// Returns an empty RString
impl Default for RString {
    fn default() -> Self {
        String::new().into()
    }
}

////////////////////

deref_coerced_impl_cmp_traits! {
    RString;
    coerce_to = str,
    [
        String,
        str,
        &str,
        RStr<'_>,
        std::borrow::Cow<'_, str>,
        crate::std_types::RCowStr<'_>,
    ]
}

////////////////////

impl_into_rust_repr! {
    impl Into<String> for RString {
        fn(this){
            this.into_string()
        }
    }
}

impl<'a> From<RString> for Cow<'a, str> {
    fn from(this: RString) -> Cow<'a, str> {
        this.into_string().piped(Cow::Owned)
    }
}

impl From<&str> for RString {
    fn from(this: &str) -> Self {
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

impl Borrow<str> for RString {
    fn borrow(&self) -> &str {
        self
    }
}

impl AsRef<str> for RString {
    fn as_ref(&self) -> &str {
        self
    }
}

impl AsRef<[u8]> for RString {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

////////////////////

impl Deref for RString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
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
    mod = string_impls
    new_type = RString[][],
    original_type = str,
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
    /// Creates an iterator that yields the chars in the `range`,
    /// removing the characters in that range in the process.
    ///
    /// # Panic
    ///
    /// Panics if the start or end of the range are not on a on a char boundary,
    /// or if either are out of bounds.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// let orig = "Not a single way";
    ///
    /// {
    ///     let mut str = RString::from(orig);
    ///     assert_eq!(str.drain(..).collect::<String>(), orig,);
    ///     assert_eq!(str.as_str(), "");
    /// }
    /// {
    ///     let mut str = RString::from(orig);
    ///     assert_eq!(str.drain(..4).collect::<String>(), "Not ",);
    ///     assert_eq!(str.as_str(), "a single way");
    /// }
    /// {
    ///     let mut str = RString::from(orig);
    ///     assert_eq!(str.drain(4..).collect::<String>(), "a single way",);
    ///     assert_eq!(str.as_str(), "Not ");
    /// }
    /// {
    ///     let mut str = RString::from(orig);
    ///     assert_eq!(str.drain(4..13).collect::<String>(), "a single ",);
    ///     assert_eq!(str.as_str(), "Not way");
    /// }
    ///
    /// ```
    pub fn drain<I>(&mut self, range: I) -> Drain<'_>
    where
        str: Index<I, Output = str>,
    {
        let string = self as *mut _;
        let slic_ = &(*self)[range];
        let start = self.offset_of_slice(slic_);
        let end = start + slic_.len();
        Drain {
            string,
            removed: start..end,
            iter: slic_.chars(),
            variance: PhantomData,
        }
    }
}

impl IntoIterator for RString {
    type Item = char;

    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        unsafe {
            // Make sure that the buffer is not deallocated as long as the iterator is accessible.
            let text: &'static str = &*(&*self as *const str);
            IntoIter {
                iter: text.chars(),
                _buf: self,
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

/// Error that happens when attempting to convert an `RVec<u8>` into an `RString`.
///
/// # Example
///
/// ```
/// use abi_stable::std_types::RString;
///
/// let err = RString::from_utf8(vec![0, 0, 0, 255]).unwrap_err();
///
/// assert_eq!(err.as_bytes(), &[0, 0, 0, 255])
///
/// ```
#[derive(Debug)]
pub struct FromUtf8Error {
    bytes: RVec<u8>,
    error: Utf8Error,
}

#[allow(clippy::missing_const_for_fn)]
impl FromUtf8Error {
    /// Unwraps this error into the bytes that failed to be converted into an `RString`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RString, RVec};
    ///
    /// let bytes: RVec<u8> = vec![72, 111, 95, 95, 95, 95, 95, 99, 107, 255].into();
    ///
    /// let err = RString::from_utf8(bytes.clone()).unwrap_err();
    ///
    /// assert_eq!(err.into_bytes(), bytes);
    ///
    /// ```
    pub fn into_bytes(self) -> RVec<u8> {
        self.bytes
    }
    /// Gets access to the bytes that failed to be converted into an `RString`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// let bytes = vec![99, 114, 121, 115, 116, 97, 108, 255];
    ///
    /// let err = RString::from_utf8(bytes.clone()).unwrap_err();
    ///
    /// assert_eq!(err.as_bytes(), &bytes[..]);
    ///
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Gets a Utf8Error with information about the conversion error.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// let err = RString::from_utf8(vec![0, 0, 255]).unwrap_err();
    ///
    /// assert_eq!(err.error().valid_up_to(), 2);
    ///
    /// ```
    pub fn error(&self) -> Utf8Error {
        self.error
    }
}

impl fmt::Display for FromUtf8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.error, f)
    }
}

impl std::error::Error for FromUtf8Error {}
