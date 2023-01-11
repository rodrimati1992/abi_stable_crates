#![allow(dead_code)]

use std::{
    error::Error as ErrorTrait,
    fmt::{self, Debug, Display},
};

#[allow(unused_imports)]
pub use abi_stable_shared::test_utils::{must_panic, ShouldHavePanickedAt, ThreadError};

//////////////////////////////////////////////////////////////////

/// Checks that `left` and `right` produce the exact same Display and Debug output.
pub fn check_formatting_equivalence<T, U>(left: &T, right: &U)
where
    T: Debug + Display + ?Sized,
    U: Debug + Display + ?Sized,
{
    assert_eq!(format!("{:?}", left), format!("{:?}", right));
    assert_eq!(format!("{:#?}", left), format!("{:#?}", right));
    assert_eq!(format!("{}", left), format!("{}", right));
    assert_eq!(format!("{:#}", left), format!("{:#}", right));
}

/// Returns the address this dereferences to.
pub fn deref_address<D>(ptr: &D) -> usize
where
    D: ::std::ops::Deref,
{
    (&**ptr) as *const _ as *const u8 as usize
}

//////////////////////////////////////////////////////////////////

/// A wrapper type which uses `T`'s Display formatter in its Debug impl
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, StableAbi)]
pub struct AlwaysDisplay<T>(pub T);

impl<T> Display for AlwaysDisplay<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T> Debug for AlwaysDisplay<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

//////////////////////////////////////////////////////////////////

macro_rules! if_impls_impls {
    ($($const:ident, $trait:path);* $(;)*) => {
        pub trait GetImplsHelper {
            $(const $const: bool = false;)*
        }

        impl<T> GetImplsHelper for T {}

        pub struct GetImpls<S>(S);

        $(
            impl<S> GetImpls<S>
            where
                S: $trait
            {
                pub const $const: bool = true;
            }
        )*
    };
}

if_impls_impls! {
    IMPLS_SEND, Send;
    IMPLS_SYNC, Sync;
    IMPLS_UNPIN, Unpin;
    IMPLS_CLONE, Clone;
    IMPLS_DISPLAY, std::fmt::Display;
    IMPLS_DEBUG, std::fmt::Debug;
    IMPLS_SERIALIZE, serde::Serialize;
    IMPLS_EQ, std::cmp::Eq;
    IMPLS_PARTIAL_EQ, std::cmp::PartialEq;
    IMPLS_ORD, std::cmp::Ord;
    IMPLS_PARTIAL_ORD, std::cmp::PartialOrd;
    IMPLS_HASH, std::hash::Hash;
    IMPLS_DESERIALIZE, serde::Deserialize<'static>;
    IMPLS_ITERATOR, Iterator;
    IMPLS_DOUBLE_ENDED_ITERATOR, DoubleEndedIterator;
    IMPLS_FMT_WRITE, std::fmt::Write;
    IMPLS_IO_WRITE, std::io::Write;
    IMPLS_IO_SEEK, std::io::Seek;
    IMPLS_IO_READ, std::io::Read;
    IMPLS_IO_BUF_READ, std::io::BufRead;
    IMPLS_ERROR, std::error::Error;
}

//////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Stringy {
    pub str: String,
}

impl Stringy {
    pub fn new<S>(str: S) -> Self
    where
        S: Into<String>,
    {
        Stringy { str: str.into() }
    }
}

impl Debug for Stringy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.str, f)
    }
}

impl Display for Stringy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.str, f)
    }
}

impl ErrorTrait for Stringy {}
