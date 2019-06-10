
use super::*;

use std::iter;

use crate::{
    abi_stability::{
        stable_abi_trait::{MakeGetAbiInfo,StableAbi_Bound},
    },
    std_types::RVec,
};



/////////////////////////////////////////////////////

/// Which lifetime is being referenced by a field.
/// Allows lifetimes to be renamed,so long as the "same" lifetime is being referenced.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
pub enum LifetimeIndex {
    Static,
    /// Refers to the nth lifetime parameter of the deriving type.
    Param(usize),
}


/////////////////////////////////////////////////////


/// The definition of
/// vtables and modules that can be extended in minor versions.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
pub struct TLPrefixType {
    /// The first field in the suffix,
    /// the index to the field after 
    /// the one to which `#[sabi(last_prefix_field)]` was applied to
    pub first_suffix_field:usize,
    /// Which fields are accessible when the prefix type is instantiated in 
    /// the same dynlib/binary.
    pub accessible_fields:FieldAccessibility,
    /// Which fields in the prefix 
    /// (what comes at and before `#[sabi(last_prefix_field)]`)
    /// are conditionally accessible 
    /// (with the `#[sabi(accessible_if=" expression ")]` attribute).
    pub conditional_prefix_fields:StaticSlice<IsConditional>,
    /// All the fields of the prefix-type,even if they are inaccessible.
    pub fields: TLFieldsOrSlice,
}


/////////////////////////////////////////////////////

/// The `repr(..)` attribute used on a type.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
pub enum ReprAttr{
    /// This is an Option<NonZeroType>.
    /// In which the size and alignment of the Option<_> is exactly that of its contents.
    ///
    /// When translated to C,it is equivalent to the type parameter.
    OptionNonZero,
    /// This is an ffi-safe primitive type,declared in the compiler.
    Primitive,
    /// A struct whose fields are laid out like C,
    /// optionally with the type of the discriminant of an enum(if it is one).
    C(ROption<DiscriminantRepr>),
    /// A type with the same size,alignment and function ABI as
    /// its only non-zero-sized field.
    Transparent,
    /// Means that only `repr(IntegerType)` was used.
    Int(DiscriminantRepr),
    // Added just in case that I add support in 0.4.*
    #[doc(hidden)]
    Packed{
        alignment:usize,
    }
}


impl ReprAttr{
    /// Constructs the ReprAttr for `#[repr(C)]` types.
    pub const fn c()->Self{
        ReprAttr::C(RNone)
    }
}


/////////////////////////////////////////////////////

/**
A module path.
*/
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
pub enum ModPath{
    /// An item without a path
    NoPath,
    /// An item in a module.
    In(StaticStr),
    /// An item in the prelude.
    Prelude,
}


impl ModPath{
    /// Constructs a ModPath from a string with a module path.
    pub const fn inside(path:&'static str)->Self{
        ModPath::In(StaticStr::new(path))
    }
}


/////////////////////////////////////////////////////



/// The layout of an enum variant.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
pub struct TLEnumVariant {
    /// The name of the variant.
    pub name: StaticStr,
    /// The discriminant of the variant.
    pub discriminant:TLDiscriminant,
    pub field_count:usize,
}



impl TLEnumVariant {
    pub const fn new(name: &'static str, field_count: usize) -> Self {
        Self {
            name: StaticStr::new(name),
            discriminant:TLDiscriminant::Default_,
            field_count,
        }
    }

    pub const fn set_discriminant(mut self,discriminant:TLDiscriminant)->Self{
        self.discriminant=discriminant;
        self
    }
}


///////////////////////////


/// The discriminant of an enum variant.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
pub enum TLDiscriminant{
    /// The default,compiler assigned value for the discriminant.
    Default_,
    /// The assigned value of a discriminant in a `#[repr(isize)]` enum.
    Isize(isize),
    /// The assigned value of a discriminant in a `#[repr(usize)]` enum.
    Usize(usize),
    /// The assigned value of a discriminant in a `#[repr(i8/i16/i32/i64)]` enum.
    Signed(i64),
    /// The assigned value of a discriminant in a `#[repr(u8/u16/u32/u64)]` enum.
    Unsigned(u64),
}

