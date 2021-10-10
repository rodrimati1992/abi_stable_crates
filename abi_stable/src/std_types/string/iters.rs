use super::*;

/////////////////////////////////////////////////////////////////////////////

/// An Iterator created by `<RString as IntoIterator>::into_iter`,
/// which yields all the characters from the `RString`,
/// consuming it in the process.
pub struct IntoIter {
    pub(super) _buf: RString,
    pub(super) iter: Chars<'static>,
}

unsafe impl Send for IntoIter {}
unsafe impl Sync for IntoIter {}

impl IntoIter {
    /// Returns a string slice over the remainder of the string that is being iterated over.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// let mut iter = RString::from("abcd").into_iter();
    ///
    /// assert_eq!(iter.as_str(), "abcd");
    ///
    /// assert_eq!(iter.next(), Some('a'));
    /// assert_eq!(iter.as_str(), "bcd");
    ///
    /// assert_eq!(iter.next_back(), Some('d'));
    /// assert_eq!(iter.as_str(), "bc");
    ///
    /// ```
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
    pub(super) variance: PhantomData<&'a mut [char]>,
}

impl<'a> fmt::Debug for Drain<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Drain { .. }")
    }
}

unsafe impl<'a> Sync for Drain<'a> {}
unsafe impl<'a> Send for Drain<'a> {}

impl<'a> Drain<'a> {
    /// Returns a string slice over the remainder of the string that is being drained.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RString;
    ///
    /// let mut string = RString::from("abcdefg");
    /// let mut iter = string.drain(2..6);
    ///
    /// assert_eq!(iter.as_str(), "cdef");
    ///
    /// assert_eq!(iter.next(), Some('c'));
    /// assert_eq!(iter.as_str(), "def");
    ///
    /// assert_eq!(iter.next_back(), Some('f'));
    /// assert_eq!(iter.as_str(), "de");
    ///
    /// drop(iter);
    ///
    /// assert_eq!(string.as_str(), "abg")
    ///
    /// ```
    pub fn as_str(&self) -> &str {
        self.iter.as_str()
    }
}

impl<'a> Drop for Drain<'a> {
    fn drop(&mut self) {
        unsafe {
            let self_vec = &mut (*self.string).inner;
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
