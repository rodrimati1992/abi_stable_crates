/*!
Where miscellaneous traits reside.
*/

use std::ops::Deref;

#[allow(unused_imports)]
use core_extensions::prelude::*;

use crate::pointer_trait::TransmuteElement;


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
        P:TransmuteElement<Self>
    {
        p.transmute_element(Self::T)
    }

    #[inline]
    unsafe fn into_unerased<P>(p:P)->P::TransmutedPtr
    where 
        P:Deref<Target=Self>,
        P:TransmuteElement<Self::Unerased>,
    {
        p.transmute_element(Self::Unerased::T)
    }


    #[inline]
    unsafe fn run_as_unerased<P,F,R>(p:P,func:F)->R
    where 
        P:Deref<Target=Self>,
        P:TransmuteElement<Self::Unerased>,
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




/**
Type used as the inline storage of a RSmallBox<>/NonExhaustive<>.

# Safety

Implementors must:

- Be types for which all bitpatterns are valid.

- Not implement Drop,and have no drop glue.

*/
pub unsafe trait InlineStorage{}


macro_rules! impl_for_arrays {
    ( ty=$ty:ty , len[ $($len:expr),* $(,)* ] ) => (
        $(
            unsafe impl InlineStorage for [$ty;$len] {}
        )*
    )
}


impl_for_arrays!{
    ty=u8,
    len[
        0,1,2,3,4,5,6,7,8,9,
        10,11,12,13,14,15,16,17,18,19,
        20,21,22,23,24,25,26,27,28,29,
        30,31,32,33,34,35,36,37,38,39,
        40,41,42,43,44,45,46,47,48,49,
        50,51,52,53,54,55,56,57,58,59,
        60,61,62,63,64,
    ]
}

impl_for_arrays!{
    ty=u32,
    len[
        0,1,2,3,4,5,6,7,8,9,
        10,11,12,13,14,15,16,17,18,19,
        20,21,22,23,24,25,26,27,28,29,
        30,31,32,33,34,35,36,37,38,39,
        40,41,42,43,44,45,46,47,48,
    ]
}

impl_for_arrays!{
    ty=u64,
    len[
        0,1,2,3,4,5,6,7,8,9,
        10,11,12,13,14,15,16,17,18,19,
        20,21,22,23,24,
    ]
}

impl_for_arrays!{
    ty=usize,
    len[
        0,1,2,3,4,5,6,7,8,9,
        10,11,12,13,14,15,16,17,18,19,
        20,21,22,23,24,25,26,27,28,29,
        30,31,32,33,34,35,36,37,38,39,
        40,41,42,43,44,45,46,47,48,
    ]
}


macro_rules! declare_alignments {
    (
        $(( $aligner:ident, $alignment:expr ),)*
    ) => (
        $(
            #[repr(C)]
            #[repr(align($alignment))]
            pub struct $aligner<Inline>{
                inline:Inline,
            }
            
            unsafe impl<Inline> InlineStorage for $aligner<Inline>
            where
                Inline:InlineStorage,
            {}
        )*
    )
}


/// Helper types related to the alignemnt of inline storage.
pub mod alignment{
    use super::*;
    
    declare_alignments!{
        ( AlignTo1,1 ),
        ( AlignTo2,2 ),
        ( AlignTo4,4 ),
        ( AlignTo8,8 ),
        ( AlignTo16,16 ),
        ( AlignTo32,32 ),
        ( AlignTo64,64 ),
        ( AlignTo128,128 ),
    }
}

