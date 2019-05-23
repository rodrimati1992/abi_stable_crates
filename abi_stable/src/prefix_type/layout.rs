use std::{
    mem,
};

use crate::{
    ignored_wrapper::CmpIgnored,
    std_types::{StaticSlice,StaticStr},
    version::VersionStrings,
};



/// Represents the layout of a prefix-type,for use in error messages.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
pub struct PTStructLayout {
    pub name: StaticStr,
    pub generics:CmpIgnored<StaticStr>,
    pub package: StaticStr,
    pub package_version: VersionStrings,
    pub file:CmpIgnored<StaticStr>, // This is for the Debug string
    pub line:CmpIgnored<u32>, // This is for the Debug string
    pub size: usize,
    pub alignment: usize,
    pub fields:StaticSlice<PTField>,
}


/// Parameters to construct a PTStructLayout.
pub struct PTStructLayoutParams{
    pub name: &'static str,
    pub generics:&'static str,
    pub package: &'static str,
    pub package_version: VersionStrings,
    pub file:&'static str, // This is for the Debug string
    pub line:u32, // This is for the Debug string
    pub fields:&'static [PTField],
}


/// Represents a field of a prefix-type,for use in error messages.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
pub struct PTField {
    pub name:StaticStr,
    pub ty:CmpIgnored<StaticStr>,
    pub size:usize,
    pub alignment:usize,
}



//////////////////////////////////////////////////////////////


impl PTStructLayout{
    pub const fn new<T>(params:PTStructLayoutParams)->Self{
        Self{
            name:StaticStr::new(params.name),
            generics:CmpIgnored::new(StaticStr::new(params.generics)),
            package:StaticStr::new(params.package),
            package_version:params.package_version,
            file:CmpIgnored::new(StaticStr::new(params.file)),
            line:CmpIgnored::new(params.line),
            size:mem::size_of::<T>(),
            alignment:mem::align_of::<T>(),
            fields:StaticSlice::new(params.fields),
        }
    }
}


impl PTField{
    pub const fn new<T>(name:&'static str ,ty:&'static str)->Self{
        Self{
            name:StaticStr::new(name),
            ty:CmpIgnored::new(StaticStr::new(ty)),
            size:mem::size_of::<T>(),
            alignment:mem::align_of::<T>(),
        }
    }
}
