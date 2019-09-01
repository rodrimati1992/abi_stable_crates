
use super::*;

use crate::{
    abi_stability::{
        stable_abi_trait::{GetTypeLayoutCtor},
    },
    std_types::{StaticSlice,RVec},
};



/////////////////////////////////////////////////////

/// Which lifetime is being referenced by a field.
/// Allows lifetimes to be renamed,so long as the "same" lifetime is being referenced.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub enum LifetimeIndex {
    Static,
    /// Refers to the nth lifetime parameter of the deriving type.
    Param(u8),
}


/////////////////////////////////////////////////////


/// The definition of
/// vtables and modules that can be extended in minor versions.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
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
    pub conditional_prefix_fields:RSlice<'static,IsConditional>,
    /// All the fields of the prefix-type,even if they are inaccessible.
    pub fields: TLFieldsOrSlice,
}


impl Display for TLPrefixType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f,"first_suffix_field:{}",self.first_suffix_field)?;
        write!(f,"accessible_fields:\n    ")?;
        f.debug_list()
         .entries(self.accessible_fields.iter_field_count(self.fields.len()))
         .finish()?;
        writeln!(f,)?;
        writeln!(f,"conditional_prefix_fields:\n    {:?}",self.conditional_prefix_fields)?;
        writeln!(f,"fields:\n{}",self.fields.to_string().left_padder(4))?;
        Ok(())
    }
}



/////////////////////////////////////////////////////

/// The `repr(..)` attribute used on a type.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
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
    // Added just in case that I add support for it
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
#[sabi(unsafe_sabi_opaque_fields)]
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


impl Display for ModPath{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModPath::NoPath=>Display::fmt("<no path>",f),
            ModPath::In(mod_)=>Display::fmt(mod_,f),
            ModPath::Prelude=>Display::fmt("<prelude>",f),
        }
    }
}


/////////////////////////////////////////////////////


/// Represents all the generic parameters of a type.
/// 
/// If the ammount of lifetimes change,the layouts are considered incompatible,
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq,StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct GenericParams {
    /// The names of the lifetimes declared by a type.
    pub lifetime: StaticStr,
    /// The type parameters of a type.
    pub type_: RSlice<'static,&'static TypeLayout>,
    /// The values of const parameters,this will work properly eventually.
    pub const_: RSlice<'static,StaticStr>,
}

impl GenericParams {
    /// Constructs a `GenericParams`.
    ///
    /// The preferred way of constructing a `GenericParams` is with the 
    /// `tl_genparams` macro.
    pub const fn new(
        lifetime: StaticStr,
        type_: RSlice<'static,&'static TypeLayout>,
        const_: RSlice<'static,StaticStr>,
    ) -> Self {
        Self {
            lifetime,
            type_,
            const_,
        }
    }

    /// Whether this contains any generic parameters
    pub fn is_empty(&self) -> bool {
        self.lifetime.is_empty() && self.type_.is_empty() && self.const_.is_empty()
    }

    pub fn lifetimes(&self)-> impl Iterator<Item=&'static str>+Clone+Send+Sync+'static {
        self.lifetime.as_str().split(',').filter(|x| !x.is_empty() )
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

        for (i, param) in self.lifetimes().enumerate() {
            fmt::Display::fmt(param, &mut *f)?;
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
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
    Enum (&'static TLEnum),
    /// vtables and modules that can be extended in minor versions.
    PrefixType(TLPrefixType),
}


impl Display for TLData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TLData::Primitive(prim)=>{
                writeln!(f,"Primitive:{:?}",prim)
            },
            TLData::Opaque=>{
                writeln!(f,"Opaque data")
            },
            TLData::Struct{fields}=>{
                writeln!(f,"Struct with Fields:\n{}",fields.to_string().left_padder(4))
            },
            TLData::Union{fields}=>{
                writeln!(f,"Union with Fields:\n{}",fields.to_string().left_padder(4))
            },
            TLData::Enum (tlenum)=>{
                writeln!(f,"Enum:")?;
                Display::fmt(tlenum,f)
            },
            TLData::PrefixType(prefix)=>{
                writeln!(f,"Prefix type:")?;
                Display::fmt(prefix,f)
            },
        }
    }
}


///////////////////////////////////////////////////////////////////////////////


/// Types defined in the compiler
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq,StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
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
#[derive(Debug, Copy, Clone, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
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


impl Eq for CustomPrimitive{}

impl PartialEq for CustomPrimitive{
    fn eq(&self,other:&Self)->bool{
        std::ptr::eq(self,other)
    }
}


///////////////////////////////////////////////////////////////////////////////


