use crate::pointer_trait::{AsPtr, CanTransmuteElement, GetPointerKind, PK_Reference};

use std::{
    fmt::{self, Display},
    ops::Deref,
    ptr::NonNull,
};

/// A wrapper type for vtable static references,
/// and other constants that have `non-'static` generic parameters
/// but are safe to reference for the lifetime of `T`.
///
/// # Purpose
///
/// This type is necessary because Rust doesn't understand that vtables live for `'static`,
/// even though they have `non-'static` type parameters.
///
/// # Example
///
/// This defines a non-extensible vtable,using a StaticRef as the pointer to the vtable.
///
/// ```
/// use abi_stable::{
///     marker_type::ErasedObject,
///     prefix_type::{PrefixTypeTrait, WithMetadata},
///     sabi_extern_fn,
///     sabi_types::StaticRef,
///     staticref, StableAbi,
/// };
///
/// use std::{marker::PhantomData, ops::Deref};
///
/// fn main() {
///     let boxed = BoxLike::new("foo".to_string());
///     assert_eq!(boxed.as_str(), "foo");
/// }
///
/// /// An ffi-safe `Box<T>`
/// #[repr(C)]
/// #[derive(StableAbi)]
/// pub struct BoxLike<T> {
///     data: *mut T,
///
///     vtable: StaticRef<VTable<T>>,
///
///     _marker: PhantomData<T>,
/// }
///
/// impl<T> BoxLike<T> {
///     pub fn new(value: T) -> Self {
///         Self {
///             data: Box::into_raw(Box::new(value)),
///             vtable: VTable::<T>::VTABLE,
///             _marker: PhantomData,
///         }
///     }
/// }
///
/// impl<T> Drop for BoxLike<T> {
///     fn drop(&mut self) {
///         unsafe {
///             (self.vtable.drop_)(self.data);
///         }
///     }
/// }
///
/// impl<T> Deref for BoxLike<T> {
///     type Target = T;
///
///     fn deref(&self) -> &T {
///         unsafe { &*self.data }
///     }
/// }
///
/// #[repr(C)]
/// #[derive(StableAbi)]
/// pub struct VTable<T> {
///     drop_: unsafe extern "C" fn(*mut T),
/// }
///
/// impl<T> VTable<T> {
///     // The `staticref` macro declares a `StaticRef<VTable<T>>` constant.
///     staticref!(const VTABLE: Self = Self{
///         drop_: drop_box::<T>,
///     });
/// }
///
/// #[sabi_extern_fn]
/// unsafe fn drop_box<T>(object: *mut T) {
///     drop(Box::from_raw(object));
/// }
///
/// ```
#[repr(transparent)]
#[derive(StableAbi)]
pub struct StaticRef<T> {
    ref_: NonNull<T>,
}

impl<T> Display for StaticRef<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<T> Clone for StaticRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for StaticRef<T> {}

unsafe impl<'a, T: 'a> Sync for StaticRef<T> where &'a T: Sync {}

unsafe impl<'a, T: 'a> Send for StaticRef<T> where &'a T: Send {}

shared_impls! {
    mod=static_ref_impls
    new_type=StaticRef[][T],
    original_type=AAAA,
}

impl<T> StaticRef<T> {
    /// Constructs this StaticRef from a raw pointer.
    ///
    /// # Safety
    ///
    /// You must ensure that the raw pointer is valid for the entire program's lifetime.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::StaticRef;
    ///
    /// struct GetPtr<T>(T);
    ///
    /// impl<T> GetPtr<T> {
    ///     const PTR: *const Option<T> = &None;
    ///
    ///     const STATIC: StaticRef<Option<T>> = unsafe { StaticRef::from_raw(Self::PTR) };
    /// }
    /// {}
    /// ```
    pub const unsafe fn from_raw(ref_: *const T) -> Self {
        Self {
            ref_: unsafe { NonNull::new_unchecked(ref_ as *mut T) },
        }
    }

    /// Constructs this StaticRef from a static reference
    ///
    /// This implicitly requires that `T:'static`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::StaticRef;
    ///
    /// struct GetPtr<T>(T);
    ///
    /// impl<T> GetPtr<T>
    /// where
    ///     T: 'static,
    /// {
    ///     const REF: &'static Option<T> = &None;
    ///
    ///     const STATIC: StaticRef<Option<T>> = StaticRef::new(Self::REF);
    /// }
    ///
    /// ```
    pub const fn new(ref_: &'static T) -> Self {
        Self {
            ref_: unsafe { NonNull::new_unchecked(ref_ as *const T as *mut T) },
        }
    }

