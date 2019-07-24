/*!
The traits releated to nonexhaustive enums.
*/

use std::{
    cmp::{Eq,Ord},
    fmt::Debug,
};


use crate::{
    std_types::{StaticStr,StaticSlice,RBoxError},
    type_level::{
        impl_enum::{Implemented,Unimplemented},
        trait_marker,
    },
    InterfaceType,
};


/**
Gets the type with the type layout of Self when it's stored in `NonExhaustive<>`.

# Safety

`Self::NonExhaustive` must describe the layout of this enum,
with the size and alignment of `Storage`,
storing the size and alignment of this enum in the 
`TypeLayout.data.TLData::Enum.exhaustiveness.IsExhaustive::nonexhaustive` field .
*/
pub unsafe trait GetNonExhaustive<Storage>:GetEnumInfo{
    /// This is the marker type used as the layout of Self in `NonExhaustive<>`
    type NonExhaustive;
}



/**
Describes the discriminant of an enum,and its valid values.

# Safety

This must be an enum with a `#[repr(C)]` or `#[repr(SomeIntegerType)]` attribute.

The type of the discriminant must match `Self::NonExhaustive`.


*/
pub unsafe trait GetEnumInfo:Sized{
    /// The type of the discriminant.
    type Discriminant:ValidDiscriminant;

    /// The default storage type,
    /// used to store this enum inside `NonExhaustive<>`,
    /// and allow the enum to grow in size in newer ABI compatible versions.
    type DefaultStorage;
    
    /// The default InterfaceType,
    /// used to determine the traits that are required when constructing a `NonExhaustive<>`,
    /// and are then usable afterwards.
    type DefaultInterface;

    /// Information about the enum.
    const ENUM_INFO:&'static EnumInfo;
    
    /// The values of the discriminants of each variant.
    fn discriminants()->&'static [Self::Discriminant];

    /// Whether `discriminant` is one of the valid discriminants for this enum in this context.
    fn is_valid_discriminant(discriminant:Self::Discriminant)->bool;
}


/// Contains miscelaneous information about an enum.
#[derive(StableAbi)]
#[repr(C)]
pub struct EnumInfo{
    /// The name of a type,eg:`Vec` for a `Vec<u8>`.
    pub type_name:StaticStr,

    /// The names of the variants of the enum.
    pub variants:StaticSlice<StaticStr>,
}

impl EnumInfo{
    #[doc(hidden)]
    pub const fn _for_derive(
        type_name:&'static str,
        variants:&'static [StaticStr],
    )->Self{
        Self{
            type_name:StaticStr::new(type_name),
            variants:StaticSlice::new(variants),
        }
    }
}


    
/////////////////////////////////////////////////////////////


/// Marker trait for types that abi_stable supports as enum discriminants.
///
/// This trait cannot be implemented outside of this module.
pub trait ValidDiscriminant:Sealed+Copy+Eq+Ord+Debug+Send+Sync+'static{}

mod sealed{
    pub trait Sealed{}
}
use self::sealed::Sealed;


macro_rules! impl_valid_discriminant {
    (
        $($ty:ty),* $(,)*
    ) => (
        $(
            impl ValidDiscriminant for $ty{}
            impl Sealed for $ty{}
        )*
    )
}


impl_valid_discriminant!{u8,i8,u16,i16,u32,i32,u64,i64,usize,isize}


///////////////////////////////////////////////////////////////////////////////


/**
Describes how some enum is serialized.

This is generally implemented by the interface of an enum
(Enum_Interface,that implement InterfaceType).
*/
pub trait SerializeEnum<NE>{
    /// The intermediate type the `NE` is converted into,to serialize it.
    type Proxy;

    /// Serializes an enum into its proxy type.
    fn serialize_enum(this:&NE) -> Result<Self::Proxy, RBoxError>;
}


#[doc(hidden)]
pub trait GetSerializeEnumProxy<NE>:InterfaceType{
    type ProxyType;
}

impl<I,NE,PT> GetSerializeEnumProxy<NE> for I
where
    I:InterfaceType,
    I:GetSerializeEnumProxyHelper<
        NE,
        <I as InterfaceType>::Serialize,
        ProxyType=PT
    >,
{
    type ProxyType=PT;
}

#[doc(hidden)]
pub trait GetSerializeEnumProxyHelper<NE,IS>:InterfaceType{
    type ProxyType;
}

impl<I,NE> 
    GetSerializeEnumProxyHelper<NE,Implemented<trait_marker::Serialize>> 
for I
where
    I:InterfaceType,
    I:SerializeEnum<NE>,
{
    type ProxyType=<I as SerializeEnum<NE>>::Proxy;
}

impl<I,NE> 
    GetSerializeEnumProxyHelper<NE,Unimplemented<trait_marker::Serialize>> 
for I
where
    I:InterfaceType,
{
    type ProxyType=();
}


///////////////////////////////////////


/**
Describes how a nonexhaustive enum is deserialized.

Generally this delegates to a library function,
so that the implementation can be delegated
to the `implementation crate`.

This is generally implemented by the interface of an enum
(Enum_Interface,that implement InterfaceType).

*/
pub trait DeserializeEnum<'borr,NE> {
    /// The intermediate type the `NE` is converted from,to deserialize it.
    type Proxy;

    /// Deserializes an enum from its proxy type.
    fn deserialize_enum(s: Self::Proxy) -> Result<NE, RBoxError>;
}

#[doc(hidden)]
pub trait GetDeserializeEnumProxy<'borr,NE>:InterfaceType{
    type ProxyType;
}

impl<'borr,I,NE,PT> GetDeserializeEnumProxy<'borr,NE> for I
where
    I:InterfaceType,
    I:GetDeserializeEnumProxyHelper<
        'borr,
        NE,
        <I as InterfaceType>::Deserialize,
        ProxyType=PT
    >,
{
    type ProxyType=PT;
}


#[doc(hidden)]
pub trait GetDeserializeEnumProxyHelper<'borr,NE,IS>:InterfaceType{
    type ProxyType;
}

impl<'borr,I,NE> 
    GetDeserializeEnumProxyHelper<'borr,NE,Implemented<trait_marker::Deserialize>> 
for I
where
    I:InterfaceType,
    I:DeserializeEnum<'borr,NE>
{
    type ProxyType=<I as DeserializeEnum<'borr,NE>>::Proxy;
}

impl<'borr,I,NE> 
    GetDeserializeEnumProxyHelper<'borr,NE,Unimplemented<trait_marker::Deserialize>> 
for I
where
    I:InterfaceType,
{
    type ProxyType=();
}
