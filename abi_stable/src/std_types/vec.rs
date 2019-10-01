/*!
Contains an ffi-safe equivalent of `Vec<T>`.
*/

use std::{
    borrow::{Cow,Borrow,BorrowMut},
    cmp::Ordering,
    io,
    iter::FromIterator,
    marker::PhantomData,
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut, Index, IndexMut},
    ptr,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use core_extensions::prelude::*;

use crate::{
    sabi_types::{Constructor,StaticRef},
    std_types::{RSlice, RSliceMut,utypeid::{UTypeId,new_utypeid}},
    prefix_type::{PrefixTypeTrait,WithMetadata},
};

#[cfg(test)]
// #[cfg(all(test,not(feature="only_new_tests")))]
mod tests;

mod iters;

use self::iters::{RawValIter,DrainFilter};

pub use self::iters::{Drain, IntoIter};

mod private {
    use super::*;
    
/**
Ffi-safe equivalent of `std::vec::Vec`.

# Example

Here is a function that partitions numbers by whether they are even or odd.

```

use abi_stable::{
    std_types::{RSlice,RVec},
    StableAbi,
    sabi_extern_fn,
};

#[repr(C)]
#[derive(StableAbi)]
pub struct Partitioned{
    pub even:RVec<u32>,
    pub odd :RVec<u32>,
}

#[sabi_extern_fn]
pub fn partition_evenness(numbers:RSlice<'_,u32>)->Partitioned{
    let (even,odd)=numbers.iter().cloned().partition(|n| *n % 2 == 0);

    Partitioned{even,odd}
}

```

*/
    #[repr(C)]
    #[derive(StableAbi)]
    // #[sabi(debug_print)]
    pub struct RVec<T> {
        buffer: *mut T,
        pub(super) length: usize,
        capacity: usize,
        vtable: StaticRef<VecVTable<T>>,
        _marker: PhantomData<T>,
    }

    impl<T> RVec<T> {
        #[allow(dead_code)]
        // Used to test functions that change behavior when the vtable changes
        pub(super) fn set_vtable_for_testing(mut self) -> Self {
            self.vtable = WithMetadata::as_prefix(
                VTableGetter::<T>::LIB_VTABLE_FOR_TESTING
            );
            self
        }

        #[inline(always)]
        pub(super) fn vtable<'a>(&self) -> &'a VecVTable<T>{
            self.vtable.get()
        }

        #[inline(always)]
        pub(super) fn buffer(&self) -> *const T {
            self.buffer
        }
        
        pub(super) fn buffer_mut(&mut self) -> *mut T {
            self.buffer
        }

        /// This returns the ammount of elements this RVec can store without reallocating.
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::std_types::RVec;
        ///
        /// let mut list=RVec::new();
        ///
        /// assert_eq!(list.capacity(),0);
        ///
        /// list.push(0);
        /// assert_ne!(list.capacity(),0);
        ///
        /// ```
        pub const fn capacity(&self) -> usize {
            self.capacity
        }

        /// Constructs a vec to do operations on the underlying buffer.
        ///
        /// # Safety
        ///
        /// This must not be called outside of functions that get stored in the vtable.
        pub(super) unsafe fn with_vec<U, F>(&mut self, f: F) -> U
        where
            F: FnOnce(&mut Vec<T>) -> U,
        {
            unsafe {
                let mut old = mem::replace(self, RVec::new()).piped(ManuallyDrop::new);
                let mut list = Vec::<T>::from_raw_parts(
                    old.buffer_mut(), 
                    old.len(), 
                    old.capacity()
                );
                let ret = f(&mut list);
                ptr::write(self, list.into());
                ret
            }
        }
    }
    impl_from_rust_repr! {
        impl[T] From<Vec<T>> for RVec<T>{
            fn(this){
                let mut this=ManuallyDrop::new(this);
                RVec {
                    vtable: WithMetadata::as_prefix( VTableGetter::<T>::LIB_VTABLE ),
                    buffer: this.as_mut_ptr(),
                    length: this.len(),
                    capacity: this.capacity(),
                    _marker: Default::default(),
                }
            }
        }
    }

}

