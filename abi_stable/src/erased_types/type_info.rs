//! Contains TypeInfo,metadata for a type.

use std::fmt;

use crate::{
    sabi_types::{Constructor, MaybeCmp, VersionStrings},
    std_types::{utypeid::UTypeId, RStr},
};

/// Metadata stored in the vtable of `DynTrait<_>`
#[derive(Debug, Eq, PartialEq)]
#[repr(C)]
#[derive(StableAbi)]
pub struct TypeInfo {
    pub size: usize,
    pub alignment: usize,
    #[doc(hidden)]
    pub _uid: Constructor<MaybeCmp<UTypeId>>,
    pub type_name: Constructor<RStr<'static>>,
    pub module: RStr<'static>,
    pub package: RStr<'static>,
    pub package_version: VersionStrings,
    #[doc(hidden)]
    pub _private_field: (),
}

impl TypeInfo {
    /// Whether the `self` is the TypeInfo for the same type as `other`
    pub fn is_compatible(&self, other: &Self) -> bool {
        self._uid == other._uid
    }
}

impl fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "type:{ty}\n\
             size:{size} alignment:{alignment}\n\
             module:'{module}'\n\
             package:'{package}'\n\
             package_version:{package_version}\n\
             ",
            ty = self.type_name,
            size = self.size,
            alignment = self.alignment,
            module = self.module,
            package = self.package,
            package_version = self.package_version
        )
    }
}

////////////////////////////////////////////