impl TLDiscriminant{
    /// Constructs a discriminant of a `#[repr(u8)]` enum.
    pub const fn from_u8(n:u8)->Self{
        TLDiscriminant::Unsigned(n as u64)
    }
    /// Constructs a discriminant of a `#[repr(u16)]` enum.
    pub const fn from_u16(n:u16)->Self{
        TLDiscriminant::Unsigned(n as u64)
    }
    /// Constructs a discriminant of a `#[repr(u32)]` enum.
    pub const fn from_u32(n:u32)->Self{
        TLDiscriminant::Unsigned(n as u64)
    }
    /// Constructs a discriminant of a `#[repr(u64)]` enum.
    pub const fn from_u64(n:u64)->Self{
        TLDiscriminant::Unsigned(n)
    }
    /// Constructs a discriminant of a `#[repr(usize)]` enum.
    pub const fn from_usize(n:usize)->Self{
        TLDiscriminant::Usize(n)
    }

    /// Constructs a discriminant of a `#[repr(i8)]` enum.
    pub const fn from_i8(n:i8)->Self{
        TLDiscriminant::Signed(n as i64)
    }
    /// Constructs a discriminant of a `#[repr(i16)]` enum.
    pub const fn from_i16(n:i16)->Self{
        TLDiscriminant::Signed(n as i64)
    }
    /// Constructs a discriminant of a `#[repr(i32)]` enum.
    pub const fn from_i32(n:i32)->Self{
        TLDiscriminant::Signed(n as i64)
    }
    /// Constructs a discriminant of a `#[repr(i64)]` enum.
    pub const fn from_i64(n:i64)->Self{
        TLDiscriminant::Signed(n)
    }
    /// Constructs a discriminant of a `#[repr(usize)]` enum.
    pub const fn from_isize(n:isize)->Self{
        TLDiscriminant::Isize(n)
    }
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


///////////////////////////


/// Represents all the generic parameters of a type.
/// 
/// If the ammount of lifetimes change,the layouts are considered incompatible,
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
pub struct GenericParams {
    /// The names of the lifetimes declared by a type.
    pub lifetime: StaticSlice<StaticStr>,
    /// The type parameters of a type.
    pub type_: StaticSlice<&'static TypeLayout>,
    /// The values of const parameters,
    /// this currently is special cased for arrays up to 32 elements.
    pub const_: StaticSlice<StaticStr>,
}

impl GenericParams {
    /// Constructs a `GenericParams`.
    ///
    /// The preferred way of constructing a `GenericParams` is with the 
    /// `tl_genparams` macro.
    pub const fn new(
        lifetime: &'static [StaticStr],
        type_: &'static [&'static TypeLayout],
        const_: &'static [StaticStr],
    ) -> Self {
        Self {
            lifetime: StaticSlice::new(lifetime),
            type_: StaticSlice::new(type_),
            const_: StaticSlice::new(const_),
        }
    }

    /// Whether this contains any generic parameters
    pub fn is_empty(&self) -> bool {
        self.lifetime.is_empty() && self.type_.is_empty() && self.const_.is_empty()
    }
}

impl Display for GenericParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("<", f)?;

        let post_iter = |i: usize, len: usize, f: &mut Formatter<'_>| -> fmt::Result {
            if i + 1 < len {
                fmt::Display::fmt(", ", f)?;
            }
            Ok(())
        };

        for (i, param) in self.lifetime.iter().cloned().enumerate() {
            fmt::Display::fmt(param.as_str(), &mut *f)?;
            post_iter(i, self.lifetime.len(), &mut *f)?;
        }
        for (i, param) in self.type_.iter().cloned().enumerate() {
            fmt::Debug::fmt(&param.full_type(), &mut *f)?;
            post_iter(i, self.type_.len(), &mut *f)?;
        }
        for (i, param) in self.const_.iter().cloned().enumerate() {
            fmt::Display::fmt(param.as_str(), &mut *f)?;
            post_iter(i, self.const_.len(), &mut *f)?;
        }
        fmt::Display::fmt(">", f)?;
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////