pub use self::private::RVec;

impl<T> RVec<T> {
    /// Creates a new,empty `RVec<T>`.
    ///
    /// This function does not allocate.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let list=RVec::<u32>::new();
    ///
    /// ```
    pub fn new() -> Self {
        Vec::new().into()
    }

    /// Creates a new,empty `RVec<T>`,with a capacity of `cap`.
    ///
    /// This function does not allocate if `cap`==0.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut list=RVec::<u32>::with_capacity(7);
    ///
    /// assert_eq!(list.len(),0);
    /// assert_eq!(list.capacity(),7);
    ///
    /// list.extend( std::iter::repeat(11).take(7) );
    /// assert_eq!(list.len(),7);
    /// assert_eq!(list.capacity(),7);
    ///
    /// list.push(17);
    /// assert_ne!(list.capacity(),7);
    /// ```
    pub fn with_capacity(cap: usize) -> Self {
        Vec::with_capacity(cap).into()
    }

    unsafe fn entire_buffer(&mut self) -> &mut [T] {
        ::std::slice::from_raw_parts_mut(self.buffer_mut(), self.capacity())
    }

    /// Creates an `RSlice<'a,T>` with access to the `range` range of
    /// elements of the `RVec<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RSlice,RVec};
    ///
    /// let list=RVec::from(vec![0,1,2,3,4,5,6,7,8]);
    ///
    /// assert_eq!( list.slice(..), RSlice::from_slice(&[0,1,2,3,4,5,6,7,8]) );
    /// assert_eq!( list.slice(..4), RSlice::from_slice(&[0,1,2,3]) );
    /// assert_eq!( list.slice(4..), RSlice::from_slice(&[4,5,6,7,8]) );
    /// assert_eq!( list.slice(4..7), RSlice::from_slice(&[4,5,6]) );
    ///
    /// ```
    #[inline]
    pub fn slice<'a, I>(&'a self, range: I) -> RSlice<'a, T>
    where
        [T]: Index<I, Output = [T]>,
    {
        (&self[range]).into()
    }

    /// Creates an `RSliceMut<'a,T>` with access to the `range` range of 
    /// elements of the `RVec<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RSliceMut,RVec};
    ///
    /// let mut list=RVec::from(vec![0,1,2,3,4,5,6,7,8]);
    ///
    /// assert_eq!( list.slice_mut(..), RSliceMut::from_mut_slice(&mut [0,1,2,3,4,5,6,7,8]) );
    /// assert_eq!( list.slice_mut(..4), RSliceMut::from_mut_slice(&mut [0,1,2,3]) );
    /// assert_eq!( list.slice_mut(4..), RSliceMut::from_mut_slice(&mut [4,5,6,7,8]) );
    /// assert_eq!( list.slice_mut(4..7), RSliceMut::from_mut_slice(&mut [4,5,6]) );
    ///
    /// ```
    #[inline]
    pub fn slice_mut<'a, I>(&'a mut self, i: I) -> RSliceMut<'a, T>
    where
        [T]: IndexMut<I, Output = [T]>,
    {
        (&mut self[i]).into()
    }

    /// Creates a `&[T]` with access to all the elements of the `RVec<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let list=RVec::from(vec![0,1,2,3]);
    /// assert_eq!(list.as_slice(), &[0,1,2,3]);
    ///
    /// ```
    pub fn as_slice(&self) -> &[T] {
        unsafe { ::std::slice::from_raw_parts(self.buffer(), self.len()) }
    }

    /// Creates a `&mut [T]` with access to all the elements of the `RVec<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut list=RVec::from(vec![0,1,2,3]);
    /// assert_eq!(list.as_mut_slice(), &mut [0,1,2,3]);
    ///
    /// ```
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { ::std::slice::from_raw_parts_mut(self.buffer_mut(), self.len()) }
    }

    /// Creates an `RSlice<'_,T>` with access to all the elements of the `RVec<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RSlice,RVec};
    ///
    /// let list=RVec::from(vec![0,1,2,3]);
    /// assert_eq!(list.as_rslice(), RSlice::from_slice(&[0,1,2,3]));
    ///
    /// ```
    pub fn as_rslice(&self) -> RSlice<'_, T> {
        self.as_slice().into()
    }

    /// Creates an `RSliceMut<'_,T>` with access to all the elements of the `RVec<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RSliceMut,RVec};
    ///
    /// let mut list=RVec::from(vec![0,1,2,3]);
    /// assert_eq!(list.as_mut_rslice(), RSliceMut::from_mut_slice(&mut [0,1,2,3]));
    ///
    /// ```
    pub fn as_mut_rslice(&mut self) -> RSliceMut<'_, T> {
        self.as_mut_slice().into()
    }

    /// Returns the ammount of elements of the `RVec<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut list=RVec::<u64>::new();
    ///
    /// assert_eq!(list.len(),0);
    ///
    /// list.push(0xDEAFBEEF);
    /// assert_eq!(list.len(),1);
    ///
    /// list.push(0xCAFE);
    /// assert_eq!(list.len(),2);
    ///
    /// ```
    pub const fn len(&self) -> usize {
        self.length
    }

    /// Sets the length field of `RVec<T>` to `new_len`.
    ///
    /// # Safety
    ///
    /// `new_len` must be less than or equal to `self.capacity()`.
    ///
    /// The elements at old_len..new_len must be initialized.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut list=RVec::<u64>::new();
    ///
    /// list.reserve_exact(10);
    ///
    /// unsafe{
    ///     let start=list.as_mut_ptr();
    ///     for i in 0..10 {
    ///         start.add(i as usize).write(i);
    ///     }
    ///     list.set_len(10);
    /// }
    ///
    /// assert_eq!( list, (0..10).collect::<RVec<u64>>() );
    ///
    /// ```
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.length = new_len;
    }

    /// Shrinks the capacity of the RString to match its length.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut list=RVec::<u32>::with_capacity(7);
    ///
    /// list.extend( std::iter::repeat(11).take(4) );
    /// assert_eq!(list.len(),4);
    /// assert_eq!(list.capacity(),7);
    ///
    /// list.shrink_to_fit();
    /// assert_eq!(list.len(),4);
    /// assert_eq!(list.capacity(),4);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        let vtable = self.vtable();
        vtable.shrink_to_fit()(self);
    }

    /// Whether the length of the `RVec<T>` is 0.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut list=RVec::<u64>::new();
    ///
    /// assert_eq!(list.is_empty(),true);
    ///
    /// list.push(0x1337);
    /// assert_eq!(list.is_empty(),false);
    ///
    /// list.push(0xC001);
    /// assert_eq!(list.is_empty(),false);
    ///
    /// ```
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Returns a `Vec<T>`,consuming `self`.
    ///
    /// # Allocation
    ///
    /// If this is invoked outside of the dynamic library/binary that created it,
    /// it will allocate a new `Vec<T>` and move the data into it.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut list=RVec::<u64>::new();
    /// 
    /// list.push(0);
    /// list.push(1);
    /// list.push(2);
    ///
    /// assert_eq!(list.into_vec(), vec![0,1,2]);
    ///
    /// ```
    pub fn into_vec(self) -> Vec<T> {
        let mut this = ManuallyDrop::new(self);

        unsafe {
            let this_vtable =this.vtable();
            let other_vtable=WithMetadata::as_prefix(VTableGetter::LIB_VTABLE).get();
            if ::std::ptr::eq(this_vtable,other_vtable)||
                this_vtable.type_id()==other_vtable.type_id()
            {
                Vec::from_raw_parts(this.buffer_mut(), this.len(), this.capacity())
            } else {
                let len = this.length;
                let mut ret = Vec::with_capacity(len);
                ptr::copy_nonoverlapping(this.as_ptr(), ret.as_mut_ptr(), len);
                ret.set_len(len);
                this.length = 0;
                ManuallyDrop::drop(&mut this);
                ret
            }
        }
    }

    /// Creates a `Vec<T>`,copying all the elements of this `RVec<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut list=RVec::<u64>::new();
    /// 
    /// list.extend( (4..=7).rev() );
    ///
    /// assert_eq!(list.to_vec(), vec![7,6,5,4] );
    ///
    /// ```
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.as_slice().to_vec()
    }

    /// Clones a `&[T]` into a new RVec.
    ///
    /// This function was defined to aid type inference,
    /// because eg:`&[0,1]` is a `&[i32;2]` not a `&[i32]`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let slic=&[99,88,77,66];
    /// let list=RVec::<u64>::from_slice(slic);
    /// 
    /// assert_eq!(list.as_slice(),slic);
    /// ```
    #[inline]
    pub fn from_slice(slic:&[T])->RVec<T>
    where
        T:Clone
    {
        slic.into()
    }

    /// Inserts the `value` value at `index` position. 
    ///
    /// # Panics
    ///
    /// Panics if self.len() < index.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut list=RVec::from(vec![0,1,2,3]);
    ///
    /// list.insert(2,22);
    /// assert_eq!(list.as_slice(),&[0,1,22,2,3]);
    ///
    /// list.insert(5,55);
    /// assert_eq!(list.as_slice(),&[0,1,22,2,3,55]);
    ///
    /// ```
    pub fn insert(&mut self, index: usize, value: T) {
        assert!(
            index <= self.length,
            "index out of bounds,index={} len={} ",
            index,
            self.length
        );
        if self.capacity() == self.length {
            self.grow_capacity_to_1();
        }

        unsafe {
            let buffer=self.buffer_mut();
            if index < self.length {
                ptr::copy(
                    buffer.offset(index as isize),
                    buffer.offset(index as isize + 1),
                    self.length - index,
                );
            }
            ptr::write(buffer.offset(index as isize), value);
            self.length += 1;
        }
    }

    /// Attemps to remove the element at `index` position,
    /// returns None if self.len() <= index.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut list=RVec::from(vec![0,1,2,3]);
    ///
    /// assert_eq!(list.try_remove(4),None);
    /// assert_eq!(list.try_remove(3),Some(3));
    /// assert_eq!(list.try_remove(1),Some(1));
    ///
    /// assert_eq!(list.as_slice(), &[0,2]);
    /// ```
    pub fn try_remove(&mut self, index: usize) -> Option<T> {
        if self.length <= index {
            return None;
        }
        unsafe {
            let buffer=self.buffer_mut();
            self.length -= 1;
            let result = ptr::read(buffer.offset(index as isize));
            ptr::copy(
                buffer.offset(index as isize + 1),
                buffer.offset(index as isize),
                self.length - index,
            );
            Some(result)
        }
    }

    /// Removes the element at `index` position,
    ///
    /// # Panic
    ///
    /// Panics if self.len() <= index.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     std_types::{RStr,RVec},
    ///     traits::IntoReprC,
    /// };
    ///
    /// // This type annotation is purely for the reader.
    /// let mut list:RVec<RStr<'static>>=
    ///     vec!["foo".into_c(), "bar".into(), "baz".into()].into_c();
    ///
    /// assert_eq!( list.remove(2), "baz".into_c() );
    /// assert_eq!(list.as_slice(), &["foo".into_c(), "bar".into_c()] );
    ///
    /// assert_eq!( list.remove(0), "foo".into_c() );
    /// assert_eq!(list.as_slice(), &["bar".into_c()] );
    /// ```
    pub fn remove(&mut self, index: usize) -> T {
        match self.try_remove(index) {
            Some(x) => x,
            None => panic!("index out of bounds,index={} len={} ", index, self.length),
        }
    }

    /// Swaps the element at `index` position with the last element,and then removes it.
    ///
    /// # Panic
    ///
    /// Panics if self.len() <= index.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     std_types::{RStr,RVec},
    ///     traits::IntoReprC,
    /// };
    ///
    /// // This type annotation is purely for the reader.
    /// let mut list:RVec<RStr<'static>>=
    ///     vec!["foo".into_c(), "bar".into(), "baz".into(), "geo".into()].into_c();
    ///
    /// assert_eq!( list.swap_remove(1), "bar".into_c() );
    /// assert_eq!( list.as_slice(), &["foo".into_c(), "geo".into(), "baz".into()] );
    ///
    /// assert_eq!( list.swap_remove(0), "foo".into_c() );
    /// assert_eq!( list.as_slice(), &["baz".into(), "geo".into()] );
    ///
    /// ```
    pub fn swap_remove(&mut self, index: usize) -> T {
        unsafe {
            let hole: *mut T = &mut self[index];
            let last = ptr::read(self.buffer_mut().offset((self.length - 1) as isize));
            self.length -= 1;
            ptr::replace(hole, last)
        }
    }

    /// Appends `new_val` at the end of the `RVec<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut list=RVec::<u32>::new();
    ///
    /// list.push(11);
    /// assert_eq!(list.as_slice(), &[11]);
    ///
    /// list.push(22);
    /// assert_eq!(list.as_slice(), &[11,22]);
    ///
    /// list.push(55);
    /// assert_eq!(list.as_slice(), &[11,22,55]);
    ///
    /// ```
    pub fn push(&mut self, new_val: T) {
        if self.length == self.capacity() {
            self.grow_capacity_to_1();
        }
        unsafe {
            ptr::write(self.buffer_mut().offset(self.length as isize), new_val);
        }
        self.length += 1;
    }

    /// Attempts to remove the last element,
    /// returns None if the `RVec<T>` is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RSlice,RVec};
    ///
    /// let mut list=RVec::<u32>::from_slice(&[11,22,55]);
    ///
    /// assert_eq!(list.pop(), Some(55));
    /// assert_eq!(list.as_slice(), &[11,22]);
    ///
    /// assert_eq!(list.pop(), Some(22));
    /// assert_eq!(list.as_slice(), &[11]);
    ///
    /// assert_eq!(list.pop(), Some(11));
    /// assert_eq!(list.as_rslice(), RSlice::<u32>::EMPTY );
    ///
    /// assert_eq!(list.pop(), None);
    ///
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        if self.length == 0 {
            None
        } else {
            unsafe {
                self.length -= 1;
                Some(ptr::read(self.buffer_mut().offset(self.length as isize)))
            }
        }
    }

    /// Truncates the `RVec<T>` to `to` length.
    /// Does nothing if self.len() <= to.
    ///
    /// Note:this has no effect on the capacity of the `RVec<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RSlice,RVec};
    ///
    /// let mut list=RVec::<u32>::from_slice(&[11,22,55,66,77]);
    ///
    /// list.truncate(3);
    /// assert_eq!(list.as_slice(), &[11,22,55] );
    ///
    /// list.truncate(0);
    /// assert_eq!(list.as_rslice(), RSlice::<u32>::EMPTY  );
    ///
    /// list.truncate(5555); //This is a no-op.
    /// ```    
    pub fn truncate(&mut self, to: usize) {
        if to < self.len() {
            self.truncate_inner(to);
        }
    }

    /// Removes all the elements from collection.
    ///
    /// Note:this has no effect on the capacity of the `RVec<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RSlice,RVec};
    ///
    /// let mut list=RVec::<u32>::from_slice(&[11,22,55]);
    ///
    /// assert_eq!( list.as_slice(), &[11,22,55] );
    ///
    /// list.clear();
    /// assert_eq!( list.as_rslice(), RSlice::<u32>::EMPTY );
    /// assert_ne!( list.capacity(), 0 );
    ///
    /// ```
    pub fn clear(&mut self) {
        self.truncate_inner(0);
    }

    

    /// Retains only the elements that satisfy the `pred` predicate
    ///
    /// This means that a element will be removed if `pred(that_element)` 
    /// returns false.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// {
    ///     let mut list=(0..=10).collect::<Vec<u32>>();
    ///     list.retain(|x| *x%3 ==0 );
    ///     assert_eq!(list.as_slice(), &[0,3,6,9]);
    /// }
    /// {
    ///     let mut list=(0..=10).collect::<Vec<u32>>();
    ///     list.retain(|x| *x%5 ==0 );
    ///     assert_eq!(list.as_slice(), &[0,5,10]);
    /// }
    ///
    /// ```
    pub fn retain<F>(&mut self, mut pred: F)
    where F: FnMut(&T) -> bool
    {
        let old_len = self.len();
        unsafe { 
            self.set_len(0); 
        }
        DrainFilter {
            vec: self,
            idx: 0,
            del: 0,
            old_len,
            pred: |x| !pred(x),
        };
    }

    fn truncate_inner(&mut self, to: usize) {
        let old_length = self.length;
        self.length = to;
        unsafe {
            for elem in self.get_unchecked_mut(to..old_length) {
                ptr::drop_in_place(elem);
            }
        }
    }

    /// Reserves `àdditional` additional capacity for extra elements.
    /// This may reserve more than necessary for the additional capacity.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut list=RVec::<u32>::new();
    ///
    /// list.reserve(10);
    /// assert!( list.capacity()>=10 );
    ///
    /// let cap=list.capacity();
    /// list.extend(0..10);
    /// assert_eq!( list.capacity(),cap );
    ///
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.resize_capacity(self.len() + additional, Exactness::Above)
    }

    /// Reserves `àdditional` additional capacity for extra elements.
    /// 
    /// Prefer using `reserve` for most situations.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut list=RVec::<u32>::new();
    ///
    /// list.reserve_exact(17);
    /// assert_eq!( list.capacity(),17 );
    ///
    /// let cap=list.capacity();
    /// list.extend(0..17);
    /// assert_eq!( list.capacity(),cap );
    ///
    /// ```
    pub fn reserve_exact(&mut self, additional: usize) {
        self.resize_capacity(self.len() + additional, Exactness::Exact)
    }

    #[inline]
    fn grow_capacity_to_1(&mut self) {
        let vtable = self.vtable();
        vtable.grow_capacity_to()(self, self.capacity() + 1, Exactness::Above);
    }

    fn resize_capacity(&mut self, to: usize, exactness: Exactness) {
        let vtable = self.vtable();
        if self.capacity() < to {
            vtable.grow_capacity_to()(self, to, exactness);
        }
    }
}

