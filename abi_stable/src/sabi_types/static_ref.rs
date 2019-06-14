use std::{
    ops::{Deref},
    fmt::{self,Display},
};

use crate::abi_stability::SharedStableAbi;


/**
A wrapper type for vtable static references,
and other constants that have `non-'static` generic parameters 
but are safe to reference for the lifetime of `T`.

# Purpose

This type is necessary because Rust doesn't understand that vtables live for `'static`,
even though they have `non-'static` type parameters.



*/
#[repr(transparent)]
#[derive(StableAbi)]
#[sabi(
    unconstrained(T),
    bound="T:SharedStableAbi",
)]
pub struct StaticRef<T>{
    ref_:*const T,
}

impl<T> Display for StaticRef<T>
where
    T:Display
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Display::fmt(&**self,f)
    }
}


impl<T> Clone for StaticRef<T>{
    fn clone(&self)->Self{
        *self
    }
}

impl<T> Copy for StaticRef<T>{}




shared_impls! {
    mod=static_ref_impls
    new_type=StaticRef[][T],
    original_type=AAAA,
}


impl<T> StaticRef<T>{
    /// Constructs this StaticRef from a raw pointer.
    ///
    /// # Safety
    ///
    /// You must ensure that the raw pointer is valid for the lifetime of `T`.
    pub const unsafe fn from_raw(ref_:*const T)->Self{
        Self{ref_}
    }

    /// Constructs this StaticRef from a static reference
    ///
    /// This implicitly requires that `T:'static`.
    pub const fn from_ref(ref_:&'static T)->Self{
        Self{ref_}
    }

    /// Gets access to the reference.
    ///
    /// This returns `&'a T` instead of `&'static T` to support vtables of `non-'static` types.
    pub fn get<'a>(self)->&'a T{
        unsafe{ &*self.ref_ }
    }

    /// Gets access to the referenced value,as a raw pointer.
    pub const fn get_raw<'a>(self)->*const T{
        self.ref_
    }

    /// Transmutes this StaticRef<T> to a StaticRef<U>.
    ///
    /// # Safety
    ///
    /// StaticRef has the same rules that references have regarding
    /// transmuting from one type to another:
    pub const unsafe fn transmute_ref<U>(self)->StaticRef<U>{
        StaticRef::from_raw(
            self.ref_ as *const U
        )
    }
}

impl<T> Deref for StaticRef<T>{
    type Target=T;

    fn deref(&self)->&T{
        self.get()
    }
}

