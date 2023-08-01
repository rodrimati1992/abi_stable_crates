use super::*;

use std::fmt;

#[allow(unused_imports)]
use core_extensions::SelfOps;

use crate::{
    abi_stability::PrefixStableAbi,
    erased_types::{c_functions::adapt_std_fmt, InterfaceType, MakeRequiredTraits},
    pointer_trait::{
        AsMutPtr, AsPtr, CanTransmuteElement, GetPointerKind, PK_Reference, PK_SmartPointer,
        PointerKind, TransmuteElement,
    },
    sabi_trait::vtable::{BaseVtable_Prefix, BaseVtable_Ref},
    sabi_types::{MaybeCmp, RMut, RRef},
    std_types::UTypeId,
    type_level::{
        impl_enum::{Implemented, Unimplemented},
        trait_marker,
    },
    StableAbi,
};

/// `RObject` implements ffi-safe trait objects, for a minimal selection of traits.
///
/// The main use of `RObject<_>` is as the default backend for `#[sabi_trait]`
/// generated trait objects.
///
/// # Construction
///
/// `RObject<_>` is how `#[sabi_trait]`-based ffi-safe trait objects are implemented,
/// and there's no way to construct it separate from those.
///
/// # Trait object
///
/// `RObject<'borrow, Pointer<()>, Interface, VTable>`
/// can be used as a trait object for any combination of
/// the traits listed below:
///
/// - [`Send`]
///
/// - [`Sync`]
///
/// - [`Unpin`](std::marker::Unpin)
///
/// - [`Debug`]
///
/// - [`Display`]
///
/// - [`Error`](std::error::Error)
///
/// - [`Clone`]
///
/// # Deconstruction
///
/// `RObject<_>` can be unwrapped into a concrete type,
/// within the same dynamic library/executable that constructed it,
/// using these (fallible) conversion methods:
///
/// - [`downcast_into`](#method.downcast_into):
/// Unwraps into a pointer to `T`.Requires `T: 'static`.
///
/// - [`downcast_as`](#method.downcast_as):
/// Unwraps into a `&T`.Requires `T: 'static`.
///
/// - [`downcast_as_mut`](#method.downcast_as_mut):
/// Unwraps into a `&mut T`.Requires `T: 'static`.
///
/// `RObject` can only be converted back if the trait object was constructed to allow it.
///
///
///
///
///
#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    not_stableabi(V),
    bound(V: PrefixStableAbi),
    bound(I: InterfaceType),
    extra_checks = <I as MakeRequiredTraits>::MAKE,
)]
pub struct RObject<'lt, P, I, V>
where
    P: GetPointerKind,
{
    vtable: PrefixRef<V>,
    ptr: ManuallyDrop<P>,
    _marker: PhantomData<(&'lt (), extern "C" fn() -> I)>,
}

mod clone_impl {
    pub trait CloneImpl<PtrKind> {
        fn clone_impl(&self) -> Self;
    }
}
use self::clone_impl::CloneImpl;

/// This impl is for smart pointers.
impl<'lt, P, I, V> CloneImpl<PK_SmartPointer> for RObject<'lt, P, I, V>
where
    P: AsPtr,
    I: InterfaceType<Clone = Implemented<trait_marker::Clone>>,
{
    fn clone_impl(&self) -> Self {
        let ptr =
            unsafe { self.sabi_robject_vtable()._sabi_clone().unwrap()(RRef::new(&self.ptr)) };
        Self {
            vtable: self.vtable,
            ptr: ManuallyDrop::new(ptr),
            _marker: PhantomData,
        }
    }
}

/// This impl is for references.
impl<'lt, P, I, V> CloneImpl<PK_Reference> for RObject<'lt, P, I, V>
where
    P: AsPtr + Copy,
    I: InterfaceType,
{
    fn clone_impl(&self) -> Self {
        Self {
            vtable: self.vtable,
            ptr: ManuallyDrop::new(*self.ptr),
            _marker: PhantomData,
        }
    }
}

/// Clone is implemented for references and smart pointers,
/// using `GetPointerKind` to decide whether `P` is a smart pointer or a reference.
///
/// RObject does not implement Clone if `P` == `&mut ()` :
///
///
/// ```compile_fail
/// use abi_stable::{
///     sabi_trait::{doc_examples::ConstExample_TO, TD_Opaque},
///     std_types::*,
/// };
///
/// let mut object = ConstExample_TO::from_value(10usize, TD_Opaque);
/// let borrow = object.sabi_reborrow_mut();
/// let _ = borrow.clone();
/// ```
///
/// Here is the same example with `sabi_reborrow`
///
/// ```
/// use abi_stable::{
///     sabi_trait::{doc_examples::ConstExample_TO, TD_Opaque},
///     std_types::*,
/// };
///
/// let mut object = ConstExample_TO::from_value(10usize, TD_Opaque);
/// let borrow = object.sabi_reborrow();
/// let _ = borrow.clone();
/// ```
///
///
impl<'lt, P, I, V> Clone for RObject<'lt, P, I, V>
where
    P: AsPtr,
    I: InterfaceType,
    Self: CloneImpl<<P as GetPointerKind>::Kind>,
{
    fn clone(&self) -> Self {
        self.clone_impl()
    }
}

impl<'lt, P, I, V> Debug for RObject<'lt, P, I, V>
where
    P: AsPtr<PtrTarget = ()> + AsPtr,
    I: InterfaceType<Debug = Implemented<trait_marker::Debug>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            adapt_std_fmt::<ErasedObject>(
                self.sabi_erased_ref(),
                self.sabi_robject_vtable()._sabi_debug().unwrap(),
                f,
            )
        }
    }
}

