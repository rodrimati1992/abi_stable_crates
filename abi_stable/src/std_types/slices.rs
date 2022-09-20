//! Contains the ffi-safe equivalent of `&'a [T]`.

use std::{
    borrow::Borrow,
    io::{self, BufRead, Read},
    marker::PhantomData,
    ops::{Deref, Index},
    slice::SliceIndex,
};

#[allow(unused_imports)]
use core_extensions::SelfOps;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::std_types::RVec;

mod private {
    use super::*;

    /// Ffi-safe equivalent of `&'a [T]`
    ///
    /// As of the writing this documentation the abi stability of `&[T]` is
    /// not yet guaranteed.
    ///
    /// # Lifetime problems
    ///
    /// Because `RSlice` dereferences into a slice, you can call slice methods on it.
    ///
    /// If you call a slice method that returns a borrow into the slice,
    /// it will have the lifetime of the `let slice: RSlice<'a, [T]>` variable instead of the `'a`
    /// lifetime that it's parameterized over.
    ///
    /// To get a slice with the same lifetime as an `RSlice`,
    /// one must use the `RSlice::as_slice` method.
    ///
    ///
    /// Example of what would not work:
    ///
    /// ```compile_fail
    /// use abi_stable::std_types::RSlice;
    ///
    /// fn into_slice<'a, T>(slic: RSlice<'a, T>) -> &'a [T] {
    ///     &*slic
    /// }
    /// ```
    ///
    /// Example of what would work:
    ///
    /// ```
    /// use abi_stable::std_types::RSlice;
    ///
    /// fn into_slice<'a, T>(slic: RSlice<'a, T>) -> &'a [T] {
    ///     slic.as_slice()
    /// }
    /// ```
    ///
    ///
    ///
    /// # Example
    ///
    /// Defining an extern fn that returns a reference to
    /// the first element that compares equal to a parameter.
    ///
    /// ```
    /// use abi_stable::{sabi_extern_fn, std_types::RSlice};
    ///
    /// #[sabi_extern_fn]
    /// pub fn find_first_mut<'a, T>(slice_: RSlice<'a, T>, element: &T) -> Option<&'a T>
    /// where
    ///     T: std::cmp::PartialEq,
    /// {
    ///     slice_
    ///         .iter()
    ///         .position(|x| x == element)
    ///         .map(|i| &slice_.as_slice()[i])
    /// }
    ///
    /// ```
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(bound(T: 'a))]
    //#[sabi(debug_print)]
    pub struct RSlice<'a, T> {
        data: *const T,
        length: usize,
        _marker: PhantomData<&'a T>,
    }

    impl_from_rust_repr! {
        impl['a, T] From<&'a [T]> for RSlice<'a, T> {
            fn(this){
                Self::from_slice(this)
            }
        }
    }

    impl<'a, T: 'a> RSlice<'a, T> {
        const _EMPTY_SLICE: &'a [T] = &[];

        /// An empty slice.
        pub const EMPTY: Self = RSlice {
            data: {
                let v: &[T] = Self::_EMPTY_SLICE;
                v.as_ptr()
            },
            length: 0,
            _marker: PhantomData,
        };

        /// Constructs an `RSlice<'a, T>` from a pointer to the first element,
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
        /// - The data `ptr_` points to must be valid for the `'a` lifetime.
        ///
        /// # Examples
        ///
        /// This function unsafely converts a `&[T]` to an `RSlice<T>`,
        /// equivalent to doing `RSlice::from_slice`.
        ///
        /// ```
        /// use abi_stable::std_types::RSlice;
        ///
        /// fn convert<T>(slice_: &[T]) -> RSlice<'_, T> {
        ///     unsafe { RSlice::from_raw_parts(slice_.as_ptr(), slice_.len()) }
        /// }
        ///
        /// ```
        pub const unsafe fn from_raw_parts(ptr_: *const T, len: usize) -> Self {
            Self {
                data: ptr_,
                length: len,
                _marker: PhantomData,
            }
        }

        #[doc(hidden)]
        pub const unsafe fn from_raw_parts_with_lifetime(slice: &'a [T], len: usize) -> Self {
            Self {
                data: slice.as_ptr(),
                length: len,
                _marker: PhantomData,
            }
        }
    }

    impl<'a, T> RSlice<'a, T> {
        conditionally_const! {
            feature = "rust_1_64"
            /// Creates an `&'a [T]` with access to all the elements of this slice.
            ///
            ;
            ///
            /// # Example
            ///
            /// ```
            /// use abi_stable::std_types::RSlice;
            ///
            /// assert_eq!(RSlice::from_slice(&[0, 1, 2, 3]).as_slice(), &[0, 1, 2, 3]);
            ///
            /// ```
            pub fn as_slice(&self) -> &'a [T] {
                unsafe { ::std::slice::from_raw_parts(self.data, self.length) }
            }
        }

        /// Gets a raw pointer to the start of the slice.
        pub const fn as_ptr(&self) -> *const T {
            self.data
        }

        /// The length (in elements) of this slice.
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::std_types::RSlice;
        ///
        /// assert_eq!(RSlice::<u8>::from_slice(&[]).len(), 0);
        /// assert_eq!(RSlice::from_slice(&[0]).len(), 1);
        /// assert_eq!(RSlice::from_slice(&[0, 1]).len(), 2);
        ///
        /// ```
        #[inline]
        pub const fn len(&self) -> usize {
            self.length
        }

