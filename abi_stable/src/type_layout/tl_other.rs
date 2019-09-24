
use super::*;

use crate::{
    abi_stability::{
        stable_abi_trait::{GetTypeLayoutCtor},
        ConstGeneric,
    },
    std_types::RVec,
};



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
    C,
    /// A struct whose fields are laid out like C,
    /// with the type of the discriminant of an enum.
    CAndInt(DiscriminantRepr),
    /// A type with the same size,alignment and function ABI as
    /// its only non-zero-sized field.
    Transparent,
    /// Means that only `repr(IntegerType)` was used.
    Int(DiscriminantRepr),
    // Added just in case that I add support for it
    #[doc(hidden)]
    Packed{
        /// The alignment represented as a `1 << alignment_power_of_two`.
        alignment_power_of_two:u8,
    }
}


/////////////////////////////////////////////////////


/**
A module path.
*/
#[repr(transparent)]
#[derive(Debug, Copy, Clone,Eq,PartialEq, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct ModPath(NulStr<'static>);

impl ModPath{

    /// An item without a path
    pub const NO_PATH:Self=ModPath(nul_str!("<no path>"));

    /// An item in the prelude.
    pub const PRELUDE:Self=ModPath(nul_str!("<prelude>"));

    /// Constructs a ModPath from a string with a module path.
    pub const fn inside(path:NulStr<'static>)->Self{
        ModPath(path)
    }
}


impl Display for ModPath{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0,f)
    }
}


/////////////////////////////////////////////////////


/// Represents all the generic parameters of a type.
/// 
/// If the ammount of lifetimes change,the layouts are considered incompatible,
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq,StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct CompGenericParams {
    /// The names of the lifetimes declared by a type.
    lifetime: NulStr<'static>,
    /// The type parameters of a type,getting them from the containing TypeLayout.
    types: StartLen,
    /// The const parameters of a type,getting them from the containing TypeLayout.
    consts: StartLen,
    lifetime_count:u8,
}

impl CompGenericParams{
    pub const fn new(
        lifetime: NulStr<'static>,
        lifetime_count:u8,
        types:StartLen,
        consts:StartLen,
    )->Self{
        Self{
            lifetime,
            lifetime_count,
            types,
            consts,
        }
    }

    pub fn expand(self,shared_vars:&'static SharedVars)->GenericParams{
        GenericParams{
            lifetime: self.lifetime,
            types: &shared_vars.type_layouts()[self.types.to_range()],
            consts: &shared_vars.constants()[self.consts.to_range()],
            lifetime_count:self.lifetime_count,
        }
    }
}

/// The generic parameters of a type.
#[derive(Copy,Clone,PartialEq,Eq)]
pub struct GenericParams{
    /// The names of the lifetimes declared by a type.
    lifetime: NulStr<'static>,
    /// The type parameters of a type,getting them from the containing TypeLayout.
    types: &'static [TypeLayoutCtor],
    /// The const parameters of a type,getting them from the containing TypeLayout.
    consts: &'static [ConstGeneric],
    lifetime_count:u8,
}

impl GenericParams {
    /// Whether this contains any generic parameters
    pub fn is_empty(&self) -> bool {
        self.lifetime.to_str().is_empty() && self.types.is_empty() && self.consts.is_empty()
    }

