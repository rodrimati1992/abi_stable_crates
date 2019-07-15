use std::{
    cmp::{Eq,Ord},
    fmt::Debug,
};


use crate::{
    std_types::{StaticStr,StaticSlice,RBoxError,RStr,RCow},
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

    const ENUM_INFO:&'static EnumInfo<Self::Discriminant>;

    const DISCRIMINANTS:&'static [Self::Discriminant];
    
    fn is_valid_discriminant(discriminant:Self::Discriminant)->bool;
}


#[derive(StableAbi)]
#[repr(C)]
pub struct EnumInfo<D:'static>{
    pub type_name:StaticStr,

    pub variants:StaticSlice<StaticStr>,

    pub discriminants:StaticSlice<D>,
}

impl<D> EnumInfo<D>
where
    D:'static
{
    #[doc(hidden)]
    pub const fn _for_derive(
        type_name:&'static str,
        variants:&'static [StaticStr],
        discriminants:&'static [D]
    )->Self{
        Self{
            type_name:StaticStr::new(type_name),
            variants:StaticSlice::new(variants),
            discriminants:StaticSlice::new(discriminants),
        }
    }
}


    
/////////////////////////////////////////////////////////////


/// Marker trait for types that abi_stable supports as enum discriminants.
pub trait ValidDiscriminant:Sealed+Copy+Eq+Ord+Debug+'static{}

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


/////////////////////////////////////////////////////////////


/**
Describes how some enum is serialized.
*/
pub trait SerializeEnum<E>{
    fn serialize_enum<'a>(this:&'a E) -> Result<RCow<'a, str>, RBoxError>;
}

/**
Describes how a nonexhaustive enum is deserialized.

Generally this delegates to a library function,
so that the implementation can be delegated
to the `implementation crate`.

*/
pub trait DeserializeOwned<E,S,I> {
    fn deserialize_enum(s: RStr<'_>) -> Result<NonExhaustive<E,S,I>, RBoxError>
    where E:GetEnumInfo;
}

/**
Describes how a nonexhaustive enum is deserialized,
borrowing from the input RStr.

Generally this delegates to a library function,
so that the implementation can be delegated
to the `implementation crate`.

*/
pub trait DeserializeBorrowed<'borr,E,S,I> {
    fn deserialize_enum(s: RStr<'borr>) -> Result<NonExhaustive<E,S,I>, RBoxError> 
    where E:GetEnumInfo+'borr;
}
