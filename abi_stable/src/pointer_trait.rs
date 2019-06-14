/*!
Traits for pointers.
*/
use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::sabi_types::MovePtr;

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
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
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

- If it implements DerefMut,it always returns the same memory address,
    so long as no `&mut self``method other than `DerefMut::deref_mut` is called.

- The inline layout of the pointer cannot change depending on its referent.


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
What kind of pointer this is.

The valid kinds are:

- Reference:a `&T`,or a `Copy` wrapper struct containing a `&T`

- MutReference:a `&mut T`,or a non-`Drop` wrapper struct containing a `&mut T`

- SmartPointer: Any pointer type that's not a reference or a mutable reference.

*/
pub unsafe trait GetPointerKind{
    type Kind:PointerKindVariant;

    const KIND:PointerKind=<Self::Kind as PointerKindVariant>::VALUE;
}

/// A type-level equivalent of a PointerKind variant.
pub trait PointerKindVariant:Sealed{
    /// The value of the PointerKind variant Self is equivalent to.
    const VALUE:PointerKind;
}

use self::sealed::Sealed;
mod sealed{
    pub trait Sealed{}
}


/// Describes the kind of a pointer.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash,StableAbi)]
#[repr(u8)]
pub enum PointerKind{
    /// a `&T`,or a `Copy` wrapper struct containing a `&T`
    Reference,
    /// a `&mut T`,or a non-`Drop` wrapper struct containing a `&mut T`
    MutReference,
    /// Any pointer type that's not a reference or a mutable reference.
    SmartPointer
}

/// The type-level equivalent of `PointerKind::Reference`.
#[allow(non_camel_case_types)]
pub struct PK_Reference;

/// The type-level equivalent of `PointerKind::MutReference`.
#[allow(non_camel_case_types)]
pub struct PK_MutReference;

/// The type-level equivalent of `PointerKind::SmartPointer`.
#[allow(non_camel_case_types)]
pub struct PK_SmartPointer;

impl Sealed for PK_Reference{}
impl Sealed for PK_MutReference{}
impl Sealed for PK_SmartPointer{}

impl PointerKindVariant for PK_Reference{
    const VALUE:PointerKind=PointerKind::Reference;
}

impl PointerKindVariant for PK_MutReference{
    const VALUE:PointerKind=PointerKind::MutReference;
}

impl PointerKindVariant for PK_SmartPointer{
    const VALUE:PointerKind=PointerKind::SmartPointer;
}

unsafe impl<'a,T> GetPointerKind for &'a T{
    type Kind=PK_Reference;
}

unsafe impl<'a,T> GetPointerKind for &'a mut T{
    type Kind=PK_MutReference;
}



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
pub unsafe trait TransmuteElement<T>: StableDeref + GetPointerKind {
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


///////////////////////////////////////////////////////////////////////////////


/**
For owned pointers,allows extracting their contents separate from deallocating them.

# Safety for implementor

- The pointer type is either `!Drop`(no drop glue either),
    or it uses a vtable to Drop the referent and deallocate the memory correctly.
*/
pub unsafe trait OwnedPointer:Sized{
    /// The type of the value this owns.
    type Target;

    /// Gets a move pointer to the contents of this pointer.
    ///
    /// # Safety
    ///
    /// This function moves the owned contents out of this pointer,
    /// the only safe thing that can be done with the pointer afterwads 
    /// is to call OwnedPointer::drop_allocation.
    unsafe fn get_move_ptr(&mut self)->MovePtr<'_,Self::Target>;

    /// Deallocates the pointer without dropping its owned contents.
    ///
    /// Note that if `Self::get_move_ptr` has not been called this will 
    /// leak the values owned by the referent of the pointer. 
    ///
    fn drop_allocation(this:ManuallyDrop<Self>);

    #[inline]
    fn with_moved_ptr<F,R>(mut this:ManuallyDrop<Self>,f:F)->R
    where 
        F:FnOnce(MovePtr<'_,Self::Target>)->R
    {
        unsafe{
            let ret=f(Self::get_move_ptr(&mut this));
            Self::drop_allocation(this);
            ret
        }
    }
}