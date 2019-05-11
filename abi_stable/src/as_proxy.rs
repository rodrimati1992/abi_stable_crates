/*
Right now this doesn't work,
wait at least a while after generic associated types are usable before trying this again.
*/

use std::{
    cmp::{Eq,PartialEq},
    hash::Hash,
};

use core_extensions::TypeIdentity;

use crate::{
    std_types::*,
};

pub trait AsProxy<'a>{
    type Proxy:'a;
    
    fn as_proxy(&'a self)->Self::Proxy;
}


pub type AsProxy_<'a,This>=
    <This as AsProxy<'a>>::Proxy;


impl<'a> AsProxy<'a> for RString {
    type Proxy=RStr<'a>;
    
    fn as_proxy(&'a self)->RStr<'a>{
        self.as_rstr()
    }
}

impl<'a,'b> AsProxy<'b> for RStr<'a> {
    type Proxy=RStr<'b>;
    
    fn as_proxy(&'b self)->RStr<'b>{
        *self
    }
}



impl<'a,'b:'a> AsProxy<'b> for RCow<'a,str>{
    type Proxy= RStr<'b> ;
    
    fn as_proxy(&'b self)-> RStr<'b>{
        self.borrowed()
    }
}

impl<'a,'b:'a,T> AsProxy<'b> for RCow<'a,[T]>
where
    T:Clone+'b,
{
    type Proxy= RSlice<'b,T> ;
    
    fn as_proxy(&'b self)-> RSlice<'b,T>{
        self.borrowed()
    }
}

impl<'a,'b:'a,T> AsProxy<'b> for RCow<'a,T>
where
    T:Clone+'b,
{
    type Proxy= &'b T ;
    
    fn as_proxy(&'b self)-> &'b T {
        self.borrowed()
    }
}



impl<'a,T:'a> AsProxy<'a> for RVec<T>{
    type Proxy=RSlice<'a,T>;
    
    fn as_proxy(&'a self)->RSlice<'a,T>{
        self.as_rslice()
    }
}

impl<'a,'b,T:'b> AsProxy<'b> for RSlice<'a,T>{
    type Proxy=RSlice<'b,T>;
    
    fn as_proxy(&'b self)->RSlice<'b,T>{
        *self
    }
}


///////////////////////////////////////////////////////////////////////////////


#[repr(transparent)]
#[derive(Debug,PartialEq,Eq,Ord,PartialOrd,Hash)]
pub struct IdentityProxy<T>(T);


impl IdentityProxy<()>{
    #[inline]
    pub fn from_ref<'a,T>(value:&'a T)->&'a IdentityProxy<T>{
        unsafe{
            &*(value as *const T as *const IdentityProxy<T>)
        }
    }

    #[inline]
    pub fn from_mut<'a,T>(value:&'a mut T)->&'a mut IdentityProxy<T>{
        unsafe{
            &mut *(value as *mut T as *mut IdentityProxy<T>)
        }
    }
}

impl<T> IdentityProxy<T>{
    #[inline]
    pub fn into_inner(self)->T{
        self.0
    }
}


impl<T> From<T> for IdentityProxy<T>{
    #[inline]
    fn from(value:T)->Self{
        IdentityProxy(value)
    }
}


impl<'b,T:'b> AsProxy<'b> for IdentityProxy<T>{
    type Proxy= &'b T ;
    
    #[inline]
    fn as_proxy(&'b self)-> &'b T {
        &self.0
    }
}




///////////////////////////////////////////////////////////////////////////////


use crate::{
    StableAbi,
    derive_macro_reexports::*,
};

use std::{fmt,marker::PhantomData};


/// A marker type which pretends to have the layout of `AsProxy_<T>`.
#[repr(C)]
#[derive(PartialEq,Eq,Ord,PartialOrd,Hash)]
pub struct PhantomProxy<T>(PhantomData<T>);



impl<T> Copy for PhantomProxy<T>{}
impl<T> Clone for PhantomProxy<T>{
    fn clone(&self)->Self{
        *self
    }
}

impl<T> fmt::Debug for PhantomProxy<T>{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        fmt::Debug::fmt("PhantomProxy",f)
    }
}

impl<T> PhantomProxy<T>{
    pub const NEW:Self=PhantomProxy(PhantomData);
}


unsafe impl<'a,T> SharedStableAbi for PhantomProxy<T>
where
    T:StableAbi+'a+AsProxy<'a>,
    AsProxy_<'a,T>:StableAbi,
{
    type IsNonZeroType=False;
    type Kind=ValueKind;
    type StaticEquivalent=PhantomProxy<T::StaticEquivalent>;
    const S_LAYOUT: &'static TypeLayout=AsProxy_::<'a,T>::S_LAYOUT;
}
