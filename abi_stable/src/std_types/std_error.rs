/*! 
Ffi-safe version of 
`Box<::std::error::Error+Sync+Send+'static>` and `Box<::std::error::Error+'static>`
*/
use std::{
    error::Error as ErrorTrait,
    fmt::{self, Debug, Display},
    marker::PhantomData,
    mem,
};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use crate::{
    erased_types::{
        c_functions::{adapt_std_fmt, debug_impl, display_impl},
        FormattingMode,
    },
    marker_type::{SyncSend, UnsyncUnsend},
    ErasedObject, OpaqueType, 
    std_types::{RBox, RResult, RString},
};


/// Ffi-safe version of `Box<::std::error::Error>` 
/// whose `Send+Sync`ness is determined by the `M` type parameter.
///
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
#[sabi(phantom(M))]
pub struct RBoxError_<M = SyncSend> {
    value: RBox<ErasedObject>,
    vtable: RErrorVTable<ErasedObject>,
    _marker: PhantomData<OpaqueType<M>>,
}

/// Ffi safe equivalent to Box<::std::error::Error>.
pub type UnsyncRBoxError = RBoxError_<UnsyncUnsend>;

/// Ffi safe equivalent to Box<::std::error::Error+Send+Sync>.
pub type RBoxError = RBoxError_<SyncSend>;

unsafe impl Send for RBoxError_<SyncSend> {}
unsafe impl Sync for RBoxError_<SyncSend> {}

impl<M> RBoxError_<M> {
    /// Constructs an error containing the Debug and Display messages without
    /// storing the error value.
    pub fn from_fmt<T>(value: T) -> Self
    where
        T: Display + Debug,
    {
        DebugDisplay {
            debug: format!("{:#?}", value),
            display: format!("{:#}", value),
        }
        .piped(Self::new_inner)
    }

    fn new_inner<T>(value: T) -> Self
    where
        T: ErrorTrait + 'static,
    {
        unsafe {
            let vtable = RErrorVTable::new::<T>();
            let value = value
                .piped(RBox::new)
                .piped(|x| mem::transmute::<RBox<T>, RBox<ErasedObject>>(x));

            Self {
                value,
                vtable,
                _marker: PhantomData,
            }
        }
    }
}

impl RBoxError_<SyncSend> {
    pub fn new<T>(value: T) -> Self
    where
        T: ErrorTrait + Send + Sync + 'static,
    {
        Self::new_inner(value)
    }
}

impl RBoxError_<UnsyncUnsend> {
    pub fn new<T>(value: T) -> Self
    where
        T: ErrorTrait + 'static,
    {
        Self::new_inner(value)
    }
}

impl<M> ErrorTrait for RBoxError_<M> {}

impl<M> Display for RBoxError_<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        adapt_std_fmt(&*self.value, self.vtable.display, f)
    }
}

impl<M> Debug for RBoxError_<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        adapt_std_fmt(&*self.value, self.vtable.debug, f)
    }
}

////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
struct RErrorVTable<T> {
    debug: extern "C" fn(&T, FormattingMode, &mut RString) -> RResult<(), ()>,
    display: extern "C" fn(&T, FormattingMode, &mut RString) -> RResult<(), ()>,
}

impl RErrorVTable<ErasedObject> {
    unsafe fn new<T>() -> Self
    where
        T: ErrorTrait + 'static,
    {
        let this = RErrorVTable {
            debug: debug_impl::<T>,
            display: display_impl::<T>,
        };
        mem::transmute::<RErrorVTable<T>, RErrorVTable<ErasedObject>>(this)
    }
}

////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Clone)]
struct DebugDisplay {
    debug: String,
    display: String,
}

impl Display for DebugDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.display, f)
    }
}

impl Debug for DebugDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.debug, f)
    }
}

impl ErrorTrait for DebugDisplay {}

////////////////////////////////////////////////////////////////////////
