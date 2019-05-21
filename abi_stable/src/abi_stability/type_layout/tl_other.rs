
use super::*;


/// Which lifetime is being referenced by a field.
/// Allows lifetimes to be renamed,so long as the "same" lifetime is being referenced.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
pub enum LifetimeIndex {
    Static,
    Param(usize),
}


/////////////////////////////////////////////////////


/// vtables and modules that can be extended in minor versions.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
pub struct TLPrefixType {
    /// The first field in the suffix
    pub first_suffix_field:usize,
    pub accessible_fields:FieldAccessibility,
    pub conditional_prefix_fields:StaticSlice<IsConditional>,
    pub fields: StaticSlice<TLField>,
}


/////////////////////////////////////////////////////


#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
pub enum ReprAttr{
    C(ROption<DiscriminantRepr>),
    Transparent,
    /// Means that only `repr(IntegerType)` was used.
    Int(DiscriminantRepr),
    // Added just in case that I add support in 0.4.*
    #[doc(hidden)]
    Packed{
        alignment:usize,
    }
}

/////////////////////////////////////////////////////

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
pub enum ModPath{
    No,
    With(StaticStr),
    Prelude,
}


impl ModPath{
    pub const fn with(path:&'static str)->Self{
        ModPath::With(StaticStr::new(path))
    }
}


/////////////////////////////////////////////////////



/// The layout of an enum variant.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
pub struct TLEnumVariant {
    pub name: StaticStr,
    pub discriminant:TLDiscriminant,
    pub fields: StaticSlice<TLField>,
}



impl TLEnumVariant {
    pub const fn new(name: &'static str, fields: &'static [TLField]) -> Self {
        Self {
            name: StaticStr::new(name),
            discriminant:TLDiscriminant::No,
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
    No,
    Isize(isize),
    Usize(usize),
    Signed(i64),
    Unsigned(u64),
}

impl TLDiscriminant{
    pub const fn from_u8(n:u8)->Self{
        TLDiscriminant::Unsigned(n as u64)
    }
    pub const fn from_u16(n:u16)->Self{
        TLDiscriminant::Unsigned(n as u64)
    }
    pub const fn from_u32(n:u32)->Self{
        TLDiscriminant::Unsigned(n as u64)
    }
    pub const fn from_u64(n:u64)->Self{
        TLDiscriminant::Unsigned(n)
    }

    pub const fn from_i8(n:i8)->Self{
        TLDiscriminant::Signed(n as i64)
    }
    pub const fn from_i16(n:i16)->Self{
        TLDiscriminant::Signed(n as i64)
    }
    pub const fn from_i32(n:i32)->Self{
        TLDiscriminant::Signed(n as i64)
    }
    pub const fn from_i64(n:i64)->Self{
        TLDiscriminant::Signed(n)
    }
}

/// How the discriminant of an enum is represented.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
pub enum DiscriminantRepr {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    /// Reserved,just in case that u128 gets a c-compatible layout
    U128,
    /// Reserved,just in case that i128 gets a c-compatible layout
    I128,
    Usize,
    /// This is the default discriminant type for `repr(C)`.
    Isize,
}



/// What primitive type this is.Used mostly for printing the type.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
pub enum RustPrimitive {
    Reference,
    MutReference,
    ConstPtr,
    MutPtr,
    Array,
}

///////////////////////////


/// Represents all the generic parameters of a type.
/// 
/// This is different for every different generic parameter,
/// if any one of them changes it won't compare equal,
/// `<Vec<u32>>::ABI_INFO.get().layout.full_type.generics`
/// ẁon't compare equal to
/// `<Vec<()>>::ABI_INFO.get().layout.full_type.generics`
/// 
///
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
pub struct GenericParams {
    pub lifetime: StaticSlice<StaticStr>,
    pub type_: StaticSlice<&'static TypeLayout>,
    pub const_: StaticSlice<StaticStr>,
}

impl GenericParams {
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
    /// All the bytes for the type are valid (not necessarily all bit patterns).
    ///
    /// If you use this variant,
    /// you must ensure the continuing validity of the same bit-patterns.
    Primitive,
    /// For structs and unions.
    Struct { fields: StaticSlice<TLField> },
    /// For enums.
    Enum {
        variants: StaticSlice<TLEnumVariant>,
    },
    /// vtables and modules that can be extended in minor versions.
    PrefixType(TLPrefixType),
}


/// A discriminant-only version of TLData.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
pub enum TLDataDiscriminant {
    Primitive,
    Struct,
    Enum,
    PrefixType,
}


impl TLData {
    pub const fn struct_(fields: &'static [TLField]) -> Self {
        TLData::Struct {
            fields: StaticSlice::new(fields),
        }
    }
    pub const fn enum_(variants: &'static [TLEnumVariant]) -> Self {
        TLData::Enum {
            variants: StaticSlice::new(variants),
        }
    }

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

    pub fn as_discriminant(&self) -> TLDataDiscriminant {
        match self {
            TLData::Primitive { .. } => TLDataDiscriminant::Primitive,
            TLData::Struct { .. } => TLDataDiscriminant::Struct,
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
    pub name: StaticStr,
    pub primitive: ROption<RustPrimitive>,
    pub generics: GenericParams,
}


impl FullType {
    pub const fn new(
        name: &'static str,
        primitive: ROption<RustPrimitive>,
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
        let (typename, start_gen, before_ty, ty_sep, end_gen) = match self.primitive {
            RSome(RustPrimitive::Reference) => ("&", "", " ", " ", " "),
            RSome(RustPrimitive::MutReference) => ("&", "", " mut ", " ", " "),
            RSome(RustPrimitive::ConstPtr) => ("*const", " ", "", " ", " "),
            RSome(RustPrimitive::MutPtr) => ("*mut", " ", "", " ", " "),
            RSome(RustPrimitive::Array) => ("", "[", "", ";", "]"),
            RNone => (self.name.as_str(), "<", "", ", ", ">"),
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




#[repr(C)]
#[derive(Debug,Copy, Clone, PartialEq, StableAbi)]
pub struct TLFunction{
    /// The name of the field this is used inside of.
    pub name: StaticStr,
    
    /// The named lifetime parameters of function itself.
    pub bound_lifetimes: StaticSlice<StaticStr>,

    /// The parameters of the function,with names.
    /// 
    /// Lifetime indices at and after `bound_lifetimes.len()`
    /// are lifetimes declared in the function pointer.
    pub params:StaticSlice<TLField>,

    /// The return value of the function.
    /// 
    /// Lifetime indices at and after `bound_lifetimes.len()`
    /// are lifetimes declared in the function pointer.
    pub returns:ROption<TLField>,
}


impl TLFunction{
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