//! Traits for pointers.

use std::{mem::ManuallyDrop, ptr::NonNull};

use crate::{
    marker_type::NonOwningPhantom,
    sabi_types::{MovePtr, RMut, RRef},
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum Deallocate {
    No,
    Yes,
}

///////////

/// What kind of pointer this is.
///
/// # Safety
///
/// Each associated item describes their requirements for the implementor.
///
///
pub unsafe trait GetPointerKind: Sized {
    /// The kind of the pointer.
    ///
    /// # Safety for implementor
    ///
    /// This is what each kind requires to be used as this associated type:
    ///
    /// - [`PK_Reference`]: `Self` must be a `&T`,
    /// or a `Copy` and `#[repr(transparent)]` wrapper around a primitive pointer,
    /// with `&T` semantics
    ///
    /// - [`PK_MutReference`]: `Self` must be a `&mut T`,
    /// or a non-`Drop` and `#[repr(transparent)]` wrapper around a
    /// primitive pointer, with `&mut T` semantics.
    ///
    /// - [`PK_SmartPointer`]: Any pointer type that's neither of the two other kinds.
    ///
    ///
    /// [`PK_Reference`]: ./struct.PK_Reference.html
    /// [`PK_MutReference`]: ./struct.PK_MutReference.html
    /// [`PK_SmartPointer`]: ./struct.PK_SmartPointer.html
    type Kind: PointerKindVariant;

    /// What this pointer points to.
    ///
    /// This is here so that pointers don't *have to* implement `Deref`.
    ///
    /// # Safety for implementor
    ///
    /// If the type implements `std::ops::Deref` this must be the same as
    /// `<Self as Deref>::Target`.
    ///
    type PtrTarget;

    /// The value-level version of the [`Kind`](#associatedtype.Kind) associated type.
    ///
    /// # Safety for implementor
    ///
    /// This must not be overriden.
    const KIND: PointerKind = <Self::Kind as PointerKindVariant>::VALUE;
}

/// For restricting types to the type-level equivalents of [`PointerKind`] variants.
///
/// This trait is sealed, cannot be implemented outside this module,
/// and won't be implemented for any more types.
///
/// [`PointerKind`]: ./enum.PointerKind.html
pub trait PointerKindVariant: Sealed {
    /// The value of the PointerKind variant Self is equivalent to.
    const VALUE: PointerKind;
}

use self::sealed::Sealed;
mod sealed {
    pub trait Sealed {}
}

/// Describes the kind of a pointer.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, StableAbi)]
#[repr(u8)]
pub enum PointerKind {
    /// A `&T`-like pointer
    Reference,
    /// a `&mut T`-like pointer
    MutReference,
    /// Any pointer type that's neither of the other variants
    SmartPointer,
}

/// The type-level equivalent of [`PointerKind::Reference`].
///
/// [`PointerKind::Reference`]: ./enum.PointerKind.html#variant.Reference
#[allow(non_camel_case_types)]
pub struct PK_Reference;

/// The type-level equivalent of [`PointerKind::MutReference`].
///
/// [`PointerKind::MutReference`]: ./enum.PointerKind.html#variant.MutReference
#[allow(non_camel_case_types)]
pub struct PK_MutReference;

/// The type-level equivalent of [`PointerKind::SmartPointer`].
///
/// [`PointerKind::SmartPointer`]: ./enum.PointerKind.html#variant.SmartPointer
#[allow(non_camel_case_types)]
pub struct PK_SmartPointer;

impl Sealed for PK_Reference {}
impl Sealed for PK_MutReference {}
impl Sealed for PK_SmartPointer {}

impl PointerKindVariant for PK_Reference {
    const VALUE: PointerKind = PointerKind::Reference;
}

impl PointerKindVariant for PK_MutReference {
    const VALUE: PointerKind = PointerKind::MutReference;
}

impl PointerKindVariant for PK_SmartPointer {
    const VALUE: PointerKind = PointerKind::SmartPointer;
}

unsafe impl<'a, T> GetPointerKind for &'a T {
    type Kind = PK_Reference;
    type PtrTarget = T;
}

unsafe impl<'a, T> GetPointerKind for &'a mut T {
    type Kind = PK_MutReference;
    type PtrTarget = T;
}

///////////

/// Whether the pointer can be transmuted to an equivalent pointer with `T` as the referent type.
///
/// # Safety
///
/// Implementors of this trait must ensure that:
///
/// - The memory layout of this
/// type is the same regardless of the type of the referent.
///
/// - The pointer type is either `!Drop`(no drop glue either),
/// or it uses a vtable to Drop the referent and deallocate the memory correctly.
///
/// - `transmute_element_` must return a pointer to the same allocation as `self`,
/// at the same offset,
/// and with no reduced provenance
/// (the range of addresses that are valid to dereference with pointers
/// derived from the returned pointer).
///
/// # Example
///
/// ```rust
/// use abi_stable::{
///     pointer_trait::{
///         PK_Reference,
///         AsPtr, CanTransmuteElement, GetPointerKind, TransmuteElement,
///     },
///     sabi_types::StaticRef,
///     std_types::{Tuple2, Tuple4},
/// };
///
/// fn main() {
///     let reff = FooRef::new(&Tuple4::<u8, u16, u32, u64>(3, 5, 8, 13));
///     
///     // safety: `Tuple2<u8, u16>` is a compatible prefix of `Tuple4<u8, u16, u32, u64>`
///     let smaller = unsafe{ reff.transmute_element::<Tuple2<u8, u16>>() };
///     assert_eq!(smaller.get(), &Tuple2(3u8, 5u16));
/// }
///
///
/// #[derive(Debug, Copy, Clone)]
/// #[repr(transparent)]
/// struct FooRef<T>(StaticRef<T>);
///
/// impl<T: 'static> FooRef<T> {
///     pub const fn new(reff: &'static T) -> Self {
///         Self(StaticRef::new(reff))
///     }
///     pub fn get(self) -> &'static T {
///         self.0.get()
///     }
/// }
///
/// unsafe impl<T: 'static> GetPointerKind for FooRef<T> {
///     type PtrTarget = T;
///     type Kind = PK_Reference;
/// }
///
/// unsafe impl<T, U> CanTransmuteElement<U> for FooRef<T>
/// where
///     T: 'static,
///     U: 'static,
/// {
///     type TransmutedPtr = FooRef<U>;
///     
///     unsafe fn transmute_element_(self) -> Self::TransmutedPtr {
///         FooRef(self.0.transmute_element_())
///     }
/// }
///
/// unsafe impl<T: 'static> AsPtr for FooRef<T> {
///     fn as_ptr(&self) -> *const T {
///         self.0.as_ptr()
///     }
/// }
///
///
///
///
/// ```
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
pub trait TransmuteElement {
    /// Transmutes the element type of this pointer..
    ///
    /// # Safety
    ///
    /// Callers must ensure that it is valid to convert from a pointer to `Self::PtrTarget`
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
    #[inline(always)]
    unsafe fn transmute_element<T>(self) -> <Self as CanTransmuteElement<T>>::TransmutedPtr
    where
        Self: CanTransmuteElement<T>,
    {
        self.transmute_element_()
    }
}

impl<This: ?Sized> TransmuteElement for This {}

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
/// # Example
///
/// ```rust
/// use abi_stable::{
///     erased_types::interfaces::DebugDefEqInterface,
///     pointer_trait::{
///         PK_Reference,
///         AsPtr, CanTransmuteElement, GetPointerKind, TransmuteElement,
///     },
///     sabi_types::StaticRef,
///     DynTrait,
/// };
///
/// fn main() {
///     let reff: DynTrait<BarRef<()>, DebugDefEqInterface> =
///         DynTrait::from_any_ptr(BarRef::new(&1234i32), DebugDefEqInterface);
///     
///     assert_eq!(format!("{:?}", reff), "1234");
/// }
///
///
/// #[derive(Debug, Copy, Clone)]
/// #[repr(transparent)]
/// struct BarRef<T>(StaticRef<T>);
///
/// impl<T: 'static> BarRef<T> {
///     pub const fn new(reff: &'static T) -> Self {
///         Self(StaticRef::new(reff))
///     }
/// }
///
/// unsafe impl<T: 'static> GetPointerKind for BarRef<T> {
///     type PtrTarget = T;
///     type Kind = PK_Reference;
/// }
///
/// unsafe impl<T, U> CanTransmuteElement<U> for BarRef<T>
/// where
///     T: 'static,
///     U: 'static,
/// {
///     type TransmutedPtr = BarRef<U>;
///     
///     unsafe fn transmute_element_(self) -> Self::TransmutedPtr {
///         BarRef(self.0.transmute_element_())
///     }
/// }
///
/// unsafe impl<T: 'static> AsPtr for BarRef<T> {
///     fn as_ptr(&self) -> *const T {
///         self.0.as_ptr()
///     }
/// }
///
///
/// ```
pub unsafe trait AsPtr: GetPointerKind {
    /// Gets a const raw pointer to the value that this points to.
    fn as_ptr(&self) -> *const Self::PtrTarget;

    /// Converts this pointer to an `RRef`.
    #[inline(always)]
    fn as_rref(&self) -> RRef<'_, Self::PtrTarget> {
        unsafe { RRef::from_raw(self.as_ptr()) }
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
/// # Example
///
/// ```rust
/// use abi_stable::{
///     erased_types::interfaces::DEIteratorInterface,
///     pointer_trait::{
///         PK_MutReference,
///         AsPtr, AsMutPtr, CanTransmuteElement, GetPointerKind, TransmuteElement,
///     },
///     sabi_types::RMut,
///     DynTrait,
/// };
///
/// fn main() {
///     let mut iter = 0..=5;
///     let reff: DynTrait<QuxMut<()>, DEIteratorInterface<_>> =
///         DynTrait::from_any_ptr(QuxMut::new(&mut iter), DEIteratorInterface::NEW);
///     
///     assert_eq!(reff.collect::<Vec<u32>>(), [0, 1, 2, 3, 4, 5]);
///
///     assert_eq!(iter.next(), None);
/// }
///
///
/// #[derive(Debug)]
/// #[repr(transparent)]
/// struct QuxMut<'a, T>(RMut<'a, T>);
///
/// impl<'a, T> QuxMut<'a, T> {
///     pub fn new(reff: &'a mut T) -> Self {
///         Self(RMut::new(reff))
///     }
/// }
///
/// unsafe impl<T> GetPointerKind for QuxMut<'_, T> {
///     type PtrTarget = T;
///     type Kind = PK_MutReference;
/// }
///
/// unsafe impl<'a, T: 'a, U: 'a> CanTransmuteElement<U> for QuxMut<'a, T> {
///     type TransmutedPtr = QuxMut<'a, U>;
///     
///     unsafe fn transmute_element_(self) -> Self::TransmutedPtr {
///         QuxMut(self.0.transmute_element_())
///     }
/// }
///
/// unsafe impl<T> AsPtr for QuxMut<'_, T> {
///     fn as_ptr(&self) -> *const T {
///         self.0.as_ptr()
///     }
/// }
///
/// unsafe impl<T> AsMutPtr for QuxMut<'_, T> {
///     fn as_mut_ptr(&mut self) -> *mut T {
///         self.0.as_mut_ptr()
///     }
/// }
///
///
/// ```
pub unsafe trait AsMutPtr: AsPtr {
    /// Gets a mutable raw pointer to the value that this points to.
    fn as_mut_ptr(&mut self) -> *mut Self::PtrTarget;

    /// Converts this pointer to an `RRef`.
    #[inline(always)]
    fn as_rmut(&mut self) -> RMut<'_, Self::PtrTarget> {
        unsafe { RMut::from_raw(self.as_mut_ptr()) }
    }
}

///////////////////////////////////////////////////////////////////////////////

/// For owned pointers, allows extracting their contents separate from deallocating them.
///
/// # Safety
///
/// Implementors must:
///
/// - Implement this trait such that `get_move_ptr` can be called before `drop_allocation`.
///
/// - Not override `with_move_ptr`
///
/// - Not override `in_move_ptr`
///
/// # Example
///
/// Implementing this trait for a Box-like type.
///
/// ```rust
/// use abi_stable::{
///     pointer_trait::{
///         CallReferentDrop, PK_SmartPointer,
///         GetPointerKind, AsPtr, AsMutPtr, OwnedPointer,
///     },
///     sabi_types::MovePtr,
///     std_types::RString,
///     StableAbi,
/// };
///
/// use std::{
///     alloc::{self, Layout},
///     marker::PhantomData,
///     mem::ManuallyDrop,
/// };
///
///
/// fn main(){
///     let this = BoxLike::new(RString::from("12345"));
///     
///     let string: RString = this.in_move_ptr(|x: MovePtr<'_, RString>|{
///         MovePtr::into_inner(x)
///     });
///
///     assert_eq!(string, "12345");
/// }
///
///
/// #[repr(C)]
/// #[derive(StableAbi)]
/// pub struct BoxLike<T> {
///     ptr: *mut T,
///     
///     dropper: unsafe extern "C" fn(*mut T, CallReferentDrop),
///
///     _marker: PhantomData<T>,
/// }
///
///
/// impl<T> BoxLike<T>{
///     pub fn new(value:T)->Self{
///         let box_ = Box::new(value);
///         
///         Self{
///             ptr: Box::into_raw(box_),
///             dropper: destroy_box::<T>,
///             _marker:PhantomData,
///         }
///     }
/// }
///
/// unsafe impl<T> GetPointerKind for BoxLike<T> {
///     type PtrTarget = T;
///     type Kind = PK_SmartPointer;
/// }
///
/// unsafe impl<T> AsPtr for BoxLike<T> {
///     fn as_ptr(&self) -> *const T {
///         self.ptr
///     }
/// }
///
/// unsafe impl<T> AsMutPtr for BoxLike<T> {
///     fn as_mut_ptr(&mut self) -> *mut T {
///         self.ptr
///     }
/// }
///
/// unsafe impl<T> OwnedPointer for BoxLike<T> {
///     unsafe fn get_move_ptr(this: &mut ManuallyDrop<Self>) -> MovePtr<'_,Self::PtrTarget>{
///         MovePtr::from_raw(this.ptr)
///     }
///     
///     unsafe fn drop_allocation(this: &mut ManuallyDrop<Self>) {
///         unsafe{
///             (this.dropper)(this.ptr, CallReferentDrop::No)
///         }
///     }
/// }
///
/// impl<T> Drop for BoxLike<T>{
///     fn drop(&mut self){
///         unsafe{
///             (self.dropper)(self.ptr, CallReferentDrop::Yes)
///         }
///     }
/// }
///
/// unsafe extern "C" fn destroy_box<T>(v: *mut T, call_drop: CallReferentDrop) {
///     abi_stable::extern_fn_panic_handling! {
///         let mut box_ = Box::from_raw(v as *mut ManuallyDrop<T>);
///         if call_drop == CallReferentDrop::Yes {
///             ManuallyDrop::drop(&mut *box_);
///         }
///         drop(box_);
///     }
/// }
///
///
/// ```
pub unsafe trait OwnedPointer: Sized + AsMutPtr + GetPointerKind {
    /// Gets a move pointer to the contents of this pointer.
    ///
    /// # Safety
    ///
    /// This function logically moves the owned contents out of this pointer,
    /// the only safe thing that can be done with the pointer afterwads
    /// is to call `OwnedPointer::drop_allocation`.
    ///
    /// <span id="get_move_ptr-example"></span>
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{
    ///     pointer_trait::OwnedPointer,
    ///     sabi_types::MovePtr,
    ///     std_types::{RBox, RVec},
    ///     rvec, StableAbi,
    /// };
    ///
    /// use std::mem::ManuallyDrop;
    ///
    /// let mut this = ManuallyDrop::new(RBox::new(rvec![3, 5, 8]));
    ///
    /// // safety:
    /// // this is only called once,
    /// // and the `RVec` is never accessed again through the `RBox`.
    /// let moveptr: MovePtr<'_, RVec<u8>> = unsafe { OwnedPointer::get_move_ptr(&mut this) };
    ///
    /// let vector: RVec<u8> = MovePtr::into_inner(moveptr);
    ///
    /// // safety: this is only called once, after all uses of `this`
    /// unsafe{ OwnedPointer::drop_allocation(&mut this); }
    ///
    /// assert_eq!(vector[..], [3, 5, 8]);
    ///
    /// ```
    unsafe fn get_move_ptr(this: &mut ManuallyDrop<Self>) -> MovePtr<'_, Self::PtrTarget>;

    /// Deallocates the pointer without dropping its owned contents.
    ///
    /// Note that if `Self::get_move_ptr` has not been called this will
    /// leak the values owned by the referent of the pointer.
    ///
    /// # Safety
    ///
    /// This method must only be called once,
    /// since it'll deallocate whatever memory this pointer owns.
    ///
    /// # Example
    ///
    /// [`get_move_ptr` has an example](#get_move_ptr-example) that uses both that function
    /// and this one.
    unsafe fn drop_allocation(this: &mut ManuallyDrop<Self>);

    /// Runs a callback with the contents of this pointer, and then deallocates it.
    ///
    /// The pointer is deallocated even in the case that `func` panics
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{
    ///     pointer_trait::OwnedPointer,
    ///     sabi_types::MovePtr,
    ///     std_types::{RBox, RCow},
    /// };
    ///
    /// use std::mem::ManuallyDrop;
    ///
    /// let this = ManuallyDrop::new(RBox::new(RCow::from_slice(&[13, 21, 34])));
    ///
    /// let cow: RCow<'static, [u8]> = OwnedPointer::with_move_ptr(this, |moveptr|{
    ///     MovePtr::into_inner(moveptr)
    /// });
    ///
    /// assert_eq!(cow[..], [13, 21, 34]);
    ///
    /// ```
    #[inline]
    fn with_move_ptr<F, R>(mut this: ManuallyDrop<Self>, func: F) -> R
    where
        F: FnOnce(MovePtr<'_, Self::PtrTarget>) -> R,
    {
        unsafe {
            let guard = DropAllocationMutGuard(&mut this);
            func(Self::get_move_ptr(guard.0))
        }
    }

    /// Runs a callback with the contents of this pointer, and then deallocates it.
    ///
    /// The pointer is deallocated even in the case that `func` panics
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{
    ///     pointer_trait::OwnedPointer,
    ///     sabi_types::MovePtr,
    ///     std_types::{RBox, RCow},
    /// };
    ///
    /// let this = RBox::new(Foo(41));
    ///
    /// let cow: Foo = this.in_move_ptr(|moveptr| MovePtr::into_inner(moveptr) );
    ///
    /// assert_eq!(cow, Foo(41));
    ///
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Foo(u32);
    ///
    ///
    /// ```
    #[inline]
    fn in_move_ptr<F, R>(self, func: F) -> R
    where
        F: FnOnce(MovePtr<'_, Self::PtrTarget>) -> R,
    {
        unsafe {
            let mut guard = DropAllocationGuard(ManuallyDrop::new(self));
            func(Self::get_move_ptr(&mut guard.0))
        }
    }
}

struct DropAllocationGuard<T: OwnedPointer>(ManuallyDrop<T>);

impl<T: OwnedPointer> Drop for DropAllocationGuard<T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { T::drop_allocation(&mut self.0) }
    }
}

struct DropAllocationMutGuard<'a, T: OwnedPointer>(&'a mut ManuallyDrop<T>);

impl<T: OwnedPointer> Drop for DropAllocationMutGuard<'_, T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { T::drop_allocation(self.0) }
    }
}

///////////////////////////////////////////////////////////////////////////////

/// Trait for non-owning pointers that are shared-reference-like.
///
/// # Safety
///
/// Implementors must be `#[repr(transparent)]` wrappers around
/// `&`/`NonNull`/`impl ImmutableRef`,
/// with any amount of 1-aligned zero-sized fields.
///
/// It must be valid to transmute a `NonNull<Self::Target>` pointer obtained
/// from [`Self::to_nonnull`](#method.to_nonnull) back into `Self`,
/// producing a pointer just as usable as the original.
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
    fn to_nonnull(self) -> NonNull<Self::Target> {
        unsafe { Transmuter { from: self }.to }
    }

    /// Constructs this pointer from a `NonNull`.
    ///
    /// # Safety
    ///
    /// `from` must be a pointer from a call to `ImmutableRef::to_nonnull` or
    /// `ImmutableRef::to_raw_ptr` on an instance of `Self`.
    #[inline(always)]
    unsafe fn from_nonnull(from: NonNull<Self::Target>) -> Self {
        Transmuter { from }.to
    }

    /// Converts this pointer to a raw pointer.
    #[inline(always)]
    fn to_raw_ptr(self) -> *const Self::Target {
        unsafe { Transmuter { from: self }.to }
    }

    /// Constructs this pointer from a raw pointer.
    ///
    /// # Safety
    ///
    /// This has the same safety requirements as [`from_nonnull`](#method.from_nonnull)
    #[inline(always)]
    unsafe fn from_raw_ptr(from: *const Self::Target) -> Option<Self> {
        Transmuter { from }.to
    }
}

