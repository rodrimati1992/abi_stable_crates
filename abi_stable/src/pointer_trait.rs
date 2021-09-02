/*!
Traits for pointers.
*/
use std::{
    mem::ManuallyDrop,
    ptr::NonNull,
};

use crate::{
    marker_type::NonOwningPhantom,
    sabi_types::{MovePtr, RRef, RMut},
    utils::Transmuter,
};

#[allow(unused_imports)]
use core_extensions::utils::transmute_ignore_size;

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


/// Determines whether the pointer is deallocated.
#[repr(u8)]
#[derive(Debug,Clone,Copy,PartialEq,Eq,StableAbi)]
pub enum Deallocate{
    No,
    Yes,
}


///////////


/// What kind of pointer this is.
/// 
/// The valid kinds are:
/// 
/// - Reference:a `&T`,or a `Copy` wrapper struct containing a `&T`
/// 
/// - MutReference:a `&mut T`,or a non-`Drop` wrapper struct containing a `&mut T`
/// 
/// - SmartPointer: Any pointer type that's not a reference or a mutable reference.
/// 
/// 
pub unsafe trait GetPointerKind: Sized {
    /// The kind of the pointer.
    type Kind: PointerKindVariant;

    /// What this pointer points to,
    /// if the type implements `std::ops::Deref` it must be the same as
    /// `<Self as Deref>::Target`.
    /// 
    /// This is here so that pointers don't *have to* implement `Deref`.
    type PtrTarget;

    /// The kind of the pointer.
    const KIND: PointerKind = <Self::Kind as PointerKindVariant>::VALUE;
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
    type PtrTarget = T;
}

unsafe impl<'a,T> GetPointerKind for &'a mut T{
    type Kind=PK_MutReference;
    type PtrTarget = T;
}



///////////

/**
Whether the pointer can be transmuted to an equivalent pointer with `T` as the referent type.

# Safety for implementor

Implementors of this trait must ensure that:

- The memory layout of this
    type is the same regardless of the type of the referent .

- The pointer type is either `!Drop`(no drop glue either),
    or it uses a vtable to Drop the referent and deallocate the memory correctly.

*/
pub unsafe trait CanTransmuteElement<T>: GetPointerKind {
    /// The type of the pointer after it's element type has been changed.
    type TransmutedPtr: AsPtr<PtrTarget = T>;

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
    /// if the caller does not ensure that the reference is aligned to a multiple of 2 address.
    ///
    /// 
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::TransmuteElement,
    ///     std_types::RBox,
    /// };
    ///
    /// let signed:RBox<u32>=unsafe{
    ///     RBox::new(1_i32)
    ///         .transmute_element::<u32>()
    /// };
    ///
    /// ```
    unsafe fn transmute_element_(self) -> Self::TransmutedPtr;
}

/// Allows transmuting pointers to point to a different type.
pub trait TransmuteElement{
    /// Transmutes the element type of this pointer..
    ///
    /// # Safety
    ///
    /// Callers must ensure that it is valid to convert from a pointer to `Self::Referent`
    /// to a pointer to `T`, and then use the pointed-to data.
    ///
    /// For example:
    ///
    /// It is undefined behavior to create unaligned references ,
    /// therefore transmuting from `&u8` to `&u16` is UB
    /// if the caller does not ensure that the reference is aligned to a multiple of 2 address.
    ///
    /// 
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::TransmuteElement,
    ///     std_types::RBox,
    /// };
    ///
    /// let signed:RBox<u32>=unsafe{
    ///     RBox::new(1_i32)
    ///         .transmute_element::<u32>()
    /// };
    ///
    /// ```
    unsafe fn transmute_element<T>(self) -> <Self as CanTransmuteElement<T>>::TransmutedPtr 
    where
        Self: CanTransmuteElement<T>,
    {
        self.transmute_element_()
    }
}

impl<This:?Sized> TransmuteElement for This{}


///////////

unsafe impl<'a, T: 'a, O: 'a> CanTransmuteElement<O> for &'a T {
    type TransmutedPtr = RRef<'a, O>;

