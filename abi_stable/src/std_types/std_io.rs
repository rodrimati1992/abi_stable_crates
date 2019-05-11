/**
Ffi-safe versions of some `std::io` types.
*/

use std::{
    error::Error as ErrorTrait,
    fmt::{self, Debug, Display},
    io::{Error as ioError,SeekFrom, ErrorKind},
};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use crate::{
    traits::{IntoReprC,IntoReprRust}, 
    std_types::{RBoxError,ROption,RSome,RNone},
};


///////////////////////////////////////////////////////////////////////////

/// Ffi safe equivalent to `std::io::ErrorKind`.
///
/// Using a struct with associated constants is the 
/// ffi-safe way of doing `#[non_exhaustive]` field-less enums.
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct RIoErrorKind {
    value: u8,
}


macro_rules! impl_error_kind {
    (
        $(
            $variant:ident,discriminant=$value:expr , message=$as_str_msg:expr ; 
        )* 
    ) => (
        /// Every (visible) variant of RIoErrorKind,equivalent to that of `std::io::ErrorKind`.
        #[allow(non_upper_case_globals)]
        impl RIoErrorKind {
            $(
                pub const $variant: Self = RIoErrorKind { value: $value };
            )*
            pub const Other: Self = RIoErrorKind { value: 0 };
        }

        impl Debug for RIoErrorKind {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let s = match *self {
                    $(
                        RIoErrorKind::$variant=> stringify!($variant),
                    )*
                    _ => "Other",
                };
                Display::fmt(s, f)
            }
        }

        impl_from_rust_repr! {
            impl From<ErrorKind> for RIoErrorKind {
                fn(this){
                    match this {
                        $(
                            ErrorKind::$variant=> RIoErrorKind::$variant,
                        )*
                        _=>RIoErrorKind::Other,
                    }
                }
            }
        }

        impl_into_rust_repr! {
            impl Into<ErrorKind> for RIoErrorKind {
                fn(this){
                    match this {
                        $(
                            RIoErrorKind::$variant=> ErrorKind::$variant,
                        )*
                        _=>ErrorKind::Other,
                    }
                }
            }
        }

        impl RIoErrorKind {
            pub(crate) fn error_message(&self) -> &'static str {
                match *self {
                    $(
                        RIoErrorKind::$variant => $as_str_msg,
                    )*
                    _=> "other os error",
                }
            }
        }
    )
}

impl_error_kind!{
    NotFound, discriminant = 1 , message="entity not found" ;
    PermissionDenied, discriminant = 2 , message="permission denied" ;
    ConnectionRefused, discriminant = 3 , message="connection refused" ;
    ConnectionReset, discriminant = 4 , message="connection reset" ;
    ConnectionAborted, discriminant = 5 , message="connection aborted" ;
    NotConnected, discriminant = 6 , message="not connected" ;
    AddrInUse, discriminant = 7 , message="address in use" ;
    AddrNotAvailable, discriminant = 8 , message="address not available" ;
    BrokenPipe, discriminant = 9 , message="broken pipe" ;
    AlreadyExists, discriminant = 10 , message="entity already exists" ;
    WouldBlock, discriminant = 11 , message="operation would block" ;
    InvalidInput, discriminant = 12 , message="invalid input parameter" ;
    InvalidData, discriminant = 13 , message="invalid data" ;
    TimedOut, discriminant = 14 , message="timed out" ;
    WriteZero, discriminant = 15 , message="write zero" ;
    Interrupted, discriminant = 16 , message="operation interrupted" ;
    UnexpectedEof, discriminant = 17 , message="unexpected end of file" ;
}


///////////////////////////////////////////////////////////////////////////


/// Ffi-safe equivalent of `std::io::SeekFrom`.
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub enum RSeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

impl_from_rust_repr! {
    impl From<SeekFrom> for RSeekFrom {
        fn(this){
            match this {
                SeekFrom::Start(x)  =>RSeekFrom::Start(x),
                SeekFrom::End(x)    =>RSeekFrom::End(x),
                SeekFrom::Current(x)=>RSeekFrom::Current(x),
            }
        }
    }
}


impl_into_rust_repr! {
    impl Into<SeekFrom> for RSeekFrom {
        fn(this){
            match this {
                RSeekFrom::Start(x)  =>SeekFrom::Start(x),
                RSeekFrom::End(x)    =>SeekFrom::End(x),
                RSeekFrom::Current(x)=>SeekFrom::Current(x),
            }
        }
    }
}



///////////////////////////////////////////////////////////////////////////

/// Ffi safe equivalent to `std::io::Error`.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct RIoError {
    kind: RIoErrorKind,
    error: ROption<RBoxError>,
}

impl_from_rust_repr! {
    impl From<ioError> for RIoError {
        fn(this){
            RIoError{
                kind:this.kind().into(),
                error:this.into_inner().map(RBoxError::from_box).into_c()
            }
        }
    }
}

impl_into_rust_repr! {
    impl Into<ioError> for RIoError {
        fn(this){
            let kind=this.kind().into_(ErrorKind::T);
            match this.into_inner() {
                Some(e)=>ioError::new(kind,RBoxError::into_box(e)),
                None=>ioError::from(kind),
            }
        }
    }
}

