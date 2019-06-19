use super::*;

use crate::{
    abi_stability::{
        abi_checking::{AbiInstability,push_err},
    },
    std_types::{StaticSlice,StaticStr,RVec},
};


///////////////////////////


/// The layout of an enum.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
pub struct TLEnum{
    /// A ';' separated list of all variant names
    pub variant_names:StaticStr,
    pub is_exhaustive:IsExhaustive,
    pub fields: TLFieldsOrSlice,
    pub discriminants:TLDiscriminants,
    pub field_count:StaticSlice<u8>,
}


impl TLEnum{
    /// Constructs a `TLData::Enum`.
    pub const fn new(
        variant_names:&'static str,
        is_exhaustive:IsExhaustive,
        fields: &'static [TLField],
        discriminants:TLDiscriminants,
        field_count:&'static [u8],
    ) -> Self {
        TLEnum {
            variant_names:StaticStr::new(variant_names),
            is_exhaustive,
            fields:TLFieldsOrSlice::from_slice(fields),
            discriminants,
            field_count:StaticSlice::new(field_count),
        }
    }

    /// Constructs a `TLData::Enum`.
    pub const fn for_derive(
        variant_names:&'static str,
        is_exhaustive:IsExhaustive,
        fields: TLFields,
        discriminants:TLDiscriminants,
        field_count:&'static [u8],
    ) -> Self {
        TLEnum {
            variant_names:StaticStr::new(variant_names),
            is_exhaustive,
            fields:TLFieldsOrSlice::TLFields(fields),
            discriminants,
            field_count:StaticSlice::new(field_count),
        }
    }

    pub fn variant_names_iter(&self)->GetVariantNames{
        GetVariantNames{
            split:self.variant_names.as_str().split(';'),
            length:self.field_count.len(),
            current:0,
        }
    }
}


///////////////////////////