/// Gets the `ImmutableRef::Target` associated type for `T`.
pub type ImmutableRefOut<T> = <T as ImmutableRef>::Target;

unsafe impl<'a, T> ImmutableRef for &'a T {
    type Target = T;

    #[inline(always)]
    #[cfg(miri)]
    fn to_raw_ptr(self) -> *const Self::Target {
        self as _
    }

    #[inline(always)]
    #[cfg(miri)]
    unsafe fn from_raw_ptr(from: *const Self::Target) -> Option<Self> {
        std::mem::transmute(from)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A marker type that can be used as a proof that the `T` type parameter of
/// `ImmutableRefTarget<T, U>`
/// implements [`ImmutableRef`]`<Target = U>`.
///
/// [`ImmutableRef`]: ./trait.ImmutableRef.html
pub struct ImmutableRefTarget<T, U>(NonOwningPhantom<(T, U)>);

impl<T, U> Copy for ImmutableRefTarget<T, U> {}
impl<T, U> Clone for ImmutableRefTarget<T, U> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, U> ImmutableRefTarget<T, U> {
    // This function is private on purpose.
    //
    // This type is only supposed to be constructed in the default initializer for
    // `ImmutableRef::TARGET`.
    #[inline(always)]
    const fn new() -> Self {
        Self(NonOwningPhantom::DEFAULT)
    }
}
