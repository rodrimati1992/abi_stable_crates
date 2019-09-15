use super::*;

use crate::abi_stability::ConstGeneric;

use std::{
    slice,
    fmt::{self,Debug},
};

////////////////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(StableAbi)]
pub struct SharedVars{
    /// Many strings,separated with ";".
    strings: *const u8,
    /// Stores the lifetime indices for lifetimes referenced in a type.
    ///
    /// Note that this only stores those indices if the type references more than 3 lifetimes,
    /// otherwise it is stored in the range itself.
    ///
    /// Lifetimes indices are stored for these in order:
    ///
    /// - For fields
    ///
    /// - For parameters and return types in function pointers in fields.
    ///
    lifetime_indices: *const LifetimeIndexPair,
    type_layouts: *const TypeLayoutCtor,
    constants: *const ConstGeneric,

    strings_len: u16,
    lifetime_indices_len: u16,
    type_layouts_len: u16,
    constants_len:u16,
}

unsafe impl Sync for SharedVars {}
unsafe impl Send for SharedVars {}

impl SharedVars{
    pub const fn new(
        strings: RStr<'static>,
        lifetime_indices: RSlice<'static,LifetimeIndexPairRepr>,
        type_layouts: RSlice<'static,TypeLayoutCtor>,
        constants: RSlice<'static,ConstGeneric>,
    )->Self{
        Self{
            strings: strings.as_ptr(),
            strings_len: strings.len() as u16,

            lifetime_indices: lifetime_indices.as_ptr() 
                as *const LifetimeIndexPairRepr 
                as *const LifetimeIndexPair,
            lifetime_indices_len: lifetime_indices.len() as u16,
            
            type_layouts: type_layouts.as_ptr(),
            type_layouts_len: type_layouts.len() as u16,

            constants: constants.as_ptr(),
            constants_len: constants.len() as u16,
        }
    }

    pub fn strings(&self)->&'static str{
        unsafe{
            let slice=slice::from_raw_parts( self.strings, self.strings_len as usize);
            std::str::from_utf8_unchecked(slice)
        }
    }

    pub fn lifetime_indices(&self)->&'static [LifetimeIndexPair]{
        unsafe{
            slice::from_raw_parts( self.lifetime_indices, self.lifetime_indices_len as usize )
        }
    }
    pub fn type_layouts(&self)->&'static [TypeLayoutCtor]{
        unsafe{
            slice::from_raw_parts( self.type_layouts, self.type_layouts_len as usize )
        }
    }
    pub fn constants(&self)->&'static [ConstGeneric]{
        unsafe{
            slice::from_raw_parts( self.constants, self.constants_len as usize )
        }
    }
}

impl Debug for SharedVars{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        f.debug_struct("SharedVars").finish()
    }
}
