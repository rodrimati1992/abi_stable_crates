/*!
Types and traits related to type erasure.
*/

use std::{
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
};


use serde::{Serialize, Serializer};

use crate::{
    traits::{IntoReprC, IntoReprRust},
    std_types::{
        RBoxError, RCmpOrdering, RErr, ROk, ROption,RResult, RString,RStr,
        RSlice, RSliceMut
    },
};


#[macro_use]
mod enabled_traits_macro;

pub(crate)mod c_functions;

/// Types that implement `InterfaceType`, used in examples.
pub mod interfaces;

pub mod trait_objects;

pub(crate) mod type_info;

pub(crate) mod iterator;

pub(crate) mod dyn_trait;

#[macro_use]
pub(crate) mod vtable;

pub(crate) mod traits;


pub use self::{
    dyn_trait::{
        DynTrait,
        DynTraitBound,
        GetVWInterface,
        UneraseError,
    },
    vtable::{ InterfaceBound,VTableDT },
    traits::{
        ImplType, InterfaceType, 
        DeserializeDyn, 
        SerializeImplType, SerializeProxyType, 
        IteratorItem,IteratorItemOrDefault,
    },
    type_info::TypeInfo,
};

#[doc(hidden)]
pub use self::vtable::GetVtable;

#[doc(no_inline)]
pub use crate::type_level::unerasability::{TU_Unerasable,TU_Opaque};


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