impl<T> RVec<T>
where
    T: Clone,
{
    /// Resizes the `RVec<T>` to `new_len` length.
    /// if new_len is larger than the current length,
    /// the `RVec<T>` is extended with clones of `value`
    /// to reach the new length.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut list=RVec::<u32>::new();
    ///
    /// list.resize(5,88);
    /// assert_eq!( list.as_slice(), &[88,88,88,88,88] );
    /// 
    /// list.resize(3,0);
    /// assert_eq!( list.as_slice(), &[88,88,88] );
    /// 
    /// list.resize(6,123);
    /// assert_eq!( list.as_slice(), &[88,88,88,123,123,123] );
    /// 
    /// ```
    pub fn resize(&mut self, new_len: usize, value: T) {
        let old_len = self.len();
        match new_len.cmp(&old_len) {
            Ordering::Less => self.truncate_inner(new_len),
            Ordering::Equal => {}
            Ordering::Greater => unsafe {
                self.resize_capacity(new_len, Exactness::Above);
                // Using new_len instead of the capacity because resize_capacity may
                // grow the capacity more than requested.
                //
                // Also replaced usage of slice with raw pointers based on a 
                // comment mentioning how slices must only reference initialized memory.
                let start=self.buffer_mut();
                let mut current=start.add(old_len);
                let end=start.add(new_len);
                while current!=end {
                    ptr::write(current, value.clone());
                    current=current.add(1);
                }
                self.length = new_len;
            },
        }
    }

    /// Extends this `RVec<_>` with clones of the elements of the slice.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut list=RVec::<u64>::new();
    ///
    /// list.extend_from_slice(&[99,88]);
    /// list.extend_from_slice(&[77,66]);
    /// 
    /// assert_eq!( list.as_slice(), &[99,88,77,66] );
    /// ```
    pub fn extend_from_slice(&mut self, slic_: &[T]) {
        self.reserve(slic_.len());
        for elem in slic_ {
            self.push(elem.clone());
        }
    }
}

