//! Types and traits related to type erasure.

use std::{
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
};

use serde::{Serialize, Serializer};

use crate::{
    std_types::{
        RBoxError, RCmpOrdering, RErr, ROk, ROption, RResult, RSlice, RSliceMut, RStr, RString,
    },
    traits::{IntoReprC, IntoReprRust},
};

#[macro_use]
mod enabled_traits_macro;

pub(crate) mod c_functions;

/// Types that implement `InterfaceType`, used in examples.
pub mod interfaces;

pub mod trait_objects;

pub(crate) mod type_info;

pub(crate) mod iterator;

pub(crate) mod dyn_trait;

#[macro_use]
pub(crate) mod vtable;

pub(crate) mod traits;

#[doc(inline)]
pub use crate::DynTrait;

pub use self::{
    dyn_trait::UneraseError,
    traits::{
        DeserializeDyn, InterfaceType, IteratorItem, IteratorItemOrDefault, SerializeProxyType,
        SerializeType,
    },
    type_info::TypeInfo,
    vtable::{MakeRequiredTraits, RequiredTraits},
};

pub use self::vtable::MakeVTable;
pub use self::vtable::VTable_Ref;

#[doc(no_inline)]
pub use crate::type_level::downcasting::{TD_CanDowncast, TD_Opaque};

/// The formatting mode for all std::fmt formatters.
///
/// For Display,"{}" is Default_ "{:#}" is Alternate
///
/// For Debug,"{:?}" is Default_ "{:#?}" is Alternate
///
/// etc.
#[doc(hidden)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, StableAbi)]
pub enum FormattingMode {
    Default_,
    Alternate,
}
