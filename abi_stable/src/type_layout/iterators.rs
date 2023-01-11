#![allow(clippy::missing_const_for_fn)]

#[derive(Debug, Clone)]
pub(crate) struct ChainOnce<I, T> {
    iter: I,
    once: Option<T>,
}

impl<I> ChainOnce<I, I::Item>
where
    I: ExactSizeIterator,
{
    pub(crate) fn new(iter: I, once: I::Item) -> Self {
        Self {
            iter,
            once: Some(once),
        }
    }
    fn length(&self) -> usize {
        self.iter.len() + (self.once.is_some() as usize)
    }
}
impl<I> Iterator for ChainOnce<I, I::Item>
where
    I: ExactSizeIterator,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> {
        if let ret @ Some(_) = self.iter.next() {
            return ret;
        }
        self.once.take()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.length();
        (len, Some(len))
    }
    fn count(self) -> usize {
        self.length()
    }
}

impl<I> std::iter::ExactSizeIterator for ChainOnce<I, I::Item> where I: ExactSizeIterator {}