    unsafe fn transmute_element_(self) -> Self::TransmutedPtr {
        RRef::from_raw(self as *const T as *const O)
    }
}

///////////

unsafe impl<'a, T: 'a, O: 'a> CanTransmuteElement<O> for &'a mut T {
    type TransmutedPtr = RMut<'a, O>;


    unsafe fn transmute_element_(self) -> Self::TransmutedPtr {
        RMut::from_raw(self as *mut T as *mut O)
    }
}


///////////////////////////////////////////////////////////////////////////////


/// For getting a const raw pointer to the value that this points to.
/// 
/// # Safety
/// 
/// The implementor of this trait must return a pointer to the same data as
/// `Deref::deref`, without constructing a `&Self::Target` in `as_ptr`
/// (or any function it calls),
/// 
/// The implementor of this trait must not override the defaulted methods.
/// 
pub unsafe trait AsPtr: GetPointerKind {
    /// Gets a const raw pointer to the value that this points to.
    fn as_ptr(&self) -> *const Self::PtrTarget;

    /// Converts this pointer to an `RRef`.
    #[inline(always)]
    fn as_rref(&self) -> RRef<'_, Self::PtrTarget> {
        unsafe{ RRef::from_raw(self.as_ptr()) }
    }
}

/// For getting a mutable raw pointer to the value that this points to.
/// 
/// # Safety
/// 
/// The implementor of this trait must return a pointer to the same data as
/// `DerefMut::deref_mut`,
/// without constructing a `&mut Self::Target` in `as_mut_ptr`
/// (or any function it calls).
/// 
/// The implementor of this trait must not override the defaulted methods.
/// 
pub unsafe trait AsMutPtr: AsPtr {
    /// Gets a mutable raw pointer to the value that this points to.
    fn as_mut_ptr(&mut self) -> *mut Self::PtrTarget;

    /// Converts this pointer to an `RRef`.
    #[inline(always)]
    fn as_rmut(&mut self) -> RMut<'_, Self::PtrTarget> {
        unsafe{ RMut::from_raw(self.as_mut_ptr()) }
    }
}

///////////////////////////////////////////////////////////////////////////////


/**
For owned pointers,allows extracting their contents separate from deallocating them.

# Safety

Implementors must:

- Be implemented such that `get_move_ptr` can be called before `drop_allocation`.

- Not override `with_move_ptr`

- Not override `in_move_ptr`

*/
pub unsafe trait OwnedPointer: Sized + AsMutPtr + GetPointerKind {
    /// Gets a move pointer to the contents of this pointer.
    ///
    /// # Safety
    ///
    /// This function logically moves the owned contents out of this pointer,
    /// the only safe thing that can be done with the pointer afterwads 
    /// is to call OwnedPointer::drop_allocation.
    unsafe fn get_move_ptr(this:&mut ManuallyDrop<Self>)->MovePtr<'_,Self::PtrTarget>;

    /// Deallocates the pointer without dropping its owned contents.
    ///
    /// Note that if `Self::get_move_ptr` has not been called this will 
    /// leak the values owned by the referent of the pointer. 
    ///
    /// # Safety
    ///
    /// The allocation managed by `this` must never be accessed again.
    ///
    unsafe fn drop_allocation(this:&mut ManuallyDrop<Self>);

