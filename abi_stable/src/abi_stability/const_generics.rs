//! This module implements the trait object used to check const generics.

use crate::{
    abi_stability::{
        check_layout_compatibility,
        extra_checks::{ExtraChecksError, TypeCheckerMut},
    },
    erased_types::{
        c_functions::{adapt_std_fmt, debug_impl, partial_eq_impl},
        FormattingMode,
    },
    marker_type::ErasedObject,
    prefix_type::WithMetadata,
    sabi_types::RRef,
    std_types::{RErr, ROk, RResult, RString},
    type_layout::TypeLayout,
    StableAbi,
};

use std::{
    cmp::{Eq, PartialEq},
    fmt::{self, Debug},
};

///////////////////////////////////////////////////////////////////////////////

/// A trait object used to check equality between const generic parameters.
#[repr(C)]
#[derive(Copy, Clone, StableAbi)]
pub struct ConstGeneric {
    ptr: RRef<'static, ErasedObject>,
    vtable: ConstGenericVTable_Ref,
}

unsafe impl Send for ConstGeneric {}
unsafe impl Sync for ConstGeneric {}

impl ConstGeneric {
    /// Constructs a ConstGeneric from a reference.
    pub const fn new<T>(this: &'static T) -> Self
    where
        T: StableAbi + Eq + PartialEq + Debug + Send + Sync + 'static,
    {
        Self {
            ptr: unsafe { RRef::from_raw(this as *const T as *const ErasedObject) },
            vtable: ConstGenericVTableFor::<T>::VTABLE,
        }
    }

    /// Compares this to another `ConstGeneric` for equality,
    /// returning an error if the type layout of `self` and `other` is not compatible.
    pub fn is_equal(
        &self,
        other: &Self,
        mut checker: TypeCheckerMut<'_>,
    ) -> Result<bool, ExtraChecksError> {
        match checker.check_compatibility(self.vtable.layout(), other.vtable.layout()) {
            ROk(_) => unsafe { Ok(self.vtable.partial_eq()(self.ptr, other.ptr)) },
            RErr(e) => Err(e),
        }
    }
}

impl Debug for ConstGeneric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe { adapt_std_fmt::<ErasedObject>(self.ptr, self.vtable.debug(), f) }
    }
}

// Make sure that this isn't called within `check_layout_compatibility` itself,
// since it would cause infinite recursion.
impl PartialEq for ConstGeneric {
    fn eq(&self, other: &Self) -> bool {
        if check_layout_compatibility(self.vtable.layout(), other.vtable.layout()).is_err() {
            false
        } else {
            unsafe { self.vtable.partial_eq()(self.ptr, other.ptr) }
        }
    }
}

impl Eq for ConstGeneric {}

///////////////////////////////////////////////////////////////////////////////

/// The vtable of `ConstGeneric`
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix))]
#[sabi(missing_field(panic))]
struct ConstGenericVTable {
    layout: &'static TypeLayout,
    partial_eq: unsafe extern "C" fn(RRef<'_, ErasedObject>, RRef<'_, ErasedObject>) -> bool,
    #[sabi(last_prefix_field)]
    debug: unsafe extern "C" fn(
        RRef<'_, ErasedObject>,
        FormattingMode,
        &mut RString,
    ) -> RResult<(), ()>,
}

/// A type that contains the vtable stored in the `ConstGeneric` constructed from a `T`.
/// This is used as a workaround for `const fn` not allowing trait bounds.
struct ConstGenericVTableFor<T>(T);

impl<T> ConstGenericVTableFor<T>
where
    T: StableAbi + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    const _VTABLE_STATIC: &'static WithMetadata<ConstGenericVTable> = &{
        WithMetadata::new(ConstGenericVTable {
            layout: <T as StableAbi>::LAYOUT,
            partial_eq: partial_eq_impl::<T>,
            debug: debug_impl::<T>,
        })
    };

    /// Constructs a `ConstGenericVTableFor`
    const VTABLE: ConstGenericVTable_Ref =
        ConstGenericVTable_Ref(Self::_VTABLE_STATIC.static_as_prefix());
}
