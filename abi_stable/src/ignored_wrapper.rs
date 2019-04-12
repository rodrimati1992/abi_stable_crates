/*!
Wrapper type(s) where their value is ignored in some trait impls .
*/

use std::{
    ops::{Deref,DerefMut},
    fmt::{self,Debug,Display},
    cmp::{Ordering,Eq,PartialEq,Ord,PartialOrd},
    hash::{Hash,Hasher},
};

/// Wrapper type used to ignore its contents in comparisons.
///
/// It also:
///
/// - replaces the hash of T with the hash of `()`.
///
#[repr(transparent)]
#[derive(Default,Copy,Clone,StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct CmpIgnored<T>{
    pub value:T,
}


impl<T> CmpIgnored<T>{
    pub const fn new(value:T)->Self{
        Self{value}
    }
}


impl<T> From<T> for CmpIgnored<T>{
    fn from(value:T)->Self{
        Self{value}
    }
}


impl<T> Deref for CmpIgnored<T> {
    type Target=T;

    fn deref(&self)->&Self::Target{
        &self.value
    }
}

impl<T> DerefMut for CmpIgnored<T> {
    fn deref_mut(&mut self)->&mut Self::Target{
        &mut self.value
    }
}

impl<T> Display for CmpIgnored<T>
where
    T:Display,
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Display::fmt(&**self,f)
    }
}


impl<T> Debug for CmpIgnored<T>
where
    T:Debug,
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Debug::fmt(&**self,f)
    }
}

impl<T> Eq for CmpIgnored<T> {}


impl<T> PartialEq for CmpIgnored<T> {
    fn eq(&self, _other: &Self) -> bool{
        true
    }
}


impl<T> Ord for CmpIgnored<T>{
    fn cmp(&self, _other: &Self) -> Ordering{
        Ordering::Equal
    }
}


impl<T> PartialOrd for CmpIgnored<T>{
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering>{
        Some(Ordering::Equal)
    }
}


impl<T> Hash for CmpIgnored<T>{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher
    {
        ().hash(state)
    }
}
