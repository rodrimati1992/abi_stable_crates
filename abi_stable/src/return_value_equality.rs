use std::{
    cmp::{Eq, PartialEq},
};

/// Wrapper type for an `extern fn()->T` using the return value for comparisons.
#[repr(transparent)]
#[derive(Debug, StableAbi)]
pub struct ReturnValueEquality<T> {
    pub function: extern "C" fn() -> T,
}

impl<T> Copy for ReturnValueEquality<T>{}
impl<T> Clone for ReturnValueEquality<T>{
    fn clone(&self)->Self{
        *self
    }
}

impl<T: Eq> Eq for ReturnValueEquality<T> {}

impl<T: PartialEq> PartialEq for ReturnValueEquality<T> {
    fn eq(&self, other: &Self) -> bool {
        (self.function)() == (other.function)()
    }
}
