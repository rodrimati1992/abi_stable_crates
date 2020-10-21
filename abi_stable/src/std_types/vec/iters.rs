use super::*;

use crate::utils::distance_from;

use std::slice;

pub(super) struct RawValIter<T> {
    pub(super) start: *const T,
    pub(super) end: *const T,
}

unsafe impl<T: Send> Send for RawValIter<T> {}
unsafe impl<T: Sync> Sync for RawValIter<T> {}

impl<T> RawValIter<T> {
    /// # Safety
    ///
    /// Must remember to keep the underlying allocation alive.
    pub(super) unsafe fn new(slice: &[T]) -> Self {
        RawValIter {
            start: slice.as_ptr(),
            end: if mem::size_of::<T>() == 0 {
                ((slice.as_ptr() as usize) + slice.len()) as *const _
            } else if slice.is_empty() {
                slice.as_ptr()
            } else {
                slice.as_ptr().offset(slice.len() as isize)
            },
        }
    }

    fn calculate_length(&self)->usize{
        let elem_size = mem::size_of::<T>();
        let distance = self.end as usize - self.start as usize;
        let stride_size = if elem_size == 0 { 1 } else { elem_size };
        distance / stride_size
    }

    fn as_slice(&self)->&[T]{
        let len=self.calculate_length();
        unsafe{
            unsafe { ::std::slice::from_raw_parts(self.start,len ) }
        }
    }

    fn as_mut_slice(&mut self)->&mut [T]{
        let len=self.calculate_length();
        unsafe{
            unsafe { ::std::slice::from_raw_parts_mut(self.start as *mut T,len ) }
        }
    }
}

impl<T> Iterator for RawValIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let result = ptr::read(self.start);
                self.start = if mem::size_of::<T>() == 0 {
                    (self.start as usize + 1) as *const _
                } else {
                    self.start.offset(1)
                };
                Some(result)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.calculate_length();
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for RawValIter<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                self.end = if mem::size_of::<T>() == 0 {
                    (self.end as usize - 1) as *const _
                } else {
                    self.end.offset(-1)
                };
                Some(ptr::read(self.end))
            }
        }
    }
}

///////////////////////////////////////////////////

/// An Iterator returned by `<RVec<T> as IntoIterator>::into_iter`,
/// which yields all the elements from the `RVec<T>`,
/// consuming it in the process.
pub struct IntoIter<T> {
    pub(super) _buf: ManuallyDrop<RVec<T>>,
    pub(super) iter: RawValIter<T>,
}


impl<T> IntoIter<T>{
    /// Returns a slice over the remainder of the `Vec<T>` that is being iterated over.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut iter=RVec::from(vec![0,1,2,3]).into_iter();
    ///
    /// assert_eq!( iter.as_slice(), &[0,1,2,3] );
    ///
    /// assert_eq!( iter.next(), Some(0) );
    /// assert_eq!( iter.as_slice(), &[1,2,3] );
    ///
    /// assert_eq!( iter.next_back(), Some(3) );
    /// assert_eq!( iter.as_slice(), &[1,2] );
    ///
    /// ```
    pub fn as_slice(&self)->&[T]{
        self.iter.as_slice()
    }

    /// Returns a mutable slice over the remainder of the `Vec<T>` that is being iterated over.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut iter=RVec::from(vec![0,1,2,3]).into_iter();
    ///
    /// assert_eq!( iter.as_mut_slice(), &mut [0,1,2,3] );
    ///
    /// assert_eq!( iter.next(), Some(0) );
    /// assert_eq!( iter.as_mut_slice(), &mut [1,2,3] );
    ///
    /// assert_eq!( iter.next_back(), Some(3) );
    /// assert_eq!( iter.as_mut_slice(), &mut [1,2] );
    ///
    /// ```
    pub fn as_mut_slice(&mut self)->&mut [T]{
        self.iter.as_mut_slice()
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        self.by_ref().for_each(drop);
        self._buf.length = 0;
        unsafe { ManuallyDrop::drop(&mut self._buf) }
    }
}

