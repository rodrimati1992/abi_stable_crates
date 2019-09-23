use super::*;

use crate::{
    abi_stability::stable_abi_trait::GetTypeLayoutCtor,
    traits::IntoReprC,
    std_types::RVec,
};

use std::{
    cmp::{PartialEq,Eq},
    ops::Range,
};

///////////////////////////////////////////////////////////////////////////////

/// All the function pointer types in a type declaration.
#[repr(C)]
#[derive(Debug,Copy, Clone, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct TLFunctions{
    pub functions:RSlice<'static,CompTLFunction>,
    /// The range of `CompTLFunction` that each field in TLFields owns.
    pub field_fn_ranges:RSlice<'static,StartLen>,
}




impl TLFunctions {
    /// Constructs a TLFunctions.
    pub const fn new(
        functions:RSlice<'static,CompTLFunction>,
        field_fn_ranges:RSlice<'static,StartLenRepr>,
    )->Self{
        Self{
            functions,
            field_fn_ranges:unsafe{
                field_fn_ranges.transmute_ref::<StartLen>()
            },
        }
    }

    /// The the `nth` TLFunction in this `TLFunctions`.
    /// Returns None if there is not `nth` TLFunction.
    pub fn get(&'static self,nth:usize,shared_vars:&'static SharedVars)->Option<TLFunction>{
        let func=self.functions.get(nth)?;
        Some(func.expand(shared_vars))
    }

    /// The the `nth` TLFunction in this `TLFunctions`.
    ///
    /// # Panics
    ///
    /// This function panics if `nth` is out of bounds (`self.len() <= nth`)
    pub fn index(&'static self,nth:usize,shared_vars:&'static SharedVars)->TLFunction{
        self.functions[nth].expand(shared_vars)
    }

    /// Gets the amount of `TLFunction`s in this TLFunctions.
    #[inline]
    pub fn len(&'static self)->usize{
        self.functions.len()
    }
}


///////////////////////////////////////////////////////////////////////////////

/**
A slice of functions from a TLFunctions.
*/
#[repr(C)]
#[derive(Copy,Clone,StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct TLFunctionSlice{
    functions:Option<&'static TLFunctions>,
    shared_vars:&'static SharedVars,
    fn_range:StartLen,
}


impl TLFunctionSlice{
    /// Constructs an empty `TLFunctionSlice`.
    pub const fn empty(shared_vars:&'static SharedVars,)->Self{
        Self{
            functions:None,
            shared_vars,
            fn_range:StartLen::EMPTY,
        }
    }

    /// Constructs the TLFunctionSlice for the function pointers in the `i`th field.
    pub fn for_field(
        i:usize,
        functions:Option<&'static TLFunctions>,
        shared_vars:&'static SharedVars,
    )->Self{
        let fn_range=functions
            .and_then(|fns| fns.field_fn_ranges.get(i).cloned() )
            .unwrap_or(StartLen::EMPTY);

        Self{functions,fn_range,shared_vars}
    }
    
    /// Gets the `&'static SharedVars` associated with this slice.
    pub const fn shared_vars(&self)->&'static SharedVars{
        self.shared_vars
    }
    /// Returns an iterator over the `TLFunction`s in the slice.
    #[inline]
    pub fn iter(self)->TLFunctionIter{
        TLFunctionIter::new(self.fn_range,self.functions,self.shared_vars)
    }

    /// Gets a TLFunction at the `index`.This returns None if `index` is outside the slice.
    pub fn get(self,index:usize)->Option<TLFunction>{
        self.functions?.get( self.fn_range.start_usize()+index, self.shared_vars )
    }

    /// Gets a TLFunction at the `index`.
    ///
    /// # Panic
    ///
    /// This panics if the TLFunction is outside the slice.
    pub fn index(self,index:usize)->TLFunction{
        self.functions
            .expect("self.functions must be Some(..) to index a TLFunctionSlice")
            .index( self.fn_range.start_usize()+index, self.shared_vars )
    }

    /// Gets the length of this slice.
    #[inline]
    pub fn len(self)->usize{
        self.fn_range.len_usize()
    }
    /// Gets whether this slice is empty.
    #[inline]
    pub fn is_empty(self)->bool{
        self.fn_range.len()==0
    }
}


impl IntoIterator for TLFunctionSlice{
    type IntoIter=TLFunctionIter;
    type Item=TLFunction;

    #[inline]
    fn into_iter(self)->TLFunctionIter{
        self.iter()
    }
}


