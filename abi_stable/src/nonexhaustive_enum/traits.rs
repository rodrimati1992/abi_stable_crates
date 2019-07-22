use std::{
    cmp::{Eq,Ord},
    fmt::Debug,
};


use crate::{
    std_types::{StaticStr,StaticSlice,RBoxError,RStr,RCow},
    type_level::{
        impl_enum::{Implemented,Unimplemented},
        trait_marker,
    },
    InterfaceType,
};

use super::{NonExhaustive};

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

This must be an enum with a `#[repr(C)]` and `#[repr(SomeIntegerType)]` attribute.

The type of the discriminant must match `Self::NonExhaustive`.


*/
pub unsafe trait GetEnumInfo:Sized{
    type Discriminant:ValidDiscriminant;

    type DefaultStorage;
    
    type DefaultInterface;

    const ENUM_INFO:&'static EnumInfo;
    
    fn discriminants()->&'static [Self::Discriminant];

    fn is_valid_discriminant(discriminant:Self::Discriminant)->bool;
}


#[derive(StableAbi)]
#[repr(C)]
pub struct EnumInfo{
    pub type_name:StaticStr,

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
*/
pub trait SerializeEnum<NE>{
    type Proxy;

    fn serialize_enum(this:&NE) -> Result<Self::Proxy, RBoxError>;
}


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

*/
pub trait DeserializeEnum<'borr,NE> {
    type Proxy;

    fn deserialize_enum(s: Self::Proxy) -> Result<NE, RBoxError>;
}


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