    #[inline]
    fn with_move_ptr<F,R>(mut this:ManuallyDrop<Self>,f:F)->R
    where 
        F:FnOnce(MovePtr<'_,Self::PtrTarget>)->R,
    {
        unsafe{
            let ret=f(Self::get_move_ptr(&mut this));
            Self::drop_allocation(&mut this);
            ret
        }
    }

    #[inline]
    fn in_move_ptr<F,R>(self,f:F)->R
    where 
        F:FnOnce(MovePtr<'_, Self::PtrTarget>)->R,
    {
        unsafe{
            let mut this=ManuallyDrop::new(self);
            let ret=f(Self::get_move_ptr(&mut this));
            Self::drop_allocation(&mut this);
            ret
        }
    }
}


///////////////////////////////////////////////////////////////////////////////


/// Trait for non-owning pointers that are shared-reference-like.
///
/// # Safety 
///
/// Implementors must only contain a non-null pointer [(*1)](#clarification1).
/// Meaning that they must be `#[repr(transparent)]` wrappers around 
/// `&`/`NonNull`/`impl ImmutableRef`.
///
/// <span id="clarification1">(*1)</span>
/// They can also contain any amount of zero-sized fields with an alignement of 1.
//
// # Implementation notes
//
// The default methods use `Transmuter` instead of:
// - `std::mem::transmute` because the compiler doesn't know that the size of 
//   `*const ()` and `Self` is the same
// - `std::mem::transmute_copy`: incurrs function call overhead in unoptimized builds,
// which is unnacceptable.
//
// These methods have been defined to compile to a couple of `mov`s in debug builds.
pub unsafe trait ImmutableRef: Copy {
    /// The referent of the pointer, what it points to.
    type Target;

    /// A marker type that can be used as a proof that the `T` type parameter of
    /// `ImmutableRefTarget<T, U>` implements `ImmutableRef<Target = U>`.
    const TARGET: ImmutableRefTarget<Self, Self::Target> = ImmutableRefTarget::new();

    /// Converts this pointer to a `NonNull`.
    #[inline(always)]
    fn to_nonnull(self)->NonNull<Self::Target> {
        unsafe{ Transmuter{from: self}.to }
    }

    /// Constructs this pointer from a `NonNull`.
    /// 
    /// # Safety 
    /// 
    /// `from` must be one of these:
    ///
    /// - A pointer from a call to `ImmutableRef::to_nonnull` or 
    /// `ImmutableRef::to_raw_ptr` on an instance of `Self`,
    /// with the same lifetime.
    ///
    /// - Valid to transmute to Self.
    #[inline(always)]
    unsafe fn from_nonnull(from: NonNull<Self::Target>)->Self{
        unsafe{ Transmuter{from}.to }
    }

    /// Converts this pointer to a raw pointer.
    #[inline(always)]
    fn to_raw_ptr(self)->*const Self::Target {
        unsafe{ Transmuter{from: self}.to }
    }

    /// Constructs this pointer from a raw pointer.
    /// 
    /// # Safety
    /// 
    /// This has the same safety requirements as [`from_nonnull`](#method.from_nonnull)
    #[inline(always)]
    unsafe fn from_raw_ptr(from: *const Self::Target)-> Option<Self> {
        unsafe{ Transmuter{from}.to }
    }
}

/// Gets the `ImmutableRef::Target` associated type for `T`.
pub type ImmutableRefOut<T> = <T as ImmutableRef>::Target;


unsafe impl<'a, T> ImmutableRef for &'a T {
    type Target = T;

    #[inline(always)]
    #[cfg(miri)]
    fn to_raw_ptr(self)->*const Self::Target {
        self as _
    }
    
    #[inline(always)]
    #[cfg(miri)]
    unsafe fn from_raw_ptr(from: *const Self::Target)-> Option<Self> {
        std::mem::transmute(from)
    }
}


////////////////////////////////////////////////////////////////////////////////


/// A marker type that can be used as a proof that the `T` type parameter of
/// `ImmutableRefTarget<T, U>`
/// implements `ImmutableRef<Target = U>`.
pub struct ImmutableRefTarget<T, U>(NonOwningPhantom<(T, U)>);

impl<T, U> Copy for ImmutableRefTarget<T, U> {}
impl<T, U> Clone for ImmutableRefTarget<T, U> {
    fn clone(&self)->Self{
        *self
    }
}

impl<T, U> ImmutableRefTarget<T, U> {
    // This function is private on purpose.
    //
    // This type is only supposed to be constructed in the default initializer for 
    // `ImmutableRef::TARGET`.
    #[inline(always)]
    const fn new()->Self{
        Self(NonOwningPhantom::DEFAULT)
    }
}