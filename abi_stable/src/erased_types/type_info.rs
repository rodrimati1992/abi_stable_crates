/*!
Contains TypeInfo,metadata for a type.
*/

use std::fmt;

use crate::{
    version::VersionStrings, 
    sabi_types::MaybeCmp,
    std_types::{StaticStr,utypeid::UTypeId},
    return_value_equality::ReturnValueEquality,
};


/// Metadata stored in the vtable of `DynTrait<_>`
#[derive(Debug, Eq, PartialEq)]
#[repr(C)]
#[derive(StableAbi)]
pub struct TypeInfo {
    pub size: usize,
    pub alignment: usize,
    #[doc(hidden)]
    pub _uid: ReturnValueEquality<MaybeCmp<UTypeId>>,
    pub name: StaticStr,
    pub file: StaticStr,
    pub package: StaticStr,
    pub package_version: VersionStrings,
    #[doc(hidden)]
    pub _private_field: (),
}

impl TypeInfo {
    /// Whether the `self` is the TypeInfo for the same type as `other`
    pub fn is_compatible(&self, other: &Self) -> bool {
        self._uid==other._uid
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

