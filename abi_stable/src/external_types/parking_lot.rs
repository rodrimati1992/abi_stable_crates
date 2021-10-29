//! Ffi-safe synchronization primitives,most of which are ffi-safe wrappers of
//! [parking_lot](https://crates.io/crates/parking_lot) types

pub mod mutex;
pub mod once;
pub mod rw_lock;

pub use self::{mutex::RMutex, once::ROnce, rw_lock::RRwLock};

/////////////////////////////////////////////////////////////////////////////////

use std::mem;

use crate::StableAbi;

#[cfg_attr(target_pointer_width = "128", repr(C, align(16)))]
#[cfg_attr(target_pointer_width = "64", repr(C, align(8)))]
#[cfg_attr(target_pointer_width = "32", repr(C, align(4)))]
#[cfg_attr(target_pointer_width = "16", repr(C, align(2)))]
#[derive(Copy, Clone, StableAbi)]
struct Overaligner;

const RAW_LOCK_SIZE: usize = mem::size_of::<usize>();

#[repr(C)]
#[derive(Copy, Clone, StableAbi)]
#[sabi(unsafe_unconstrained(T))]
struct UnsafeOveralignedField<T, P> {
    #[sabi(unsafe_opaque_field)]
    value: T,
    /// Manual padding to ensure that the bytes are copied,
    /// even if Rust thinks there is nothing in the padding.
    _padding: P,
    _alignment: Overaligner,
}

impl<T, P> UnsafeOveralignedField<T, P> {
    const fn new(value: T, _padding: P) -> Self {
        Self {
            value,
            _padding,
            _alignment: Overaligner,
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////
