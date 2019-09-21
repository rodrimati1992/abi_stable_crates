/*!
Where miscellaneous traits reside.
*/

use std::ops::Deref;

#[allow(unused_imports)]
use core_extensions::prelude::*;

use crate::pointer_trait::{CanTransmuteElement,TransmuteElement};


///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////

/// Converts a #[repr(Rust)] type into its #[repr(C)] equivalent.
pub trait IntoReprC {
    type ReprC;

    fn into_c(self) -> Self::ReprC;
}

/// Converts a #[repr(C)] type into its #[repr(Rust)] equivalent.
pub trait IntoReprRust {
    type ReprRust;

    fn into_rust(self) -> Self::ReprRust;
}

///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////

macro_rules! impl_from_rust_repr {
    (
        $(#[$meta:meta])*
        impl$([ $($impl_header:tt)* ])? From<$from_ty:ty> for $into_ty:ty
        $( where [ $( $where_clause:tt )* ] )?
        {
            fn($this:pat) $function_contents:block
        }


    ) => (
        $(#[$meta])*
        impl $(< $($impl_header)* >)? From<$from_ty> for $into_ty
        $(where $($where_clause)*)?
        {
            #[inline]
            fn from($this:$from_ty)->$into_ty{
                $function_contents
            }
        }

        $(#[$meta])*
        impl $(< $($impl_header)* >)?  $crate::traits::IntoReprC for $from_ty
        $(where $($where_clause)*)?
        {
            type ReprC=$into_ty;
            #[inline]
            fn into_c(self)->Self::ReprC{
                self.into()
            }
        }
    )
}

macro_rules! impl_into_rust_repr {
    (
        $(#[$meta:meta])*
        impl$([ $($impl_header:tt)* ])? Into<$into_ty:ty> for $from_ty:ty
        $( where [ $( $where_clause:tt )* ] )?
        {
            fn($this:pat){
                $($function_contents:tt)*
            }
        }

    ) => (
        $(#[$meta])*
        impl $(< $($impl_header)* >)?  Into<$into_ty> for $from_ty
        $(where $($where_clause)*)?
        {
            #[inline]
            fn into(self)->$into_ty{
                let $this=self;
                $($function_contents)*
            }
        }

        $(#[$meta])*
        impl $(< $($impl_header)* >)?  $crate::traits::IntoReprRust for $from_ty
        $(where $($where_clause)*)?
        {
            type ReprRust=$into_ty;
            #[inline]
            fn into_rust(self)->Self::ReprRust{
                self.into()
            }
        }
    )
}


///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////



pub(crate) unsafe trait ErasedType<'a>:Sized{
    type Unerased;

    #[inline]
    unsafe fn from_unerased<P>(p:P)->P::TransmutedPtr
    where 
        P:Deref<Target=Self::Unerased>,
        P:CanTransmuteElement<Self>
    {
        p.transmute_element::<Self>()
    }

    #[inline]
    unsafe fn into_unerased<P>(p:P)->P::TransmutedPtr
    where 
        P:Deref<Target=Self>,
        P:CanTransmuteElement<Self::Unerased>,
    {
        p.transmute_element::<Self::Unerased>()
    }


    #[inline]
    unsafe fn run_as_unerased<P,F,R>(p:P,func:F)->R
    where 
        P:Deref<Target=Self>,
        P:CanTransmuteElement<Self::Unerased>,
        F:FnOnce(P::TransmutedPtr)->R,
    {
        func(Self::into_unerased(p))
    }


}



///////////////////////////////////////////////////////////////////////////

/// Unwraps a type into its owned value.
pub trait IntoInner{
    /// The type of the value this owns.
    type Element;

    /// Unwraps this type into its owned value.
    fn into_inner_(self)->Self::Element;
}


///////////////////////////////////////////////////////////////////////////