/// A discriminant-only version of TLData.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
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
            fields: TLFieldsOrSlice::Slice(RSlice::EMPTY),
        };
 
    /// Constructs `TLData::Struct` from a slice of its fields.
    pub const fn struct_(fields: RSlice<'static,TLField>) -> Self {
        TLData::Struct {
            fields: TLFieldsOrSlice::Slice(fields),
        }
    }
    
    /// Constructs `TLData::Union` from a slice of its fields.
    pub const fn union_(fields: RSlice<'static,TLField>) -> Self {
        TLData::Union {
            fields: TLFieldsOrSlice::Slice(fields),
        }
    }
    
    /// Constructs a `TLData::PrefixType`
    pub const fn prefix_type(
        first_suffix_field:usize,
        accessible_fields:FieldAccessibility,
        conditional_prefix_fields:RSlice<'static,IsConditional>,
        fields: RSlice<'static,TLField>,
    )->Self{
        TLData::PrefixType(TLPrefixType{
            first_suffix_field,
            accessible_fields,
            conditional_prefix_fields,
            fields:TLFieldsOrSlice::Slice(fields),
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

    /// Constructs a `TLData::PrefixType`
    pub const fn prefix_type_derive(
        first_suffix_field:usize,
        accessible_fields:FieldAccessibility,
        conditional_prefix_fields:RSlice<'static,IsConditional>,
        fields: TLFields,
    )->Self{
        TLData::PrefixType(TLPrefixType{
            first_suffix_field,
            accessible_fields,
            conditional_prefix_fields,
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
#[derive(Copy, Clone, PartialEq, Eq, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct FullType {
    /// The name of the type.
    pub name: StaticStr,
    /// Whether the type is a primitive,and which one.
    pub primitive:ROption<TLPrimitive>,
    /// The generic parameters of the type
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

            for param in self.generics.lifetimes() {
                fmt::Display::fmt(param, &mut *f)?;
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


/**
Either a TLField or a TLFunction.
*/
#[repr(u8)]
#[derive(Copy,Clone,Debug,Eq,PartialEq,StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub enum TLFieldOrFunction{
    Field(TLField),
    Function(TLFunction),
}


impl From<TLField> for TLFieldOrFunction{
    fn from(x:TLField)->Self{
        TLFieldOrFunction::Field(x)
    }
}

impl From<TLFunction> for TLFieldOrFunction{
    fn from(x:TLFunction)->Self{
        TLFieldOrFunction::Function(x)
    }
}


impl Display for TLFieldOrFunction{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        match self {
            TLFieldOrFunction::Field(x)=>Display::fmt(x,f),
            TLFieldOrFunction::Function(x)=>Display::fmt(x,f),
        }
    }
}


impl TLFieldOrFunction{
    /// Outputs this into a String with `Display` formatting.
    pub fn formatted_layout(&self)->String{
        match self {
            TLFieldOrFunction::Field(x)=>x.layout.get().to_string(),
            TLFieldOrFunction::Function(x)=>x.to_string(),
        }
    }
}


////////////////////////////////////



/// A function pointer in a field.
#[repr(C)]
#[derive(Copy,Clone,Debug,Eq,StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct TLFunction{
    /// The name of the field this is used inside of.
    pub name: RStr<'static>,
    
    /// The named lifetime parameters of the function itself,separated by ';'.
    pub bound_lifetimes: RStr<'static>,

    /// A ';' separated list of all the parameter names.
    pub param_names: RStr<'static>,

    pub param_type_layouts: RSlice<'static,GetTypeLayout>,
    pub paramret_lifetime_indices: RSlice<'static,LifetimeIndex>,

    /// The return value of the function.
    /// 
    /// Lifetime indices inside mention lifetimes of the function after 
    /// the ones from the deriving type
    pub return_type_layout:ROption<GetTypeLayout>,

}





impl PartialEq for TLFunction{
    fn eq(&self,other:&Self)->bool{
        self.name==other.name&&
        self.bound_lifetimes==other.bound_lifetimes&&
        self.param_names==other.param_names&&
        self.get_params_ret_iter().eq(other.get_params_ret_iter())&&
        self.paramret_lifetime_indices==other.paramret_lifetime_indices&&
        self.return_type_layout.map(|x| x.get() )==other.return_type_layout.map(|x| x.get() )
    }
}


impl TLFunction{
    pub(crate) fn get_param_names(&self)->GetParamNames{
        GetParamNames{
            split:self.param_names.as_str().split(';'),
            length:self.param_type_layouts.len(),
            current:0,
        }
    }

    /// Gets the parameter types
    pub(crate) fn get_params(&self)->impl ExactSizeIterator<Item=TLField>+Clone+Debug {
        self.get_param_names()
            .zip(self.param_type_layouts.as_slice().iter().cloned())
            .map(|(param_name,layout)|{
                TLField::new(param_name,RSlice::EMPTY,layout)
            })
    }
    
    pub(crate) fn get_return(&self)->TLField{
        const UNIT_GET_ABI_INFO:GetTypeLayout=GetTypeLayoutCtor::<()>::STABLE_ABI;
        TLField::new(
            "__returns",
            RSlice::EMPTY,
            self.return_type_layout.unwrap_or(UNIT_GET_ABI_INFO)
        )
    }

    /// Gets the parameters and return types 
    pub(crate) fn get_params_ret_iter(&self)->
        ChainOnce<impl ExactSizeIterator<Item=TLField>+Clone+Debug,TLField>
    {
        ChainOnce::new(self.get_params(),self.get_return())
    }

    /// Gets the parameters and return types 
    #[allow(dead_code)]
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
            Display::fmt(&param.name,f)?;
            Display::fmt(&": ",f)?;
            Display::fmt(&param.full_type(),f)?;
            if param_i+1!=param_count {
                Display::fmt(&", ",f)?;
            }
        }
        write!(f,")")?;
        
        let returns=self.get_return(); 
        Display::fmt(&"->",f)?;
        Display::fmt(&returns.full_type(),f)?;

        if !self.paramret_lifetime_indices.is_empty() {
            writeln!(f,"\nlifetime indices:{:?}",self.paramret_lifetime_indices)?;
        }

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