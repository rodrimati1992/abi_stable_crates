//! Ffi-safe trait objects for individual traits.

use std::{
    fmt::{self, Debug, Display},
    marker::PhantomData,
};

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
    write_fns: &'static WriteFns,
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
            write_fns: MakeWriteFns::<T>::V,
            finish: finish_Hasher::<T>,
        }
    }

    /// Reborrows this `HasherObject` with a smaller lifetime.
    pub fn as_mut<'b: 'a>(&'b mut self) -> HasherObject<'b> {
        Self {
            this: self.this.reborrow(),
            write_fns: self.write_fns,
            finish: self.finish,
        }
    }
}

macro_rules! impl_write {
    ( $(($ty:ty, $fn:ident)),* ) => {
        $(
            fn $fn(&mut self, val: $ty) {
                unsafe { (self.write_fns.$fn)(self.this.reborrow(), val) }
            }
        )*
    }
}

impl<'a> Hasher for HasherObject<'a> {
    fn finish(&self) -> u64 {
        unsafe { (self.finish)(self.this.as_rref()) }
    }
    fn write(&mut self, bytes: &[u8]) {
        unsafe { (self.write_fns.write)(self.this.reborrow(), bytes.into()) }
    }

    impl_write!(
        (i16, write_i16),
        (i32, write_i32),
        (i64, write_i64),
        (i8, write_i8),
        (isize, write_isize),
        (u16, write_u16),
        (u32, write_u32),
        (u64, write_u64),
        (u8, write_u8),
        (usize, write_usize)
    );
}

/// The write variations for the hasher. Even if `write` is the only required
/// function in the trait, the rest must also be explicitly implemented in the
/// hasher object so that the original behaviour is maintained (since the
/// default impls may have been overridden).
///
/// They are declared and constructed separately from the object for
/// cleanliness.
#[repr(C)]
#[derive(StableAbi)]
struct WriteFns {
    write: unsafe extern "C" fn(RMut<'_, ErasedObject>, RSlice<'_, u8>),
    // No c-compatible layout for i128 yet
    // write_i128: unsafe extern "C" fn(RMut<'_, ErasedObject>, i128),
    write_i16: unsafe extern "C" fn(RMut<'_, ErasedObject>, i16),
    write_i32: unsafe extern "C" fn(RMut<'_, ErasedObject>, i32),
    write_i64: unsafe extern "C" fn(RMut<'_, ErasedObject>, i64),
    write_i8: unsafe extern "C" fn(RMut<'_, ErasedObject>, i8),
    write_isize: unsafe extern "C" fn(RMut<'_, ErasedObject>, isize),
    // No c-compatible layout for u128 yet
    // write_u128: unsafe extern "C" fn(RMut<'_, ErasedObject>, u128),
    write_u16: unsafe extern "C" fn(RMut<'_, ErasedObject>, u16),
    write_u32: unsafe extern "C" fn(RMut<'_, ErasedObject>, u32),
    write_u64: unsafe extern "C" fn(RMut<'_, ErasedObject>, u64),
    write_u8: unsafe extern "C" fn(RMut<'_, ErasedObject>, u8),
    write_usize: unsafe extern "C" fn(RMut<'_, ErasedObject>, usize),
}

struct MakeWriteFns<T>(PhantomData<T>);

impl<T: Hasher> MakeWriteFns<T> {
    const V: &'static WriteFns = &WriteFns {
        write: write_Hasher::<T>,
        write_i16: write_i16_Hasher::<T>,
        write_i32: write_i32_Hasher::<T>,
        write_i64: write_i64_Hasher::<T>,
        write_i8: write_i8_Hasher::<T>,
        write_isize: write_isize_Hasher::<T>,
        write_u16: write_u16_Hasher::<T>,
        write_u32: write_u32_Hasher::<T>,
        write_u64: write_u64_Hasher::<T>,
        write_u8: write_u8_Hasher::<T>,
        write_usize: write_usize_Hasher::<T>,
    };
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
