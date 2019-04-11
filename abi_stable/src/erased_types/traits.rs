
/*!
Traits for types wrapped in `VirtualWrapper<_>`
*/

use crate::{
    erased_types::{GetImplFlags, VirtualWrapperTrait},
    type_info::GetTypeInfo,
    std_types::{RBoxError, RCow, RStr},
};

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