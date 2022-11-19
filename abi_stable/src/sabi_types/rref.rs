use std::{
    fmt::{self, Display},
    marker::PhantomData,
    ptr::NonNull,
};

use crate::{
    pointer_trait::{AsPtr, CanTransmuteElement, GetPointerKind, PK_Reference},
    utils::ref_as_nonnull,
};

/// Equivalent to `&'a T`,
/// which allows a few more operations without causing Undefined Behavior.
///
/// # Purpose
///
/// This type is used as the `&self` parameter in abi_stable trait objects
/// because it can be soundly transmuted
/// to point to other smaller but compatible types, then back to the original type.
///
/// This crate is tested with [miri] to detect bugs in unsafe code,
/// which implements the  [Stacked Borrows model].
/// Because that model forbids `&T` to `&()`  to `&T` transmutes (when `T` isn't zero-sized),
/// it required defining `RRef` to allow a reference-like type that can be transmuted.
///
/// # Example
///
/// This example demonstrates how a simple `&dyn Any`-like type can be implemented.
///
/// ```rust
/// use abi_stable::{marker_type::ErasedObject, std_types::UTypeId, RRef};
///
/// fn main() {
///     let value = WithTypeId::new(5u32);
///     let erased = value.erase();
///
///     assert_eq!(WithTypeId::downcast::<i32>(erased), None);
///     assert_eq!(WithTypeId::downcast::<bool>(erased), None);
///     assert_eq!(WithTypeId::downcast::<u32>(erased), Some(&value));
/// }
///
/// // `#[repr(C))]` with a trailing `T` field is required for soundly transmuting from
/// // `RRef<'a, WithTypeId<T>>` to `RRef<'a, WithTypeId<ErasedObject>>`.
/// #[repr(C)]
/// #[derive(Debug, PartialEq)]
/// struct WithTypeId<T> {
///     type_id: UTypeId,
///     value: T,
/// }
///
/// impl<T> WithTypeId<T> {
///     pub fn new(value: T) -> Self
///     where
///         T: 'static,
///     {
///         Self {
///             type_id: UTypeId::new::<T>(),
///             value,
///         }
///     }
///
///     pub fn erase(&self) -> RRef<'_, WithTypeId<ErasedObject>> {
///         unsafe { RRef::new(self).transmute::<WithTypeId<ErasedObject>>() }
///     }
/// }
///
/// impl WithTypeId<ErasedObject> {
///     pub fn downcast<T>(this: RRef<'_, Self>) -> Option<&WithTypeId<T>>
///     where
///         T: 'static,
///     {
///         if this.get().type_id == UTypeId::new::<T>() {
///             // safety: we checked that type parameter was `T`
///             unsafe { Some(this.transmute_into_ref::<WithTypeId<T>>()) }
///         } else {
///             None
///         }
///     }
/// }
///
///
/// ```
///
/// <span id="type-prefix-exp"></span>
/// # Type Prefix
///
/// A type parameter `U` is considered a prefix of `T` in all of these cases:
///
/// - `U` is a zero-sized type with an alignment equal or lower than `T`
///
/// - `U` is a `#[repr(transparent)]` wrapper over `T`
///
/// - `U` and `T` are both `#[repr(C)]` structs,
/// in which `T` starts with the fields of `U` in the same order,
/// and `U` has an alignment equal to or lower than `T`.
///
/// Please note that it can be unsound to transmute a non-local
/// type if it has private fields,
/// since it may assume it was constructed in a particular way.
///
/// [Stacked Borrows model]:
/// https://github.com/rust-lang/unsafe-code-guidelines/blob/master/wip/stacked-borrows.md
///
/// [miri]: https://github.com/rust-lang/miri
///
#[repr(transparent)]
#[derive(StableAbi)]
#[sabi(bound(T:'a))]
pub struct RRef<'a, T> {
    ref_: NonNull<T>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Display for RRef<'a, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.get(), f)
    }
}

impl<'a, T> Clone for RRef<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for RRef<'a, T> {}

unsafe impl<'a, T> Sync for RRef<'a, T> where &'a T: Sync {}

unsafe impl<'a, T> Send for RRef<'a, T> where &'a T: Send {}

