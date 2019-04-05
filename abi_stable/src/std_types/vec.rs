use std::{
    borrow::Cow,
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

use crate::{RSlice, RSliceMut};

#[cfg(test)]
mod tests;

mod iters;

use self::iters::RawValIter;

pub use self::iters::{Drain, IntoIter};

mod private {
    /// Type used to represent a Vec<u8> in any language.
    ///
    /// This allows sharing a Vec<u8> between different versions of Rust,
    /// even ones with a different allocator
    use super::*;
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(inside_abi_stable_crate)]
    // #[sabi(debug_print)]
    pub struct RVec<T> {
        /// Look at the documentation for ErasedObject for why ErasedObject instead of T.
        buffer: *mut T,
        pub(super) length: usize,
        capacity: usize,
        vtable: *const BufferVTable<T>,
        _marker: PhantomData<T>,
    }

    impl<T> RVec<T> {
        #[allow(dead_code)]
        // Used to test functions that change behavior when the vtable changes
        pub(super) fn set_vtable_for_testing(mut self) -> Self {
            self.vtable = VTableGetter::<T>::LIB_VTABLE_FOR_TESTING as *const _;
            self
        }

        #[inline(always)]
        pub(super) fn vtable<'a>(&self) -> &'a BufferVTable<T>
        where
            T: 'a,
        {
            unsafe { &*self.vtable }
        }

        #[inline(always)]
        pub(super) fn buffer(&self) -> *mut T {
            self.buffer
        }

        pub fn capacity(&self) -> usize {
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
                let old = mem::replace(self, RVec::new()).piped(ManuallyDrop::new);
                let mut list = Vec::<T>::from_raw_parts(old.buffer(), old.len(), old.capacity());
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
                    vtable: VTableGetter::<T>::LIB_VTABLE,
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
    pub fn new() -> Self {
        Vec::new().into()
    }

    pub fn with_capacity(cap: usize) -> Self {
        Vec::with_capacity(cap).into()
    }

    unsafe fn entire_buffer(&mut self) -> &mut [T] {
        ::std::slice::from_raw_parts_mut(self.buffer(), self.capacity())
    }

    #[inline]
    pub fn slice<'a, I>(&'a self, i: I) -> RSlice<'a, T>
    where
        [T]: Index<I, Output = [T]>,
    {
        (&self[i]).into()
    }

    #[inline]
    pub fn slice_mut<'a, I>(&'a mut self, i: I) -> RSliceMut<'a, T>
    where
        [T]: IndexMut<I, Output = [T]>,
    {
        (&mut self[i]).into()
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { ::std::slice::from_raw_parts(self.buffer(), self.len()) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { ::std::slice::from_raw_parts_mut(self.buffer(), self.len()) }
    }

    pub fn as_rslice(&self) -> RSlice<'_, T> {
        self.as_slice().into()
    }

    pub fn as_rslice_mut(&mut self) -> RSliceMut<'_, T> {
        self.as_slice_mut().into()
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.length = new_len;
    }

    pub fn shrink_to_fit(&mut self) {
        let vtable = self.vtable();
        (vtable.shrink_to_fit)(self);
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Returns a Vec<T>,consuming `self`.
    ///
    /// # Allocation
    ///
    /// Moves all the elements into a newly allocated Vec<T>
    /// if the destructor is not the one for Vec<T>.
    pub fn into_vec(self) -> Vec<T> {
        let mut this = ManuallyDrop::new(self);

        unsafe {
            if this.vtable().destructor as usize == destructor_vec::<T> as usize {
                Vec::from_raw_parts(this.buffer(), this.len(), this.capacity())
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

    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.as_slice().to_vec()
    }

    /// Appends another collection,clearing it in the process
    pub fn append<C>(&mut self, other: &mut C)
    where
        C: Appendable<T>,
    {
        let slic_ = other.as_slice();
        let additional = slic_.len();
        let old_len = self.len();
        self.reserve(additional);
        unsafe {
            ptr::copy_nonoverlapping(slic_.as_ptr(), self.get_unchecked_mut(old_len), additional)
        }
        self.length += additional;
    }

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
            if index < self.length {
                ptr::copy(
                    self.buffer().offset(index as isize),
                    self.buffer().offset(index as isize + 1),
                    self.length - index,
                );
            }
            ptr::write(self.buffer().offset(index as isize), value);
            self.length += 1;
        }
    }

    pub fn try_remove(&mut self, index: usize) -> Option<T> {
        if self.length <= index {
            return None;
        }
        unsafe {
            self.length -= 1;
            let result = ptr::read(self.buffer().offset(index as isize));
            ptr::copy(
                self.buffer().offset(index as isize + 1),
                self.buffer().offset(index as isize),
                self.length - index,
            );
            Some(result)
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        match self.try_remove(index) {
            Some(x) => x,
            None => panic!("index out of bounds,index={} len={} ", index, self.length),
        }
    }

    pub fn swap_remove(&mut self, index: usize) -> T {
        unsafe {
            let hole: *mut T = &mut self[index];
            let last = ptr::read(self.buffer().offset((self.length - 1) as isize));
            self.length -= 1;
            ptr::replace(hole, last)
        }
    }

    pub fn push(&mut self, new_val: T) {
        if self.length == self.capacity() {
            self.grow_capacity_to_1();
        }
        unsafe {
            ptr::write(self.buffer().offset(self.length as isize), new_val);
        }
        self.length += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.length == 0 {
            None
        } else {
            unsafe {
                self.length -= 1;
                Some(ptr::read(self.buffer().offset(self.length as isize)))
            }
        }
    }

    pub fn truncate(&mut self, to: usize) {
        if to < self.len() {
            self.truncate_inner(to);
        }
    }

    fn truncate_inner(&mut self, to: usize) {
        let old_length = self.length;
        self.length = to;
        unsafe {
            for elem in &mut self.entire_buffer()[to..old_length] {
                ptr::drop_in_place(elem);
            }
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        self.resize_capacity(self.len() + additional, Exactness::Above)
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.resize_capacity(self.len() + additional, Exactness::Exact)
    }

    #[inline]
    fn grow_capacity_to_1(&mut self) {
        let vtable = self.vtable();
        (vtable.grow_capacity_to)(self, self.capacity() + 1, Exactness::Above);
    }

    fn resize_capacity(&mut self, to: usize, exactness: Exactness) {
        let vtable = self.vtable();
        if self.capacity() < to {
            (vtable.grow_capacity_to)(self, to, exactness);
        }
    }
}

impl<T> RVec<T>
where
    T: Clone,
{
    pub fn resize(&mut self, new_len: usize, value: T) {
        let old_len = self.len();
        match new_len.cmp(&old_len) {
            Ordering::Less => self.truncate_inner(new_len),
            Ordering::Equal => {}
            Ordering::Greater => unsafe {
                self.resize_capacity(new_len, Exactness::Above);
                // Using new_len instead of the capacity because resize_capacity may
                // grow the capacity more than requested.
                let new_elems = &mut self.entire_buffer()[old_len..new_len];
                for elem_ptr in new_elems {
                    ptr::write(elem_ptr, value.clone());
                }
                self.length = new_len;
            },
        }
    }

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
    pub fn extend_from_copy_slice(&mut self, slic_: &[T]) {
        self.reserve(slic_.len());
        let old_len = self.len();
        unsafe {
            let entire = self.get_unchecked_mut(old_len);
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
        self.as_slice_mut()
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
        (vtable.destructor)(self)
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
        self.reserve(lower.saturating_add(1));
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

pub unsafe trait Appendable<T> {
    fn as_slice(&self) -> &[T];
    unsafe fn clear_forget(&mut self);
}

unsafe impl<T> Appendable<T> for RVec<T> {
    fn as_slice(&self) -> &[T] {
        &**self
    }
    unsafe fn clear_forget(&mut self) {
        self.set_len(0);
    }
}

unsafe impl<T> Appendable<T> for Vec<T> {
    fn as_slice(&self) -> &[T] {
        &**self
    }
    unsafe fn clear_forget(&mut self) {
        self.set_len(0);
    }
}

////////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
#[sabi(inside_abi_stable_crate)]
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
    // The VTABLE for this type in this executable/library
    const LIB_VTABLE: &'a BufferVTable<T> = &BufferVTable {
        destructor: destructor_vec,
        grow_capacity_to: grow_capacity_to_vec,
        shrink_to_fit: shrink_to_fit_vec,
    };

    // Used to test functions that change behavior based on the vtable being used
    const LIB_VTABLE_FOR_TESTING: &'a BufferVTable<T> = &BufferVTable {
        destructor: destructor_vec_for_testing,
        ..*Self::LIB_VTABLE
    };
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate, kind(unsafe_Prefix))]
pub struct BufferVTable<T> {
    destructor: extern "C" fn(&mut RVec<T>),
    grow_capacity_to: extern "C" fn(&mut RVec<T>, usize, Exactness),
    shrink_to_fit: extern "C" fn(&mut RVec<T>),
}

impl<T> Copy for BufferVTable<T> {}
impl<T> Clone for BufferVTable<T> {
    fn clone(&self) -> Self {
        *self
    }
}

extern "C" fn destructor_vec<T>(this: &mut RVec<T>) {
    extern_fn_panic_handling! {
        unsafe {
            drop(Vec::from_raw_parts(
                this.buffer(),
                this.len(),
                this.capacity(),
            ));
        }
    }
}

// Used to test functions that change behavior based on the vtable being used
extern "C" fn destructor_vec_for_testing<T>(this: &mut RVec<T>) {
    destructor_vec(this)
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
