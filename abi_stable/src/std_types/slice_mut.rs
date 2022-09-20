//! Contains the ffi-safe equivalent of `&'a mut [T]`.

use std::{
    borrow::{Borrow, BorrowMut},
    io::{self, Write},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::SliceIndex,
};

use serde::{Serialize, Serializer};

#[allow(unused_imports)]
use core_extensions::SelfOps;

use crate::std_types::{RSlice, RVec};

mod privacy {
    use super::*;

    /// Ffi-safe equivalent of `&'a mut [T]`
    ///
    /// As of the writing this documentation the abi stability of `&mut [T]` is
    /// not yet guaranteed.
    ///
    /// # Lifetime problems
    ///
    /// Because `RSliceMut` dereferences into a mutable slice, you can call slice methods on it.
    ///
    /// If you call a slice method that returns a borrow into the slice,
    /// it will have the lifetime of the `let slice: RSliceMut<'a, [T]>` variable instead of the `'a`
    /// lifetime that it's parameterized over.
    ///
    /// To get a slice with the same lifetime as an `RSliceMut`,
    /// one must use one of the `RSliceMut::{into_slice, into_mut_slice}` methods.
    ///
    ///
    /// Example of what would not work:
    ///
    /// ```compile_fail
    /// use abi_stable::std_types::RSliceMut;
    ///
    /// fn into_slice<'a, T>(slic: RSliceMut<'a, T>) -> &'a [T] {
    ///     &*slic
    /// }
    ///
    /// fn into_mut_slice<'a, T>(slic: RSliceMut<'a, T>) -> &'a mut [T] {
    ///     &mut *slic
    /// }
    /// ```
    ///
    /// Example of what would work:
    ///
    /// ```
    /// use abi_stable::std_types::RSliceMut;
    ///
    /// fn into_slice<'a, T>(slic: RSliceMut<'a, T>) -> &'a [T] {
    ///     slic.into_slice()
    /// }
    ///
    /// fn into_mut_slice<'a, T>(slic: RSliceMut<'a, T>) -> &'a mut [T] {
    ///     slic.into_mut_slice()
    /// }
    ///
    /// ```
    ///
    ///
    /// # Example
    ///
    /// Defining an extern fn that returns a mutable reference to
    /// the first element that compares equal to a parameter.
    ///
    /// ```
    /// use abi_stable::{sabi_extern_fn, std_types::RSliceMut};
    ///
    /// #[sabi_extern_fn]
    /// pub fn find_first_mut<'a, T>(
    ///     slice_: RSliceMut<'a, T>,
    ///     element: &T,
    /// ) -> Option<&'a mut T>
    /// where
    ///     T: std::cmp::PartialEq,
    /// {
    ///     slice_
    ///         .iter()
    ///         .position(|x| x == element)
    ///         .map(|i| &mut slice_.into_mut_slice()[i])
    /// }
    /// ```
    ///
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(bound(T: 'a))]
    pub struct RSliceMut<'a, T> {
        data: *mut T,
        length: usize,
        _marker: PhantomData<MutWorkaround<'a, T>>,
    }

    /// Used as a workaround to make `from_raw_parts_mut` a const fn.
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(bound(T: 'a))]
    struct MutWorkaround<'a, T>(&'a mut T);

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
        pub(super) const fn data(&self) -> *mut T {
            self.data
        }

        /// Gets a raw pointer to the start of the slice.
        pub const fn as_ptr(&self) -> *const T {
            self.data
        }

        /// Gets a mutable raw pointer to the start of the slice.
        pub fn as_mut_ptr(&mut self) -> *mut T {
            self.data
        }

        /// Gets a mutable raw pointer to the start of the slice.
        pub const fn into_mut_ptr(self) -> *mut T {
            self.data
        }

        /// The length (in elements) of this slice.
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::std_types::RSliceMut;
        ///
        /// assert_eq!(RSliceMut::<u8>::from_mut_slice(&mut []).len(), 0);
        /// assert_eq!(RSliceMut::from_mut_slice(&mut [0]).len(), 1);
        /// assert_eq!(RSliceMut::from_mut_slice(&mut [0, 1]).len(), 2);
        ///
        /// ```
        #[inline(always)]
        pub const fn len(&self) -> usize {
            self.length
        }

        /// Whether this slice is empty.
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::std_types::RSliceMut;
        ///
        /// assert_eq!(RSliceMut::<u8>::from_mut_slice(&mut []).is_empty(), true);
        /// assert_eq!(RSliceMut::from_mut_slice(&mut [0]).is_empty(), false);
        /// assert_eq!(RSliceMut::from_mut_slice(&mut [0, 1]).is_empty(), false);
        ///
        /// ```
        #[inline]
        pub const fn is_empty(&self) -> bool {
            self.length == 0
        }

        /// Constructs an `RSliceMut<'a, T>` from a pointer to the first element,
        /// and a length.
        ///
        /// # Safety
        ///
        /// Callers must ensure that:
        ///
        /// - `ptr_` points to valid memory,
        ///
        /// - `ptr_ .. ptr+len` range is accessible memory.
        ///
        /// - `ptr_` is aligned to `T`.
        ///
        /// - the data `ptr_` points to must be valid for the lifetime of this `RSlice<'a, T>`
        ///
        /// # Examples
        ///
        /// This function unsafely converts a `&mut [T]` to an `RSliceMut<T>`,
        /// equivalent to doing `RSliceMut::from_mut_slice`.
        ///
        /// ```
        /// use abi_stable::std_types::RSliceMut;
        ///
        /// fn convert<T>(slice_: &mut [T]) -> RSliceMut<'_, T> {
        ///     let len = slice_.len();
        ///     unsafe { RSliceMut::from_raw_parts_mut(slice_.as_mut_ptr(), len) }
        /// }
        ///
        /// ```
        pub const unsafe fn from_raw_parts_mut(ptr_: *mut T, len: usize) -> Self {
            Self {
                data: ptr_,
                length: len,
                // WHAT!?
                // error[E0723]: mutable references in const fn are unstable (see issue #57563)
                _marker: PhantomData,
            }
        }
    }
}
pub use self::privacy::RSliceMut;

impl<'a, T> RSliceMut<'a, T> {
    // pub const fn empty() -> Self {
    //     Self::EMPTY
    // }

