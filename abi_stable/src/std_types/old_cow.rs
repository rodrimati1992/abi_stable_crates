use std::{borrow::Cow, fmt, ops::Deref};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use crate::{
    StableAbi, 
    SharedStableAbi, 
    std_types::{RSlice, RStr, RString, RVec},
};

////////////////////////////////////////////////////////////////////


/// The ffi-safe borrowed and owned types this is associated with,
/// as well as conversions to and from those types.
///
/// # Safety
///
/// The StaticEquivalent associated type must be implemented correctly.
///
pub unsafe trait BorrowOwned<'a>:'a+Copy {
    type ROwned;
    type Borrowed:'a+?Sized+ToOwned;

    fn to_rowned(self)->Self::ROwned;
    fn from_rowned(owned:&'a Self::ROwned)->Self;
    
    fn rowned_to_owned(owned:Self::ROwned)-><Self::Borrowed as ToOwned>::Owned;
    fn owned_to_rowned(owned:<Self::Borrowed as ToOwned>::Owned)->Self::ROwned;

    fn from_borrowed(this:&'a Self::Borrowed)->Self;
    
    fn deref_borrowed(self)->&'a Self::Borrowed;
    fn deref_owned(owned:&Self::ROwned)->&Self::Borrowed;
}

unsafe impl<'a> BorrowOwned<'a> for RStr<'a> {
    type ROwned = RString;
    type Borrowed = str;
    

    #[inline]
    fn to_rowned(self)->Self::ROwned{
        self.into()
    }

    #[inline]
    fn from_rowned(owned:&'a Self::ROwned)->Self{
        owned.as_rstr()
    }

    #[inline]
    fn rowned_to_owned(owned:Self::ROwned)->String{
        owned.into()
    }

    
    #[inline]
    fn owned_to_rowned(owned:String)->Self::ROwned {
        owned.into()
    }

    fn from_borrowed(this:&'a Self::Borrowed)->Self{
        this.into()
    }

    #[inline]
    fn deref_borrowed(self)->&'a Self::Borrowed{
        self.as_str()
    }

    #[inline]
    fn deref_owned(owned:&Self::ROwned)->&Self::Borrowed{
        owned
    }
}

unsafe impl<'a, T: 'a> BorrowOwned<'a> for RSlice<'a, T>
where
    T: StableAbi+Clone,
{
    type Borrowed = [T];
    type ROwned = RVec<T>;
    
    #[inline]
    fn to_rowned(self)->Self::ROwned{
        self.to_vec().into()
    }

    #[inline]
    fn from_rowned(owned:&'a Self::ROwned)->Self{
        owned.as_rslice()
    }

    #[inline]
    fn rowned_to_owned(owned:Self::ROwned)->Vec<T>{
        owned.into()
    }
    
    #[inline]
    fn owned_to_rowned(owned:Vec<T>)->Self::ROwned {
        owned.into()
    }

    fn from_borrowed(this:&'a Self::Borrowed)->Self{
        this.into()
    }

    #[inline]
    fn deref_borrowed(self)->&'a Self::Borrowed{
        self.as_slice()
    }

    #[inline]
    fn deref_owned(owned:&Self::ROwned)->&Self::Borrowed{
        owned
    }   
}

unsafe impl<'a, T: 'a> BorrowOwned<'a> for &'a T
where
    T: StableAbi+Clone,
{
    type Borrowed = T;
    type ROwned = T;
    
    #[inline]
    fn to_rowned(self)->Self::ROwned{
        self.clone()
    }

    #[inline]
    fn from_rowned(owned:&'a Self::ROwned)->Self{
        owned
    }

    #[inline]
    fn rowned_to_owned(owned:Self::ROwned)->T{
        owned
    }

    #[inline]
    fn owned_to_rowned(owned:T)->Self::ROwned {
        owned
    }

    fn from_borrowed(this:&'a Self::Borrowed)->Self{
        this
    }

    #[inline]
    fn deref_borrowed(self)->&'a Self::Borrowed{
        self
    }

    #[inline]
    fn deref_owned(owned:&Self::ROwned)->&Self::Borrowed{
        owned
    }   
}

////////////////////////////////////////////////////////////////////