/// What kind of type this is.struct/enum/etc.
///
/// Unions are currently treated as structs.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
pub enum TLData {
    /// Types defined in the compiler.
    Primitive(TLPrimitive),
    /// The type can't be inspected,and has no properties other than size/alignment.
    ///
    /// When translated to C,this would be a struct with a single array field
    /// whose element type is the alignment in this layout,
    /// with the same byte length as this layout .
    Opaque,
    /// For structs.
    Struct { 
        fields: TLFieldsOrSlice 
    },
    /// For unions.
    Union { 
        fields: TLFieldsOrSlice 
    },
    /// For enums.
    Enum {
        fields: TLFieldsOrSlice,
        variants: StaticSlice<TLEnumVariant>,
    },
    /// vtables and modules that can be extended in minor versions.
    PrefixType(TLPrefixType),
}

/// Types defined in the compiler
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
pub enum TLPrimitive{
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    Usize,
    Isize,
    Bool,
    /// A `&T`
    SharedRef,
    /// A `&mut T`
    MutRef,
    /// A `*const T`
    ConstPtr,
    /// A `*mut T`
    MutPtr,
    /// An array.
    Array{
        len:usize,
    },
    /// A "custom" primitive type.
    Custom(&'static CustomPrimitive)
}


/// The properties of a custom primitive.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
pub struct CustomPrimitive{
    /// The printed type name of this primitive
    pub typename:StaticStr,
    /// The token before the generic parameters of this primitive.Eg:"<"
    pub start_gen:StaticStr,
    /// The token separating generic parameters for this primitive.Eg:", "
    pub ty_sep:StaticStr,
    /// The token after the generic parameters of this primitive.Eg:">"
    pub end_gen:StaticStr,
}


/// A discriminant-only version of TLData.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
pub enum TLDataDiscriminant {
    Primitive,
    Opaque,
    Struct,
    Union,
    Enum,
    PrefixType,
}


impl TLData {
    pub const EMPTY:Self=
        TLData::Struct {
            fields: TLFieldsOrSlice::from_slice(&[]),
        };
 
    /// Constructs `TLData::Struct` from a slice of its fields.
    pub const fn struct_(fields: &'static [TLField]) -> Self {
        TLData::Struct {
            fields: TLFieldsOrSlice::from_slice(fields),
        }
    }
    
    /// Constructs `TLData::Union` from a slice of its fields.
    pub const fn union_(fields: &'static [TLField]) -> Self {
        TLData::Union {
            fields: TLFieldsOrSlice::from_slice(fields),
        }
    }
    
    /// Constructs a `TLData::Enum` from a slice to its variants.
    pub const fn enum_(fields:&'static [TLField],variants: &'static [TLEnumVariant]) -> Self {
        TLData::Enum {
            fields:TLFieldsOrSlice::from_slice(fields),
            variants: StaticSlice::new(variants),
        }
    }

    /// Constructs a `TLData::PrefixType`
    pub const fn prefix_type(
        first_suffix_field:usize,
        accessible_fields:FieldAccessibility,
        conditional_prefix_fields:&'static [IsConditional],
        fields: &'static [TLField],
    )->Self{
        TLData::PrefixType(TLPrefixType{
            first_suffix_field,
            accessible_fields,
            conditional_prefix_fields:StaticSlice::new(conditional_prefix_fields),
            fields:TLFieldsOrSlice::from_slice(fields),
        })
    }
 
    /// Constructs `TLData::Struct` from a slice of its fields.
    pub const fn struct_derive(fields: TLFields) -> Self {
        TLData::Struct {
            fields: TLFieldsOrSlice::TLFields(fields),
        }
    }
    
