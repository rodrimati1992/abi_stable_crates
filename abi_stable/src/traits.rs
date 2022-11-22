//! Where miscellaneous traits reside.

use std::{borrow::Borrow, ops::Deref};

#[allow(unused_imports)]
use core_extensions::SelfOps;

use crate::{
    pointer_trait::{CanTransmuteElement, TransmuteElement},
    sabi_types::{RMut, RRef},
    std_types::{RSlice, RStr, RString, RVec},
};

///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////

/// For cloning a reference-like type into a (preferably ffi-safe) owned type.
pub trait IntoOwned: Copy + Deref {
    /// The owned equivalent of this type.
    type ROwned: Borrow<Self::Target>;

    /// Performs the colne.
    fn into_owned(self) -> Self::ROwned;
}

impl<T: Clone> IntoOwned for &T {
    type ROwned = T;

    fn into_owned(self) -> T {
        self.clone()
    }
}

impl IntoOwned for RStr<'_> {
    type ROwned = RString;

    fn into_owned(self) -> RString {
        self.into()
    }
}

impl<T: Clone> IntoOwned for RSlice<'_, T> {
    type ROwned = RVec<T>;

    fn into_owned(self) -> RVec<T> {
        self.to_rvec()
    }
}

///////////////////////////////////////////////////////////////////////////

/// Converts a `#[repr(Rust)]` type into its `#[repr(C)]` equivalent.
///
/// `#[repr(Rust)]` is the default representation for data types.
pub trait IntoReprC {
    /// The `#[repr(C)]` equivalent.
    type ReprC;

    /// Performs the conversion
    fn into_c(self) -> Self::ReprC;
}

/// Converts a `#[repr(C)]` type into its `#[repr(Rust)]` equivalent.
///
/// `#[repr(Rust)]` is the default representation for data types.
pub trait IntoReprRust {
    /// The `#[repr(Rust)]` equivalent.
    type ReprRust;

    /// Performs the conversion
    fn into_rust(self) -> Self::ReprRust;
}

///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////

macro_rules! impl_from_rust_repr {
    (
        $(#[$meta:meta])*
        impl$([ $($impl_header:tt)* ])? From<$from_ty:ty> for $into_ty:ty
        $( where [ $( $where_clause:tt )* ] )?
        {
            fn($this:pat) $function_contents:block
        }


    ) => (
        $(#[$meta])*
        impl $(< $($impl_header)* >)? From<$from_ty> for $into_ty
        $(where $($where_clause)*)?
        {
            #[inline]
            fn from($this:$from_ty)->$into_ty{
                $function_contents
            }
        }

        $(#[$meta])*
        impl $(< $($impl_header)* >)?  $crate::traits::IntoReprC for $from_ty
        $(where $($where_clause)*)?
        {
            type ReprC=$into_ty;
            #[inline]
            fn into_c(self)->Self::ReprC{
                self.into()
            }
        }
    )
}

macro_rules! impl_into_rust_repr {
    (
        $(#[$meta:meta])*
        impl$([ $($impl_header:tt)* ])? Into<$into_ty:ty> for $from_ty:ty
        $( where [ $( $where_clause:tt )* ] )?
        {
            fn($this:pat){
                $($function_contents:tt)*
            }
        }

    ) => (
        $(#[$meta])*
        impl $(< $($impl_header)* >)? From<$from_ty> for $into_ty
        $(where $($where_clause)*)?
        {
            #[inline]
            fn from($this: $from_ty) -> $into_ty{
                $($function_contents)*
            }
        }

        $(#[$meta])*
        impl $(< $($impl_header)* >)?  $crate::traits::IntoReprRust for $from_ty
        $(where $($where_clause)*)?
        {
            type ReprRust=$into_ty;
            #[inline]
            fn into_rust(self)->Self::ReprRust{
                self.into()
            }
        }
    )
}

///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////

pub(crate) trait ErasedType<'a>: Sized {
    type Unerased;

    #[inline]
    unsafe fn from_unerased<P>(p: P) -> P::TransmutedPtr
    where
        P: CanTransmuteElement<Self, PtrTarget = Self::Unerased>,
    {
        unsafe { p.transmute_element::<Self>() }
    }

    #[inline]
    unsafe fn downcast_into<P>(p: P) -> P::TransmutedPtr
    where
        P: CanTransmuteElement<Self::Unerased, PtrTarget = Self>,
    {
        unsafe { p.transmute_element::<Self::Unerased>() }
    }

    #[inline]
    unsafe fn run_downcast_as<'b, F, R>(p: RRef<'b, Self>, func: F) -> R
    where
        Self::Unerased: 'b,
        F: FnOnce(&'b Self::Unerased) -> R,
    {
        unsafe { func(p.transmute_into_ref::<Self::Unerased>()) }
    }

    #[inline]
    unsafe fn run_downcast_as_mut<'b, F, R>(p: RMut<'b, Self>, func: F) -> R
    where
        Self::Unerased: 'b,
        F: FnOnce(&'b mut Self::Unerased) -> R,
    {
        unsafe { func(p.transmute_into_mut()) }
    }
}

///////////////////////////////////////////////////////////////////////////

/// Unwraps a type into its owned value.
pub trait IntoInner {
    /// The type of the value this owns.
    type Element;

    /// Unwraps this type into its owned value.
    fn into_inner_(self) -> Self::Element;
}

///////////////////////////////////////////////////////////////////////////
