/**
Ffi-safe versions of some `std::io` types.
*/

use std::{
    error::Error as ErrorTrait,
    fmt::{self, Debug, Display},
    io::{Error as ioError, ErrorKind},
};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use crate::{traits::{IntoReprC}, std_types::{RBoxError}};

/// Ffi safe equivalent to ::std::io::ErrorKind.
///
/// Using a struct with associated constants is the 
/// ffi-safe way of doing `#[non_exhaustive]` field-less enums.
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct RIoErrorKind {
    value: u8,
}

#[allow(non_upper_case_globals)]
impl RIoErrorKind {
    pub const Other: Self = RIoErrorKind { value: 0 };
    pub const NotFound: Self = RIoErrorKind { value: 1 };
    pub const PermissionDenied: Self = RIoErrorKind { value: 2 };
    pub const ConnectionRefused: Self = RIoErrorKind { value: 3 };
    pub const ConnectionReset: Self = RIoErrorKind { value: 4 };
    pub const ConnectionAborted: Self = RIoErrorKind { value: 5 };
    pub const NotConnected: Self = RIoErrorKind { value: 6 };
    pub const AddrInUse: Self = RIoErrorKind { value: 7 };
    pub const AddrNotAvailable: Self = RIoErrorKind { value: 8 };
    pub const BrokenPipe: Self = RIoErrorKind { value: 9 };
    pub const AlreadyExists: Self = RIoErrorKind { value: 10 };
    pub const WouldBlock: Self = RIoErrorKind { value: 11 };
    pub const InvalidInput: Self = RIoErrorKind { value: 12 };
    pub const InvalidData: Self = RIoErrorKind { value: 13 };
    pub const TimedOut: Self = RIoErrorKind { value: 14 };
    pub const WriteZero: Self = RIoErrorKind { value: 15 };
    pub const Interrupted: Self = RIoErrorKind { value: 16 };
    pub const UnexpectedEof: Self = RIoErrorKind { value: 17 };
}

impl Debug for RIoErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            RIoErrorKind::NotFound => "NotFound",
            RIoErrorKind::PermissionDenied => "PermissionDenied",
            RIoErrorKind::ConnectionRefused => "ConnectionRefused",
            RIoErrorKind::ConnectionReset => "ConnectionReset",
            RIoErrorKind::ConnectionAborted => "ConnectionAborted",
            RIoErrorKind::NotConnected => "NotConnected",
            RIoErrorKind::AddrInUse => "AddrInUse",
            RIoErrorKind::AddrNotAvailable => "AddrNotAvailable",
            RIoErrorKind::BrokenPipe => "BrokenPipe",
            RIoErrorKind::AlreadyExists => "AlreadyExists",
            RIoErrorKind::WouldBlock => "WouldBlock",
            RIoErrorKind::InvalidInput => "InvalidInput",
            RIoErrorKind::InvalidData => "InvalidData",
            RIoErrorKind::TimedOut => "TimedOut",
            RIoErrorKind::WriteZero => "WriteZero",
            RIoErrorKind::Interrupted => "Interrupted",
            RIoErrorKind::UnexpectedEof => "UnexpectedEof",
            _ => "Other",
        };
        Display::fmt(s, f)
    }
}

impl_from_rust_repr! {
    impl From<ErrorKind> for RIoErrorKind {
        fn(this){
            match this {
                ErrorKind::NotFound=>RIoErrorKind::NotFound,
                ErrorKind::PermissionDenied=>RIoErrorKind::PermissionDenied,
                ErrorKind::ConnectionRefused=>RIoErrorKind::ConnectionRefused,
                ErrorKind::ConnectionReset=>RIoErrorKind::ConnectionReset,
                ErrorKind::ConnectionAborted=>RIoErrorKind::ConnectionAborted,
                ErrorKind::NotConnected=>RIoErrorKind::NotConnected,
                ErrorKind::AddrInUse=>RIoErrorKind::AddrInUse,
                ErrorKind::AddrNotAvailable=>RIoErrorKind::AddrNotAvailable,
                ErrorKind::BrokenPipe=>RIoErrorKind::BrokenPipe,
                ErrorKind::AlreadyExists=>RIoErrorKind::AlreadyExists,
                ErrorKind::WouldBlock=>RIoErrorKind::WouldBlock,
                ErrorKind::InvalidInput=>RIoErrorKind::InvalidInput,
                ErrorKind::InvalidData=>RIoErrorKind::InvalidData,
                ErrorKind::TimedOut=>RIoErrorKind::TimedOut,
                ErrorKind::WriteZero=>RIoErrorKind::WriteZero,
                ErrorKind::Interrupted=>RIoErrorKind::Interrupted,
                ErrorKind::UnexpectedEof=>RIoErrorKind::UnexpectedEof,
                _=>RIoErrorKind::Other,
            }
        }
    }
}

impl_into_rust_repr! {
    impl Into<ErrorKind> for RIoErrorKind {
        fn(this){
            match this {
                RIoErrorKind::NotFound=>ErrorKind::NotFound,
                RIoErrorKind::PermissionDenied=>ErrorKind::PermissionDenied,
                RIoErrorKind::ConnectionRefused=>ErrorKind::ConnectionRefused,
                RIoErrorKind::ConnectionReset=>ErrorKind::ConnectionReset,
                RIoErrorKind::ConnectionAborted=>ErrorKind::ConnectionAborted,
                RIoErrorKind::NotConnected=>ErrorKind::NotConnected,
                RIoErrorKind::AddrInUse=>ErrorKind::AddrInUse,
                RIoErrorKind::AddrNotAvailable=>ErrorKind::AddrNotAvailable,
                RIoErrorKind::BrokenPipe=>ErrorKind::BrokenPipe,
                RIoErrorKind::AlreadyExists=>ErrorKind::AlreadyExists,
                RIoErrorKind::WouldBlock=>ErrorKind::WouldBlock,
                RIoErrorKind::InvalidInput=>ErrorKind::InvalidInput,
                RIoErrorKind::InvalidData=>ErrorKind::InvalidData,
                RIoErrorKind::TimedOut=>ErrorKind::TimedOut,
                RIoErrorKind::WriteZero=>ErrorKind::WriteZero,
                RIoErrorKind::Interrupted=>ErrorKind::Interrupted,
                RIoErrorKind::UnexpectedEof=>ErrorKind::UnexpectedEof,
                _=>ErrorKind::Other,
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////

/// Ffi safe equivalent to ::std::io::Error.
#[repr(C)]
#[derive(Debug, StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct RIoError {
    kind: RIoErrorKind,
    error: RBoxError,
}

impl_from_rust_repr! {
    impl From<ioError> for RIoError {
        fn(this){
            RIoError{
                kind:this.kind().into(),
                error:this.piped(RBoxError::new)
            }
        }
    }
}

impl RIoError {
    pub fn new<E>(kind: ErrorKind, error: E) -> Self
    where
        E: ErrorTrait + Send + Sync + 'static,
    {
        RIoError {
            kind: kind.into_c(),
            error: error.piped(RBoxError::new),
        }
    }
}

impl Display for RIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl ErrorTrait for RIoError {}