    /// Converts a mutable reference to `T` to a single element `RSliceMut<'a, T>`.
    ///
    /// Note: this function does not copy anything.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RSliceMut;
    ///
    /// assert_eq!(
    ///     RSliceMut::from_mut(&mut 0),
    ///     RSliceMut::from_mut_slice(&mut [0])
    /// );
    /// assert_eq!(
    ///     RSliceMut::from_mut(&mut 1),
    ///     RSliceMut::from_mut_slice(&mut [1])
    /// );
    /// assert_eq!(
    ///     RSliceMut::from_mut(&mut 2),
    ///     RSliceMut::from_mut_slice(&mut [2])
    /// );
    ///
    /// ```
    pub fn from_mut(ref_: &'a mut T) -> Self {
        unsafe { Self::from_raw_parts_mut(ref_, 1) }
    }

    /// Converts a `&'a mut [T]` to a `RSliceMut<'a, T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RSliceMut;
    ///
    /// let empty: &mut [u8] = &mut [];
    ///
    /// assert_eq!(
    ///     RSliceMut::<u8>::from_mut_slice(&mut []).as_mut_slice(),
    ///     empty
    /// );
    /// assert_eq!(
    ///     RSliceMut::from_mut_slice(&mut [0]).as_mut_slice(),
    ///     &mut [0][..]
    /// );
    /// assert_eq!(
    ///     RSliceMut::from_mut_slice(&mut [0, 1]).as_mut_slice(),
    ///     &mut [0, 1][..]
    /// );
    ///
    /// ```
    #[inline]
    pub fn from_mut_slice(slic: &'a mut [T]) -> Self {
        slic.into()
    }

    /// Creates an `RSlice<'b, T>` with access to the `range` range of elements.
    ///
    /// This is an inherent method instead of an implementation of the
    /// `std::ops::Index` trait because it does not return a reference.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RSlice, RSliceMut};
    ///
    /// let slic = &mut [0, 1, 2, 3];
    /// let slic = RSliceMut::from_mut_slice(slic);
    ///
    /// assert_eq!(slic.slice(..), RSlice::from_slice(&[0, 1, 2, 3]));
    /// assert_eq!(slic.slice(..2), RSlice::from_slice(&[0, 1]));
    /// assert_eq!(slic.slice(2..), RSlice::from_slice(&[2, 3]));
    /// assert_eq!(slic.slice(1..3), RSlice::from_slice(&[1, 2]));
    ///
    /// ```
    #[allow(clippy::needless_lifetimes)]
    pub fn slice<'b, I>(&'b self, i: I) -> RSlice<'b, T>
    where
        [T]: Index<I, Output = [T]>,
    {
        self.as_slice().index(i).into()
    }