impl<'lt, P, I, V> Display for RObject<'lt, P, I, V>
where
    P: AsPtr<PtrTarget = ()>,
    I: InterfaceType<Display = Implemented<trait_marker::Display>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            adapt_std_fmt::<ErasedObject>(
                self.sabi_erased_ref(),
                self.sabi_robject_vtable()._sabi_display().unwrap(),
                f,
            )
        }
    }
}

impl<'lt, P, I, V> std::error::Error for RObject<'lt, P, I, V>
where
    P: AsPtr<PtrTarget = ()>,
    I: InterfaceType<
        Display = Implemented<trait_marker::Display>,
        Debug = Implemented<trait_marker::Debug>,
        Error = Implemented<trait_marker::Error>,
    >,
{
}

unsafe impl<'lt, P, I, V> Send for RObject<'lt, P, I, V>
where
    P: GetPointerKind,
    I: InterfaceType<Send = Implemented<trait_marker::Send>>,
{
}

unsafe impl<'lt, P, I, V> Sync for RObject<'lt, P, I, V>
where
    P: GetPointerKind,
    I: InterfaceType<Sync = Implemented<trait_marker::Sync>>,
{
}

impl<'lt, P, I, V> Unpin for RObject<'lt, P, I, V>
where
    // `Unpin` is a property of the referent
    P: GetPointerKind,
    I: InterfaceType<Unpin = Implemented<trait_marker::Unpin>>,
{
}

impl<'lt, P, I, V> RObject<'lt, P, I, V>
where
    P: AsPtr<PtrTarget = ()>,
{
    /// Constructs an RObject from a pointer and an extra vtable.
    ///
    /// This is mostly intended to be called by `#[sabi_trait]` generated trait objects.
    ///
    /// # Safety
    ///
    /// These are the requirements for the caller:
    ///
    /// - `P` must be a pointer to the type that the vtable functions
    ///     take as the first parameter.
    ///
    /// - The vtable must not come from a reborrowed `RObject`
    ///     (created using `RObject::reborrow` or `RObject::reborrow_mut`).
    ///
    /// - The vtable must be the `SomeVTableName` of a struct declared with
    ///     `#[derive(StableAbi)] #[sabi(kind(Prefix(prefix_ref= SomeVTableName)))]`.
    ///
    /// - The vtable must have `RObjectVtable_Ref` as its first declared field
    ///
    pub unsafe fn with_vtable<OrigPtr>(ptr: OrigPtr, vtable: PrefixRef<V>) -> RObject<'lt, P, I, V>
    where
        OrigPtr: CanTransmuteElement<(), TransmutedPtr = P>,
        OrigPtr::PtrTarget: Sized + 'lt,
        P: AsPtr<PtrTarget = ()>,
    {
        RObject {
            vtable,
            ptr: ManuallyDrop::new(unsafe { ptr.transmute_element::<()>() }),
            _marker: PhantomData,
        }
    }
}