    /// Creates a StaticRef by heap allocating and leaking `val`.
    pub fn leak_value(val: T) -> Self {
        // Safety: This is safe, because the value is a leaked heap allocation.
        unsafe { Self::from_raw(crate::utils::leak_value(val)) }
    }

    /// Gets access to the reference.
    ///
    /// This returns `&'a T` instead of `&'static T` to support vtables of `non-'static` types.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::StaticRef;
    ///
    /// struct GetPtr<T>(T);
    ///
    /// impl<T> GetPtr<T> {
    ///     const PTR: *const Option<T> = &None;
    ///
    ///     const STATIC: StaticRef<Option<T>> = unsafe { StaticRef::from_raw(Self::PTR) };
    /// }
    ///
    /// let reference: &'static Option<String> = GetPtr::<String>::STATIC.get();
    ///
    /// ```
    pub const fn get<'a>(self) -> &'a T {
        unsafe { crate::utils::deref!(self.ref_.as_ptr() as *const T) }
    }

    /// Gets access to the referenced value,as a raw pointer.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::StaticRef;
    /// use std::convert::Infallible;
    ///
    /// struct GetPtr<T>(T);
    ///
    /// impl<T> GetPtr<T> {
    ///     const PTR: *const Option<T> = &None;
    ///
    ///     const STATIC: StaticRef<Option<T>> = unsafe { StaticRef::from_raw(Self::PTR) };
    /// }
    ///
    /// let reference: *const Option<Infallible> = GetPtr::<Infallible>::STATIC.as_ptr();
    ///
    /// ```
    pub const fn as_ptr(self) -> *const T {
        self.ref_.as_ptr() as *const T
    }

    /// Transmutes this `StaticRef<T>` to a `StaticRef<U>`.
    ///
    /// # Safety
    ///
    /// StaticRef has the same rules that references have regarding
    /// transmuting from one type to another:
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::StaticRef;
    ///
    /// struct GetPtr<T>(T);
    ///
    /// impl<T> GetPtr<T> {
    ///     const PTR: *const Option<T> = &None;
    ///
    ///     const STATIC: StaticRef<Option<T>> = unsafe { StaticRef::from_raw(Self::PTR) };
    /// }
    ///
    /// let reference: StaticRef<Option<[(); 0xFFF_FFFF]>> =
    ///     unsafe { GetPtr::<()>::STATIC.transmute::<Option<[(); 0xFFF_FFFF]>>() };
    ///
    /// ```
    pub const unsafe fn transmute<U>(self) -> StaticRef<U> {
        unsafe { StaticRef::from_raw(self.ref_.as_ptr() as *const T as *const U) }
    }
}

impl<T> Deref for StaticRef<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.get()
    }
}

unsafe impl<T> AsPtr for StaticRef<T> {
    fn as_ptr(&self) -> *const T {
        self.ref_.as_ptr() as *const T
    }
}

unsafe impl<T> GetPointerKind for StaticRef<T> {
    type Kind = PK_Reference;

    type PtrTarget = T;
}

unsafe impl<T, U> CanTransmuteElement<U> for StaticRef<T> {
    type TransmutedPtr = StaticRef<U>;

    #[inline(always)]
    unsafe fn transmute_element_(self) -> StaticRef<U> {
        unsafe { self.transmute() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_test() {
        unsafe {
            let three: *const i32 = &3;
            assert_eq!(*StaticRef::from_raw(three), 3);
        }

        assert_eq!(*StaticRef::new(&5), 5);

        assert_eq!(*StaticRef::leak_value(8), 8);
    }

    #[test]
    fn access() {
        let reference = StaticRef::new(&8);
        const SREF: StaticRef<u8> = StaticRef::new(&8);
        const REF: &u8 = SREF.get();

        assert_eq!(*reference.get(), 8);
        assert_eq!(*REF, 8);
        unsafe {
            assert_eq!(*reference.as_ptr(), 8);
        }
    }

    #[test]
    fn transmutes() {
        let reference = StaticRef::new(&(!0u32));

        unsafe {
            assert_eq!(*reference.transmute::<i32>(), -1);
        }
    }
}
