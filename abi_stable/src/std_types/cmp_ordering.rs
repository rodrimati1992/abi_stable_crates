/*!
Contains the ffi-safe equivalent of `std::cmp::Ordering`.
*/


use std::cmp::Ordering;

/**
Ffi-safe equivalent of ::std::cmp::Ordering.

# Example

This defines an extern function,which compares a slice to another.

```

use abi_stable::{
    std_types::{RCmpOrdering,RSlice},
    sabi_extern_fn,
};
use std::cmp::Ord;


#[sabi_extern_fn]
pub fn compare_slices<T>(l:RSlice<'_,T>, r:RSlice<'_,T>)->RCmpOrdering
where
    T:Ord
{
    l.cmp(&r)
     .into()
}


```
*/
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[repr(u8)]
#[derive(StableAbi)]
pub enum RCmpOrdering {
    Less,
    Equal,
    Greater,
}

impl RCmpOrdering{
    /// Converts this `RCmpOrdering` into a `std::cmp::Ordering`;
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RCmpOrdering;
    /// use std::cmp::Ordering;
    /// 
    ///
    /// assert_eq!( RCmpOrdering::Less.into_ordering(), Ordering::Less );
    /// assert_eq!( RCmpOrdering::Equal.into_ordering(), Ordering::Equal );
    /// assert_eq!( RCmpOrdering::Greater.into_ordering(), Ordering::Greater );
    ///
    /// ```
    #[inline]
    pub fn into_ordering(self)->Ordering{
        self.into()
    }
}

impl_from_rust_repr! {
    impl From<Ordering> for RCmpOrdering {
        fn(this){
            match this {
                Ordering::Less=>RCmpOrdering::Less,
                Ordering::Equal=>RCmpOrdering::Equal,
                Ordering::Greater=>RCmpOrdering::Greater,
            }
        }
    }
}

impl_into_rust_repr! {
    impl Into<Ordering> for RCmpOrdering {
        fn(this){
            match this {
                RCmpOrdering::Less=>Ordering::Less,
                RCmpOrdering::Equal=>Ordering::Equal,
                RCmpOrdering::Greater=>Ordering::Greater,
            }
        }
    }
}