impl<'borr, 'a, I, V> RObject<'borr, RRef<'a, ()>, I, V> {
    /// This function allows constructing an RObject in a constant/static.
    ///
    /// This is mostly intended for `#[sabi_trait]`-generated trait objects
    ///
    /// # Safety
    ///
    /// This has the same safety requirements as `RObject::with_vtable`
    ///
    /// # Example
    ///
    /// Because this is intended for `#[sabi_trait]` generated trait objects,
    /// this demonstrates how to construct one in a constant.
    ///
    /// ```
    /// use abi_stable::sabi_trait::{
    ///     doc_examples::ConstExample_CTO,
    ///     prelude::TD_Opaque,
    /// };
    ///
    /// const EXAMPLE0: ConstExample_CTO<'static, 'static> =
    ///     ConstExample_CTO::from_const(&0usize, TD_Opaque);
    ///
    /// ```
    pub const unsafe fn with_vtable_const<T, Downcasting>(ptr: &'a T, vtable: PrefixRef<V>) -> Self
    where
        T: 'borr,
    {
        RObject {
            vtable,
            ptr: {
                let x = unsafe { RRef::new(ptr).transmute::<()>() };
                ManuallyDrop::new(x)
            },
            _marker: PhantomData,
        }
    }
}

impl<'lt, P, I, V> RObject<'lt, P, I, V>
where
    P: GetPointerKind,
{
    /// The uid in the vtable has to be the same as the one for T,
    /// otherwise it was not created from that T in the library that
    /// declared the trait object.
    fn sabi_check_same_utypeid<T>(&self) -> Result<(), UneraseError<()>>
    where
        T: 'static,
    {
        let expected_typeid = self.sabi_robject_vtable()._sabi_type_id()();
        let actual_typeid = UTypeId::new::<T>();
        if expected_typeid == MaybeCmp::Just(actual_typeid) {
            Ok(())
        } else {
            Err(UneraseError {
                robject: (),
                expected_typeid,
                actual_typeid,
            })
        }
    }

    /// Attempts to unerase this trait object into the pointer it was constructed with.
    ///
    /// # Errors
    ///
    /// This will return an error in any of these conditions:
    ///
    /// - It is called in a dynamic library/binary outside
    /// the one from which this RObject was constructed.
    ///
    /// - The trait object wrapping this `RObject` was constructed with a
    /// `TD_CanDowncast` argument.
    ///
    /// - `T` is not the concrete type this `RObject<_>` was constructed with.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{
    ///     sabi_trait::doc_examples::Doer_TO, std_types::RBox,
    ///     type_level::downcasting::TD_CanDowncast,
    /// };
    ///
    /// let to = || Doer_TO::from_value(5usize, TD_CanDowncast);
    ///
    /// // `to.obj` is an RObject
    /// assert_eq!(
    ///     to().obj.downcast_into::<usize>().ok(),
    ///     Some(RBox::new(5usize))
    /// );
    /// assert_eq!(to().obj.downcast_into::<u8>().ok(), None);
    ///
    /// ```
    pub fn downcast_into<T>(self) -> Result<P::TransmutedPtr, UneraseError<Self>>
    where
        T: 'static,
        P: AsPtr<PtrTarget = ()> + CanTransmuteElement<T>,
    {
        check_unerased!(self, self.sabi_check_same_utypeid::<T>());
        unsafe {
            let this = ManuallyDrop::new(self);
            Ok(ptr::read(&*this.ptr).transmute_element::<T>())
        }
    }

    /// Attempts to unerase this trait object into a reference of
    /// the value was constructed with.
    ///
    /// # Errors
    ///
    /// This will return an error in any of these conditions:
    ///
    /// - It is called in a dynamic library/binary outside
    /// the one from which this RObject was constructed.
    ///
    /// - The trait object wrapping this `RObject` was constructed with a
    /// `TD_CanDowncast` argument.
    ///
    /// - `T` is not the concrete type this `RObject<_>` was constructed with.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{
    ///     sabi_trait::doc_examples::Doer_TO, std_types::RArc,
    ///     type_level::downcasting::TD_CanDowncast, RMut, RRef,
    /// };
    ///
    /// {
    ///     let to: Doer_TO<'_, RArc<()>> =
    ///         Doer_TO::from_ptr(RArc::new(8usize), TD_CanDowncast);
    ///
    ///     // `to.obj` is an RObject
    ///     assert_eq!(to.obj.downcast_as::<usize>().ok(), Some(&8usize));
    ///     assert_eq!(to.obj.downcast_as::<u8>().ok(), None);
    /// }
    /// {
    ///     // `#[sabi_trait]` trait objects constructed from `&`
    ///     // use `RRef<'_, ()>` instead of `&'_ ()`
    ///     // since `&T` can't soundly be transmuted back and forth into `&()`
    ///     let to: Doer_TO<'_, RRef<'_, ()>> = Doer_TO::from_ptr(&13usize, TD_CanDowncast);
    ///
    ///     assert_eq!(to.obj.downcast_as::<usize>().ok(), Some(&13usize));
    ///     assert_eq!(to.obj.downcast_as::<u8>().ok(), None);
    /// }
    /// {
    ///     let mmut = &mut 21usize;
    ///     // `#[sabi_trait]` trait objects constructed from `&mut`
    ///     // use `RMut<'_, ()>` instead of `&'_ mut ()`
    ///     // since `&mut T` can't soundly be transmuted back and forth into `&mut ()`
    ///     let to: Doer_TO<'_, RMut<'_, ()>> = Doer_TO::from_ptr(mmut, TD_CanDowncast);
    ///
    ///     assert_eq!(to.obj.downcast_as::<usize>().ok(), Some(&21usize));
    ///     assert_eq!(to.obj.downcast_as::<u8>().ok(), None);
    /// }
    ///
    /// ```
    pub fn downcast_as<T>(&self) -> Result<&T, UneraseError<&Self>>
    where
        T: 'static,
        P: AsPtr<PtrTarget = ()> + CanTransmuteElement<T>,
    {
        check_unerased!(self, self.sabi_check_same_utypeid::<T>());
        unsafe { Ok(&*(self.ptr.as_ptr() as *const T)) }
    }

    /// Attempts to unerase this trait object into a mutable reference of
    /// the value was constructed with.
    ///
    /// # Errors
    ///
    /// This will return an error in any of these conditions:
    ///
    /// - It is called in a dynamic library/binary outside
    /// the one from which this RObject was constructed.
    ///
    /// - The trait object wrapping this `RObject` was constructed with a
    /// `TD_CanDowncast` argument.
    ///
    /// - `T` is not the concrete type this `RObject<_>` was constructed with.
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{
    ///     sabi_trait::doc_examples::Doer_TO, std_types::RBox,
    ///     type_level::downcasting::TD_CanDowncast, RMut, RRef,
    /// };
    ///
    /// {
    ///     let mut to: Doer_TO<'_, RBox<()>> =
    ///         Doer_TO::from_value(34usize, TD_CanDowncast);
    ///
    ///     // `to.obj` is an RObject
    ///     assert_eq!(to.obj.downcast_as_mut::<usize>().ok(), Some(&mut 34usize));
    ///     assert_eq!(to.obj.downcast_as_mut::<u8>().ok(), None);
    /// }
    /// {
    ///     let mmut = &mut 55usize;
    ///     // `#[sabi_trait]` trait objects constructed from `&mut`
    ///     // use `RMut<'_, ()>` instead of `&'_ mut ()`
    ///     // since `&mut T` can't soundly be transmuted back and forth into `&mut ()`
    ///     let mut to: Doer_TO<'_, RMut<'_, ()>> = Doer_TO::from_ptr(mmut, TD_CanDowncast);
    ///
    ///     assert_eq!(to.obj.downcast_as_mut::<usize>().ok(), Some(&mut 55usize));
    ///     assert_eq!(to.obj.downcast_as_mut::<u8>().ok(), None);
    /// }
    ///
    /// ```
    pub fn downcast_as_mut<T>(&mut self) -> Result<&mut T, UneraseError<&mut Self>>
    where
        T: 'static,
        P: AsMutPtr<PtrTarget = ()> + CanTransmuteElement<T>,
    {
        check_unerased!(self, self.sabi_check_same_utypeid::<T>());
        unsafe { Ok(&mut *(self.ptr.as_mut_ptr() as *mut T)) }
    }

    /// Unwraps the `RObject<_>` into a pointer to T,
    /// without checking whether `T` is the type that the RObject was constructed with.
    ///
    /// # Safety
    ///
    /// You must check that `T` is the type that RObject was constructed
    /// with through other means.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{
    ///     sabi_trait::doc_examples::Doer_TO, std_types::RBox,
    ///     type_level::downcasting::TD_Opaque,
    /// };
    ///
    /// let to = || Doer_TO::from_value(5usize, TD_Opaque);
    ///
    /// unsafe {
    ///     // `to.obj` is an RObject
    ///     assert_eq!(
    ///         to().obj.unchecked_downcast_into::<usize>(),
    ///         RBox::new(5usize)
    ///     );
    /// }
    /// ```
    #[inline]
    pub unsafe fn unchecked_downcast_into<T>(self) -> P::TransmutedPtr
    where
        P: AsPtr<PtrTarget = ()> + CanTransmuteElement<T>,
    {
        let this = ManuallyDrop::new(self);
        unsafe { ptr::read(&*this.ptr).transmute_element::<T>() }
    }

    /// Unwraps the `RObject<_>` into a reference to T,
    /// without checking whether `T` is the type that the RObject was constructed with.
    ///
    /// # Safety
    ///
    /// You must check that `T` is the type that RObject was constructed
    /// with through other means.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{
    ///     sabi_trait::doc_examples::Doer_TO, std_types::RArc,
    ///     type_level::downcasting::TD_Opaque, RMut, RRef,
    /// };
    ///
    /// {
    ///     let to: Doer_TO<'_, RArc<()>> = Doer_TO::from_ptr(RArc::new(8usize), TD_Opaque);
    ///
    ///     unsafe {
    ///         // `to.obj` is an RObject
    ///         assert_eq!(to.obj.unchecked_downcast_as::<usize>(), &8usize);
    ///     }
    /// }
    /// {
    ///     // `#[sabi_trait]` trait objects constructed from `&`
    ///     // use `RRef<'_, ()>` instead of `&'_ ()`
    ///     // since `&T` can't soundly be transmuted back and forth into `&()`
    ///     let to: Doer_TO<'_, RRef<'_, ()>> = Doer_TO::from_ptr(&13usize, TD_Opaque);
    ///
    ///     unsafe {
    ///         assert_eq!(to.obj.unchecked_downcast_as::<usize>(), &13usize);
    ///     }
    /// }
    /// {
    ///     let mmut = &mut 21usize;
    ///     // `#[sabi_trait]` trait objects constructed from `&mut`
    ///     // use `RMut<'_, ()>` instead of `&'_ mut ()`
    ///     // since `&mut T` can't soundly be transmuted back and forth into `&mut ()`
    ///     let to: Doer_TO<'_, RMut<'_, ()>> = Doer_TO::from_ptr(mmut, TD_Opaque);
    ///
    ///     unsafe {
    ///         assert_eq!(to.obj.unchecked_downcast_as::<usize>(), &21usize);
    ///     }
    /// }
    ///
    /// ```
    #[inline]
    pub unsafe fn unchecked_downcast_as<T>(&self) -> &T
    where
        P: AsPtr<PtrTarget = ()>,
    {
        unsafe { &*(self.ptr.as_ptr() as *const T) }
    }

    /// Unwraps the `RObject<_>` into a mutable reference to T,
    /// without checking whether `T` is the type that the RObject was constructed with.
    ///
    /// # Safety
    ///
    /// You must check that `T` is the type that RObject was constructed
    /// with through other means.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{
    ///     sabi_trait::doc_examples::Doer_TO, std_types::RBox,
    ///     type_level::downcasting::TD_Opaque, RMut, RRef,
    /// };
    ///
    /// {
    ///     let mut to: Doer_TO<'_, RBox<()>> = Doer_TO::from_value(34usize, TD_Opaque);
    ///
    ///     unsafe {
    ///         // `to.obj` is an RObject
    ///         assert_eq!(to.obj.unchecked_downcast_as_mut::<usize>(), &mut 34usize);
    ///     }
    /// }
    /// {
    ///     let mmut = &mut 55usize;
    ///     // `#[sabi_trait]` trait objects constructed from `&mut`
    ///     // use `RMut<'_, ()>` instead of `&'_ mut ()`
    ///     // since `&mut T` can't soundly be transmuted back and forth into `&mut ()`
    ///     let mut to: Doer_TO<'_, RMut<'_, ()>> = Doer_TO::from_ptr(mmut, TD_Opaque);
    ///
    ///     unsafe {
    ///         assert_eq!(to.obj.unchecked_downcast_as_mut::<usize>(), &mut 55usize);
    ///     }
    /// }
    ///
    /// ```
    #[inline]
    pub unsafe fn unchecked_downcast_as_mut<T>(&mut self) -> &mut T
    where
        P: AsMutPtr<PtrTarget = ()>,
    {
        unsafe { &mut *(self.ptr.as_mut_ptr() as *mut T) }
    }
}

mod private_struct {
    pub struct PrivStruct;
}
use self::private_struct::PrivStruct;

/// This is used to make sure that reborrowing does not change
/// the Send-ness or Sync-ness of the pointer.
pub trait ReborrowBounds<SendNess, SyncNess> {}

// If it's reborrowing, it must have either both Sync+Send or neither.
impl ReborrowBounds<Unimplemented<trait_marker::Send>, Unimplemented<trait_marker::Sync>>
    for PrivStruct
{
}

impl ReborrowBounds<Implemented<trait_marker::Send>, Implemented<trait_marker::Sync>>
    for PrivStruct
{
}

impl<'lt, P, I, V> RObject<'lt, P, I, V>
where
    P: GetPointerKind,
    I: InterfaceType,
{
    /// Creates a shared reborrow of this RObject.
    ///
    /// This is only callable if `RObject` is either `Send + Sync` or `!Send + !Sync`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{
    ///     sabi_trait::doc_examples::Doer_TO, std_types::RBox,
    ///     type_level::downcasting::TD_Opaque, RMut, RRef,
    /// };
    ///
    /// let mut to: Doer_TO<'_, RBox<()>> = Doer_TO::from_value(13usize, TD_Opaque);
    ///
    /// // `to.obj` is an RObject
    /// assert_eq!(debug_string(to.obj.reborrow()), "13");
    /// assert_eq!(debug_string(to.obj.reborrow()), "13");
    ///
    /// // `#[sabi_trait]` trait objects have an equivalent `sabi_reborrow` method.
    /// assert_eq!(debug_string(to.sabi_reborrow()), "13");
    /// assert_eq!(debug_string(to.sabi_reborrow()), "13");
    ///
    /// fn debug_string<T>(to: T) -> String
    /// where
    ///     T: std::fmt::Debug,
    /// {
    ///     format!("{:?}", to)
    /// }
    ///
    /// ```
    pub fn reborrow<'re>(&'re self) -> RObject<'lt, RRef<'re, ()>, I, V>
    where
        P: AsPtr<PtrTarget = ()>,
        PrivStruct: ReborrowBounds<I::Send, I::Sync>,
    {
        // Reborrowing will break if I add extra functions that operate on `P`.
        RObject {
            vtable: self.vtable,
            ptr: ManuallyDrop::new(self.ptr.as_rref()),
            _marker: PhantomData,
        }
    }

    /// Creates a mutable reborrow of this RObject.
    ///
    /// The reborrowed RObject cannot use these methods:
    ///
    /// - RObject::clone
    ///
    /// This is only callable if `RObject` is either `Send + Sync` or `!Send + !Sync`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{
    ///     sabi_trait::doc_examples::{Doer, Doer_TO},
    ///     std_types::RBox,
    ///     type_level::downcasting::TD_Opaque,
    ///     RMut, RRef,
    /// };
    ///
    /// let mut to: Doer_TO<'_, RBox<()>> = Doer_TO::from_value(2usize, TD_Opaque);
    ///
    /// // `#[sabi_trait]` trait objects have an equivalent `sabi_reborrow_mut` method,
    /// // which delegate to this method.
    /// assert_eq!(increment(to.sabi_reborrow_mut()).value(), 3);
    /// assert_eq!(increment(to.sabi_reborrow_mut()).value(), 4);
    ///
    /// fn increment<T>(mut to: T) -> T
    /// where
    ///     T: Doer,
    /// {
    ///     to.add_into(1);
    ///     to
    /// }
    ///
    /// ```
    pub fn reborrow_mut<'re>(&'re mut self) -> RObject<'lt, RMut<'re, ()>, I, V>
    where
        P: AsMutPtr<PtrTarget = ()>,
        PrivStruct: ReborrowBounds<I::Send, I::Sync>,
    {
        // Reborrowing will break if I add extra functions that operate on `P`.
        RObject {
            vtable: self.vtable,
            ptr: ManuallyDrop::new(self.ptr.as_rmut()),
            _marker: PhantomData,
        }
    }
}