    /// Constructs `TLData::Union` from a slice of its fields.
    pub const fn union_derive(fields: TLFields) -> Self {
        TLData::Union {
            fields: TLFieldsOrSlice::TLFields(fields),
        }
    }
    
    /// Constructs a `TLData::Enum` from a slice to its variants.
    pub const fn enum_derive(
        fields:TLFields,
        variants: &'static [TLEnumVariant]
    ) -> Self {
        TLData::Enum {
            fields:TLFieldsOrSlice::TLFields(fields),
            variants: StaticSlice::new(variants),
        }
    }

    /// Constructs a `TLData::PrefixType`
    pub const fn prefix_type_derive(
        first_suffix_field:usize,
        accessible_fields:FieldAccessibility,
        conditional_prefix_fields:&'static [IsConditional],
        fields: TLFields,
    )->Self{
        TLData::PrefixType(TLPrefixType{
            first_suffix_field,
            accessible_fields,
            conditional_prefix_fields:StaticSlice::new(conditional_prefix_fields),
            fields:TLFieldsOrSlice::TLFields(fields),
        })
    }

    /// Converts this a TLDataDiscriminant,allowing one to query which discriminant this is
    /// (without either copying TLData or keeping a reference).
    pub fn as_discriminant(&self) -> TLDataDiscriminant {
        match self {
            TLData::Primitive { .. } => TLDataDiscriminant::Primitive,
            TLData::Opaque { .. } => TLDataDiscriminant::Opaque,
            TLData::Struct { .. } => TLDataDiscriminant::Struct,
            TLData::Union { .. } => TLDataDiscriminant::Union,
            TLData::Enum { .. } => TLDataDiscriminant::Enum,
            TLData::PrefixType { .. } => TLDataDiscriminant::PrefixType,
        }
    }
}

///////////////////////////



/// The typename and generics of the type this layout is associated to,
/// used for printing types.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, StableAbi)]
pub struct FullType {
    /// The name of the type.
    pub name: StaticStr,
    /// Whether the type is a primitive,and which one.
    pub primitive:ROption<TLPrimitive>,
    /// The generic parameters of the type ().
    pub generics: GenericParams,
}


impl FullType {
    /// Constructs a `FullType`.
    pub const fn new(
        name: &'static str,
        primitive: ROption<TLPrimitive>,
        generics: GenericParams,
    ) -> Self {
        Self {
            name: StaticStr::new(name),
            primitive,
            generics,
        }
    }
}

impl Display for FullType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl Debug for FullType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::TLPrimitive as TLP;

        let (typename, start_gen, before_ty, ty_sep, end_gen) = match self.primitive {
            RSome(TLP::SharedRef) => ("&", "", " ", " ", " "),
            RSome(TLP::MutRef) => ("&", "", " mut ", " ", " "),
            RSome(TLP::ConstPtr) => ("*const", " ", "", " ", " "),
            RSome(TLP::MutPtr) => ("*mut", " ", "", " ", " "),
            RSome(TLP::Array{..}) => ("", "[", "", ";", "]"),
            RSome(TLP::Custom(c))=>{
                (
                    c.typename.as_str(),
                    c.start_gen.as_str(),
                    "",
                    c.ty_sep.as_str(),
                    c.end_gen.as_str(),
                )
            }
             RSome(TLP::U8)|RSome(TLP::I8)
            |RSome(TLP::U16)|RSome(TLP::I16)
            |RSome(TLP::U32)|RSome(TLP::I32)
            |RSome(TLP::U64)|RSome(TLP::I64)
            |RSome(TLP::Usize)|RSome(TLP::Isize)
            |RSome(TLP::Bool)
            |RNone => (self.name.as_str(), "<", "", ", ", ">"),
        };

