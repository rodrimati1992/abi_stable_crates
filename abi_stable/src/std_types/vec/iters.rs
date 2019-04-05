use super::*;

use core_extensions::SliceExt;

pub(super) struct RawValIter<T> {
    pub(super) start: *const T,
    pub(super) end: *const T,
}

impl<T> RawValIter<T> {
    /// # Safety
    ///
    /// Must remember to keep the underlying allocation alive.
    pub(super) unsafe fn new(slice: &[T]) -> Self {
        RawValIter {
            start: slice.as_ptr(),
            end: if mem::size_of::<T>() == 0 {
                ((slice.as_ptr() as usize) + slice.len()) as *const _
            } else if slice.len() == 0 {
                slice.as_ptr()
            } else {
                slice.as_ptr().offset(slice.len() as isize)
            },
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
        let elem_size = mem::size_of::<T>();
        let distance = self.end as usize - self.start as usize;
        let stride_size = if elem_size == 0 { 1 } else { elem_size };
        let len = distance / stride_size;
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

pub struct IntoIter<T> {
    pub(super) _buf: ManuallyDrop<RVec<T>>,
    pub(super) iter: RawValIter<T>,
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

#[repr(C)]
pub struct Drain<'a, T> {
    pub(super) vec: &'a mut RVec<T>,
    pub(super) iter: RawValIter<T>,
    pub(super) len: usize,
    pub(super) removed_start: *mut T,
    pub(super) slice_len: usize,
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
            let end_index = self.vec.entire_buffer().index_of(removed_start) + self.slice_len;
            ptr::copy(removed_end, removed_start, self.len - end_index);
            self.vec.set_len(self.len - self.slice_len);
        }
    }
}
