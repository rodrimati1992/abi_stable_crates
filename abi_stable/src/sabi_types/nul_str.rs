//! A nul-terminated string,which is just a pointer to the string data,
//! it doesn't know the length of the string.

use crate::std_types::RStr;

use std::{
    cmp::{PartialEq,Eq},
    fmt::{self, Debug, Display},
    marker::PhantomData,
};



/// A utf8 null-terminated string slice.
#[repr(transparent)]
#[derive(Copy,Clone,StableAbi)]
pub struct NulStr<'a>{
    ptr:*const u8,
    _marker:PhantomData<&'a u8>,
}

unsafe impl Sync for NulStr<'_>{}
unsafe impl Send for NulStr<'_>{}


impl NulStr<'static>{
    /// An empty string.
    pub const EMPTY:Self=NulStr{ptr:&0,_marker:PhantomData};
}

impl<'a> NulStr<'a>{
    /// Constructs an NulStr from a slice.
    /// 
    /// # Safety
    /// 
    /// `str` must be nul terminated(a 0 byte).
    pub const unsafe fn from_str(str: &'a str) -> Self{
        Self{
            ptr:str.as_ptr(),
            _marker:PhantomData,
        }
    }

    /// Constructs an NulStr from a pointer.
    /// 
    /// # Safety
    /// 
    /// The pointer must point to a utf8 and nul terminated (a 0 byte) sequence of bytes.
    pub const unsafe fn from_ptr(ptr: *const u8) -> Self{
        Self{
            ptr,
            _marker:PhantomData,
        }
    }

    /// Converts this `NulStr<'a>` to a `&'a str`.
    ///
    /// # Performance
    ///
    /// This conversion requires traversing through the entire string to 
    /// find the nul byte.
    pub fn to_str(self)->&'a str{
        unsafe{
            let bytes=std::ffi::CStr::from_ptr(self.ptr as *const i8).to_bytes();
            std::str::from_utf8_unchecked(bytes)
        }
    }

    /// Converts this `NulStr<'a>` to a `RStr<'a>`.
    ///
    /// # Performance
    ///
    /// This conversion requires traversing through the entire string to 
    /// find the nul byte.
    pub fn to_rstr(self)->RStr<'a>{
        self.as_str().into()
    }
}


impl<'a> PartialEq for NulStr<'a>{
    fn eq(&self,other:&Self)->bool{
        self.ptr==other.ptr ||
        self.as_str()==other.as_str()
    }
}

impl<'a> Eq for NulStr<'a>{}



impl Display for NulStr<'_> {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Display::fmt(self.as_str(),f)
    }
}

impl Debug for NulStr<'_> {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Debug::fmt(self.as_str(),f)
    }
}