impl From<RIoErrorKind> for RIoError{
    fn from(kind:RIoErrorKind)->Self{
        Self{ 
            kind, 
            error:RNone,
        }
    }
}

impl From<ErrorKind> for RIoError{
    fn from(kind:ErrorKind)->Self{
        Self{ 
            kind:kind.into(),
            error:RNone
        }
    }
}

impl RIoError {
    /// Constructs an RIoError from an error and a `std::io::ErrorKind`.
    pub fn new<E>(kind: ErrorKind, error: E) -> Self
    where
        E: ErrorTrait + Send + Sync + 'static,
    {
        RIoError {
            kind: kind.into_c(),
            error: RSome(RBoxError::new(error)),
        }
    }

    /// Constructs an RIoError from a `std::io::ErrorKind`.
    pub fn from_kind(kind: ErrorKind)->Self{
        Self{
            kind:kind.into_c(),
            error:RNone,
        }
    }

    /// Constructs an RIoError from a 
    /// `Box<dyn ErrorTrait+Send+Sync+'static>` and a `std::io::ErrorKind`.
    pub fn with_box(kind: ErrorKind, error: Box<dyn ErrorTrait+Send+Sync+'static>) -> Self{
        RIoError {
            kind: kind.into_c(),
            error: RSome(RBoxError::from_box(error)),
        }
    }

    /// Constructs an RIoError from an `RBoxError` and a `std::io::ErrorKind`.
    pub fn with_rboxerror(kind: ErrorKind, error: RBoxError) -> Self{
        RIoError {
            kind: kind.into_c(),
            error:RSome(error),
        }
    }

    /// Retrieves the kind of io error.
    pub fn kind(&self)->RIoErrorKind{
        self.kind
    }


    /// Gets the internal error,
    /// returning None if this was constructed with `RIoError::from_kind`.
    pub fn get_ref(&self) -> Option<&RBoxError>{
        self.error.as_ref().into_rust()
    }
    
    /// Gets the internal error,
    /// returning None if this was constructed with `RIoError::from_kind`.
    pub fn get_mut(&mut self) -> Option<&mut RBoxError>{
        self.error.as_mut().into_rust()
    }

    /// Converts this into the internal error,
    /// returning None if this was constructed with `RIoError::from_kind`.
    pub fn into_inner(self) -> Option<RBoxError>{
        self.error.into_rust()
    }

}

impl Debug for RIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.error.as_ref() {
            RSome(c) => Debug::fmt(&c, f),
            RNone => f.debug_tuple("Kind").field(&self.kind).finish(),
        }
    }
}


impl Display for RIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.error.as_ref() {
            RSome(c) => Display::fmt(&c, f),
            RNone => Display::fmt(self.kind.error_message(),f),
        }
    }
}

impl ErrorTrait for RIoError {}


///////////////////////////////////////////////////////////////////////////////////


#[cfg(all(test,not(feature="only_new_tests")))]
mod error_kind_tests{
    use super::*;

    #[test]
    fn conversions(){
        for (from,to) in
            vec![
                (ErrorKind::NotConnected,RIoErrorKind::NotConnected),
                (ErrorKind::AddrInUse,RIoErrorKind::AddrInUse),
                (ErrorKind::Other,RIoErrorKind::Other),
            ]
        {
            assert_eq!(RIoErrorKind::from(from) , to);
            assert_eq!(to.into_(ErrorKind::T) , from);
        }
    }
}

#[cfg(all(test,not(feature="only_new_tests")))]
mod io_error_tests{
    use super::*;

    use crate::test_utils::{
        check_formatting_equivalence,
        deref_address,
        Stringy,
    };


    #[test]
    fn from_error_kind(){
        for kind in 
            vec![
                ErrorKind::NotConnected,
                ErrorKind::AddrInUse,
                ErrorKind::Other,
            ]
        {
            let err=kind.piped(RIoError::from_kind);

            assert_eq!(err.kind(), kind.into_c());
        }
    }

    #[test]
    fn from_value(){
        let err=Stringy::new("What\nis\ra\tline");
        let e0=RIoError::new(ErrorKind::Other,err.clone());

        check_formatting_equivalence(&err,&e0);
    }

    #[test]
    fn from_boxerror(){
        let err=Stringy::new("What\nis\ra\tline");
        let box_=err.clone().piped(Box::new);
        let addr=deref_address(&box_);
        let ioerr=RIoError::with_box(ErrorKind::Other,box_);

        check_formatting_equivalence(&err,&ioerr);

        assert_eq!(addr, ioerr.into_inner().unwrap().into_box().piped_ref(deref_address));
    }

    #[test]
    fn from_rboxerror(){
        let err=Stringy::new("What\nis\ra\tline");
        let rbox=err.clone().piped(RBoxError::new);
        let addr=rbox.heap_address();
        let mut ioerr=RIoError::with_rboxerror(ErrorKind::Other,rbox);

        check_formatting_equivalence(&err,&ioerr);

        assert_eq!(addr, ioerr.get_ref().unwrap().heap_address());

        assert_eq!(addr, ioerr.get_mut().unwrap().heap_address());

        assert_eq!(addr, ioerr.into_inner().unwrap().heap_address());
    }


}