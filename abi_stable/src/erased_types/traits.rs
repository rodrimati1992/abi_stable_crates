
/*!
Traits for types wrapped in `DynTrait<_>`
*/

use std::{mem,marker::PhantomData};

use crate::{
    erased_types::{DynTraitBound},
    sabi_types::MaybeCmp,
    std_types::{
        RBoxError, 
        RCow, RStr,StaticStr,
        utypeid::{UTypeId,no_utypeid,some_utypeid},
    },
    version::VersionStrings,
    return_value_equality::ReturnValueEquality,
};

use super::TypeInfo;

#[allow(unused_imports)]
use crate::type_level::bools::{False, True};

/**
An `implementation type`,
with an associated `interface type` which describes the traits that 
must be implemented when constructing a `DynTrait` from Self,
using the `from_value` and `from_ptr` constructors,
so as to pass an opaque type across ffi.

To initialize `INFO` you can use the `impl_get_type_info` macro.

# Uniqueness

Users of this trait can't enforce that they are the only ones with the same interface,
therefore they should handle the `Err(..)`s returned
from the `DynTrait::*_unerased` functions whenever
the convert back and forth between `Self` and `Self::Interface`.


*/
pub trait ImplType: Sized  {
    type Interface: InterfaceType;

    const INFO: &'static TypeInfo;
}


