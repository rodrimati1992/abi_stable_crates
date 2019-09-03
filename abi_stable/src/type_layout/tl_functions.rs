use super::*;

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
    /// The strings of function/bound_lifetime/parameter names.
    pub strings: StaticStr,
    pub functions:RSlice<'static,CompTLFunction>,
    /// The range of `CompTLFunction` that each field in TLFields owns.
    pub field_fn_ranges:RSlice<'static,StartLen>,
    pub type_layouts: RSlice<'static,GetTypeLayout>,
    pub paramret_lifetime_indices: RSlice<'static,LifetimeIndex>,
}




impl TLFunctions {
    /// Constructs a TLFunctions.
    pub const fn new(
        strings: &'static str,
        functions:RSlice<'static,CompTLFunction>,
        field_fn_ranges:RSlice<'static,StartLen>,
        type_layouts: RSlice<'static,GetTypeLayout>,
        paramret_lifetime_indices: RSlice<'static,LifetimeIndex>,
    )->Self{
        Self{
            strings:StaticStr::new(strings),
            functions,
            field_fn_ranges,
            type_layouts,
            paramret_lifetime_indices,
        }
    }

    /// The the `nth` TLFunction in this `TLFunctions`.
    /// Returns None if there is not `nth` TLFunction.
    pub fn get(&'static self,nth:usize)->Option<TLFunction>{
        let func=self.functions.get(nth)?;
        Some(func.expand(self))
    }

    /// The the `nth` TLFunction in this `TLFunctions`.
    ///
    /// # Panics
    ///
    /// This function panics if `nth` is out of bounds (`self.len() <= nth`)
    pub fn index(&'static self,nth:usize)->TLFunction{
        self.functions[nth].expand(self)
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
    bound_lifetimes:StartLen,
    param_names:StartLen,
    param_type_layouts:StartLen,
    paramret_lifetime_indices:StartLen,
    return_type_layout:ROption<u16>,
}


impl CompTLFunction{
    /// Constructs a CompTLFunction.
    pub const fn new(
        name:StartLen,
        bound_lifetimes:StartLen,
        param_names:StartLen,
        param_type_layouts:StartLen,
        paramret_lifetime_indices:StartLen,
        return_type_layout:ROption<u16>,
    )->Self{
        Self{
            name,
            bound_lifetimes,
            param_names,
            param_type_layouts,
            paramret_lifetime_indices,
            return_type_layout,
        }
    }

    /// Decompresses this CompTLFunction into a TLFunction.
    pub fn expand(&self,with:&'static TLFunctions)->TLFunction{
        let strings=with.strings.as_rstr();
        TLFunction{
            name: strings.slice(self.name.to_range()),
            bound_lifetimes: strings.slice(self.bound_lifetimes.to_range()),
            param_names: strings.slice(self.param_names.to_range()),
            param_type_layouts: with.type_layouts.slice(self.param_type_layouts.to_range()),
            paramret_lifetime_indices:
                with.paramret_lifetime_indices.slice(self.paramret_lifetime_indices.to_range()),
            return_type_layout:
                self.return_type_layout.map(|x| with.type_layouts[x as usize] ),
        }
    }
}



///////////////////////////////////////////////////////////////////////////////



/// The start and length of a slice into `TLFunctions`.
#[repr(C)]
#[derive(Copy,Clone,Debug,PartialEq,Eq,Ord,PartialOrd,StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct StartLen{
    pub start:u16,
    pub len:u16,
}


impl StartLen{
    /// An empty range.
    pub const EMPTY:Self=Self{start:0,len:0};

    /// Constructs a range.
    pub const fn new(start:u16,len:u16)->Self{
        Self{start,len}
    }

    /// The start of this range.
    #[inline]
    pub const fn start(self)->usize{
        self.start as usize
    }

    /// The length of this range.
    #[inline]
    pub const fn len(self)->usize{
        self.len as usize
    }

    /// The exclusive end of this range.
    #[inline]
    pub const fn end(self)->usize{
        (self.start+self.len) as usize
    }

    /// Converts this range to a `std::ops::Range`.
    #[inline]
    pub const fn to_range(self)->Range<usize>{
        self.start()..self.end()
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
    fn_range:StartLen,
}


impl TLFunctionSlice{
    /// An empty slice of `TLFunction`s
    pub const EMPTY:Self=Self{
        functions:None,
        fn_range:StartLen::EMPTY,
    };

    /// Constructs this slice from a range and an optional `TLFunctions`.
    #[inline]
    pub const fn new(fn_range:StartLen,functions:Option<&'static TLFunctions>)->Self{
        Self{functions,fn_range}
    }

    /// Returns an iterator over the `TLFunction`s in the slice.
    #[inline]
    pub fn iter(self)->TLFunctionIter{
        TLFunctionIter::new(self.fn_range,self.functions)
    }

    /// Gets a TLFunction at the `index`.This returns None if `index` is outside the slice.
    pub fn get(self,index:usize)->Option<TLFunction>{
        self.functions?.get(self.fn_range.start()+index)
    }

    /// Gets a TLFunction at the `index`.
    ///
    /// # Panic
    ///
    /// This panics if the TLFunction is outside the slice.
    pub fn index(self,index:usize)->TLFunction{
        self.functions
            .expect("self.functions must be Some(..) to index a TLFunctionSlice")
            .index(self.fn_range.start()+index)
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