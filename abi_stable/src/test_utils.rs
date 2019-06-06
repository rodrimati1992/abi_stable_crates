use std::{
    any::Any,
    error::Error as ErrorTrait,
    fmt::{self, Debug, Display},
    panic::{catch_unwind, AssertUnwindSafe},
};


#[derive(Debug, Clone)]
pub struct FileSpan {
    pub file: &'static str,
    pub line: u32,
}

pub type ThreadError = Box<dyn Any + Send + 'static>;

#[derive(Debug, Clone)]
pub struct ShouldHavePanickedAt {
    pub span: FileSpan,
}

macro_rules! file_span {
    () => {{
        use crate::test_utils::FileSpan;
        FileSpan {
            file: file!(),
            line: line!(),
        }
    }};
}

pub fn must_panic<F, R>(span: FileSpan, f: F) -> Result<ThreadError, ShouldHavePanickedAt>
where
    F: FnOnce() -> R,
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(_) => Err(ShouldHavePanickedAt { span }),
        Err(e) => Ok(e),
    }
}

#[test]
fn test_must_panic() {
    assert!(must_panic(file_span!(), || panic!()).is_ok());
    assert!(must_panic(file_span!(), || ()).is_err());
}


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