impl<T> RVec<T>
where
    T: Copy,
{
    /// Extends this `RVec<_>` with copies of the elements of the slice.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     std_types::{RStr,RVec},
    ///     traits::IntoReprC,
    /// };
    ///
    /// let mut list=RVec::<RStr<'_>>::new();
    ///
    /// list.extend_from_slice(&["foo".into_c(), "bar".into()]);
    /// list.extend_from_slice(&["baz".into_c(), "goo".into()]);
    /// 
    /// assert_eq!( 
    ///     list.as_slice(),
    ///     &["foo".into_c(), "bar".into(), "baz".into(), "goo".into()],
    /// );
    /// ```
    pub fn extend_from_copy_slice(&mut self, slic_: &[T]) {
        self.reserve(slic_.len());
        let old_len = self.len();
        unsafe {
            let entire:*mut T = self.buffer_mut().offset(old_len as isize);
            ptr::copy_nonoverlapping(slic_.as_ptr(), entire, slic_.len());
            self.length = old_len + slic_.len();
        }
    }
}

impl<T> Clone for RVec<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        self.to_vec().into()
    }
}

impl<T> Default for RVec<T> {
    fn default() -> Self {
        Vec::new().into()
    }
}

impl<T> Deref for RVec<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for RVec<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T> AsRef<[T]> for RVec<T> {
    fn as_ref(&self) -> &[T] {
        self
    }
}
impl<T> AsMut<[T]> for RVec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T> Borrow<[T]> for RVec<T> {
    fn borrow(&self) -> &[T] {
        self
    }
}

