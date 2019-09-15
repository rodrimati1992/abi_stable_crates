
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
        self.lifetime.as_str().is_empty() && self.types.is_empty() && self.consts.is_empty()
    }

    pub fn lifetimes(&self)-> impl Iterator<Item=&'static str>+Clone+Send+Sync+'static {
        self.lifetime.as_str().split(',').filter(|x| !x.is_empty() )
    }
    pub fn lifetime_count(&self)->usize{
        self.lifetime_count as usize
    }
    pub fn type_params(&self)->&'static [TypeLayoutCtor]{
        self.types
    }
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
    pub fn name(&self)->&'static str{
        self.name
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


////////////////////////////////////



/// A function pointer in a field.
#[repr(C)]
#[derive(Copy,Clone,Debug,Eq,StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct TLFunction{
    pub(super) shared_vars:CmpIgnored<&'static SharedVars>,

    /// The name of the field this is used inside of.
    pub name: RStr<'static>,
    
    /// The named lifetime parameters of the function itself,separated by ';'.
    pub bound_lifetimes: RStr<'static>,

    /// A ';' separated list of all the parameter names.
    pub param_names: RStr<'static>,

    pub param_type_layouts: MultipleTypeLayouts<'static>,
    pub paramret_lifetime_indices: LifetimeArrayOrSlice<'static>,

    /// The return value of the function.
    /// 
    /// Lifetime indices inside mention lifetimes of the function after 
    /// the ones from the deriving type
    pub return_type_layout:Option<TypeLayoutCtor>,

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
        let shared_vars=*self.shared_vars;
        self.get_param_names()
            .zip(self.param_type_layouts.iter())
            .map(move|(param_name,layout)|{
                TLField::new(param_name.into(),layout,shared_vars)
            })
    }
    
    pub(crate) fn get_return(&self)->TLField{
        const UNIT_GET_ABI_INFO:TypeLayoutCtor=GetTypeLayoutCtor::<()>::STABLE_ABI;
        TLField::new(
            rstr!("__returns"),
            self.return_type_layout.unwrap_or(UNIT_GET_ABI_INFO),
            *self.shared_vars,
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
            Display::fmt(&param.name(),f)?;
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