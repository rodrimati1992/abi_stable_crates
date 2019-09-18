use std::{
    ops::{Deref},
    fmt::{self,Display},
};

use crate::abi_stability::SharedStableAbi;

use super::RRef;


/**
A wrapper type for vtable static references,
and other constants that have `non-'static` generic parameters 
but are safe to reference for the lifetime of `T`.

# Purpose

This type is necessary because Rust doesn't understand that vtables live for `'static`,
even though they have `non-'static` type parameters.

# Example

This defines a vtable,using a StaticRef as the pointer to the vtable.

This example is not intended to be fully functional,
it's only to demonstrate a use for StaticRef.

```
use abi_stable::{
    marker_type::ErasedObject,
    prefix_type::{PrefixTypeTrait,WithMetadata},
    sabi_types::StaticRef,
    utils::transmute_mut_reference,
    StableAbi,
    sabi_extern_fn,
};

use std::marker::PhantomData;

/// An ffi-safe `Box<T>`
#[repr(C)]
#[derive(StableAbi)]
pub struct BoxLike<T> {
    data: *mut T,
    
    vtable: StaticRef<VTable<T>>,

    _marker: PhantomData<T>,
}



#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_struct="VTable")))]
#[sabi(missing_field(panic))]
pub struct VTableVal<T>{
    #[sabi(last_prefix_field)]
    drop_:unsafe extern "C" fn(*mut T),
}


mod vtable{
    use super::*;

    impl<T> VTableVal<T> {
        const VALUE:Self=
            Self{
                drop_:drop_::<T>,
            };

        const VTABLE:StaticRef<WithMetadata<Self>>=unsafe{
            StaticRef::from_raw(
                &WithMetadata::new(PrefixTypeTrait::METADATA,Self::VALUE)
            )
        };

        pub(super)fn vtable()->StaticRef<VTable<T>> {
            WithMetadata::staticref_as_prefix(Self::VTABLE)
        }
    }
}




#[sabi_extern_fn]
unsafe fn drop_<T>(object:*mut T){
    std::ptr::drop_in_place(object);
}

# fn main(){}

```

*/
#[repr(transparent)]
#[derive(StableAbi)]
#[sabi(
    not_stableabi(T),
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

unsafe impl<'a,T:'a> Sync for StaticRef<T>
where &'a T:Sync
{}

unsafe impl<'a,T:'a> Send for StaticRef<T>
where &'a T:Send
{}


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
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::StaticRef;
    ///
    /// struct GetPtr<T>(T);
    ///
    /// impl<T> GetPtr<T>{
    ///     const PTR:*const Option<T>=&None;
    ///
    ///     const STATIC:StaticRef<Option<T>>=unsafe{
    ///         StaticRef::from_raw(Self::PTR)
    ///     };
    /// }
    ///
    /// ```
    pub const unsafe fn from_raw(ref_:*const T)->Self{
        Self{ref_}
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
    ///     T:'static
    /// {
    ///     const REF:&'static Option<T>=&None;
    ///
    ///     const STATIC:StaticRef<Option<T>>=
    ///         StaticRef::from_ref(Self::REF);
    /// }
    ///
    /// ```
    pub const fn from_ref(ref_:&'static T)->Self{
        Self{ref_}
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
    /// impl<T> GetPtr<T>{
    ///     const PTR:*const Option<T>=&None;
    ///
    ///     const STATIC:StaticRef<Option<T>>=unsafe{
    ///         StaticRef::from_raw(Self::PTR)
    ///     };
    /// }
    ///
    /// let reference:&'static Option<String>=
    ///     GetPtr::<String>::STATIC.get();
    ///
    /// ```
    pub fn get<'a>(self)->&'a T{
        unsafe{ &*self.ref_ }
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
    /// impl<T> GetPtr<T>{
    ///     const PTR:*const Option<T>=&None;
    ///
    ///     const STATIC:StaticRef<Option<T>>=unsafe{
    ///         StaticRef::from_raw(Self::PTR)
    ///     };
    /// }
    ///
    /// let reference:*const Option<Infallible>=
    ///     GetPtr::<Infallible>::STATIC.get_raw();
    ///
    /// ```
    pub const fn get_raw<'a>(self)->*const T{
        self.ref_
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
    /// impl<T> GetPtr<T>{
    ///     const PTR:*const Option<T>=&None;
    ///
    ///     const STATIC:StaticRef<Option<T>>=unsafe{
    ///         StaticRef::from_raw(Self::PTR)
    ///     };
    /// }
    ///
    /// let reference:StaticRef<Option<[();0xFFF_FFFF]>>=unsafe{
    ///     GetPtr::<()>::STATIC
    ///         .transmute_ref::<Option<[();0xFFF_FFFF]>>()
    /// };
    ///
    /// ```
    pub const unsafe fn transmute_ref<U>(self)->StaticRef<U>{
        StaticRef::from_raw(
            self.ref_ as *const U
        )
    }

    /// Converts this `StaticRef<T>` to an `RRef<'a,T>`.
    pub const fn to_rref<'a>(self)->RRef<'a,T>
    where
        T:'a,
    {
        unsafe{
            RRef::from_raw(self.ref_)
        }
    }
}

impl<T> From<RRef<'static,T>> for StaticRef<T>
where
    T:'static,
{
    #[inline]
    fn from(v:RRef<'static,T>)->Self{
        v.to_staticref()
    }
}

impl<T> Deref for StaticRef<T>{
    type Target=T;

    fn deref(&self)->&T{
        self.get()
    }
}