macro_rules! declare_InterfaceType {
    (

        $(#[$attrs:meta])*

        assoc_types[ 
            $( 
                $(#[$assoc_attrs:meta])*
                type $trait_:ident ;
            )* 
        ]
    ) => (
        $(#[$attrs])*
        pub trait InterfaceType: Sized {
            $(
                $(#[$assoc_attrs])*
                type $trait_;
            )*

            #[doc(hidden)]
            type define_this_in_the_impl_InterfaceType_macro;
        }


    )
}


declare_InterfaceType!{


/**
Defines the usable/required traits when creating a 
`DynTrait<Pointer<()>,ThisInterfaceType>`
from a type that implements `ImplType<Interface= ThisInterfaceType >` .

This trait can only be implemented within the `impl_InterfaceType` macro,
giving a default value to each associated type,
so that adding associated types is not a breaking change.

The value of every associated type is `True`/`False`.

On `True`,the trait would be required by and usable in `DynTrait`.

On `False`,the trait would not be required by and not usable in `DynTrait`.

# Example

```

use abi_stable::{
    StableAbi,
    impl_InterfaceType,
    erased_types::InterfaceType,
    type_level::bools::*,
};

#[repr(C)]
#[derive(StableAbi)]
pub struct FooInterface;

impl_InterfaceType!{
    impl InterfaceType for FooInterface {
        type Clone=True;

        type Debug=True;

        /////////////////////////////////////    
        //// defaulted associated types
        /////////////////////////////////////

        // Changing this to require/unrequire in minor versions,is an abi breaking change.
        // type Send=True;

        // Changing this to require/unrequire in minor versions,is an abi breaking change.
        // type Sync=True;

        // type Iterator=False;

        // type DoubleEndedIterator=False;

        // type Default=False;

        // type Display=False;

        // type Serialize=False;

        // type Eq=False;

        // type PartialEq=False;

        // type Ord=False;

        // type PartialOrd=False;

        // type Hash=False;

        // type Deserialize=False;

        // type FmtWrite=False;
        
        // type IoWrite=False;
        
        // type IoSeek=False;
        
        // type IoRead=False;

        // type IoBufRead=False;
    }
}

# fn main(){}


```


*/


    assoc_types[
        /// Changing this to require/unrequire in minor versions,is an abi breaking change.
        type Send;

        /// Changing this to require/unrequire in minor versions,is an abi breaking change.
        type Sync;

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

        type Iterator;
        
        type DoubleEndedIterator;

        type FmtWrite;
        
        type IoWrite;
        
        type IoSeek;
        
        type IoRead;

        type IoBufRead;
    ]


}



/**
Describes how this `implementation type` is serialized.
*/
pub trait SerializeImplType {
    fn serialize_impl<'a>(&'a self) -> Result<RCow<'a, str>, RBoxError>;
}

/**
Describes how this `interface type` is deserialized,
not borrowing from the input RStr.

Generally this delegates to a library function,
so that the implementation can be delegated
to the `implementation crate`.

*/
pub trait DeserializeOwnedInterface<'borr>: InterfaceType<Deserialize = True> {
    type Deserialized: DynTraitBound<'borr,Interface = Self>+'borr;

    fn deserialize_impl(s: RStr<'_>) -> Result<Self::Deserialized, RBoxError>;
}

/**
Describes how this `interface type` is deserialized,
borrowing from the input RStr.

Generally this delegates to a library function,
so that the implementation can be delegated
to the `implementation crate`.

*/
pub trait DeserializeBorrowedInterface<'borr>: InterfaceType<Deserialize = True> {
    type Deserialized: DynTraitBound<'borr,Interface = Self>+'borr;

    fn deserialize_impl(s: RStr<'borr>) -> Result<Self::Deserialized, RBoxError> ;
}



/////////////////////////////////////////////////////////////////////


/// The way to specify the expected Iterator::Item type for an InterfaceType.
///
/// This is a separate trait to allow iterators that yield borrowed elements.
pub trait IteratorItem<'a>:InterfaceType{
    type Item:'a;
}



/// Gets the Item type of an Iterator.
///
/// Used by `DynTrait`'s vtable to give its iter a default type,
/// when `I:InterfaceType<Iterator=False>`.
pub trait IteratorItemOrDefault<'borr,ImplIsRequired>:InterfaceType{
    type Item:'borr;
}


impl<'borr,I,Item> IteratorItemOrDefault<'borr,True> for I
where
    I:InterfaceType+IteratorItem<'borr,Item=Item>,
    Item:'borr,
{
    type Item=Item;
}


impl<'borr,I> IteratorItemOrDefault<'borr,False> for I
where I:InterfaceType
{
    type Item=();
}


//////////////////////////////////////////////////////////////////


pub use self::interface_for::InterfaceFor;

pub mod interface_for{
    use super::*;

    /// Helper struct to get an `ImplType` implementation for any type.
    pub struct InterfaceFor<T,Interface,IsStatic>(
        PhantomData<fn()->(T,Interface,IsStatic)>
    );

    impl<T,Interface,IsStatic> ImplType for InterfaceFor<T,Interface,IsStatic>
    where 
        Interface:InterfaceType,
        T:GetUTID<IsStatic>,
    {
        type Interface=Interface;
        
        /// The `&'static TypeInfo` constant,used when unerasing `DynTrait`s into a type.
        const INFO:&'static TypeInfo=&TypeInfo{
            size:mem::size_of::<T>(),
            alignment:mem::align_of::<T>(),
            _uid:<T as GetUTID<IsStatic>>::UID,
            name:StaticStr::new("<erased>"),
            file:StaticStr::new("<unavailable>"),
            package:StaticStr::new("<unavailable>"),
            package_version:VersionStrings{
                major:StaticStr::new("99"),
                minor:StaticStr::new("99"),
                patch:StaticStr::new("99"),
            },
            _private_field:(),
        };
    }

    /// Gets the `ReturnValueEquality<MaybeCmp<UTypeId>>` to construct a `TypeInfo`.
    pub trait GetUTID<IsStatic>{
        const UID:ReturnValueEquality<MaybeCmp<UTypeId>>;
    }


    impl<T> GetUTID<True> for T
    where T:'static
    {
        const UID:ReturnValueEquality<MaybeCmp<UTypeId>>=ReturnValueEquality{
            function:some_utypeid::<T>
        };
    }

    impl<T> GetUTID<False> for T{
        const UID:ReturnValueEquality<MaybeCmp<UTypeId>>=ReturnValueEquality{
            function:no_utypeid
        };
    }
}





/////////////////////////////////////////////////////////////////////

crate::impl_InterfaceType!{
    impl crate::erased_types::InterfaceType for () {
        type Send=True;
        type Sync=True;
    }
}
