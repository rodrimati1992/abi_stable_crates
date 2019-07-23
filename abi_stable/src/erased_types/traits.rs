
/*!
Traits for types wrapped in `DynTrait<_>`
*/

use std::{mem,marker::PhantomData};

use crate::{
    sabi_types::VersionStrings,
    std_types::{RBoxError, StaticStr},
};

use super::TypeInfo;

#[allow(unused_imports)]
use crate::type_level::{
    bools::{False, True},
    impl_enum::{Implemented,Unimplemented},
    trait_marker,
};

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
        
        type Error;
    ]


}



///////////////////////////////////////////////////////////////////////////////


/**
Describes how this `implementation type` is serialized.
*/
pub trait SerializeImplType{
    type Interface:SerializeProxyType;

    fn serialize_impl(
        &self
    ) -> Result<<Self::Interface as SerializeProxyType>::Proxy, RBoxError>;
}



pub trait SerializeProxyType:InterfaceType{
    type Proxy;
}

pub trait GetSerializeProxyType:InterfaceType{
    type ProxyType;
}

impl<I,PT> GetSerializeProxyType for I
where
    I:InterfaceType,
    I:GetSerializeProxyTypeHelper<
        <I as InterfaceType>::Serialize,
        ProxyType=PT
    >,
{
    type ProxyType=PT;
}

#[doc(hidden)]
pub trait GetSerializeProxyTypeHelper<IS>:InterfaceType{
    type ProxyType;
}

impl<I> GetSerializeProxyTypeHelper<Implemented<trait_marker::Serialize>> for I
where
    I:SerializeProxyType,
{
    type ProxyType=<I as SerializeProxyType>::Proxy;
}

impl<I> GetSerializeProxyTypeHelper<Unimplemented<trait_marker::Serialize>> for I
where
    I:InterfaceType,
{
    type ProxyType=();
}


///////////////////////////////////////


/**
Describes how something is deserialized,using a proxy to do so.

Generally this delegates to a library function,
so that the implementation can be delegated
to the `implementation crate`.

*/
pub trait DeserializeDyn<'borr,D> {
    type Proxy;

    fn deserialize_dyn(s: Self::Proxy) -> Result<D, RBoxError>;
}


pub trait GetDeserializeDynProxy<'borr,D>:InterfaceType{
    type ProxyType;
}

impl<'borr,I,D,PT> GetDeserializeDynProxy<'borr,D> for I
where
    I:InterfaceType,
    I:GetDeserializeDynProxyHelper<
        'borr,
        D,
        <I as InterfaceType>::Deserialize,
        ProxyType=PT
    >,
{
    type ProxyType=PT;
}


#[doc(hidden)]
pub trait GetDeserializeDynProxyHelper<'borr,D,IS>:InterfaceType{
    type ProxyType;
}

impl<'borr,I,D> 
    GetDeserializeDynProxyHelper<'borr,D,Implemented<trait_marker::Deserialize>> 
for I
where
    I:InterfaceType,
    I:DeserializeDyn<'borr,D>
{
    type ProxyType=<I as DeserializeDyn<'borr,D>>::Proxy;
}

impl<'borr,I,D> 
    GetDeserializeDynProxyHelper<'borr,D,Unimplemented<trait_marker::Deserialize>> 
for I
where
    I:InterfaceType,
{
    type ProxyType=();
}


/////////////////////////////////////////////////////////////////////


/// The way to specify the expected Iterator::Item type for an InterfaceType.
///
/// This is a separate trait to allow iterators that yield borrowed elements.
pub trait IteratorItem<'a>:InterfaceType{
    type Item;
}



/// Gets the Item type of an Iterator.
///
/// Used by `DynTrait`'s vtable to give its iter a default type,
/// when `I:InterfaceType<Iterator=Implemented<_>>`.
pub trait IteratorItemOrDefault<'borr>:InterfaceType{
    type Item;
}


impl<'borr,I,Item> IteratorItemOrDefault<'borr> for I
where 
    I:InterfaceType,
    I:IteratorItemOrDefaultHelper<
        'borr,
        <I as InterfaceType>::Iterator,
        Item=Item,
    >
{
    type Item=Item;
}


#[doc(hidden)]
pub trait IteratorItemOrDefaultHelper<'borr,ImplIsRequired>{
    type Item;
}

impl<'borr,I,Item> IteratorItemOrDefaultHelper<'borr,Implemented<trait_marker::Iterator>> for I
where
    I:IteratorItem<'borr,Item=Item>,
{
    type Item=Item;
}


impl<'borr,I> IteratorItemOrDefaultHelper<'borr,Unimplemented<trait_marker::Iterator>> for I{
    type Item=();
}



//////////////////////////////////////////////////////////////////


pub use self::interface_for::InterfaceFor;

pub mod interface_for{
    use super::*;

    use crate::type_level::unerasability::GetUTID;

    /// Helper struct to get an `ImplType` implementation for any type.
    pub struct InterfaceFor<T,Interface,Unerasability>(
        PhantomData<fn()->(T,Interface,Unerasability)>
    );

    impl<T,Interface,Unerasability> ImplType for InterfaceFor<T,Interface,Unerasability>
    where 
        Interface:InterfaceType,
        Unerasability:GetUTID<T>,
    {
        type Interface=Interface;
        
        /// The `&'static TypeInfo` constant,used when unerasing `DynTrait`s into a type.
        const INFO:&'static TypeInfo=&TypeInfo{
            size:mem::size_of::<T>(),
            alignment:mem::align_of::<T>(),
            _uid:<Unerasability as GetUTID<T>>::UID,
            name:StaticStr::new("<erased>"),
            module:StaticStr::new("<unavailable>"),
            package:StaticStr::new("<unavailable>"),
            package_version:VersionStrings::new("99.99.99"),
            _private_field:(),
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
