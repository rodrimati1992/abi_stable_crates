//! Contains TypeInfo,metadata for a type.

use std::fmt;

use crate::{
    marker_type::NonOwningPhantom,
    sabi_types::{Constructor, MaybeCmp},
    std_types::{utypeid::UTypeId, RStr},
    type_level::downcasting::GetUTID,
    InterfaceType,
};

/// Metadata about a type.
#[derive(Debug, Eq, PartialEq)]
#[repr(C)]
#[derive(StableAbi)]
pub struct TypeInfo {
    ///
    pub size: usize,
    ///
    pub alignment: usize,
    #[doc(hidden)]
    pub _uid: Constructor<MaybeCmp<UTypeId>>,
    ///
    pub type_name: Constructor<RStr<'static>>,
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
             ",
            ty = self.type_name,
            size = self.size,
            alignment = self.alignment,
        )
    }
}

////////////////////////////////////////////

#[doc(hidden)]
pub struct TypeInfoFor<T, Interface, Downcasting>(NonOwningPhantom<(T, Interface, Downcasting)>);

impl<T, Interface, Downcasting> TypeInfoFor<T, Interface, Downcasting>
where
    Interface: InterfaceType,
    Downcasting: GetUTID<T>,
{
    /// The `&'static TypeInfo` constant, used when unerasing `DynTrait`s into a type.
    pub const INFO: &'static TypeInfo = &TypeInfo {
        size: std::mem::size_of::<T>(),
        alignment: std::mem::align_of::<T>(),
        _uid: Constructor(<Downcasting as GetUTID<T>>::UID),
        type_name: Constructor(crate::utils::get_type_name::<T>),
        _private_field: (),
    };
}