    /// Creates an `RSliceMut<'a, T>` with access to the `range` range of elements.
    ///
    /// This is an inherent method instead of an implementation of the
    /// `std::ops::IndexMut` trait because it does not return a reference.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RSliceMut;
    ///
    /// let slic = &mut [0, 1, 2, 3];
    /// let mut slic = RSliceMut::from_mut_slice(slic);
    ///
    /// assert_eq!(
    ///     slic.slice_mut(..),
    ///     RSliceMut::from_mut_slice(&mut [0, 1, 2, 3])
    /// );
    /// assert_eq!(slic.slice_mut(..2), RSliceMut::from_mut_slice(&mut [0, 1]));
    /// assert_eq!(slic.slice_mut(2..), RSliceMut::from_mut_slice(&mut [2, 3]));
    /// assert_eq!(slic.slice_mut(1..3), RSliceMut::from_mut_slice(&mut [1, 2]));
    ///
    /// ```
    #[allow(clippy::needless_lifetimes)]
    pub fn slice_mut<'b, I>(&'b mut self, i: I) -> RSliceMut<'b, T>
    where
        [T]: IndexMut<I, Output = [T]>,
    {
        self.as_mut_slice().index_mut(i).into()
    }

    /// Creates a new `RVec<T>` and clones all the elements of this slice into it.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RSliceMut, RVec};
    ///
    /// let slic = &mut [0, 1, 2, 3];
    /// let slic = RSliceMut::from_mut_slice(slic);
    ///
    /// assert_eq!(slic.slice(..).to_rvec(), RVec::from_slice(&[0, 1, 2, 3]));
    /// assert_eq!(slic.slice(..2).to_rvec(), RVec::from_slice(&[0, 1]));
    /// assert_eq!(slic.slice(2..).to_rvec(), RVec::from_slice(&[2, 3]));
    /// assert_eq!(slic.slice(1..3).to_rvec(), RVec::from_slice(&[1, 2]));
    ///
    /// ```
    pub fn to_rvec(&self) -> RVec<T>
    where
        T: Clone,
    {
        self.to_vec().into()
    }

    conditionally_const! {
        feature = "rust_1_64"
        ;
        unsafe fn as_slice_unbounded_lifetime(&self) -> &'a [T] {
            unsafe { ::std::slice::from_raw_parts(self.data(), self.len()) }
        }
    }

    unsafe fn as_mut_slice_unbounded_lifetime(&mut self) -> &'a mut [T] {
        unsafe { ::std::slice::from_raw_parts_mut(self.data(), self.len()) }
    }

    conditionally_const! {
        feature = "rust_1_64"
        /// Creates an `&'_ [T]` with access to all the elements of this slice.
        ///
        ;
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::std_types::RSliceMut;
        ///
        /// assert_eq!(
        ///     RSliceMut::from_mut_slice(&mut [0, 1, 2, 3]).as_slice(),
        ///     &[0, 1, 2, 3]
        /// );
        ///
        /// ```
        pub fn as_slice(&self) -> &[T] {
            unsafe { self.as_slice_unbounded_lifetime() }
        }
    }

    conditionally_const! {
        feature = "rust_1_64"
        /// Creates an `&'a [T]` with access to all the elements of this slice.
        ///
        /// This is different to `as_slice` in that the returned lifetime of
        /// this function  is larger.
        ///
        ;
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::std_types::RSliceMut;
        ///
        /// assert_eq!(
        ///     RSliceMut::from_mut_slice(&mut [0, 1, 2, 3]).into_slice(),
        ///     &[0, 1, 2, 3]
        /// );
        ///
        /// ```
        pub fn into_slice(self) -> &'a [T] {
            unsafe { self.as_slice_unbounded_lifetime() }
        }
    }

    /// Creates an `RSlice<'_, T>` with access to all the elements of this slice.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RSlice, RSliceMut};
    ///
    /// assert_eq!(
    ///     RSliceMut::from_mut_slice(&mut [0, 1, 2, 3]).as_rslice(),
    ///     RSlice::from_slice(&[0, 1, 2, 3]),
    /// );
    ///
    /// ```
    pub fn as_rslice(&self) -> RSlice<'_, T> {
        self.as_slice().into()
    }

    /// Creates an `RSlice<'a, T>` with access to all the elements of this slice.
    ///
    /// This is different to `as_rslice` in that the returned lifetime of
    /// this function  is larger.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RSlice, RSliceMut};
    ///
    /// assert_eq!(
    ///     RSliceMut::from_mut_slice(&mut [0, 1, 2, 3]).into_rslice(),
    ///     RSlice::from_slice(&[0, 1, 2, 3]),
    /// );
    ///
    /// ```
    pub fn into_rslice(self) -> RSlice<'a, T> {
        self.into_slice().into()
    }

    /// Creates a `&'_ mut [T]` with access to all the elements of this slice.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RSliceMut;
    ///
    /// assert_eq!(
    ///     RSliceMut::from_mut_slice(&mut [0, 1, 2, 3]).as_mut_slice(),
    ///     &mut [0, 1, 2, 3]
    /// );
    ///
    /// ```
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { self.as_mut_slice_unbounded_lifetime() }
    }

    /// Creates a `&'a mut [T]` with access to all the elements of this slice.
    ///
    /// This is different to `as_mut_slice` in that the returned lifetime of
    /// this function is larger.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RSliceMut;
    ///
    /// assert_eq!(
    ///     RSliceMut::from_mut_slice(&mut [0, 1, 2, 3]).into_mut_slice(),
    ///     &mut [0, 1, 2, 3]
    /// );
    ///
    /// ```
    pub fn into_mut_slice(mut self) -> &'a mut [T] {
        unsafe { self.as_mut_slice_unbounded_lifetime() }
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
        self.into_mut_slice().iter_mut()
    }
}