impl<T> BorrowMut<[T]> for RVec<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self
    }
}

shared_impls! {
    mod=buffer_impls
    new_type=RVec[][T],
    original_type=Vec,
}

impl_into_rust_repr! {
    impl[T] Into<Vec<T>> for RVec<T> {
        fn(this){
            this.into_vec()
        }
    }
}

impl<'a, T> From<&'a [T]> for RVec<T>
where
    T: Clone,
{
    fn from(this: &'a [T]) -> Self {
        this.to_vec().into()
    }
}

impl<'a, T> From<Cow<'a, [T]>> for RVec<T>
where
    T: Clone,
{
    fn from(this: Cow<'a, [T]>) -> Self {
        this.into_owned().into()
    }
}

unsafe impl<T> Send for RVec<T> where T: Send {}
unsafe impl<T> Sync for RVec<T> where T: Sync {}

impl<T> Drop for RVec<T> {
    fn drop(&mut self) {
        let vtable = self.vtable();
        vtable.destructor()(self)
    }
}

impl<'de, T> Deserialize<'de> for RVec<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Vec<T>>::deserialize(deserializer).map(Self::from)
    }
}

impl<T> Serialize for RVec<T>
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

/////////////////////////////////////////////////////////////////////////////////////
//             Iteration implementation based on the nomicon                       //
/////////////////////////////////////////////////////////////////////////////////////

