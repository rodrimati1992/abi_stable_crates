/*!
Where most traits reside.
*/

#[allow(unused_imports)]
use core_extensions::prelude::*;

pub use core_extensions::type_level_bool::{False, True};


///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////

/// Converts a #[repr(Rust)] type into its #[repr(C)] equivalent.
pub trait IntoReprC {
    type ReprC;

    fn into_c(self) -> Self::ReprC;
}

/// Converts a #[repr(Rust)] type into its #[repr(C)] equivalent.
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
