use super::*;

use crate::const_utils::{min_u16,min_u8};

use std::{
    ops::{Range,RangeInclusive},
};

////////////////////////////////////////////////////////////////////////////////

/// The start and length of a slice into `TLFunctions`.
#[repr(C)]
#[derive(Copy,Clone,Debug,PartialEq,Eq,Ord,PartialOrd,StableAbi)]
pub struct StartLen{
    pub start:u16,
    pub len:u16,
}

impl StartLen{
    /// Constructs a range.
    pub const fn new(start:u16,len:u16)->Self{
        Self{start,len}
    }

    /// Converts this range to a `std::ops::Range`.
    #[inline]
    pub const fn to_range(self)->Range<usize>{
        self.start()..self.end()
    }

    /// An empty range.
    pub const EMPTY:Self=Self{start:0,len:0};

    abi_stable_shared::declare_start_len_bit_methods!{}
}



/// Used to convert the stuff passed to the `tl_genparams` macro to a `StartLen`.
pub struct StartLenConverter<T>(pub T);

impl StartLenConverter<()>{
    pub const fn to_start_len(self)->StartLen{
        StartLen::EMPTY
    }
}

impl StartLenConverter<usize>{
    pub const fn to_start_len(self)->StartLen{
        StartLen::new(self.0 as u16,1)
    }
}

impl StartLenConverter<Range<usize>>{
    pub const fn to_start_len(self)->StartLen{
        let start=self.0.start as u16;
        let len=(self.0.end-self.0.start) as u16;
        StartLen::new(start,len)
    }
}

impl StartLenConverter<RangeInclusive<usize>>{
    pub const fn to_start_len(self)->StartLen{
        let start=*self.0.start();
        let end=*self.0.end()+1;
        StartLen::new(start as u16,(end-start)as u16)
    }
}

impl StartLenConverter<StartLen>{
    pub const fn to_start_len(self)->StartLen{
        self.0
    }
}


/////////////////////////////////////////////////////////////////////////////////////////////

#[repr(transparent)]
#[derive(Copy,Clone,PartialEq,Eq,Ord,PartialOrd,StableAbi)]
pub struct OptionU16(u16);

impl OptionU16{
    #[allow(non_uppercase_globals)]
    pub const None:Self=OptionU16(!0);

    const MAX_VAL:u16=!0-1;
    
    pub const fn some(value:u16)->Self{
        OptionU16( min_u16(value,Self::MAX_VAL))
    }
    
    pub fn is_some(self)->bool{
        self!=Self::None
    }
    pub fn is_none(self)->bool{
        self==Self::None
    }
    
    pub fn to_option(self)->Option<u16>{
        if self.is_some() {
            Some(self.0)
        }else{
            None
        }
    }
}


impl Debug for OptionU16{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Debug::fmt(&self.to_option(),f)
    }
}

impl Display for OptionU16{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        if self.is_some() {
            Display::fmt("None",f)
        }else{
            Display::fmt(&self.0,f)
        }        
    }
}


/////////////////////////////////////////////////////////////////////////////////////////////

#[repr(transparent)]
#[derive(Copy,Clone,PartialEq,Eq,Ord,PartialOrd,StableAbi)]
pub struct OptionU8(u8);

impl OptionU8{
    #[allow(non_uppercase_globals)]
    pub const None:Self=OptionU8(!0);
    
    const MAX_VAL:u8=!0-1;
    
    pub const fn some(value:u8)->Self{
        OptionU8( min_u8(value,Self::MAX_VAL))
    }
    
    pub fn is_some(self)->bool{
        self!=Self::None
    }
    pub fn is_none(self)->bool{
        self==Self::None
    }
    
    pub fn to_option(self)->Option<u8>{
        if self.is_some() {
            Some(self.0)
        }else{
            None
        }
    }
}


impl Debug for OptionU8{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Debug::fmt(&self.to_option(),f)
    }
}

impl Display for OptionU8{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        if self.is_some() {
            Display::fmt("None",f)
        }else{
            Display::fmt(&self.0,f)
        }        
    }
}


