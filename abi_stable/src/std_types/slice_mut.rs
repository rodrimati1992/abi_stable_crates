use std::{
    io::{self, Write},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut, Index, IndexMut},
};

use serde::{Serialize, Serializer};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use crate::std_types::{RSlice, RVec};

mod privacy {
    use super::*;
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(inside_abi_stable_crate)]
    #[sabi(bound = "T:'a")]
    pub struct RSliceMut<'a, T> {
        data: *mut T,
        length: usize,
        _marker: PhantomData<&'a mut T>,
    }

    impl_from_rust_repr! {
        impl['a, T] From<&'a mut [T]> for RSliceMut<'a, T> {
            fn(this){
                RSliceMut {
                    data: this.as_mut_ptr(),
                    length: this.len(),
                    _marker: Default::default(),
                }
            }
        }
    }

    impl<'a, T> RSliceMut<'a, T> {
        #[inline(always)]
        pub(super) fn data(&self) -> *mut T {
            self.data
        }

        #[inline(always)]
        pub fn len(&self) -> usize {
            self.length
        }
    }
}
pub use self::privacy::RSliceMut;

impl<'a, T> RSliceMut<'a, T> {
    // pub const fn empty() -> Self {
    //     Self::EMPTY
    // }

    /// For slicing RSliceMut.
    ///
    /// This is an inherent method instead of an implementation of the
    /// ::std::ops::Index trait because it does not return a reference.
    pub fn slice<I>(&self, i: I) -> RSlice<'_, T>
    where
        [T]: Index<I, Output = [T]>,
    {
        self.as_slice().index(i).into()
    }

    pub fn slice_mut<'b, I>(&'b mut self, i: I) -> RSliceMut<'b, T>
    where
        [T]: IndexMut<I, Output = [T]>,
    {
        self.as_slice_mut().index_mut(i).into()
    }

    pub fn to_rvec(&self) -> RVec<T>
    where
        T: Clone,
    {
        self.to_vec().into()
    }

    unsafe fn as_slice_unbounded(&self) -> &'a [T] {
        ::std::slice::from_raw_parts(self.data(), self.len())
    }

    unsafe fn as_slice_mut_unbounded(&mut self) -> &'a mut [T] {
        ::std::slice::from_raw_parts_mut(self.data(), self.len())
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { self.as_slice_unbounded() }
    }

    pub fn into_slice(self) -> &'a [T] {
        unsafe { self.as_slice_unbounded() }
    }

    pub fn into_rslice(self) -> RSlice<'a, T> {
        self.into_slice().into()
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { self.as_slice_mut_unbounded() }
    }

    pub fn into_slice_mut(mut self) -> &'a mut [T] {
        unsafe { self.as_slice_mut_unbounded() }
    }
}

unsafe impl<'a, T> Send for RSliceMut<'a, T> where &'a mut [T]: Send {}
unsafe impl<'a, T> Sync for RSliceMut<'a, T> where &'a mut [T]: Sync {}

impl<'a, T> Default for RSliceMut<'a, T> {
    fn default() -> Self {
        (&mut [][..]).into()
    }
}

impl<'a, T> IntoIterator for RSliceMut<'a, T> {
    type Item = &'a mut T;

    type IntoIter = ::std::slice::IterMut<'a, T>;

    fn into_iter(self) -> ::std::slice::IterMut<'a, T> {
        self.into_slice_mut().into_iter()
    }
}

impl<'a, T> Deref for RSliceMut<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'a, T> DerefMut for RSliceMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

////////////////////////////

impl_into_rust_repr! {
    impl['a, T] Into<&'a mut [T]> for RSliceMut<'a, T> {
        fn(this){
            this.into_slice_mut()
        }
    }
}

impl<'a, T> Into<&'a [T]> for RSliceMut<'a, T> {
    fn into(self) -> &'a [T] {
        self.into_slice()
    }
}


////////////////////


impl<'a,T:'a> AsRef<[T]> for RSliceMut<'a,T>{
    fn as_ref(&self)->&[T]{
        self
    }
}

impl<'a,T:'a> AsMut<[T]> for RSliceMut<'a,T>{
    fn as_mut(&mut self)->&mut [T]{
        self
    }
}


////////////////////////////
impl<'a, T> Serialize for RSliceMut<'a, T>
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

impl<'a> Write for RSliceMut<'a, u8> {
    #[inline]
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let mut this = mem::replace(self, Self::default()).into_slice_mut();
        let ret = this.write(data);
        *self = this.into();
        ret
    }

    #[inline]
    fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        let mut this = mem::replace(self, Self::default()).into_slice_mut();
        let ret = this.write_all(data);
        *self = this.into();
        ret
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
type SliceMut<'a, T> = &'a mut [T];

shared_impls! {
    mod=slice_impls
    new_type=RSliceMut['a][T],
    original_type=SliceMut,
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_to_slice() {
        let a = b"what the hell".to_vec();
        let mut a_clone = a.clone();
        let a_addr = a_clone.as_ptr();
        let mut b = RSliceMut::from(&mut a_clone[..]);

        assert_eq!(&*a, &*b);
        assert_eq!(&*a, &mut *b);
        assert_eq!(a_addr, b.data());
        assert_eq!(a.len(), b.len());
    }
}