impl<'lt, P, I, V> RObject<'lt, P, I, V>
where
    P: GetPointerKind,
{
    /// Gets the vtable.
    #[inline]
    pub const fn sabi_et_vtable(&self) -> PrefixRef<V> {
        self.vtable
    }

    /// The vtable common to all `#[sabi_trait]` generated trait objects.
    #[inline]
    pub fn sabi_robject_vtable(&self) -> RObjectVtable_Ref<(), P, I> {
        unsafe { BaseVtable_Ref(self.vtable.cast::<BaseVtable_Prefix<(), P, I>>())._sabi_vtable() }
    }

    #[inline]
    fn sabi_into_erased_ptr(self) -> ManuallyDrop<P> {
        let __this = ManuallyDrop::new(self);
        unsafe { ptr::read(&__this.ptr) }
    }

    /// Gets an `RRef` pointing to the erased object.
    pub fn sabi_erased_ref(&self) -> RRef<'_, ErasedObject<()>>
    where
        P: AsPtr<PtrTarget = ()>,
    {
        unsafe { RRef::from_raw(self.ptr.as_ptr() as *const _) }
    }

    /// Gets an `RMut` pointing to the erased object.
    pub fn sabi_erased_mut(&mut self) -> RMut<'_, ErasedObject<()>>
    where
        P: AsMutPtr<PtrTarget = ()>,
    {
        unsafe { RMut::from_raw(self.ptr.as_mut_ptr() as *mut _) }
    }

    /// Gets an `RRef` pointing to the erased object.
    pub fn sabi_as_rref(&self) -> RRef<'_, ()>
    where
        P: AsPtr<PtrTarget = ()>,
    {
        self.ptr.as_rref()
    }

    /// Gets an `RMut` pointing to the erased object.
    pub fn sabi_as_rmut(&mut self) -> RMut<'_, ()>
    where
        P: AsMutPtr<PtrTarget = ()>,
    {
        self.ptr.as_rmut()
    }

    /// Calls the `f` callback with an `MovePtr` pointing to the erased object.
    #[inline]
    pub fn sabi_with_value<F, R>(self, f: F) -> R
    where
        P: OwnedPointer<PtrTarget = ()>,
        F: FnOnce(MovePtr<'_, ()>) -> R,
    {
        OwnedPointer::with_move_ptr(self.sabi_into_erased_ptr(), f)
    }
}

impl<'lt, I, V> RObject<'lt, crate::std_types::RArc<()>, I, V> {
    /// Does a shallow clone of the object, just incrementing the reference counter
    pub fn shallow_clone(&self) -> Self {
        Self {
            vtable: self.vtable,
            ptr: self.ptr.clone(),
            _marker: PhantomData,
        }
    }
}

impl<P, I, V> Drop for RObject<'_, P, I, V>
where
    P: GetPointerKind,
{
    fn drop(&mut self) {
        // This condition is necessary because if the RObject was reborrowed,
        // the destructor function would take a different pointer type.
        if <P as GetPointerKind>::KIND == PointerKind::SmartPointer {
            let destructor = self.sabi_robject_vtable()._sabi_drop();
            unsafe {
                destructor(RMut::<P>::new(&mut self.ptr));
            }
        }
    }
}

//////////////////////////////////////////////////////////////////

/// Error for `RObject<_>` being downcasted into the wrong type
/// with one of the `*downcast*` methods.
#[derive(Copy, Clone)]
pub struct UneraseError<T> {
    robject: T,
    expected_typeid: MaybeCmp<UTypeId>,
    actual_typeid: UTypeId,
}

#[allow(clippy::missing_const_for_fn)]
impl<T> UneraseError<T> {
    fn map<F, U>(self, f: F) -> UneraseError<U>
    where
        F: FnOnce(T) -> U,
    {
        UneraseError {
            robject: f(self.robject),
            expected_typeid: self.expected_typeid,
            actual_typeid: self.actual_typeid,
        }
    }

    /// Extracts the RObject, to handle the failure to unerase it.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.robject
    }
}

impl<D> fmt::Debug for UneraseError<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UneraseError")
            .field("dyn_trait", &"<not shown>")
            .field("expected_typeid", &self.expected_typeid)
            .field("actual_typeid", &self.actual_typeid)
            .finish()
    }
}

impl<D> fmt::Display for UneraseError<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl<D> ::std::error::Error for UneraseError<D> {}

//////////////////////////////////////////////////////////////////
