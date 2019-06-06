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
    pub field_names:StaticStr,
}


/// Parameters to construct a PTStructLayout.
pub struct PTStructLayoutParams{
    pub name: &'static str,
    pub generics:&'static str,
    pub package: &'static str,
    pub package_version: VersionStrings,
    pub file:&'static str, // This is for the Debug string
    pub line:u32, // This is for the Debug string
    pub field_names:&'static str,
}


//////////////////////////////////////////////////////////////


impl PTStructLayout{
    pub const fn new(params:PTStructLayoutParams)->Self{
        Self{
            name:StaticStr::new(params.name),
            generics:CmpIgnored::new(StaticStr::new(params.generics)),
            package:StaticStr::new(params.package),
            package_version:params.package_version,
            file:CmpIgnored::new(StaticStr::new(params.file)),
            line:CmpIgnored::new(params.line),
            field_names:StaticStr::new(params.field_names),
        }
    }

    pub fn get_field_names(&self)->impl Iterator<Item=&'static str>{
        self.field_names.as_str().split(';').filter(|x| !x.is_empty() )
    }

    pub fn get_field_names_vec(&self)->Vec<&'static str>{
        self.get_field_names().collect()
    }

    pub fn get_field_name(&self,field_index:usize)->Option<&'static str>{
        self.get_field_names().nth(field_index)
    }
}
