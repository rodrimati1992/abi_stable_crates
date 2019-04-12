/*!
Contains TypeInfo,metadata for a type.
*/

use std::fmt;

use crate::{
    version::VersionStrings, 
    std_types::{StaticStr,utypeid::UTypeId},
    return_value_equality::ReturnValueEquality,
};


/// Metadata stored in the vtable of `VirtualWrapper<_>`
#[derive(Debug, Eq, PartialEq)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct TypeInfo {
    pub size: usize,
    pub alignment: usize,
    pub uid: ReturnValueEquality<UTypeId>,
    pub name: StaticStr,
    pub file: StaticStr,
    pub package: StaticStr,
    pub package_version: VersionStrings,
    #[doc(hidden)]
    pub _private_field: (),
}

impl TypeInfo {
    pub fn is_compatible(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "type:{}\n\
             size:{} alignment:{}\n\
             path:'{}'\n\
             package:'{}'\n\
             package_version:{}\n\
             ",
            self.name, self.size, self.alignment, self.file, self.package, self.package_version
        )
    }
}


////////////////////////////////////////////

