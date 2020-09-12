#![allow(dead_code)]

use std::{
    error::Error as ErrorTrait,
    fmt::{self, Debug, Display},
};

#[allow(unused_imports)]
pub use abi_stable_shared::test_utils::{
    FileSpan,
    ThreadError,
    ShouldHavePanickedAt,
    must_panic,
};

#[allow(unused_imports)]
pub use abi_stable_shared::file_span;


//////////////////////////////////////////////////////////////////


/// Checks that `left` and `right` produce the exact same Display and Debug output.
pub fn check_formatting_equivalence<T,U>(left:&T,right:&U)
where 
    T:Debug+Display+?Sized,
    U:Debug+Display+?Sized,
{
    assert_eq!(format!("{:?}",left), format!("{:?}",right));
    assert_eq!(format!("{:#?}",left), format!("{:#?}",right));
    assert_eq!(format!("{}",left), format!("{}",right));
    assert_eq!(format!("{:#}",left), format!("{:#}",right));
}

/// Returns the address this dereferences to.
pub fn deref_address<D>(ptr:&D)->usize
where
    D: ::std::ops::Deref,
{
    (&**ptr)as *const _ as *const u8 as usize
}


//////////////////////////////////////////////////////////////////


/// A wrapper type which uses `T`'s Display formatter in its Debug impl
#[repr(transparent)]
#[derive(Copy,Clone,PartialEq,Eq,Ord,PartialOrd,Hash,StableAbi)]
pub struct AlwaysDisplay<T>(pub T);

impl<T> Display for AlwaysDisplay<T> 
where
    T:Display
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0,f)
    }
}


impl<T> Debug for AlwaysDisplay<T> 
where
    T:Display
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0,f)
    }
}

//////////////////////////////////////////////////////////////////


#[derive(Clone)]
pub struct Stringy{
    pub str:String
}

impl Stringy{
    pub fn new<S>(str:S)->Self
    where S:Into<String>
    {
        Stringy{
            str:str.into(),
        }
    }
}


impl Debug for Stringy{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Debug::fmt(&self.str,f)
    }
}

impl Display for Stringy{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Display::fmt(&self.str,f)
    }
}

impl ErrorTrait for Stringy{}

