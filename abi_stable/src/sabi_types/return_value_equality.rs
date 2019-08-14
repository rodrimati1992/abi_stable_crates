use std::{
    cmp::{Eq, PartialEq},
};

/**
Wrapper type for an `extern fn()->T` using the return value for comparisons.

# Example

```
use abi_stable::{
    sabi_types::ReturnValueEquality,
    std_types::{ROption,RNone,RSome},
};


extern fn returns_100()->ROption<u32>{
    RSome(100)
}

extern fn returns_100b()->ROption<u32>{
    RSome(100)
}

extern fn returns_200()->ROption<u32>{
    RSome(200)
}

extern fn returns_none()->ROption<u32>{
    RNone
}


let a=ReturnValueEquality::new(returns_100);
let b=ReturnValueEquality::new(returns_100b);
let c=ReturnValueEquality::new(returns_200);
let d=ReturnValueEquality::new(returns_none);

assert_eq!(a,a);
assert_eq!(b,b);
assert_eq!(c,c);
assert_eq!(d,d);

assert_eq!(a,b);

assert_ne!(a,c);
assert_ne!(a,d);
assert_ne!(b,c);
assert_ne!(c,d);


```
*/
#[repr(transparent)]
#[derive(Debug, StableAbi)]
pub struct ReturnValueEquality<T> {
    pub function: extern "C" fn() -> T,
}

impl<T> ReturnValueEquality<T>{
    /// Constructs a `ReturnValueEquality`.
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::{
    ///     sabi_types::ReturnValueEquality,
    ///     std_types::{ROption,RSome},
    /// };
    /// extern fn returns_100()->ROption<u32>{
    ///     RSome(100)
    /// }
    ///
    /// static RVE:ReturnValueEquality<ROption<u32>>=
    ///     ReturnValueEquality{function:returns_100};
    ///
    /// let rve=ReturnValueEquality::new(returns_100);
    ///
    /// assert_eq!(RVE,rve);
    ///
    /// ```
    pub fn new(function: extern "C" fn() -> T)->Self{
        Self{function}
    }
    
    /// Gets the value returned by the function.
    pub fn get(&self)->T{
        (self.function)()
    }
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