impl Debug for TLFunctionSlice{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        f.debug_list()
         .entries(self.iter())
         .finish()
    }
}

impl Eq for TLFunctionSlice{}

impl PartialEq for TLFunctionSlice{
    fn eq(&self,other:&Self)->bool{
        self.fn_range.len()==other.fn_range.len()&&
        self.iter().eq(other.iter())
    }
}


///////////////////////////////////////////////////////////////////////////////

/// A compressed version of a function pointer type,
/// which can be expanded into a TLFunction by calling the expand method.
#[repr(C)]
#[derive(Copy,Clone,Debug,PartialEq,Eq,Ord,PartialOrd,StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct CompTLFunction{
    name:StartLen,
    contiguous_strings_offset:u16,
    bound_lifetimes_len:u16,
    param_names_len:u16,
    /// Stores `!0` if the return type is `()`.
    return_type_layout:u16,
    paramret_lifetime_range:LifetimeRange,
    param_type_layouts:TypeLayoutRange,
}



impl CompTLFunction{
    /// Constructs a CompTLFunction.
    pub const fn new(
        name:StartLenRepr,
        contiguous_strings_offset:u16,
        bound_lifetimes_len:u16,
        param_names_len:u16,
        return_type_layout:u16,
        paramret_lifetime_range:u32,
        param_type_layouts:u64,
    )->Self{
        Self{
            name:StartLen::from_u32(name),
            contiguous_strings_offset,
            bound_lifetimes_len,
            param_names_len,
            return_type_layout,
            paramret_lifetime_range: LifetimeRange::from_u21(paramret_lifetime_range),
            param_type_layouts: TypeLayoutRange::from_u64(param_type_layouts),
        }
    }

    /// Decompresses this CompTLFunction into a TLFunction.
    pub fn expand(&self,shared_vars:&'static SharedVars)->TLFunction{
        let strings=shared_vars.strings().into_c();
        let lifetime_indices=shared_vars.lifetime_indices();
        let type_layouts=shared_vars.type_layouts();

        let cs_offset=self.contiguous_strings_offset as usize;

        let bound_lifetimes=cs_offset..cs_offset+(self.bound_lifetimes_len as usize);
        let param_names=
            bound_lifetimes.end..bound_lifetimes.end+(self.param_names_len as usize);

        TLFunction{
            shared_vars:CmpIgnored::new(shared_vars),
            name: strings.slice(self.name.to_range()),
            bound_lifetimes: strings.slice(bound_lifetimes),
            param_names: strings.slice(param_names),
            param_type_layouts: self.param_type_layouts.expand(type_layouts),
            paramret_lifetime_indices: self.paramret_lifetime_range.slicing(lifetime_indices),
            return_type_layout: type_layouts.get(self.return_type_layout as usize).cloned(),
        }
    }
}




///////////////////////////////////////////////////////////////////////////////


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
    pub(crate) fn get_params_ret_iter(&self)->impl ExactSizeIterator<Item=TLField>+Clone+Debug{
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

///////////////////////////////////////////////////////////////////////////////


/// An iterator over a range of `TLFunction`s.
pub struct TLFunctionIter{
    start:usize,
    end:usize,
    functions:Option<&'static TLFunctions>,
    shared_vars:&'static SharedVars,
}


impl TLFunctionIter{
    fn new(
        start_len:StartLen,
        functions:Option<&'static TLFunctions>,
        shared_vars:&'static SharedVars,
    )->Self{
        let Range{start,end}=start_len.to_range();
        if let Some(functions)=functions {
            assert!(start <= functions.len(),"{} < {}",start,functions.len());
            assert!(end   <= functions.len(),"{} < {}",end  ,functions.len());
        }
        Self{
            start,
            end,
            functions,
            shared_vars,
        }
    }
    fn length(&self)->usize{
        self.end-self.start
    }
}

impl Iterator for TLFunctionIter{
    type Item=TLFunction;

    fn next(&mut self)->Option<TLFunction>{
        let functions=self.functions?;
        if self.start>=self.end {
            return None;
        }
        let ret=functions.index(self.start,self.shared_vars);
        self.start+=1;
        Some(ret)
    }

    fn size_hint(&self)->(usize,Option<usize>){
        let len=self.length();
        (len,Some(len))
    }

    fn count(self)->usize{
        self.length()
    }
}

impl ExactSizeIterator for TLFunctionIter{}



////////////////////////////////////


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

