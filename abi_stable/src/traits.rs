#[allow(unused_imports)]
use core_extensions::prelude::*;

use crate::{
    erased_types::{GetImplFlags, VirtualWrapperTrait},
    type_info::GetTypeInfo,
    std_types::{RBoxError, RCow, RStr},
};

pub use core_extensions::type_level_bool::{False, True};

/**
An `implementation type`,
with an associated `interface type` which describes the traits that must be implemented by Self.

This trait allows a type to be wrapped in a `VirtualWrapper<_,_>` 
using the `from_value` and `from_ptr`,so as to pass an opaque type across ffi.

# Uniqueness

Users of this trait can't enforce that they are the only ones with the same interface,
therefore they should handle the `Err(..)`s returned
from the `VirtualWrapper::*_unerased` functions whenever
the convert back and forth between `Self` and `Self::Interface`.


*/
pub trait ImplType: Sized + 'static + GetTypeInfo + Send + Sync {
    type Interface: InterfaceType;
}

/**
Defines the usable/required traits when creating a 
`VirtualWrapper<Pointer<OpaqueType< ThisType >>>`
from a type that implements `ImplType<Interface= ThisType >` .

The value of every one of these associated types is `True`/`False`.

On `True`,the trait would be required by and usable in `VirtualWrapper`.

On `False`,the trait would not be required by and not usable in `VirtualWrapper`.

*/
pub trait InterfaceType: Sized + 'static + Send + Sync + GetImplFlags {
    type Clone;

    type Default;

    type Display;

    type Debug;

    type Serialize;

    type Eq;

    type PartialEq;

    type Ord;

    type PartialOrd;

    type Hash;

    type Deserialize;

    // type FmtWrite;
    // type IoWrite;
    // type IoRead;
    // type IoBufRead;
}


/**
Describes how this `implementation type` is serialized.
*/
pub trait SerializeImplType: ImplType {
    fn serialize_impl<'a>(&'a self) -> Result<RCow<'a, str>, RBoxError>;
}

/**
Describes how this `interface type` is deserialized.

Generally this delegates to a library function,so that the implementation can be delegated
to the `implementation crate`.

*/
pub trait DeserializeImplType: InterfaceType<Deserialize = True> {
    type Deserialized: VirtualWrapperTrait<Interface = Self>;

    fn deserialize_impl(s: RStr<'_>) -> Result<Self::Deserialized, RBoxError>;
}

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