slice_like_impl_cmp_traits! {
    impl[] RSliceMut<'_, T>,
    where[];
    Vec<U>,
    [U],
    &[U],
    RSlice<'_, U>,
}

slice_like_impl_cmp_traits! {
    impl[const N: usize] RSliceMut<'_, T>,
    where[];
    [U; N],
}

slice_like_impl_cmp_traits! {
    impl[] RSliceMut<'_, T>,
    where[T: Clone, U: Clone];
    std::borrow::Cow<'_, [U]>,
    crate::std_types::RCowSlice<'_, U>,
}

impl<'a, T> Deref for RSliceMut<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'a, T> DerefMut for RSliceMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

////////////////////////////

impl_into_rust_repr! {
    impl['a, T] Into<&'a mut [T]> for RSliceMut<'a, T> {
        fn(this){
            this.into_mut_slice()
        }
    }
}

impl<'a, T> From<RSliceMut<'a, T>> for &'a [T] {
    fn from(this: RSliceMut<'a, T>) -> &'a [T] {
        this.into_slice()
    }
}

////////////////////

impl<'a, T: 'a> Borrow<[T]> for RSliceMut<'a, T> {
    fn borrow(&self) -> &[T] {
        self
    }
}

impl<'a, T: 'a> BorrowMut<[T]> for RSliceMut<'a, T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<'a, T: 'a> AsRef<[T]> for RSliceMut<'a, T> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<'a, T: 'a> AsMut<[T]> for RSliceMut<'a, T> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<'a, T, I: SliceIndex<[T]>> Index<I> for RSliceMut<'a, T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<'a, T, I: SliceIndex<[T]>> IndexMut<I> for RSliceMut<'a, T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
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
        let mut this = mem::take(self).into_mut_slice();
        let ret = this.write(data);
        *self = this.into();
        ret
    }

    #[inline]
    fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        let mut this = mem::take(self).into_mut_slice();
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
    mod = slice_impls
    new_type = RSliceMut['a][T],
    original_type = SliceMut,
}

////////////////////////////////////////////////////////////////////////////////

//#[cfg(test)]
#[cfg(all(test, not(feature = "only_new_tests")))]
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

    #[cfg(feature = "rust_1_64")]
    #[test]
    fn const_as_slice_test() {
        const RSM: RSliceMut<'_, u8> =
            unsafe { RSliceMut::from_raw_parts_mut(std::ptr::NonNull::dangling().as_ptr(), 0) };

        const SLICE_A: &[u8] = RSM.as_slice();
        const SLICE_B: &[u8] = RSM.into_slice();

        assert_eq!(SLICE_A, [0u8; 0]);
        assert_eq!(SLICE_B, [0u8; 0]);
    }

    #[test]
    fn test_index() {
        let mut v = vec![1, 2, 3, 4, 5];
        let s = RSliceMut::from_mut_slice(&mut v[..]);

        assert_eq!(s.index(0), &1);
        assert_eq!(s.index(4), &5);
        assert_eq!(s.index(..2), &mut [1, 2]);
        assert_eq!(s.index(1..2), &mut [2]);
        assert_eq!(s.index(3..), &mut [4, 5]);
    }

    #[test]
    fn test_index_mut() {
        let mut v = vec![1, 2, 3, 4, 5];
        let mut s = RSliceMut::from_mut_slice(&mut v[..]);

        assert_eq!(s.index_mut(0), &mut 1);
        assert_eq!(s.index_mut(4), &mut 5);
        assert_eq!(s.index_mut(..2), &mut [1, 2]);
        assert_eq!(s.index_mut(1..2), &mut [2]);
        assert_eq!(s.index_mut(3..), &mut [4, 5]);
    }
}