impl<T> RVec<T> {

    /**
Creates a draining iterator that removes the specified range in 
the `RVec<T>` and yields the removed items.

# Panic

Panics if the index is out of bounds or if the start of the range is 
greater than the end of the range.

# Consumption

The elements in the range will be removed even if the iterator 
was dropped before yielding them.

# Example

```
use abi_stable::std_types::{RSlice,RVec};

{
    let mut list=RVec::from(vec![0,1,2,3,4,5]);
    assert_eq!( list.drain(2..4).collect::<Vec<_>>(), vec![2,3] );
    assert_eq!( list.as_slice(), &[0,1,4,5] );
}
{
    let mut list=RVec::from(vec![0,1,2,3,4,5]);
    assert_eq!( list.drain(2..).collect::<Vec<_>>(), vec![2,3,4,5] );
    assert_eq!( list.as_slice(), &[0,1] );
}
{
    let mut list=RVec::from(vec![0,1,2,3,4,5]);
    assert_eq!( list.drain(..2).collect::<Vec<_>>(), vec![0,1] );
    assert_eq!( list.as_slice(), &[2,3,4,5] );
}
{
    let mut list=RVec::from(vec![0,1,2,3,4,5]);
    assert_eq!( list.drain(..).collect::<Vec<_>>(), vec![0,1,2,3,4,5] );
    assert_eq!( list.as_rslice(), RSlice::<u32>::EMPTY );
}

```


    */
    pub fn drain<'a, I>(&'a mut self, index: I) -> Drain<'a, T>
    where
        [T]: IndexMut<I, Output = [T]>,
    {
        unsafe {
            let slic_ = &mut self[index];
            let removed_start = slic_.as_mut_ptr();
            let slice_len = slic_.len();
            let iter = RawValIter::new(slic_);
            let old_length = self.length;
            self.length = 0;

            Drain {
                removed_start,
                slice_len,
                iter: iter,
                vec: self,
                len: old_length,
            }
        }
    }
}

