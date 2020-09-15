use std::{
    ops::{Deref, DerefMut},
    fmt::{self,Display},
    marker::PhantomData,
    ptr::NonNull,
};

use crate::pointer_trait::{CanTransmuteElement,GetPointerKind,PK_MutReference};

/// A StableAbi type equivalent to `&'a mut T`,
#[repr(transparent)]
#[derive(StableAbi)]
#[sabi(
    bound="T:'a",
)]
pub struct RMut<'a,T>{
    ref_: NonNull<T>,
    _marker:PhantomData<&'a mut T>,
}

impl<'a,T> Display for RMut<'a,T>
where
    T:Display
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Display::fmt(&**self,f)
    }
}

unsafe impl<'a,T> Sync for RMut<'a,T>
where &'a T:Sync
{}

unsafe impl<'a,T> Send for RMut<'a,T>
where &'a T:Send
{}


shared_impls! {
    mod=static_ref_impls
    new_type=RMut['a][T],
    original_type=AAAA,
}


impl<'a,T> RMut<'a,T>{
    /// Constructs this RMut from a raw pointer.
    ///
    /// # Safety
    ///
    /// You must ensure that the raw pointer is valid for the `'a` lifetime.
    ///
    #[inline]
    pub unsafe fn from_raw(ref_:*mut T)->Self
    where
        T:'a,
    {
        Self{
            ref_: NonNull::new_unchecked(ref_),
            _marker:PhantomData,
        }
    }

    /// Constructs this RMut from a static reference
    ///
    #[inline]
    pub fn new(ref_:&'a mut T)->Self{
        unsafe{ Self::from_raw(ref_) }
    }

    /// Gets access to the reference.
    ///
    /// Use this to get a `&'a T`,
    /// instead of a reference borrowing from the pointer.
    ///
    #[inline]
    pub fn get(self)->&'a T{
        unsafe{ &*(self.ref_.as_ptr() as *const T) }
    }

    /// Gets access to the mutable reference.
    ///
    /// Use this to get a `&'a mut T`,
    /// instead of a reference borrowing from the pointer.
    ///
    #[inline]
    pub fn get_mut(self)->&'a mut T{
        unsafe{ &mut *self.ref_.as_ptr() }
    }

    /// Gets access to the referenced value,as a raw pointer.
    ///
    #[inline]
    pub fn into_raw(self)->*mut T{
        self.ref_.as_ptr()
    }

    /// Accesses the referenced value as a casted raw pointer.
    #[inline]
    pub fn cast_into_raw<U>(self)->*mut U{
        self.ref_.as_ptr() as *mut U
    }
}

impl<'a,T> Deref for RMut<'a,T>{
    type Target=T;

    #[inline(always)]
    fn deref(&self)->&T{
        unsafe{ &*(self.ref_.as_ptr() as *const T) }
    }
}

impl<'a,T> DerefMut for RMut<'a,T>{
    #[inline(always)]
    fn deref_mut(&mut self)->&mut T{
        unsafe{ &mut *self.ref_.as_ptr() }
    }
}

unsafe impl<'a,T> GetPointerKind for RMut<'a,T>{
    type Kind=PK_MutReference;
}

unsafe impl<'a,T,U> CanTransmuteElement<U> for RMut<'a,T>
where
    U:'a,
{
    type TransmutedPtr= RMut<'a,U>;
}
