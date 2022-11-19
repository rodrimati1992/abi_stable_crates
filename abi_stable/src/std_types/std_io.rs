//! Ffi-safe equivalents of `std::io::{ErrorKind, Error, SeekFrom}`.
#![allow(clippy::missing_const_for_fn)]

use std::{
    error::Error as ErrorTrait,
    fmt::{self, Debug, Display},
    io::{Error as ioError, ErrorKind, SeekFrom},
};

#[allow(unused_imports)]
use core_extensions::SelfOps;

use crate::{
    std_types::{RBoxError, RNone, ROption, RSome},
    traits::{IntoReprC, IntoReprRust},
};

///////////////////////////////////////////////////////////////////////////

/// Ffi safe equivalent to `std::io::ErrorKind`.
///
/// Using a struct with associated constants is the
/// ffi-safe way of doing `#[non_exhaustive]` field-less enums.
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[repr(C)]
#[derive(StableAbi)]
pub struct RIoErrorKind {
    value: u8,
}

macro_rules! impl_error_kind {
    (
        $(
            $variant: ident, discriminant = $value: expr , message = $as_str_msg: expr ;
        )*
    ) => (
        /// Every (visible) variant of RIoErrorKind, equivalent to that of `std::io::ErrorKind`.
        #[allow(non_upper_case_globals)]
        impl RIoErrorKind {
            $(
                ///
                pub const $variant: Self = RIoErrorKind { value: $value };
            )*
            ///
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
                        _ => RIoErrorKind::Other,
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
                        _ => ErrorKind::Other,
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

impl_error_kind! {
    NotFound, discriminant = 1 , message = "entity not found" ;
    PermissionDenied, discriminant = 2 , message = "permission denied" ;
    ConnectionRefused, discriminant = 3 , message = "connection refused" ;
    ConnectionReset, discriminant = 4 , message = "connection reset" ;
    ConnectionAborted, discriminant = 5 , message = "connection aborted" ;
    NotConnected, discriminant = 6 , message = "not connected" ;
    AddrInUse, discriminant = 7 , message = "address in use" ;
    AddrNotAvailable, discriminant = 8 , message = "address not available" ;
    BrokenPipe, discriminant = 9 , message = "broken pipe" ;
    AlreadyExists, discriminant = 10 , message = "entity already exists" ;
    WouldBlock, discriminant = 11 , message = "operation would block" ;
    InvalidInput, discriminant = 12 , message = "invalid input parameter" ;
    InvalidData, discriminant = 13 , message = "invalid data" ;
    TimedOut, discriminant = 14 , message = "timed out" ;
    WriteZero, discriminant = 15 , message = "write zero" ;
    Interrupted, discriminant = 16 , message = "operation interrupted" ;
    UnexpectedEof, discriminant = 17 , message = "unexpected end of file" ;
}

///////////////////////////////////////////////////////////////////////////

/// Ffi-safe equivalent of [`std::io::SeekFrom`].
///
/// [`std::io::SeekFrom`]: https://doc.rust-lang.org/std/io/enum.SeekFrom.html
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[repr(u8)]
#[derive(StableAbi)]
pub enum RSeekFrom {
    ///
    Start(u64),
    ///
    End(i64),
    ///
    Current(i64),
}

impl_from_rust_repr! {
    impl From<SeekFrom> for RSeekFrom {
        fn(this){
            match this {
                SeekFrom::Start(x)   => RSeekFrom::Start(x),
                SeekFrom::End(x)     => RSeekFrom::End(x),
                SeekFrom::Current(x) => RSeekFrom::Current(x),
            }
        }
    }
}

impl_into_rust_repr! {
    impl Into<SeekFrom> for RSeekFrom {
        fn(this){
            match this {
                RSeekFrom::Start(x)   => SeekFrom::Start(x),
                RSeekFrom::End(x)     => SeekFrom::End(x),
                RSeekFrom::Current(x) => SeekFrom::Current(x),
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////

/// Ffi safe equivalent to `std::io::Error`.
///
/// # Example
///
/// Defining an extern function to write a slice into a writer twice.
///
/// ```
/// use abi_stable::{
///     erased_types::interfaces::IoWriteInterface,
///     rtry, sabi_extern_fn,
///     std_types::{RIoError, ROk, RResult},
///     traits::IntoReprC,
///     DynTrait, RMut,
/// };
///
/// use std::io::Write;
///
/// #[sabi_extern_fn]
/// pub fn write_slice_twice(
///     mut write: DynTrait<RMut<'_, ()>, IoWriteInterface>,
///     slice: &[u8],
/// ) -> RResult<(), RIoError> {
///     rtry!(write.write_all(slice).into_c());
///     rtry!(write.write_all(slice).into_c());
///     ROk(())
/// }
///
/// ```
///
#[repr(C)]
#[derive(StableAbi)]
pub struct RIoError {
    kind: RIoErrorKind,
    error: ROption<RBoxError>,
}

impl_from_rust_repr! {
    impl From<ioError> for RIoError {
        fn(this){
            RIoError{
                kind: this.kind().into(),
                error: this.into_inner().map(RBoxError::from_box).into_c()
            }
        }
    }
}

impl_into_rust_repr! {
    impl Into<ioError> for RIoError {
        fn(this){
            let kind = this.kind().into_::<ErrorKind>();
            match this.into_inner() {
                Some(e) => ioError::new(kind, RBoxError::into_box(e)),
                None => ioError::from(kind),
            }
        }
    }
}

impl From<RIoErrorKind> for RIoError {
    fn from(kind: RIoErrorKind) -> Self {
        Self { kind, error: RNone }
    }
}

impl From<ErrorKind> for RIoError {
    fn from(kind: ErrorKind) -> Self {
        Self {
            kind: kind.into(),
            error: RNone,
        }
    }
}

impl RIoError {
    /// Constructs an `RIoError` from an error and a `std::io::ErrorKind`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RIoError;
    /// use std::io::ErrorKind;
    ///
    /// let err = RIoError::new(ErrorKind::Other, "".parse::<u64>().unwrap_err());
    /// ```
    pub fn new<E>(kind: ErrorKind, error: E) -> Self
    where
        E: ErrorTrait + Send + Sync + 'static,
    {
        RIoError {
            kind: kind.into_c(),
            error: RSome(RBoxError::new(error)),
        }
    }

    /// Constructs an `RIoError` from a type convertible into a
    /// `Box<dyn std::error::Error + Send + Sync + 'static>`, and a `std::io::ErrorKind`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RIoError;
    /// use std::io::ErrorKind;
    ///
    /// let str_err = "Timeout receiving the response from server.";
    ///
    /// let err = RIoError::new_(ErrorKind::TimedOut, str_err);
    /// ```
    #[inline]
    pub fn new_<E>(kind: ErrorKind, error: E) -> Self
    where
        E: Into<Box<dyn ErrorTrait + Send + Sync + 'static>>,
    {
        Self::with_box(kind, error.into())
    }

    /// Constructs an `RIoError` from a `std::io::ErrorKind`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RIoError;
    /// use std::io::ErrorKind;
    ///
    /// let err = RIoError::from_kind(ErrorKind::AlreadyExists);
    /// ```
    pub fn from_kind(kind: ErrorKind) -> Self {
        Self {
            kind: kind.into_c(),
            error: RNone,
        }
    }

    /// Constructs an `RIoError` from a
    /// `Box<dyn std::error::Error + Send + Sync + 'static>` and a `std::io::ErrorKind`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RIoError;
    /// use std::io::ErrorKind;
    ///
    /// let str_err = "Could not create file \"memes.txt\" because it already exists.";
    ///
    /// let err = RIoError::with_box(ErrorKind::AlreadyExists, str_err.into());
    /// ```
    pub fn with_box(kind: ErrorKind, error: Box<dyn ErrorTrait + Send + Sync + 'static>) -> Self {
        RIoError {
            kind: kind.into_c(),
            error: RSome(RBoxError::from_box(error)),
        }
    }

    /// Constructs an `RIoError` from an `RBoxError` and a `std::io::ErrorKind`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RIoError;
    /// use std::io::ErrorKind;
    ///
    /// type DynErr = Box<dyn std::error::Error + Send + Sync>;
    ///
    /// let str_err: DynErr = "IP address `256.256.256.256` is already in use.".into();
    ///
    /// let err = RIoError::with_rboxerror(ErrorKind::AddrInUse, str_err.into());
    /// ```
    pub fn with_rboxerror(kind: ErrorKind, error: RBoxError) -> Self {
        RIoError {
            kind: kind.into_c(),
            error: RSome(error),
        }
    }

    /// Retrieves the kind of io error.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RIoError, RIoErrorKind};
    /// use std::io::ErrorKind;
    ///
    /// let err = RIoError::from_kind(ErrorKind::AlreadyExists);
    ///
    /// assert_eq!(err.kind(), RIoErrorKind::AlreadyExists);
    /// ```
    pub fn kind(&self) -> RIoErrorKind {
        self.kind
    }

    /// Gets the internal error,
    /// returning `None` if this was constructed with `RIoError::from_kind`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RBoxError, RIoError, RIoErrorKind};
    /// use std::io::ErrorKind;
    ///
    /// {
    ///     let err = RIoError::from_kind(ErrorKind::AlreadyExists);
    ///     assert_eq!(err.get_ref().map(|_| ()), None);
    /// }
    /// {
    ///     let msg = "Cannot access directory at \"/home/Steve/memes/\".";
    ///     let err = RIoError::new_(ErrorKind::PermissionDenied, msg);
    ///
    ///     assert!(err.get_ref().is_some());
    /// }
    ///
    /// ```
    pub fn get_ref(&self) -> Option<&RBoxError> {
        self.error.as_ref().into_rust()
    }

    /// Gets the internal error,
    /// returning `None` if this was constructed with `RIoError::from_kind`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RBoxError, RIoError, RIoErrorKind};
    /// use std::io::ErrorKind;
    ///
    /// {
    ///     let mut err = RIoError::from_kind(ErrorKind::AlreadyExists);
    ///     assert_eq!(err.get_mut().map(|_| ()), None);
    /// }
    /// {
    ///     let mut msg = "Cannot access directory at \"/home/Patrick/373.15K takes/\".";
    ///     let mut err = RIoError::new_(ErrorKind::PermissionDenied, msg);
    ///     assert!(err.get_mut().is_some());
    /// }
    ///
    /// ```
    pub fn get_mut(&mut self) -> Option<&mut RBoxError> {
        self.error.as_mut().into_rust()
    }

    /// Converts this into the internal error,
    /// returning `None` if this was constructed with `RIoError::from_kind`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RIoError;
    /// use std::io::ErrorKind;
    ///
    /// {
    ///     let err = RIoError::from_kind(ErrorKind::AlreadyExists);
    ///     assert_eq!(err.into_inner().map(|_| ()), None);
    /// }
    /// {
    ///     let mut msg = "Cannot access directory at \"/home/wo_boat/blog/\".";
    ///     let err = RIoError::new_(ErrorKind::PermissionDenied, msg);
    ///     assert!(err.into_inner().is_some());
    /// }
    ///
    /// ```
    pub fn into_inner(self) -> Option<RBoxError> {
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
            RNone => Display::fmt(self.kind.error_message(), f),
        }
    }
}

impl ErrorTrait for RIoError {}

///////////////////////////////////////////////////////////////////////////////////

#[cfg(all(test, not(feature = "only_new_tests")))]
mod error_kind_tests {
    use super::*;

    #[test]
    fn conversions() {
        for (from, to) in [
            (ErrorKind::NotConnected, RIoErrorKind::NotConnected),
            (ErrorKind::AddrInUse, RIoErrorKind::AddrInUse),
            (ErrorKind::Other, RIoErrorKind::Other),
        ] {
            assert_eq!(RIoErrorKind::from(from), to);
            assert_eq!(to.into_::<ErrorKind>(), from);
        }
    }
}

#[cfg(all(test, not(feature = "only_new_tests")))]
mod io_error_tests {
    use super::*;

    use crate::test_utils::{check_formatting_equivalence, deref_address, Stringy};

    #[test]
    fn from_error_kind() {
        for kind in [
            ErrorKind::NotConnected,
            ErrorKind::AddrInUse,
            ErrorKind::Other,
        ] {
            let err = kind.piped(RIoError::from_kind);

            assert_eq!(err.kind(), kind.into_c());
        }
    }

    #[test]
    fn from_value() {
        let err = Stringy::new("What\nis\ra\tline");
        let e0 = RIoError::new(ErrorKind::Other, err.clone());

        check_formatting_equivalence(&err, &e0);
    }

    #[test]
    fn from_boxerror() {
        let err = Stringy::new("What\nis\ra\tline");
        let box_ = err.clone().piped(Box::new);
        let addr = deref_address(&box_);
        let ioerr = RIoError::with_box(ErrorKind::Other, box_);

        check_formatting_equivalence(&err, &ioerr);

        assert_eq!(
            addr,
            ioerr
                .into_inner()
                .unwrap()
                .into_box()
                .piped_ref(deref_address)
        );
    }

    #[test]
    fn from_rboxerror() {
        let err = Stringy::new("What\nis\ra\tline");
        let rbox = err.clone().piped(RBoxError::new);
        let addr = rbox.heap_address();
        let mut ioerr = RIoError::with_rboxerror(ErrorKind::Other, rbox);

        check_formatting_equivalence(&err, &ioerr);

        assert_eq!(addr, ioerr.get_ref().unwrap().heap_address());

        assert_eq!(addr, ioerr.get_mut().unwrap().heap_address());

        assert_eq!(addr, ioerr.into_inner().unwrap().heap_address());
    }
}
