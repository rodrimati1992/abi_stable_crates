/*!
Contains TypeInfo,metadata for a type.
*/

use std::fmt;

use crate::{
    version::VersionStrings, 
    std_types::{StaticStr,utypeid::UTypeId,ROption,RSome,RNone},
    return_value_equality::ReturnValueEquality,
};


/// Metadata stored in the vtable of `DynTrait<_>`
#[derive(Debug, Eq, PartialEq)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct TypeInfo {
    pub size: usize,
    pub alignment: usize,
    #[doc(hidden)]
    pub _uid: ReturnValueEquality<ROption<UTypeId>>,
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
        match ((self._uid.function)(),(other._uid.function)() ) {
            (RSome(l),RSome(r))=>l==r,
            _=>false,
        }
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

