use std::{
    fmt::{self,Display},
    marker::PhantomData,
    ptr::NonNull,
};

use crate::{
    sabi_types::RRef,
    pointer_trait::{AsPtr, AsMutPtr, CanTransmuteElement,GetPointerKind,PK_MutReference},
};

/// Equivalent to `&mut T`.
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
        Display::fmt(self.get(),f)
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
    deref_approach=(method = get),
}


impl<'a,T> RMut<'a,T>{

    /// Constructs this RMut from a mutable reference
    ///
    #[inline(always)]
    pub fn new(ref_:&'a mut T)->Self{
        unsafe{
            Self{
                ref_: NonNull::new_unchecked(ref_),
                _marker:PhantomData,
            }
        }
    }

    /// Constructs this RMut from a raw pointer.
    ///
    /// # Safety
    ///
    /// You must ensure that the raw pointer is valid for the `'a` lifetime,
    /// and that this is the only active pointer to that value.
    ///
    #[inline(always)]
    pub unsafe fn from_raw(ref_:*mut T)->Self
    where
        T:'a,
    {
        Self{
            ref_: NonNull::new_unchecked(ref_),
            _marker:PhantomData,
        }
    }

    /// Reborrows this `RMut`, with a shorter lifetime.
    #[inline(always)]
    pub fn reborrow(&mut self) -> RMut<'_, T> {
        RMut{
            ref_: self.ref_,
            _marker: PhantomData,
        }
    }

    /// Reborrows this `RMut` into a shared reference.
    ///
    #[inline(always)]
    pub fn get(&self)->&T{
        unsafe{ &*(self.ref_.as_ptr() as *const T) }
    }

    /// Copies the value that this `RMut` points to.
    ///
    #[inline(always)]
    pub fn get_copy(&self) -> T
    where
        T: Copy
    {
        unsafe{ *(self.ref_.as_ptr() as *const T) }
    }

    /// Converts this `RMut<'a, T>` into a `&'a T`
    ///
    #[inline(always)]
    pub fn into_ref(self) -> &'a T{
        unsafe{ &*(self.ref_.as_ptr() as *const T) }
    }

    /// Reborrows this `RMut` into a mutable reference.
    ///
    #[inline(always)]
    pub fn get_mut(&mut self)->&mut T{
        unsafe{ &mut *self.ref_.as_ptr() }
    }

    /// Converts this `RMut<'a, T>` into a `&'a mut T`
    ///
    #[inline(always)]
    pub fn into_mut(self)->&'a mut T{
        unsafe{ &mut *self.ref_.as_ptr() }
    }

    /// Reborrows this `RMut` as a const raw pointer.
    ///
    #[inline]
    pub fn as_ptr(&self)->*const T{
        self.ref_.as_ptr()
    }

    /// Reborrows this `RMut` as a const raw pointer.
    ///
    #[inline]
    pub fn as_mut_ptr(&mut self)->*mut T{
        self.ref_.as_ptr()
    }

    /// Converts this `RMut<'a, T>` into a `*mut T`
    ///
    #[inline]
    pub fn into_raw(self)->*const T{
        self.ref_.as_ptr()
    }

    /// Converts this `RMut<'a, T>` into a `*mut T`
    ///
    #[inline]
    pub fn into_raw_mut(self)->*mut T{
        self.ref_.as_ptr()
    }

    /// Accesses the referenced value as a casted raw pointer.
    #[inline(always)]
    pub fn transmute_into_raw<U>(self)->*mut U{
        self.ref_.as_ptr() as *mut U
    }

    /// Transmutes this `RRefMut<'a,T>` to a `&'a mut U`.
    ///
    #[inline(always)]
    pub unsafe fn transmute_into_mut<U>(self)->&'a mut U
    where
        U:'a,
    {
        &mut *(self.ref_.as_ptr() as *mut U)
    }

    /// Transmutes this `RRefMut<'a,T>` to a `RRefMut<'a,U>`.
    ///
    #[inline(always)]
    pub unsafe fn transmute<U>(self)->RMut<'a,U>
    where
        U:'a,
    {
        RMut::from_raw(
            self.ref_.as_ptr() as *mut U
        )
    }

    /// Reborrows this `RMut<'a, T>` into an RRef<'a, T>
    #[inline(always)]
    pub fn as_rref<'r>(&'r self) -> RRef<'r, T> {
        unsafe{
            RRef::from_raw(self.ref_.as_ptr())
        }
    }

    /// Converts this `RMut<'a, T>` to an RRef<'a, T>
    #[inline(always)]
    pub fn into_rref(self) -> RRef<'a, T> {
        unsafe{
            RRef::from_raw(self.ref_.as_ptr())
        }
    }
}

unsafe impl<'a, T> AsPtr for RMut<'a, T> {
    #[inline(always)]
    fn as_ptr(&self) -> *const T {
        self.ref_.as_ptr() as *const T
    }
}

unsafe impl<'a, T> AsMutPtr for RMut<'a, T> {
    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut T {
        self.ref_.as_ptr()
    }

    #[inline(always)]
    fn as_rmut(&mut self) -> RMut<'_, T> {
        self.reborrow()
    }
}


unsafe impl<'a,T> GetPointerKind for RMut<'a,T>{
    type Kind=PK_MutReference;

    type PtrTarget = T;
}

unsafe impl<'a,T,U> CanTransmuteElement<U> for RMut<'a,T>
where
    U:'a,
{
    type TransmutedPtr= RMut<'a,U>;

    #[inline(always)]
    unsafe fn transmute_element_(self) -> Self::TransmutedPtr {
        self.transmute()
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_test(){
        unsafe{
            let zero: *mut i32 = &mut 3;
            assert_eq!(RMut::from_raw(zero).get_copy(), 3);
        }

        assert_eq!(RMut::new(&mut 99).get_copy(), 99);
    }

    #[test]
    fn access(){
        let mut num = 5;
        let mut mutref= RMut::new(&mut num);
        
        assert_eq!(*mutref.get_mut(), 5);
        *mutref.get_mut() = 21;
        assert_eq!(*mutref.get_mut(), 21);

        assert_eq!(*mutref.get(), 21);
        
        assert_eq!(*mutref.get_mut(), 21);
        *mutref.reborrow().get_mut() = 34;

        unsafe{
            let raw = mutref.reborrow().into_raw_mut();
            assert_eq!(*raw, 34);
            *raw = 55;
        }
        assert_eq!(num, 55);
    }

    #[test]
    fn transmutes(){
        let mut num = !1;
        let mutref= RMut::new(&mut num);

        unsafe{
            let ptr = mutref.transmute_into_raw::<i32>();

            assert_eq!(*ptr, -2);
            *ptr = 55;
        }

        assert_eq!(num, 55);
    }
}


