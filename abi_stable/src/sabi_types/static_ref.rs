use std::{
    ops::{Deref},
    fmt::{self,Debug,Display},
};


/**
A wrapper type for vtable static references,
and other constants that have `non-'static` generic parameters 
but are safe to reference for the lifetime of `T`.

*/
#[repr(transparent)]
#[derive(StableAbi)]
pub struct StaticRef<T>{
    ref_:*const T,
}

impl<T> Debug for StaticRef<T>
where
    T:Debug
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Debug::fmt(&**self,f)
    }
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
    pub fn get<'a>(&self)->&'a T{
        unsafe{ &*self.ref_ }
    }

    /// Gets access to the referenced value,as a raw pointer.
    pub const fn get_raw<'a>(&self)->*const T{
        self.ref_
    }
}

impl<T> Deref for StaticRef<T>{
    type Target=T;

    fn deref(&self)->&T{
        self.get()
    }
}