///////////////////////////////////////////////////

/// An Iterator returned by `RVec::drain` ,
/// which removes and yields all the elements in a range from the `RVec<T>`.
#[repr(C)]
pub struct Drain<'a, T> {
    pub(super) vec: &'a mut RVec<T>,
    pub(super) iter: RawValIter<T>,
    pub(super) len: usize,
    pub(super) removed_start: *mut T,
    pub(super) slice_len: usize,
}

impl<'a,T> Drain<'a,T>{
    /// Returns a slice over the remainder of the `Vec<T>` that is being drained.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut list=(0..8).collect::<RVec<u8>>();
    /// let mut iter=list.drain(3..7);
    ///
    /// assert_eq!( iter.as_slice(), &[3,4,5,6] );
    ///
    /// assert_eq!( iter.next(), Some(3) );
    /// assert_eq!( iter.as_slice(), &[4,5,6] );
    ///
    /// assert_eq!( iter.next(), Some(4) );
    /// assert_eq!( iter.as_slice(), &[5,6] );
    ///
    /// drop(iter);
    ///
    /// assert_eq!(list.as_slice(),&[0,1,2,7]);
    ///
    /// ```
    pub fn as_slice(&self)->&[T]{
        self.iter.as_slice()
    }

    /// Returns a mutable slice over the remainder of the `Vec<T>` that is being drained.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RVec;
    ///
    /// let mut list=(0..8).collect::<RVec<u8>>();
    /// let mut iter=list.drain(3..7);
    ///
    /// assert_eq!( iter.as_mut_slice(), &mut [3,4,5,6] );
    ///
    /// assert_eq!( iter.next(), Some(3) );
    /// assert_eq!( iter.as_mut_slice(), &mut [4,5,6] );
    ///
    /// assert_eq!( iter.next(), Some(4) );
    /// assert_eq!( iter.as_mut_slice(), &mut [5,6] );
    ///
    /// drop(iter);
    ///
    /// assert_eq!(list.as_slice(),&[0,1,2,7]);
    ///
    /// ```
    pub fn as_mut_slice(&mut self)->&mut [T]{
        self.iter.as_mut_slice()
    }
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        self.iter.by_ref().for_each(drop);
        unsafe {
            let removed_start = self.removed_start;
            let removed_end = self.removed_start.offset(self.slice_len as isize);
            let end_index = 
                distance_from(self.vec.as_ptr(),removed_start).unwrap_or(0)+self.slice_len;
            ptr::copy(removed_end, removed_start, self.len - end_index);
            self.vec.set_len(self.len - self.slice_len);
        }
    }
}


///////////////////////////////////////////////////



// copy-paste of the std library DrainFilter
#[derive(Debug)]
pub(crate) struct DrainFilter<'a, T, F>
    where F: FnMut(&mut T) -> bool,
{
    pub(super) vec: &'a mut RVec<T>,
    pub(super) idx: usize,
    pub(super) del: usize,
    pub(super) old_len: usize,
    pub(super) pred: F,
}

impl<T, F> Iterator for DrainFilter<'_, T, F>
    where F: FnMut(&mut T) -> bool,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        unsafe {
            while self.idx != self.old_len {
                let i = self.idx;
                self.idx += 1;
                let v = slice::from_raw_parts_mut(self.vec.as_mut_ptr(), self.old_len);
                if (self.pred)(&mut v[i]) {
                    self.del += 1;
                    return Some(ptr::read(&v[i]));
                } else if self.del > 0 {
                    let del = self.del;
                    let src: *const T = &v[i];
                    let dst: *mut T = &mut v[i - del];
                    ptr::copy_nonoverlapping(src, dst, 1);
                }
            }
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.old_len - self.idx))
    }
}

impl<T, F> Drop for DrainFilter<'_, T, F>
    where F: FnMut(&mut T) -> bool,
{
    fn drop(&mut self) {
        self.for_each(drop);
        unsafe {
            self.vec.set_len(self.old_len - self.del);
        }
    }
}

