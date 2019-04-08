use std::{
    io::{self, BufRead, Read},
    marker::PhantomData,
    ops::{Deref, Index},
};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::std_types::{RVec};

mod private {
    use super::*;

    /// Type used to represent a Vec<u8> in any language.
    ///
    /// This allows sharing a Vec<u8> between different versions of Rust,
    /// even ones with a different allocator
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(inside_abi_stable_crate)]
    #[sabi(bound = "T:'a")]
    //#[sabi(debug_print)]
    pub struct RSlice<'a, T> {
        data: *const T,
        length: usize,
        _marker: PhantomData<&'a T>,
    }

    impl_from_rust_repr! {
        impl['a, T] From<&'a [T]> for RSlice<'a, T> {
            fn(this){
                RSlice {
                    data: this.as_ptr(),
                    length: this.len(),
                    _marker: Default::default(),
                }
            }
        }
    }

    impl<'a, T: 'a> RSlice<'a, T> {
        pub const EMPTY: Self = RSlice {
            data: {
                let v: &[T] = &[];
                v.as_ptr()
            },
            length: 0,
            _marker: PhantomData,
        };
        pub const unsafe fn from_raw_parts(ptr_: *const T, len: usize) -> Self {
            Self {
                data: ptr_,
                length: len,
                _marker: PhantomData,
            }
        }
    }
    impl<T: 'static> RSlice<'static, T> {
        #[doc(hidden)]
        pub const fn _private_from_raw_parts(ptr_: *const T, len: usize) -> Self {
            unsafe { Self::from_raw_parts(ptr_, len) }
        }
    }

    impl<'a, T> RSlice<'a, T> {
        pub fn as_slice(&self) -> &'a [T] {
            unsafe { ::std::slice::from_raw_parts(self.data, self.length) }
        }

        pub fn len(&self) -> usize {
            self.length
        }
    }
}

pub use self::private::RSlice;

impl<'a, T> RSlice<'a, T> {
    pub const fn empty() -> Self {
        Self::EMPTY
    }

    /// For slicing RSlices.
    ///
    /// This is an inherent method instead of an implementation of the
    /// ::std::ops::Index trait because it does not return a reference.
    pub fn slice<I>(&self, i: I) -> RSlice<'a, T>
    where
        [T]: Index<I, Output = [T]>,
    {
        self.as_slice().index(i).into()
    }

    pub fn to_rvec(&self) -> RVec<T>
    where
        T: Clone,
    {
        self.to_vec().into()
    }
}

unsafe impl<'a, T> Send for RSlice<'a, T> where &'a [T]: Send {}
unsafe impl<'a, T> Sync for RSlice<'a, T> where &'a [T]: Sync {}

impl<'a, T> Copy for RSlice<'a, T> {}

impl<'a, T> Clone for RSlice<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Default for RSlice<'a, T> {
    fn default() -> Self {
        (&[][..]).into()
    }
}

impl<'a, T> IntoIterator for RSlice<'a, T> {
    type Item = &'a T;

    type IntoIter = ::std::slice::Iter<'a, T>;

    fn into_iter(self) -> ::std::slice::Iter<'a, T> {
        self.as_slice().into_iter()
    }
}

impl<'a, T: 'a> Deref for RSlice<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl_into_rust_repr! {
    impl['a, T] Into<&'a [T]> for RSlice<'a, T> {
        fn(this){
            this.as_slice()
        }
    }
}

////////////////////


impl<'a,T:'a> AsRef<[T]> for RSlice<'a,T>{
    fn as_ref(&self)->&[T]{
        self
    }
}


///////////////////

impl<'de, T> Deserialize<'de> for RSlice<'de, T>
where
    &'de [T]: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <&'de [T] as Deserialize<'de>>::deserialize(deserializer).map(Self::from)
    }
}

impl<'a, T> Serialize for RSlice<'a, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_slice().serialize(serializer)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<'a> Read for RSlice<'a, u8> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut this = self.as_slice();
        let ret = this.read(buf);
        *self = this.into();
        ret
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let mut this = self.as_slice();
        let ret = this.read_exact(buf);
        *self = this.into();
        ret
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut this = self.as_slice();
        let ret = this.read_to_end(buf);
        *self = this.into();
        ret
    }
}

impl<'a> BufRead for RSlice<'a, u8> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(&**self)
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        *self = self.slice(amt..);
    }
}

///////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
type Slice<'a, T> = &'a [T];

shared_impls! {
    mod=slice_impls
    new_type=RSlice['a][T],
    original_type=Slice,
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_to_slice() {
        let a = "what the hell".as_bytes();
        let b = RSlice::from(a);

        assert_eq!(a, &*b);
        assert_eq!(a.len(), b.len());
    }
}