shared_impls! {
    mod=static_ref_impls
    new_type=RRef['a][T],
    original_type=AAAA,
    deref_approach=(method = get),
}

impl<'a, T> RRef<'a, T> {
    /// Constructs this RRef from a reference.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::RRef;
    ///
    /// struct GetPtr<'a, T>(&'a T);
    ///
    /// impl<'a, T: 'a> GetPtr<'a, T> {
    ///     const REF: &'a Option<T> = &None;
    ///
    ///     const STATIC: RRef<'a, Option<T>> = RRef::new(Self::REF);
    /// }
    ///
    /// ```
    #[inline(always)]
    pub const fn new(ref_: &'a T) -> Self {
        Self {
            ref_: ref_as_nonnull(ref_),
            _marker: PhantomData,
        }
    }

    /// Constructs this RRef from a raw pointer.
    ///
    /// # Safety
    ///
    /// You must ensure that the raw pointer is valid for the `'a` lifetime,
    /// and points to a fully initialized and aligned `T`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::RRef;
    ///
    /// struct GetPtr<'a, T>(&'a T);
    ///
    /// impl<'a, T: 'a> GetPtr<'a, T> {
    ///     const PTR: *const Option<T> = &None;
    ///
    ///     const STATIC: RRef<'a, Option<T>> = unsafe { RRef::from_raw(Self::PTR) };
    /// }
    ///
    /// ```
    #[inline(always)]
    pub const unsafe fn from_raw(ref_: *const T) -> Self
    where
        T: 'a,
    {
        Self {
            ref_: unsafe { NonNull::new_unchecked(ref_ as *mut T) },
            _marker: PhantomData,
        }
    }

    /// Casts this to an equivalent reference.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::RRef;
    ///
    /// let rref = RRef::new(&89);
    ///
    /// assert_eq!(rref.get(), &89);
    ///
    /// ```
    #[inline(always)]
    pub const fn get(self) -> &'a T {
        unsafe { crate::utils::deref!(self.ref_.as_ptr()) }
    }

    /// Copies the value that this points to.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::RRef;
    ///
    /// let rref = RRef::new(&55);
    ///
    /// assert_eq!(rref.get_copy(), 55);
    ///
    /// ```
    #[inline(always)]
    pub const fn get_copy(self) -> T
    where
        T: Copy,
    {
        unsafe { *(self.ref_.as_ptr() as *const T) }
    }

    /// Casts this to an equivalent raw pointer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::RRef;
    ///
    /// let rref = RRef::new(&89);
    ///
    /// unsafe {
    ///     assert_eq!(*rref.as_ptr(), 89);
    /// }
    /// ```
    #[inline(always)]
    pub const fn as_ptr(self) -> *const T {
        self.ref_.as_ptr() as *const T
    }

    /// Transmutes this `RRef<'a,T>` to a `RRef<'a,U>`.
    ///
    /// # Safety
    ///
    /// Either of these must be the case:
    ///
    /// - [`U` is a prefix of `T`](#type-prefix-exp)
    ///
    /// - `RRef<'a, U>` was the original type of this `RRef<'a, T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::RRef;
    ///
    /// use std::num::Wrapping;
    ///
    /// let rref = RRef::new(&13u32);
    ///
    /// // safety: Wrapping is a `#[repr(transparent)]` wrapper with one `pub` field.
    /// let trans = unsafe { rref.transmute::<Wrapping<u32>>() };
    ///
    /// assert_eq!(trans, RRef::new(&Wrapping(13u32)));
    ///
    ///
    /// ```
    #[inline(always)]
    pub const unsafe fn transmute<U>(self) -> RRef<'a, U>
    where
        U: 'a,
    {
        unsafe { RRef::from_raw(self.ref_.as_ptr() as *const U) }
    }

    /// Transmutes this to a raw pointer pointing to a different type.
    #[inline(always)]
    pub const fn transmute_into_raw<U>(self) -> *const U {
        self.ref_.as_ptr() as *const T as *const U
    }

    /// Transmutes this to a reference pointing to a different type.
    ///
    /// # Safety
    ///
    /// Either of these must be the case:
    ///
    /// - [`U` is a prefix of `T`](#type-prefix-exp)
    ///
    /// - `RRef<'a, U>` was the original type of this `RRef<'a, T>`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{std_types::Tuple2, RRef};
    ///
    /// unsafe {
    ///     let reff = RRef::new(&Tuple2(3u32, 5u64));
    ///     assert_eq!(reff.transmute_into_ref::<u32>(), &3u32);
    /// }
    ///
    /// ```
    #[inline(always)]
    pub const unsafe fn transmute_into_ref<U>(self) -> &'a U
    where
        U: 'a,
    {
        unsafe { crate::utils::deref!(self.ref_.as_ptr() as *const T as *const U) }
    }
}

unsafe impl<'a, T> GetPointerKind for RRef<'a, T> {
    type Kind = PK_Reference;

    type PtrTarget = T;
}

unsafe impl<'a, T, U> CanTransmuteElement<U> for RRef<'a, T>
where
    U: 'a,
{
    type TransmutedPtr = RRef<'a, U>;

    #[inline(always)]
    unsafe fn transmute_element_(self) -> Self::TransmutedPtr {
        unsafe { self.transmute() }
    }
}

unsafe impl<T> AsPtr for RRef<'_, T> {
    #[inline(always)]
    fn as_ptr(&self) -> *const T {
        self.ref_.as_ptr() as *const T
    }

    #[inline(always)]
    fn as_rref(&self) -> RRef<'_, T> {
        *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_test() {
        unsafe {
            let three: *const i32 = &3;
            assert_eq!(RRef::from_raw(three).get_copy(), 3);
        }

        assert_eq!(RRef::new(&5).get_copy(), 5);
    }

    #[test]
    fn access() {
        let reference = RRef::new(&8);

        assert_eq!(*reference.get(), 8);
        assert_eq!(reference.get_copy(), 8);
        unsafe {
            assert_eq!(*reference.as_ptr(), 8);
        }
    }

    #[test]
    fn transmutes() {
        let reference = RRef::new(&(!0u32));

        unsafe {
            assert_eq!(reference.transmute::<i32>().get_copy(), -1);
            assert_eq!(*reference.transmute_into_raw::<i32>(), -1);
            assert_eq!(reference.transmute_into_ref::<i32>(), &-1);
        }
    }

    #[test]
    fn as_ptr_impl() {
        let reference = RRef::new(&89u32);

        unsafe {
            assert_eq!(*AsPtr::as_ptr(&reference), 89);
            assert_eq!(AsPtr::as_rref(&reference), reference);
        }
    }
}
