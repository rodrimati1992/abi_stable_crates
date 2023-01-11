//! Helper types for type_layout types.

use std::cmp::{Eq, PartialEq};

////////////////////////////////////////////////////////////////////////////////

/// A pair of length and an array,
/// which is treated as a slice of `0..self.len` in all its impls.
#[repr(C)]
#[derive(Debug, Copy, Clone, StableAbi)]
pub struct ArrayLen<A> {
    /// the length of initialized elements in `array`
    pub len: u16,
    ///
    pub array: A,
}

impl<A> ArrayLen<A> {
    /// The `len` field  casted to usize.
    pub const fn len(&self) -> usize {
        self.len as usize
    }
    /// Whether the array is empty
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<A, T> PartialEq for ArrayLen<A>
where
    A: ArrayTrait<Elem = T>,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let t_slice = &self.array.as_slice()[..self.len as usize];
        let o_slice = &other.array.as_slice()[..other.len as usize];
        t_slice == o_slice
    }
}

impl<A, T> Eq for ArrayLen<A>
where
    A: ArrayTrait<Elem = T>,
    T: Eq,
{
}

////////////////////////////////////////////////////////////////////////////////

mod array_trait {
    pub trait ArrayTrait {
        type Elem;

        fn as_slice(&self) -> &[Self::Elem];
    }
}
use self::array_trait::ArrayTrait;

macro_rules! impl_stable_abi_array {
    ($($size:expr),*)=>{
        $(
            impl<T> ArrayTrait for [T;$size] {
                type Elem=T;

                fn as_slice(&self)->&[T]{
                    self
                }
            }
        )*
    }
}

impl_stable_abi_array! {
    00,01,02,03,04,05,06,07,08
}
