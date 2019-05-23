
use super::*;


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
    pub fields: StaticSlice<TLField>,
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
    /// The fields of the variant.
    pub fields: StaticSlice<TLField>,
}



impl TLEnumVariant {
    pub const fn new(name: &'static str, fields: &'static [TLField]) -> Self {
        Self {
            name: StaticStr::new(name),
            discriminant:TLDiscriminant::Default_,
            fields: StaticSlice::new(fields),
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
        fields: StaticSlice<TLField> 
    },
    /// For unions.
    Union { 
        fields: StaticSlice<TLField> 
    },
    /// For enums.
    Enum {
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
            fields: StaticSlice::new(&[]),
        };
 
    /// Constructs `TLData::Struct` from a slice of its fields.
    pub const fn struct_(fields: &'static [TLField]) -> Self {
        TLData::Struct {
            fields: StaticSlice::new(fields),
        }
    }
    
    /// Constructs `TLData::Union` from a slice of its fields.
    pub const fn union_(fields: &'static [TLField]) -> Self {
        TLData::Union {
            fields: StaticSlice::new(fields),
        }
    }
    
    /// Constructs a `TLData::Enum` from a slice to its variants.
    pub const fn enum_(variants: &'static [TLEnumVariant]) -> Self {
        TLData::Enum {
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
            fields:StaticSlice::new(fields),
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
#[derive(Debug,Copy, Clone, PartialEq, StableAbi)]
pub struct TLFunction{
    /// The name of the field this is used inside of.
    pub name: StaticStr,
    
    /// The named lifetime parameters of the function itself.
    pub bound_lifetimes: StaticSlice<StaticStr>,

    /// The parameters of the function,with names.
    /// 
    /// Lifetime indices inside mention lifetimes of the function after 
    /// the ones from the deriving type
    pub params:StaticSlice<TLField>,

    /// The return value of the function.
    /// 
    /// Lifetime indices inside mention lifetimes of the function after 
    /// the ones from the deriving type
    pub returns:ROption<TLField>,
}


impl TLFunction{
    /// Constructs a `TLFunction`.
    pub const fn new(
        name: &'static str,
        bound_lifetimes: &'static [StaticStr],
        params:&'static [TLField],
        returns:ROption<TLField>,
    )->Self{
        Self{
            name:StaticStr::new(name),
            bound_lifetimes:StaticSlice::new(bound_lifetimes),
            params:StaticSlice::new(params),
            returns,
        }
    }
}

impl Display for TLFunction{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        write!(f,"fn(")?;
        let param_count=self.params.len();
        for (param_i,param) in self.params.iter().enumerate() {
            Display::fmt(&TLFieldAndType::new(*param),f)?;
            if param_i+1!=param_count {
                Display::fmt(&", ",f)?;
            }
        }
        write!(f,")")?;
        if let RSome(returns)=self.returns {
            Display::fmt(&"->",f)?;
            Display::fmt(&TLFieldAndType::new(returns),f)?;
        }
        Ok(())
    }
}