macro_rules! declare_tl_discriminants {
    (
        $((
            $(#[$variant_attr:meta])*
            $variant:ident ( $ty:ty ),
            $single:ident,
            $(#[$method_attr:meta])*
            $method:ident
        ))*
    ) => (
        /// The discriminant of an enum variant.
        #[repr(u8)]
        #[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
        pub enum TLDiscriminants{
            $(
                $(#[$variant_attr])*
                $variant(StaticSlice<$ty>),
            )*
        }

        impl TLDiscriminants{
            $(
                $(#[$method_attr])*
                pub const fn $method(arr:&'static [$ty])->Self{
                    TLDiscriminants::$variant(StaticSlice::new(arr))
                }
            )*

            /// Gets the type of a discriminant in this TLDiscriminants.
            pub fn discriminant_repr(&self)->DiscriminantRepr{
                match self {
                    $(
                        TLDiscriminants::$variant{..}=>DiscriminantRepr::$variant,
                    )*
                }
            }

            pub fn compare(&self,other:&Self)->Result<(),RVec<AbiInstability>>{
                let mut errs=RVec::new();
                match (self,other) {
                    $(
                        (
                            TLDiscriminants::$variant(t_discrs),
                            TLDiscriminants::$variant(o_discrs)
                        )=>{
                            for (&t_discr,&o_discr) in 
                                t_discrs.as_slice().iter().zip(o_discrs.as_slice())
                            {
                                if t_discr!=o_discr {
                                    push_err(
                                        &mut errs,
                                        t_discr,
                                        o_discr,
                                        |x| TLDiscriminant::$single(x as _),
                                        AbiInstability::EnumDiscriminant,
                                    );
                                }
                            }
                        }
                    )*
                    _=>{
                        push_err(
                            &mut errs,
                            self,
                            other,
                            |x| ReprAttr::Int(x.discriminant_repr()),
                            AbiInstability::ReprAttr
                        );
                    }
                }
                if errs.is_empty(){
                    Ok(())
                }else{
                    Err(errs)
                }
            }
        }
    )
}


declare_tl_discriminants!{
    ( U8(u8) ,Signed  , from_u8_slice )
    ( I8(i8) ,Unsigned, from_i8_slice )
    ( U16(u16) ,Signed  , from_u16_slice )
    ( I16(i16) ,Unsigned, from_i16_slice )
    ( U32(u32) ,Signed  , from_u32_slice )
    ( I32(i32) ,Unsigned, from_i32_slice )
    ( U64(u64) ,Signed  , from_u64_slice )
    ( I64(i64) ,Unsigned, from_i64_slice )
    ( Usize(usize) ,Usize, from_usize_slice )
    ( Isize(isize) ,Isize, from_isize_slice )
}



/// The discriminant of an enum variant.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
pub enum TLDiscriminant{
    /// The assigned value of a discriminant in a `#[repr(isize)]` enum.
    Isize(isize),
    /// The assigned value of a discriminant in a `#[repr(usize)]` enum.
    Usize(usize),
    /// The assigned value of a discriminant in a `#[repr(i8/i16/i32/i64)]` enum.
    Signed(i64),
    /// The assigned value of a discriminant in a `#[repr(u8/u16/u32/u64)]` enum.
    Unsigned(u64),
}




/// How the discriminant of an enum is represented.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
pub enum DiscriminantRepr {
    /// The type of the discriminant for a `#[repr(u8)]`enum
    U8,
    /// The type of the discriminant for a `#[repr(i8)]`enum
    I8,
    /// The type of the discriminant for a `#[repr(u16)]`enum
    U16,
    /// The type of the discriminant for a `#[repr(i16)]`enum
    I16,
    /// The type of the discriminant for a `#[repr(u32)]`enum
    U32,
    /// The type of the discriminant for a `#[repr(i32)]`enum
    I32,
    /// The type of the discriminant for a `#[repr(u64)]`enum
    U64,
    /// The type of the discriminant for a `#[repr(i64)]`enum
    I64,
    /// Reserved,just in case that u128 gets a c-compatible layout
    U128,
    /// Reserved,just in case that i128 gets a c-compatible layout
    I128,
    /// The type of the discriminant for a `#[repr(usize)]`enum
    Usize,
    /// The type of the discriminant for a `#[repr(isize)]`enum
    ///
    /// This is the default discriminant type for `repr(C)`.
    Isize,
}


/// Whether this enum is exhaustive,if `Yes` it can add variants in minor versions.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
pub enum IsExhaustive{
    Yes,
    No,
}


/////////////////////////////////////////////////////////////////////////////


#[derive(Debug,Clone)]
pub struct GetVariantNames{
    split:std::str::Split<'static,char>,
    length:usize,
    current:usize,
}

impl Iterator for GetVariantNames{
    type Item=&'static str;
    fn next(&mut self) -> Option<Self::Item>{
        if self.length==self.current {
            return None;
        }
        let current=self.current;
        self.current+=1;
        match self.split.next().filter(|&x| !x.is_empty()||x=="_" ) {
            Some(x)=>Some(x),
            None=>Some(VARIANT_INDEX[current]),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len=self.length-self.current;
        (len,Some(len))
    }
    fn count(self) -> usize {
        let len=self.length-self.current;
        len
    }
}


impl std::iter::ExactSizeIterator for GetVariantNames{}


static VARIANT_INDEX: [&'static str; 68] = [
    "Variant0", "Variant1", "Variant2", "Variant3", 
    "Variant4", "Variant5", "Variant6", "Variant7", 
    "Variant8", "Variant9", "Variant10", "Variant11", 
    "Variant12", "Variant13", "Variant14", "Variant15",
    "Variant16", "Variant17", "Variant18", "Variant19",
    "Variant20", "Variant21", "Variant22", "Variant23",
    "Variant24", "Variant25", "Variant26", "Variant27",
    "Variant28", "Variant29", "Variant30", "Variant31",
    "Variant32", "Variant33", "Variant34", "Variant35",
    "Variant36", "Variant37", "Variant38", "Variant39",
    "Variant40", "Variant41", "Variant42", "Variant43",
    "Variant44", "Variant45", "Variant46", "Variant47",
    "Variant48", "Variant49", "Variant50", "Variant51",
    "Variant52", "Variant53", "Variant54", "Variant55",
    "Variant56", "Variant57", "Variant58", "Variant59",
    "Variant60", "Variant61", "Variant62", "Variant63",
    "Variant64", "Variant65", "Variant66", "Variant67",
];