impl<T> IntoIterator for RVec<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        unsafe {
            let iter = RawValIter::new(&self);
            IntoIter {
                iter: iter,
                _buf: ManuallyDrop::new(self),
            }
        }
    }
}

impl<'a, T> IntoIterator for &'a RVec<T> {
    type Item = &'a T;

    type IntoIter = ::std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut RVec<T> {
    type Item = &'a mut T;

    type IntoIter = ::std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> FromIterator<T> for RVec<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        iter.piped(Vec::from_iter).piped(Self::from)
    }
}

impl<T> Extend<T> for RVec<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        // optimizable
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();
        self.reserve(lower);
        for elem in iter {
            self.push(elem);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl io::Write for RVec<u8> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.extend_from_copy_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.extend_from_copy_slice(buf);
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////



#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
enum Exactness {
    Exact,
    Above,
}

////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

/// Dummy type used to create a statically allocated constant that can only be
/// accessed of the lifetime of T.
struct VTableGetter<'a, T>(&'a T);

impl<'a, T: 'a> VTableGetter<'a, T> {
    const DEFAULT_VTABLE:VecVTableVal<T>=VecVTableVal{
        type_id:Constructor( new_utypeid::<RVec<()>> ),
        destructor: destructor_vec,
        grow_capacity_to: grow_capacity_to_vec,
        shrink_to_fit: shrink_to_fit_vec,
    };

    // The VTABLE for this type in this executable/library
    const LIB_VTABLE: StaticRef<WithMetadata<VecVTableVal<T>>> = unsafe{
        StaticRef::from_raw(&WithMetadata::new(
            PrefixTypeTrait::METADATA,
            Self::DEFAULT_VTABLE,
        ))
    };

    // Used to test functions that change behavior based on the vtable being used
    const LIB_VTABLE_FOR_TESTING: StaticRef<WithMetadata<VecVTableVal<T>>> = unsafe{
        StaticRef::from_raw(&WithMetadata::new(
            PrefixTypeTrait::METADATA,
            VecVTableVal {
                type_id:Constructor( new_utypeid::<RVec<i32>> ),
                ..Self::DEFAULT_VTABLE
            }
        ))
    };

}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_struct="VecVTable")))]
#[sabi(missing_field(panic))]
struct VecVTableVal<T> {
    type_id:Constructor<UTypeId>,
    destructor: extern "C" fn(&mut RVec<T>),
    grow_capacity_to: extern "C" fn(&mut RVec<T>, usize, Exactness),
    #[sabi(last_prefix_field)]
    shrink_to_fit: extern "C" fn(&mut RVec<T>),
}


extern "C" fn destructor_vec<T>(this: &mut RVec<T>) {
    extern_fn_panic_handling! {
        unsafe {
            drop(Vec::from_raw_parts(
                this.buffer_mut(),
                this.len(),
                this.capacity(),
            ));
        }
    }
}

extern "C" fn grow_capacity_to_vec<T>(this: &mut RVec<T>, to: usize, exactness: Exactness) {
    extern_fn_panic_handling! {
        unsafe{
            this.with_vec(|list| {
                let additional = to.saturating_sub(list.len());
                match exactness {
                    Exactness::Above => list.reserve(additional),
                    Exactness::Exact => list.reserve_exact(additional),
                }
            })
        }
    }
}

extern "C" fn shrink_to_fit_vec<T>(this: &mut RVec<T>) {
    extern_fn_panic_handling! {
        unsafe{
            this.with_vec(|list| {
                list.shrink_to_fit();
            })
        }
    }
}
