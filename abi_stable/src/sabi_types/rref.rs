use std::{
    fmt::{self,Display},
    marker::PhantomData,
    ptr::NonNull,
};

use crate::{
    pointer_trait::{AsPtr, CanTransmuteElement,GetPointerKind,PK_Reference},
    utils::ref_as_nonnull,
};

/// Equivalent to `&'a T`,
/// which allows a few more operations without causing Undefined Behavior.
#[repr(transparent)]
#[derive(StableAbi)]
#[sabi(
    bound="T:'a",
)]
pub struct RRef<'a,T>{
    ref_: NonNull<T>,
    _marker:PhantomData<&'a T>,
}

impl<'a,T> Display for RRef<'a,T>
where
    T:Display
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Display::fmt(self.get(), f)
    }
}


impl<'a,T> Clone for RRef<'a,T>{
    fn clone(&self)->Self{
        *self
    }
}

impl<'a,T> Copy for RRef<'a,T>{}

unsafe impl<'a,T> Sync for RRef<'a,T>
where &'a T:Sync
{}

unsafe impl<'a,T> Send for RRef<'a,T>
where &'a T:Send
{}


shared_impls! {
    mod=static_ref_impls
    new_type=RRef['a][T],
    original_type=AAAA,
    deref_approach=(method = get),
}


impl<'a,T> RRef<'a,T>{
    /// Constructs this RRef from a raw pointer.
    ///
    /// # Safety
    ///
    /// You must ensure that the raw pointer is valid for the `'a` lifetime.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::RRef;
    ///
    /// struct GetPtr<'a,T>(&'a T);
    ///
    /// impl<'a,T:'a> GetPtr<'a,T>{
    ///     const PTR:*const Option<T>=&None;
    ///
    ///     const STATIC:RRef<'a,Option<T>>=unsafe{
    ///         RRef::from_raw(Self::PTR)
    ///     };
    /// }
    ///
    /// ```
    #[inline(always)]
    pub const unsafe fn from_raw(ref_:*const T)->Self
    where
        T:'a,
    {
        Self{
            ref_: NonNull::new_unchecked(ref_ as *mut T),
            _marker:PhantomData,
        }
    }

    /// Constructs this RRef from a reference.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::RRef;
    ///
    /// struct GetPtr<'a,T>(&'a T);
    ///
    /// impl<'a,T:'a> GetPtr<'a,T>{
    ///     const REF:&'a Option<T>=&None;
    ///
    ///     const STATIC:RRef<'a,Option<T>>=
    ///         RRef::new(Self::REF);
    /// }
    ///
    /// ```
    #[inline(always)]
    pub const fn new(ref_:&'a T)->Self{
        Self{
            ref_: ref_as_nonnull(ref_),
            _marker:PhantomData,
        }
    }

    /// Casts this to an equivalent reference.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::RRef;
    ///
    /// struct GetPtr<'a,T>(&'a T);
    ///
    /// impl<'a,T:'a> GetPtr<'a,T>{
    ///     const NONE_REF:&'a Option<T>=&None;
    ///
    ///     const REFERENCE:RRef<'a,Option<T>>=RRef::new(Self::NONE_REF);
    ///
    ///     // This returns a reference that lives as long as T does
    ///     fn returns_ref()->&'a Option<String>{
    ///         let reference=GetPtr::<String>::REFERENCE;
    ///         reference.get()
    ///     }
    ///
    ///     // This doesn't work,it borrows the reference `variable`.
    ///     // fn returns_ref_2()->&'a Option<String>{
    ///     //     let reference=GetPtr::<String>::REFERENCE;
    ///     //     &*reference
    ///     // }
    /// }
    ///
    /// ```
    #[inline(always)]
    pub fn get(self) -> &'a T{
        unsafe{ &*(self.ref_.as_ptr() as *const T) }
    }

    /// Copies the value that this points to.
    #[inline(always)]
    pub fn get_copy(self) -> T 
    where 
        T: Copy
    {
        unsafe{ *(self.ref_.as_ptr() as *const T) }
    }

    /// Casts this to an equivalent raw pointer.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::RRef;
    /// use std::convert::Infallible;
    ///
    /// struct GetPtr<'a,T>(&'a T);
    ///
    /// impl<'a,T:'a> GetPtr<'a,T>{
    ///     const NONE_REF: &'a Option<T> = &None;
    ///
    ///     const STATIC: RRef<'a,Option<T>> =
    ///         RRef::new(Self::NONE_REF);
    /// }
    ///
    /// let reference: *const Option<Infallible>=
    ///     GetPtr::<Infallible>::STATIC.as_ptr();
    ///
    /// ```
    #[inline(always)]
    pub const fn as_ptr(self) -> *const T{
        self.ref_.as_ptr() as *const T
    }

    /// Transmutes this `RRef<'a,T>` to a `RRef<'a,U>`.
    ///
    /// This is equivalent to calling `transmute`,
    /// except that it doesn't change the lifetime parameter of `RRef`.
    ///
    /// # Safety
    ///
    /// This has the same safety problems that transmuting `&'a T` to `&'a U` has.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::RRef;
    ///
    /// struct GetPtr<'a,T>(&'a T);
    ///
    /// impl<'a,T:'a> GetPtr<'a,T>{
    ///     const PTR:*const Option<T>=&None;
    ///
    ///     const STATIC:RRef<'a,Option<T>>=unsafe{
    ///         RRef::from_raw(Self::PTR)
    ///     };
    /// }
    ///
    /// let reference:RRef<'static,[();0xFFF_FFFF]>=unsafe{
    ///     GetPtr::<'static,()>::STATIC
    ///         .transmute::<[();0xFFF_FFFF]>()
    /// };
    ///
    /// ```
    #[inline(always)]
    pub const unsafe fn transmute<U>(self)->RRef<'a,U>
    where
        U:'a,
    {
        RRef::from_raw(
            self.ref_.as_ptr() as *const U
        )
    }

    /// Transmutes this to a raw pointer pointing to a different type.
    #[inline(always)]
    pub const fn transmute_into_raw<U>(self)->*const U{
        self.ref_.as_ptr() as *const T as *const U
    }

    /// Transmutes this to a reference pointing to a different type.
    #[inline(always)]
    pub unsafe fn transmute_into_ref<U>(self) -> &'a U 
    where
        U: 'a
    {
        &*(self.ref_.as_ptr() as *const T as *const U)
    }

}

unsafe impl<'a,T> GetPointerKind for RRef<'a,T>{
    type Kind=PK_Reference;

    type PtrTarget = T;
}

unsafe impl<'a,T,U> CanTransmuteElement<U> for RRef<'a,T>
where
    U:'a,
{
    type TransmutedPtr = RRef<'a,U>;

    #[inline(always)]
    unsafe fn transmute_element_(self) -> Self::TransmutedPtr {
        self.transmute()
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
    fn construction_test(){
        unsafe{
            let three: *const i32 = &3;
            assert_eq!(RRef::from_raw(three).get_copy(), 3);
        }

        assert_eq!(RRef::new(&5).get_copy(), 5);
    }

    #[test]
    fn access(){
        let reference = RRef::new(&8);
        
        assert_eq!(*reference.get(), 8);
        unsafe{
            assert_eq!(*reference.as_ptr(), 8);
        }
    }

    #[test]
    fn transmutes(){
        let reference = RRef::new(&(!0u32));

        unsafe{
            assert_eq!(*reference.transmute_into_raw::<i32>(), -1);
            assert_eq!(reference.transmute::<i32>().get_copy(), -1);
        }
    }
}