    /// Gets an iterator over the names of the lifetime parameters of the type.
    pub fn lifetimes(&self)-> impl Iterator<Item=&'static str>+Clone+Send+Sync+'static {
        self.lifetime.to_str().split(',').filter(|x| !x.is_empty() )
    }
    /// The ammount of the lifetime of the type.
    pub fn lifetime_count(&self)->usize{
        self.lifetime_count as usize
    }
    /// The type parameters of the type.
    pub fn type_params(&self)->&'static [TypeLayoutCtor]{
        self.types
    }
    /// The const parameters of the type.
    pub fn const_params(&self)->&'static [ConstGeneric]{
        self.consts
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
            post_iter(i, self.lifetime_count(), &mut *f)?;
        }
        for (i, param) in self.types.iter().cloned().enumerate() {
            fmt::Debug::fmt(&param.get().full_type(), &mut *f)?;
            post_iter(i, self.types.len(), &mut *f)?;
        }
        for (i, param) in self.consts.iter().enumerate() {
            fmt::Debug::fmt(param, &mut *f)?;
            post_iter(i, self.consts.len(), &mut *f)?;
        }
        fmt::Display::fmt(">", f)?;
        Ok(())
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
    pub typename:RStr<'static>,
    /// The token before the generic parameters of this primitive.Eg:"<"
    pub start_gen:RStr<'static>,
    /// The token separating generic parameters for this primitive.Eg:", "
    pub ty_sep:RStr<'static>,
    /// The token after the generic parameters of this primitive.Eg:">"
    pub end_gen:RStr<'static>,
}


impl Eq for CustomPrimitive{}

impl PartialEq for CustomPrimitive{
    fn eq(&self,other:&Self)->bool{
        std::ptr::eq(self,other)
    }
}


///////////////////////////


/// The typename and generics of the type this layout is associated to,
/// used for printing types.
#[derive(Copy,Clone,PartialEq,Eq)]
pub struct FmtFullType {
    pub(super) name:&'static str,
    pub(super) generics:GenericParams,
    pub(super) primitive:Option<TLPrimitive>,
}

impl FmtFullType{
    /// The name of a type.
    pub fn name(&self)->&'static str{
        self.name
    }
    /// The generic parmaters of a type.
    pub fn generics(&self)->GenericParams{
        self.generics
    }
}

impl Display for FmtFullType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl Debug for FmtFullType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::TLPrimitive as TLP;

        let (typename, start_gen, before_ty, ty_sep, end_gen) = match self.primitive {
            Some(TLP::SharedRef) => ("&", "", " ", " ", " "),
            Some(TLP::MutRef) => ("&", "", " mut ", " ", " "),
            Some(TLP::ConstPtr) => ("*const", " ", "", " ", " "),
            Some(TLP::MutPtr) => ("*mut", " ", "", " ", " "),
            Some(TLP::Array{..}) => ("", "[", "", ";", "]"),
            Some(TLP::Custom(c))=>{
                (
                    c.typename.as_str(),
                    c.start_gen.as_str(),
                    "",
                    c.ty_sep.as_str(),
                    c.end_gen.as_str(),
                )
            }
             Some(TLP::U8)|Some(TLP::I8)
            |Some(TLP::U16)|Some(TLP::I16)
            |Some(TLP::U32)|Some(TLP::I32)
            |Some(TLP::U64)|Some(TLP::I64)
            |Some(TLP::Usize)|Some(TLP::Isize)
            |Some(TLP::Bool)
            |None => (self.name, "<", "", ", ", ">"),
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
                generics.lifetime_count()+generics.types.len()+generics.consts.len();

            for param in self.generics.lifetimes() {
                fmt::Display::fmt(param, &mut *f)?;
                post_iter(i,total_generics_len, &mut *f)?;
                i+=1;
            }
            for param in generics.types.iter().cloned() {
                let layout=param.get();
                if is_before_ty {
                    fmt::Display::fmt(before_ty, &mut *f)?;
                    is_before_ty = false;
                }
                fmt::Debug::fmt(&layout.full_type(), &mut *f)?;
                post_iter(i,total_generics_len, &mut *f)?;
                i+=1;
            }
            for param in generics.consts.iter() {
                fmt::Debug::fmt(param, &mut *f)?;
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
            TLFieldOrFunction::Field(x)=>x.layout().to_string(),
            TLFieldOrFunction::Function(x)=>x.to_string(),
        }
    }
}



//////////////////////////////////////////////////////////////////////////////
