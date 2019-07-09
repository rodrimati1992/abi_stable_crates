use super::*;

use std::{
    cmp::{PartialEq,Eq},
    ops::Range,
};

///////////////////////////////////////////////////////////////////////////////

/// A function pointer in a field.
#[repr(C)]
#[derive(Debug,Copy, Clone, StableAbi)]
pub struct TLFunctions{
    /// The strings of function/bound_lifetime/parameter names.
    pub strings: StaticStr,
    pub functions:StaticSlice<CompTLFunction>,
    /// The range of `CompTLFunction` that each field in TLFields owns.
    pub field_fn_ranges:StaticSlice<StartLen>,
    pub abi_infos: StaticSlice<GetAbiInfo>,
    pub paramret_lifetime_indices: StaticSlice<LifetimeIndex>,
}




impl TLFunctions {
    pub const fn new(
        strings: &'static str,
        functions:&'static [CompTLFunction],
        field_fn_ranges:&'static [StartLen],
        abi_infos: &'static [GetAbiInfo],
        paramret_lifetime_indices: &'static [LifetimeIndex],
    )->Self{
        Self{
            strings:StaticStr::new(strings),
            functions:StaticSlice::new(functions),
            field_fn_ranges:StaticSlice::new(field_fn_ranges),
            abi_infos:StaticSlice::new(abi_infos),
            paramret_lifetime_indices:StaticSlice::new(paramret_lifetime_indices),
        }
    }

    pub fn get(&'static self,index:usize)->Option<TLFunction>{
        let func=self.functions.get(index)?;
        Some(func.expand(self))
    }

    pub fn index(&'static self,index:usize)->TLFunction{
        self.functions[index].expand(self)
    }

    #[inline]
    pub fn len(&'static self)->usize{
        self.functions.len()
    }
}


///////////////////////////////////////////////////////////////////////////////

/// Equivalent to TLFunction,in which every field is a range into a `TLFunctions`.
#[repr(C)]
#[derive(Copy,Clone,Debug,PartialEq,Eq,Ord,PartialOrd,StableAbi)]
pub struct CompTLFunction{
    name:StartLen,
    bound_lifetimes:StartLen,
    param_names:StartLen,
    param_abi_infos:StartLen,
    paramret_lifetime_indices:StartLen,
    return_abi_info:ROption<u16>,
}


impl CompTLFunction{
    pub const fn new(
        name:StartLen,
        bound_lifetimes:StartLen,
        param_names:StartLen,
        param_abi_infos:StartLen,
        paramret_lifetime_indices:StartLen,
        return_abi_info:ROption<u16>,
    )->Self{
        Self{
            name,
            bound_lifetimes,
            param_names,
            param_abi_infos,
            paramret_lifetime_indices,
            return_abi_info,
        }
    }

    pub fn expand(&self,with:&'static TLFunctions)->TLFunction{
        TLFunction{
            name:self.name.get_str(with.strings),
            bound_lifetimes:self.bound_lifetimes.get_str(with.strings),
            param_names:self.param_names.get_str(with.strings),
            param_abi_infos:self.param_abi_infos.get_slice(with.abi_infos),
            paramret_lifetime_indices:
                self.paramret_lifetime_indices
                    .get_slice(with.paramret_lifetime_indices),
            return_abi_info:
                self.return_abi_info.map(|x| with.abi_infos[x as usize] ),
        }
    }
}



///////////////////////////////////////////////////////////////////////////////



/// The start and length of a slice into a slice in `TLFunctions`.
#[repr(C)]
#[derive(Copy,Clone,Debug,PartialEq,Eq,Ord,PartialOrd,StableAbi)]
pub struct StartLen{
    pub start:u16,
    pub len:u16,
}


impl StartLen{

    pub const EMPTY:Self=Self{start:0,len:0};

    pub const fn new(start:u16,len:u16)->Self{
        Self{start,len}
    }

    #[inline]
    pub const fn start(self)->usize{
        self.start as usize
    }
    #[inline]
    pub const fn len(self)->usize{
        self.len as usize
    }
    #[inline]
    pub const fn end(self)->usize{
        (self.start+self.len) as usize
    }
    #[inline]
    pub const fn to_range(self)->Range<usize>{
        self.start()..self.end()
    }
    #[inline]
    pub fn get_slice<T>(self,slice:StaticSlice<T>)->RSlice<'static,T>{
        slice.as_rslice().slice(self.to_range())
    }
    #[inline]
    pub fn get_str(self,str:StaticStr)->RStr<'static>{
        str.as_rstr().slice(self.to_range())
    }
}


///////////////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(Copy,Clone,StableAbi)]
pub struct TLFunctionRange{
    functions:Option<&'static TLFunctions>,
    fn_range:StartLen,
}


impl TLFunctionRange{
    pub const EMPTY:Self=Self{
        functions:None,
        fn_range:StartLen::EMPTY,
    };

    #[inline]
    pub const fn new(fn_range:StartLen,functions:Option<&'static TLFunctions>)->Self{
        Self{functions,fn_range}
    }

    #[inline]
    pub fn iter(self)->TLFunctionIter{
        TLFunctionIter::new(self.fn_range,self.functions)
    }

    pub fn get(self,index:usize)->Option<TLFunction>{
        self.functions?.get(self.fn_range.start()+index)
    }

    pub fn index(self,index:usize)->TLFunction{
        self.functions
            .expect("self.functions must be Some(..) to index a TLFunctionRange")
            .index(self.fn_range.start()+index)
    }

    #[inline]
    pub fn len(self)->usize{
        self.fn_range.len()
    }
    #[inline]
    pub fn is_empty(self)->bool{
        self.fn_range.len==0
    }
}


impl IntoIterator for TLFunctionRange{
    type IntoIter=TLFunctionIter;
    type Item=TLFunction;

    #[inline]
    fn into_iter(self)->TLFunctionIter{
        self.iter()
    }
}


impl Debug for TLFunctionRange{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        f.debug_list()
         .entries(self.iter())
         .finish()
    }
}

impl Eq for TLFunctionRange{}

impl PartialEq for TLFunctionRange{
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
}


impl TLFunctionIter{
    pub fn new(start_len:StartLen,functions:Option<&'static TLFunctions>)->Self{
        let Range{start,end}=start_len.to_range();
        if let Some(functions)=functions {
            assert!(start <= functions.len(),"{} < {}",start,functions.len());
            assert!(end   <= functions.len(),"{} < {}",end  ,functions.len());
        }
        Self{
            start,
            end,
            functions,
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
        if self.start==self.end {
            return None;
        }
        let ret=functions.index(self.start);
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