        fmt::Display::fmt(typename, f)?;
        let mut is_before_ty = true;
        let generics = self.generics;
        if !generics.is_empty() {
            fmt::Display::fmt(start_gen, f)?;

            let post_iter = |i: usize, len: usize, f: &mut Formatter<'_>| -> fmt::Result {
                if i+1 < len {
                    fmt::Display::fmt(ty_sep, f)?;
                }
                Ok(())
            };

            let mut i=0;

            let total_generics_len=
                generics.lifetime.len()+generics.type_.len()+generics.const_.len();

            for param in generics.lifetime.iter().cloned() {
                fmt::Display::fmt(param.as_str(), &mut *f)?;
                post_iter(i,total_generics_len, &mut *f)?;
                i+=1;
            }
            for param in generics.type_.iter().cloned() {
                if is_before_ty {
                    fmt::Display::fmt(before_ty, &mut *f)?;
                    is_before_ty = false;
                }
                fmt::Debug::fmt(&param.full_type(), &mut *f)?;
                post_iter(i,total_generics_len, &mut *f)?;
                i+=1;
            }
            for param in generics.const_.iter().cloned() {
                fmt::Display::fmt(param.as_str(), &mut *f)?;
                post_iter(i,total_generics_len, &mut *f)?;
                i+=1;
            }
            fmt::Display::fmt(end_gen, f)?;
        }
        Ok(())
    }
}


////////////////////////////////////



/// A function pointer in a field.
#[repr(C)]
#[derive(Debug,Copy, Clone, StableAbi)]
pub struct TLFunction{
    /// The name of the field this is used inside of.
    pub name: StaticStr,
    
    /// The named lifetime parameters of the function itself.
    pub bound_lifetimes: StaticSlice<StaticStr>,

    /// A ';' separated list of all the parameter names.
    pub param_names: StaticStr,

    pub param_abi_infos: StaticSlice<GetAbiInfo>,
    pub paramret_lifetime_indices: StaticSlice<LifetimeIndex>,

    /// The return value of the function.
    /// 
    /// Lifetime indices inside mention lifetimes of the function after 
    /// the ones from the deriving type
    pub return_abi_info:ROption<GetAbiInfo>,
}


impl PartialEq for TLFunction{
    fn eq(&self,other:&Self)->bool{
        self.name==other.name&&
        self.bound_lifetimes==other.bound_lifetimes&&
        self.param_names==other.param_names&&
        self.get_params_ret_iter().eq(other.get_params_ret_iter())&&
        self.paramret_lifetime_indices==other.paramret_lifetime_indices&&
        self.return_abi_info.map(|x| x.get() )==other.return_abi_info.map(|x| x.get() )
    }
}


impl TLFunction{
    /// Constructs a `TLFunction`.
    pub const fn new(
        name: &'static str,
        bound_lifetimes: &'static [StaticStr],
        param_names: &'static str,
        param_abi_infos: &'static [GetAbiInfo],
        paramret_lifetime_indices: &'static [LifetimeIndex],
        return_abi_info:ROption<GetAbiInfo>,
    )->Self{
        Self{
            name:StaticStr::new(name),
            bound_lifetimes:StaticSlice::new(bound_lifetimes),
            param_names:StaticStr::new(param_names),
            param_abi_infos:StaticSlice::new(param_abi_infos),
            paramret_lifetime_indices:StaticSlice::new(paramret_lifetime_indices),
            return_abi_info,
        }
    }

    pub(crate) fn get_param_names(&self)->GetParamNames{
        GetParamNames{
            split:self.param_names.as_str().split(';'),
            length:self.param_abi_infos.len(),
            current:0,
        }
    }

    /// Gets the parameter types
    pub(crate) fn get_params(&self)->impl ExactSizeIterator<Item=TLField>+Clone+Debug {
        self.get_param_names()
            .zip(self.param_abi_infos.as_slice().iter().cloned())
            .map(|(param_name,abi_info)|{
                TLField::new(param_name,&[],abi_info)
            })
    }
    
