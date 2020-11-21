use crate::marker_type::NonOwningPhantom;

use std::fmt::{self,Debug};


/// Type-level equivalent of the `Option::None` variant.
#[derive(Debug,Copy,Clone)]
pub struct None_;
pub struct Some_<T>(NonOwningPhantom<T>);


///////////////////////////////////////////////////////////////


impl None_{
    pub const NEW:Self=None_;

    pub const fn new()->Self{
        Self::NEW
    }
}


///////////////////////////////////////////////////////////////


/// Type-level equivalent of the `Option::Some` variant.
impl<T> Some_<T>{
    pub const NEW:Self=Some_(NonOwningPhantom::NEW);

    pub const fn new()->Self{
        Self::NEW
    }
}

impl<T> Default for Some_<T>{
    fn default()->Self{
        Self::NEW
    }
}

impl<T> Debug for Some_<T>{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Debug::fmt("Some_",f)
    }
}

impl<T> Copy for Some_<T>{}
impl<T> Clone for Some_<T>{
    fn clone(&self)->Self{
        Self::NEW
    }
}


///////////////////////////////////////////////////////////////



mod sealed {
    use super::*;
    pub trait Sealed {}
    impl Sealed for None_{}
    impl<T> Sealed for Some_<T>{}
}
use self::sealed::Sealed;


///////////////////////////////////////////////////////////////


/// Type-level equivalent of `Option`.
///
/// This trait is sealed and can only be implemente in the abi_stable crate.
pub trait OptionType:Sealed{}


impl OptionType for None_{}

impl<T> OptionType for Some_<T>{}


///////////////////////////////////////////////////////////////


/// To require `None_` in generic contexts.
pub trait NoneTrait:OptionType{}


/// To require `Some_<_>` in generic contexts.
pub trait SomeTrait:OptionType{
    type Value;
}


impl NoneTrait for None_{}

impl<T> SomeTrait for Some_<T>{
    type Value=T;
}


//////////////////////////////////////////////////////////////


/// Type-level equivalent of `Option::unwrap_or`.
pub trait UnwrapOr_<Default_>:OptionType{
    type Output;
}


/// Type-level equivalent of `Option::unwrap_or`.
pub type UnwrapOr<This,Default_>=
    <This as UnwrapOr_<Default_>>::Output;


impl<Default_> UnwrapOr_<Default_> for None_{
    type Output=Default_;
}

impl<T,Default_> UnwrapOr_<Default_> for Some_<T>{
    type Output=T;
}


//////////////////////////////////////////////////////////////


