use std::{
    ops::{Deref},
    fmt::{self,Display},
    marker::PhantomData,
    ptr::NonNull,
};

use crate::{
    pointer_trait::{CanTransmuteElement,GetPointerKind,PK_Reference},
    utils::ref_as_nonnull,
};

/**
A StableAbi type equivalent to `&'a T`,
defined as a workaround to allow casting from `&T` to `&U` inside a `const fn`
in stable Rust.
*/
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
        Display::fmt(&**self,f)
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
    pub const unsafe fn from_raw(ref_:*const T)->Self
    where
        T:'a,
    {
        Self{
            ref_: NonNull::new_unchecked(ref_ as *mut T),
            _marker:PhantomData,
        }
    }

    /// Constructs this RRef from a static reference
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
    pub const fn new(ref_:&'a T)->Self{
        Self{
            ref_: ref_as_nonnull(ref_),
            _marker:PhantomData,
        }
    }

    /// Gets access to the reference.
    ///
    /// Use this to get a `&'a T`,
    /// instead of a reference borrowing from the pointer.
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
    pub fn get(self)->&'a T{
        unsafe{ &*(self.ref_.as_ptr() as *const T) }
    }

    /// Gets access to the referenced value,as a raw pointer.
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
    ///     const NONE_REF:&'a Option<T>=&None;
    ///
    ///     const STATIC:RRef<'a,Option<T>>=
    ///         RRef::new(Self::NONE_REF);
    /// }
    ///
    /// let reference:*const Option<Infallible>=
    ///     GetPtr::<Infallible>::STATIC.get_raw();
    ///
    /// ```
    pub const fn get_raw(self)->*const T{
        self.ref_.as_ptr() as *const T
    }

    /// Transmutes this `RRef<'a,T>` to a `RRef<'b,U>`.
    ///
    /// # Safety
    ///
    /// This has the same safety problems that transmuting `&'a T` to `&'b U` has.
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
    /// let reference:RRef<'static,Option<[();0xFFF_FFFF]>>=unsafe{
    ///     GetPtr::<()>::STATIC
    ///         .transmute::<'static,Option<[();0xFFF_FFFF]>>()
    /// };
    ///
    /// ```
    pub const unsafe fn transmute<'b,U>(self)->RRef<'b,U>
    where
        U:'b,
    {
        RRef::from_raw(
            self.ref_.as_ptr() as *const U
        )
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
    ///         .transmute_ref::<[();0xFFF_FFFF]>()
    /// };
    ///
    /// ```
    pub const unsafe fn transmute_ref<U>(self)->RRef<'a,U>
    where
        U:'a,
    {
        RRef::from_raw(
            self.ref_.as_ptr() as *const U
        )
    }
}

impl<'a,T> Deref for RRef<'a,T>{
    type Target=T;

    #[inline(always)]
    fn deref(&self)->&T{
        self.get()
    }
}


unsafe impl<'a,T> GetPointerKind for RRef<'a,T>{
    type Kind=PK_Reference;
}

unsafe impl<'a,T,U> CanTransmuteElement<U> for RRef<'a,T>
where
    U:'a,
{
    type TransmutedPtr= RRef<'a,U>;
}
