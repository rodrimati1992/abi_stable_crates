use super::*;

/////////////////////////////////////////////////////////////////////////////

/// An Iterator created by `<RString as IntoIterator>::into_iter`,
/// which yields all the characters from the RString,
/// consuming it in the process.
pub struct IntoIter {
    pub(super) _buf: RString,
    pub(super) iter: Chars<'static>,
}

impl IntoIter {
    pub fn as_str(&self) -> &str {
        self.iter.as_str()
    }
}

impl Iterator for IntoIter {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<char> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl DoubleEndedIterator for IntoIter {
    #[inline]
    fn next_back(&mut self) -> Option<char> {
        self.iter.next_back()
    }
}

impl FusedIterator for IntoIter {}

/////////////////////////////////////////////////////////////////////////////

/// An Iterator returned by `RString::drain` ,
/// which removes and yields all the characters in a range from the RString.
pub struct Drain<'a> {
    pub(super) string: *mut RString,
    pub(super) removed: Range<usize>,
    pub(super) iter: Chars<'a>,
    pub(super) variance:PhantomData<&'a mut [char]>,
}

impl<'a> fmt::Debug for Drain<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Drain { .. }")
    }
}

unsafe impl<'a> Sync for Drain<'a> {}
unsafe impl<'a> Send for Drain<'a> {}

impl<'a> Drain<'a> {
    pub fn as_str(&self) -> &str {
        self.iter.as_str()
    }
}

impl<'a> Drop for Drain<'a> {
    fn drop(&mut self) {
        unsafe {
            let self_vec = &mut (*self.string).vec;
            if self.removed.start <= self.removed.end && self.removed.end <= self_vec.len() {
                self_vec.drain(self.removed.start..self.removed.end);
            }
        }
    }
}

impl<'a> Iterator for Drain<'a> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<char> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> DoubleEndedIterator for Drain<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<char> {
        self.iter.next_back()
    }
}

impl<'a> FusedIterator for Drain<'a> {}