    pub(crate) fn get_return(&self)->TLField{
        const UNIT_GET_ABI_INFO:GetAbiInfo=<() as MakeGetAbiInfo<StableAbi_Bound>>::CONST;
        TLField::new(
            "__returns",
            self.paramret_lifetime_indices.as_slice(),
            self.return_abi_info.unwrap_or(UNIT_GET_ABI_INFO)
        )
    }

    /// Gets the parameters and return types 
    pub(crate) fn get_params_ret_iter(&self)->
        ChainOnce<impl ExactSizeIterator<Item=TLField>+Clone+Debug,TLField>
    {
        ChainOnce::new(self.get_params(),self.get_return())
    }

    /// Gets the parameters and return types 
    pub(crate) fn get_params_ret_vec(&self)->RVec<TLField>{
        self.get_params_ret_iter().collect()
    }
}

impl Display for TLFunction{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        write!(f,"fn(")?;
        let params=self.get_params();
        let param_count=params.len();
        for (param_i,param) in params.enumerate() {
            Display::fmt(&TLFieldAndType::new(param),f)?;
            if param_i+1!=param_count {
                Display::fmt(&", ",f)?;
            }
        }
        write!(f,")")?;
        
        let returns=self.get_return(); 
        Display::fmt(&"->",f)?;
        Display::fmt(&TLFieldAndType::new(returns),f)?;

        Ok(())
    }
}


/////////////////////////////////////////////////////////////////////////////


#[derive(Debug,Clone)]
pub struct GetParamNames{
    split:std::str::Split<'static,char>,
    length:usize,
    current:usize,
}

impl Iterator for GetParamNames{
    type Item=&'static str;
    fn next(&mut self) -> Option<Self::Item>{
        if self.length==self.current{
            return None;
        }
        let current=self.current;
        self.current+=1;
        match self.split.next().filter(|&x| !x.is_empty()||x=="_" ) {
            Some(x)=>Some(x),
            None=>Some(PARAM_INDEX[current]),
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


impl std::iter::ExactSizeIterator for GetParamNames{}


static PARAM_INDEX: [&'static str; 64] = [
    "param_0", "param_1", "param_2", "param_3", "param_4", "param_5", "param_6", "param_7",
    "param_8", "param_9", "param_10", "param_11", "param_12", "param_13", "param_14", "param_15",
    "param_16", "param_17", "param_18", "param_19", "param_20", "param_21", "param_22", "param_23",
    "param_24", "param_25", "param_26", "param_27", "param_28", "param_29", "param_30", "param_31",
    "param_32", "param_33", "param_34", "param_35", "param_36", "param_37", "param_38", "param_39",
    "param_40", "param_41", "param_42", "param_43", "param_44", "param_45", "param_46", "param_47",
    "param_48", "param_49", "param_50", "param_51", "param_52", "param_53", "param_54", "param_55",
    "param_56", "param_57", "param_58", "param_59", "param_60", "param_61", "param_62", "param_63",
];


//////////////////////////////////////////////////////////////////////////////

#[derive(Debug,Clone)]
pub struct ChainOnce<I,T>{
    iter:I,
    once:Option<T>,
}

impl<I> ChainOnce<I,I::Item>
where
    I:ExactSizeIterator
{
    fn new(iter:I,once:I::Item)->Self{
        Self{
            iter,
            once:Some(once),
        }
    }
    fn length(&self)->usize{
        self.iter.len()+(self.once.is_some() as usize)
    }
}
impl<I> Iterator for ChainOnce<I,I::Item>
where
    I:ExactSizeIterator
{
    type Item=I::Item;
    fn next(&mut self) -> Option<I::Item>{
        if let ret@Some(_)=self.iter.next() {
            return ret;
        }
        self.once.take()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len=self.length();
        (len,Some(len))
    }
    fn count(self) -> usize {
        self.length()
    }
}


impl<I> std::iter::ExactSizeIterator for ChainOnce<I,I::Item>
where
    I:ExactSizeIterator
{}