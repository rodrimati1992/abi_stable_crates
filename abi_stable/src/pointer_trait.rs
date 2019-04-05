use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::{cabi_type::CAbi, OpaqueType};

#[allow(unused_imports)]
use core_extensions::{prelude::*, utils::transmute_ignore_size};

///
/// Determines whether the referent of a pointer is dropped when the
/// pointer deallocates the memory.
///
/// On Yes, the referent of the pointer is dropped.
///
/// On No,the memory the pointer owns is deallocated without calling the destructor
/// of the referent.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub enum CallReferentDrop {
    Yes,
    No,
}

pub type DestructorType<T> = unsafe extern "C" fn(*mut T, CallReferentDrop);

/**
Trait for pointers that:

- Point to a single location in memory,even after being moved.

- Deref::deref always returns the same address (for the same pointer).

- If it implements DerefMut,it always returns the same memory address.

*/
pub unsafe trait StableDeref: Deref + Sized {}

pub trait StableDerefMut: StableDeref + DerefMut {}

impl<P> StableDerefMut for P where P: StableDeref + DerefMut {}

///////////

/// Erases a pointer,casting its referent to `OpaqueType<O>`.
pub trait ErasedStableDeref<O>: StableDeref + TransmuteElement<OpaqueType<O>> {
    fn erased(self, _: VariantPhantom<O>) -> Self::TransmutedPtr {
        unsafe { self.transmute_element(PhantomData) }
    }
}

impl<P, O> ErasedStableDeref<O> for P where P: StableDeref + TransmuteElement<OpaqueType<O>> {}

///////////

/// Transmutes the element type of this pointer..
///
/// # Safety for implementor
///
/// Implementors of this trait must ensure that the memory layout of this
/// type is the same regardless of the type of the referent .
///
/// `T` is intentionally `Sized` so as to prevent transmuting pointers to DST .
pub unsafe trait TransmuteElement<T>: StableDeref {
    type TransmutedPtr: StableDeref<Target = T>;

    /// Transmutes the element type of this pointer..
    ///
    /// # Safety
    ///
    /// Callers must ensure that it is valid to convert from a pointer to `Self::Referent`
    /// to a pointer to `T` .
    ///
    /// For example:
    ///     It is undefined behavior to create unaligned references ,
    ///     therefore transmuting from `&u8` to `&u16` is UB
    ///     if the caller does not ensure that the reference was a multiple of 2.
    ///
    ///
    unsafe fn transmute_element(self, _: VariantPhantom<T>) -> Self::TransmutedPtr {
        transmute_ignore_size::<Self, Self::TransmutedPtr>(self)
    }
}

///////////

unsafe impl<P> StableDeref for CAbi<P>
where
    P: StableDeref,
    P::Target: Sized,
{
}

unsafe impl<P, T> TransmuteElement<T> for CAbi<P>
where
    P: StableDeref + TransmuteElement<T>,
    P::TransmutedPtr: StableDerefMut,
    <P as Deref>::Target: Sized,
    <P::TransmutedPtr as Deref>::Target: Sized,
{
    type TransmutedPtr = CAbi<P::TransmutedPtr>;
}

///////////

unsafe impl<T> StableDeref for Box<T> {}

///////////

unsafe impl<T> StableDeref for Arc<T> {}

///////////

unsafe impl<'a, T: 'a> StableDeref for &'a T {}

unsafe impl<'a, T: 'a, O: 'a> TransmuteElement<O> for &'a T {
    type TransmutedPtr = &'a O;
}

///////////

unsafe impl<'a, T: 'a> StableDeref for &'a mut T {}

unsafe impl<'a, T: 'a, O: 'a> TransmuteElement<O> for &'a mut T {
    type TransmutedPtr = &'a mut O;
}
