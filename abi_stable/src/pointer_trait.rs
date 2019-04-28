/*!
Traits for pointers.
*/
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};

// use crate::{cabi_type::CAbi};

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


/// The type of the destructor for every pointer type from in this crate.
pub type DestructorType<T> = unsafe extern "C" fn(data:*mut T, CallReferentDrop);

/**
Trait for pointers that:

- Point to a single location in memory,even after being moved.

- Deref::deref always returns the same address (for the same pointer).

- If it implements DerefMut,it always returns the same memory address.

- The inline layout of the pointer cannot change depending on its (Sized) referent.


Explicit non-guarantees:

- If the pointer is converted by value to another pointer type
    (ie:going from RBox<T> to Box<T>,RArc<T> to Arc<T>),
    the address cannot be relied on being the same,
    even if it implements StableDeref.


*/
pub unsafe trait StableDeref: Deref + Sized {}

/// An alias for `StableDeref + DerefMut`.
pub trait StableDerefMut: StableDeref + DerefMut {}

impl<P> StableDerefMut for P where P: StableDeref + DerefMut {}

///////////

/**
Transmutes the element type of this pointer..

# Safety for implementor

Implementors of this trait must ensure that:

- The memory layout of this
    type is the same regardless of the type of the referent .

- The pointer type is either `!Drop`(no drop glue either),
    or it uses a vtable to Drop the referent and deallocate the memory correctly.

# Safety for callers

Callers must ensure that:

- References to `T` are compatible with references to `Self::Target`.

*/
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
    ///
    /// It is undefined behavior to create unaligned references ,
    /// therefore transmuting from `&u8` to `&u16` is UB
    /// if the caller does not ensure that the reference was a multiple of 2.
    ///
    /// 
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::TransmuteElement,
    ///     reexports::SelfOps,
    ///     std_types::RBox,
    /// };
    ///
    /// let signed:RBox<u32>=unsafe{
    ///     RBox::new(1_i32)
    ///         .transmute_element(u32::T)
    /// };
    ///
    /// ```
    unsafe fn transmute_element(self, _: VariantPhantom<T>) -> Self::TransmutedPtr 
    where Self::Target:Sized
    {
        transmute_ignore_size::<Self, Self::TransmutedPtr>(self)
    }
}

///////////

// unsafe impl<P> StableDeref for CAbi<P>
// where
//     P: StableDeref,
//     P::Target: Sized,
// {
// }

// unsafe impl<P, T> TransmuteElement<T> for CAbi<P>
// where
//     P: StableDeref + TransmuteElement<T>,
//     P::TransmutedPtr: StableDerefMut,
//     <P as Deref>::Target: Sized,
//     <P::TransmutedPtr as Deref>::Target: Sized,
// {
//     type TransmutedPtr = CAbi<P::TransmutedPtr>;
// }

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