/// Ffi-safe equivalent of ::std::borrow::Cow.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
#[sabi(bound="<B as SharedStableAbi>::StaticEquivalent: BorrowOwned<'static>")]
pub enum RCow<'a, B,O=<B as BorrowOwned<'a>>::ROwned>
where
    B: BorrowOwned<'a>,
{
    Borrowed(B),
    Owned(O),
    __Impossible{
        ref_:&'a(),
        wtf:core_extensions::Void,
    },
}

use self::RCow::{Borrowed, Owned};

///////////////////////////////////////////////////////////////////////////

impl<'a, B> RCow<'a, B>
where
    B: BorrowOwned<'a>,
{
    /// Get a mutable reference to the owner form of RCow,
    /// converting to the owned form if it is currently the borrowed form.
    pub fn to_mut(&mut self) -> &mut B::ROwned {
        if let Borrowed(v) = *self {
            let owned = B::to_rowned(v);
            *self = Owned(owned)
        }
        match self {
            Borrowed(_) => loop {},
            Owned(v) => v,
            RCow::__Impossible{wtf,..}=>wtf.to(),
        }
    }
    /// Unwraps into the owned owner form of RCow,
    /// converting to the owned form if it is currently the borrowed form.
    pub fn into_owned(self) -> B::ROwned {
        match self {
            Borrowed(x) => B::to_rowned(x),
            Owned(x) => x,
            RCow::__Impossible{wtf,..}=>wtf.to(),
        }
    }
}

impl<'a, B> Copy for RCow<'a, B>
where
    B: BorrowOwned<'a>,
    B::ROwned: Copy,
{
}

impl<'a, B> Clone for RCow<'a, B>
where
    B: BorrowOwned<'a>,
    B::ROwned: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Borrowed(x) => Borrowed(x.clone()),
            Owned(x) => Owned((*x).clone()),
            RCow::__Impossible{wtf,..}=>wtf.to(),
        }
    }
}

impl<'a, B> Deref for RCow<'a, B>
where
    B: BorrowOwned<'a>,
{
    type Target = B::Borrowed;
    
    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Borrowed(x) => B::deref_borrowed(*x),
            Owned(x) => B::deref_owned(x),
            RCow::__Impossible{wtf,..}=>wtf.to(),
        }
    }
}

////////////////////


impl<'a,B> AsRef<B::Borrowed> for RCow<'a, B>
where
    B: BorrowOwned<'a>,
{
    fn as_ref(&self)->&B::Borrowed{
        self
    }
}

////////////////////////////

shared_impls! {
    mod=slice_impls
    new_type=RCow['a][] 
    extra[B]
    constrained[B::Borrowed]
    where [ B:BorrowOwned<'a> ],
    original_type=void,
}

impl_into_rust_repr! {
    impl['a,B] Into<Cow<'a,B::Borrowed>> for RCow<'a,B>
    where[
        B: BorrowOwned<'a>,
    ]{
        fn(this){
            match this{
                RCow::Borrowed(x)=>x.piped(B::deref_borrowed).piped(Cow::Borrowed),
                RCow::Owned(x)=>x.piped(B::rowned_to_owned).piped(Cow::Owned),
                RCow::__Impossible{wtf,..}=>wtf.to(),
            }
        }
    }
}

impl<'a,B> From<Cow<'a,B::Borrowed>> for RCow<'a,B>
where
    B: BorrowOwned<'a>,
{
    fn from(this:Cow<'a,B::Borrowed>)->Self{
        match this{
            Cow::Borrowed(x)=>x.piped(B::from_borrowed).piped(RCow::Borrowed),
            Cow::Owned(x)=>x.piped(B::owned_to_rowned).piped(RCow::Owned),
        }
    }
}

impl<'a, B> fmt::Display for RCow<'a, B>
where
    B: BorrowOwned<'a> ,
    B::Borrowed: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<'de, 'a, B> Deserialize<'de> for RCow<'a, B>
where
    B: BorrowOwned<'a>,
    Cow<'a, B::Borrowed>: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Cow<'a, B::Borrowed> as Deserialize<'de>>::deserialize(deserializer)
            .map(From::from)
    }
}

impl<'a, B> Serialize for RCow<'a, B>
where
    B: BorrowOwned<'a>,
    B::Borrowed: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (&**self).serialize(serializer)
    }
}

//////////////////////////////////////////////////////////////////////////////////////
