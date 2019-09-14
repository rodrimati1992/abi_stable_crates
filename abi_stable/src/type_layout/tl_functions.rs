use super::*;

use crate::traits::IntoReprC;

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
        field_fn_ranges:RSlice<'static,StartLen>,
    )->Self{
        Self{
            functions,
            field_fn_ranges,
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

/// Equivalent to TLFunction,in which every field is a range into a `TLFunctions`.
#[repr(C)]
#[derive(Copy,Clone,Debug,PartialEq,Eq,Ord,PartialOrd,StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct CompTLFunction{
    name:StartLen,
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
        name:StartLen,
        bound_lifetimes_len:u16,
        param_names_len:u16,
        return_type_layout:u16,
        paramret_lifetime_range:u32,
        param_type_layouts:u64,
    )->Self{
        Self{
            name,
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

        let bound_lifetimes=
            self.name.end()..self.name.end()+(self.bound_lifetimes_len as usize);
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
    /// Constructs this slice from a range and an optional `TLFunctions`.
    #[inline]
    pub const fn new(
        fn_range:StartLen,
        functions:Option<&'static TLFunctions>,
        shared_vars:&'static SharedVars,
    )->Self{
        Self{functions,fn_range,shared_vars}
    }

    pub const fn empty(shared_vars:&'static SharedVars,)->Self{
        Self{
            functions:None,
            shared_vars,
            fn_range:StartLen::EMPTY,
        }
    }
    
    pub const fn shared_vars(&self)->&'static SharedVars{
        self.shared_vars
    }

    pub fn for_field(
        field_index:usize,
        functions:Option<&'static TLFunctions>,
        shared_vars:&'static SharedVars,
    )->Self{
        let start_len=functions
            .and_then(|fns| fns.field_fn_ranges.get(field_index).cloned() )
            .unwrap_or(StartLen::EMPTY);

        Self::new(start_len,functions,shared_vars)
    }

    /// Returns an iterator over the `TLFunction`s in the slice.
    #[inline]
    pub fn iter(self)->TLFunctionIter{
        TLFunctionIter::new(self.fn_range,self.functions,self.shared_vars)
    }

    /// Gets a TLFunction at the `index`.This returns None if `index` is outside the slice.
    pub fn get(self,index:usize)->Option<TLFunction>{
        self.functions?.get( self.fn_range.start()+index, self.shared_vars )
    }

    /// Gets a TLFunction at the `index`.
    ///
    /// # Panic
    ///
    /// This panics if the TLFunction is outside the slice.
    pub fn index(self,index:usize)->TLFunction{
        self.functions
            .expect("self.functions must be Some(..) to index a TLFunctionSlice")
            .index( self.fn_range.start()+index, self.shared_vars )
    }

    /// Gets the length of this slice.
    #[inline]
    pub fn len(self)->usize{
        self.fn_range.len()
    }
    /// Gets whether this slice is empty.
    #[inline]
    pub fn is_empty(self)->bool{
        self.fn_range.len==0
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
        self.fn_range.len==other.fn_range.len&&
        self.iter().eq(other.iter())
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
    pub fn new(
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
