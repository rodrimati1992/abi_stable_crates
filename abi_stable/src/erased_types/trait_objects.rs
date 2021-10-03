/*!
Ffi-safe trait objects for individual traits.
*/

use std::fmt::{self, Debug, Display};

#[allow(unused_imports)]
use core_extensions::SelfOps;

use super::{c_functions::*, *};

use crate::{
    marker_type::ErasedObject,
    pointer_trait::{AsPtr, TransmuteElement},
    sabi_types::{RMut, RRef},
    std_types::RBox,
};

/////////////////////////////////////////////////////////////

/// An ffi-safe equivalent of `&mut dyn std::hash::Hasher`.
#[repr(C)]
#[derive(StableAbi)]
pub struct HasherObject<'a> {
    this: RMut<'a, ErasedObject>,
    hash_slice: unsafe extern "C" fn(RMut<'_, ErasedObject>, RSlice<'_, u8>),
    finish: unsafe extern "C" fn(RRef<'_, ErasedObject>) -> u64,
}

impl<'a> HasherObject<'a> {
    /// Constructs a `HasherObject`.
    pub fn new<T: 'a>(this: &'a mut T) -> HasherObject<'a>
    where
        T: Hasher,
    {
        HasherObject {
            this: unsafe {
                // The lifetime is tied to the input.
                this.transmute_element::<ErasedObject>()
            },
            hash_slice: hash_slice_Hasher::<T>,
            finish: finish_Hasher::<T>,
        }
    }

    /// Reborrows this `HasherObject` with a smaller lifetime.
    pub fn as_mut<'b: 'a>(&'b mut self) -> HasherObject<'b> {
        Self {
            this: self.this.reborrow(),
            hash_slice: self.hash_slice,
            finish: self.finish,
        }
    }
}

impl<'a> Hasher for HasherObject<'a> {
    fn finish(&self) -> u64 {
        unsafe { (self.finish)(self.this.as_rref()) }
    }
    fn write(&mut self, bytes: &[u8]) {
        unsafe { (self.hash_slice)(self.this.reborrow(), bytes.into()) }
    }
}

//////////////

/// An ffi-safe equivalent of `Box<dyn Debug + Display>`
/// (if `dyn Debug + Display` was possible).
#[repr(C)]
#[derive(StableAbi)]
pub struct DebugDisplayObject {
    this: RBox<ErasedObject>,
    display: unsafe extern "C" fn(
        RRef<'_, ErasedObject>,
        FormattingMode,
        &mut RString,
    ) -> RResult<(), ()>,
    debug: unsafe extern "C" fn(
        RRef<'_, ErasedObject>,
        FormattingMode,
        &mut RString,
    ) -> RResult<(), ()>,
}

impl DebugDisplayObject {
    /// Constructs this `DebugDisplayObject`.
    pub fn new<T>(value: T) -> DebugDisplayObject
    where
        T: Display + Debug + 'static,
    {
        DebugDisplayObject {
            this: unsafe {
                // The lifetime here is 'static,so it's fine to erase the type.
                RBox::new(value).transmute_element::<ErasedObject>()
            },
            display: display_impl::<T>,
            debug: debug_impl::<T>,
        }
    }

    /// Constructs a `DebugDisplayObject`.which doesn't output anything.
    pub fn no_output() -> DebugDisplayObject {
        Self::new(NoFmt)
    }
}

impl Display for DebugDisplayObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe { adapt_std_fmt::<ErasedObject>(self.this.as_rref(), self.display, f) }
    }
}

impl Debug for DebugDisplayObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe { adapt_std_fmt::<ErasedObject>(self.this.as_rref(), self.debug, f) }
    }
}

struct NoFmt;

impl Display for NoFmt {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Debug for NoFmt {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

//////////////