        /// Whether this slice is empty.
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::std_types::RSlice;
        ///
        /// assert_eq!(RSlice::<u8>::from_slice(&[]).is_empty(), true);
        /// assert_eq!(RSlice::from_slice(&[0]).is_empty(), false);
        /// assert_eq!(RSlice::from_slice(&[0, 1]).is_empty(), false);
        ///
        /// ```
        #[inline]
        pub const fn is_empty(&self) -> bool {
            self.length == 0
        }
    }
}

pub use self::private::RSlice;

impl<'a, T> RSlice<'a, T> {
    /// Creates an empty slice
    pub const fn empty() -> Self {
        Self::EMPTY
    }

    /// Converts a reference to `T` to a single element `RSlice<'a, T>`.
    ///
    /// Note: this function does not copy anything.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RSlice;
    ///
    /// assert_eq!(RSlice::from_ref(&0), RSlice::from_slice(&[0]));
    /// assert_eq!(RSlice::from_ref(&1), RSlice::from_slice(&[1]));
    /// assert_eq!(RSlice::from_ref(&2), RSlice::from_slice(&[2]));
    ///
    /// ```
    pub const fn from_ref(ref_: &'a T) -> Self {
        unsafe { Self::from_raw_parts(ref_, 1) }
    }

    /// Converts a `&[T]` to an `RSlice<'_, T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RSlice;
    ///
    /// let empty: &[u8] = &[];
    ///
    /// assert_eq!(RSlice::<u8>::from_slice(&[]).as_slice(), empty);
    /// assert_eq!(RSlice::from_slice(&[0]).as_slice(), &[0][..]);
    /// assert_eq!(RSlice::from_slice(&[0, 1]).as_slice(), &[0, 1][..]);
    ///
    /// ```
    #[inline]
    pub const fn from_slice(slic: &'a [T]) -> Self {
        unsafe { RSlice::from_raw_parts(slic.as_ptr(), slic.len()) }
    }

    /// Creates an `RSlice<'a, T>` with access to the `range` range of elements.
    ///
    /// This is an inherent method instead of an implementation of the
    /// `std::ops::Index` trait because it does not return a reference.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RSlice;
    ///
    /// let slic = RSlice::from_slice(&[0, 1, 2, 3]);
    ///
    /// assert_eq!(slic.slice(..), RSlice::from_slice(&[0, 1, 2, 3]));
    /// assert_eq!(slic.slice(..2), RSlice::from_slice(&[0, 1]));
    /// assert_eq!(slic.slice(2..), RSlice::from_slice(&[2, 3]));
    /// assert_eq!(slic.slice(1..3), RSlice::from_slice(&[1, 2]));
    ///
    /// ```
    pub fn slice<I>(&self, i: I) -> RSlice<'a, T>
    where
        [T]: Index<I, Output = [T]>,
    {
        self.as_slice().index(i).into()
    }

    /// Creates a new `RVec<T>` and clones all the elements of this slice into it.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RSlice, RVec};
    ///
    /// let slic = RSlice::from_slice(&[0, 1, 2, 3]);
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

    /// Transmutes n `RSlice<'a, T>` to a `RSlice<'a, U>`
    ///
    /// # Safety
    ///
    /// This has the same safety requirements as calling [`std::mem::transmute`] to
    /// transmute a `&'a [T]` to a `&'a [U]`.
    ///
    /// [`std::mem::transmute`]: https://doc.rust-lang.org/std/mem/fn.transmute.html
    pub const unsafe fn transmute<U>(self) -> RSlice<'a, U>
    where
        U: 'a,
    {
        let len = self.len();
        unsafe { RSlice::from_raw_parts(self.as_ptr() as *const T as *const U, len) }
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
        self.as_slice().iter()
    }
}

impl<'a, T, I: SliceIndex<[T]>> Index<I> for RSlice<'a, T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.get(index).expect("Index out of bounds")
    }
}

slice_like_impl_cmp_traits! {
    impl[] RSlice<'_, T>,
    where[];
    Vec<U>,
    [U],
    &[U],
}

slice_like_impl_cmp_traits! {
    impl[const N: usize] RSlice<'_, T>,
    where[];
    [U; N],
}

slice_like_impl_cmp_traits! {
    impl[] RSlice<'_, T>,
    where[T: Clone, U: Clone];
    std::borrow::Cow<'_, [U]>,
    crate::std_types::RCowSlice<'_, U>,
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

impl<'a, T: 'a> Borrow<[T]> for RSlice<'a, T> {
    fn borrow(&self) -> &[T] {
        self
    }
}

impl<'a, T: 'a> AsRef<[T]> for RSlice<'a, T> {
    fn as_ref(&self) -> &[T] {
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
    mod = slice_impls
    new_type = RSlice['a][T],
    original_type = Slice,
}

////////////////////////////////////////////////////////////////////////////////

//#[cfg(test)]
#[cfg(all(test, not(feature = "only_new_tests")))]
mod test {
    use super::*;

    #[test]
    fn from_to_slice() {
        let a = "what the hell".as_bytes();
        let b = RSlice::from(a);

        assert_eq!(a, &*b);
        assert_eq!(a.len(), b.len());
    }

    #[cfg(feature = "rust_1_64")]
    #[test]
    fn const_as_slice_test() {
        const RS: RSlice<'_, u8> = RSlice::from_slice(&[3, 5, 8]);
        const SLICE: &[u8] = RS.as_slice();

        assert_eq!(SLICE, [3, 5, 8]);
    }

    #[test]
    fn test_index() {
        let s = rslice![1, 2, 3, 4, 5];

        assert_eq!(s.index(0), &1);
        assert_eq!(s.index(4), &5);
        assert_eq!(s.index(..2), rslice![1, 2]);
        assert_eq!(s.index(1..2), rslice![2]);
        assert_eq!(s.index(3..), rslice![4, 5]);
    }
}
