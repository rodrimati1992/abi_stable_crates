use std::{
    error::Error as ErrorTrait,
    fmt::{self, Debug, Display},
};

#[allow(unused_imports)]
pub(crate) use abi_stable_shared::test_utils::{
    FileSpan,
    ThreadError,
    ShouldHavePanickedAt,
    must_panic,
};


//////////////////////////////////////////////////////////////////


/// Checks that `left` and `right` produce the exact same Display and Debug output.
pub(crate) fn check_formatting_equivalence<T,U>(left:&T,right:&U)
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
pub(crate) fn deref_address<D>(ptr:&D)->usize
where
    D: ::std::ops::Deref,
{
    (&**ptr)as *const _ as *const u8 as usize
}


//////////////////////////////////////////////////////////////////


#[derive(Clone)]
pub(crate) struct Stringy{
    str:String
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

