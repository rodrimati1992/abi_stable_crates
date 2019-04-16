/*!
Types and traits related to type erasure.
*/

use std::{
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    mem,
    ops::Deref,
};

use core_extensions::type_level_bool::{Boolean, False, True};

use serde::{Serialize, Serializer};

use crate::{
    traits::{IntoReprC, IntoReprRust},
    ZeroSized, 
    std_types::{RBoxError, RCmpOrdering, RCow, RErr, ROk, ROption,RResult, RSlice, RString,RStr},
};

pub(crate)mod c_functions;
pub mod trait_objects;
pub mod type_info;
pub mod virtual_wrapper;
pub mod vtable;
pub mod traits;

pub use self::{
    virtual_wrapper::{VirtualWrapper, VirtualWrapperTrait},
    vtable::{ GetVtable },
    traits::{ImplType, InterfaceType, SerializeImplType, DeserializeInterfaceType},
    type_info::TypeInfo,
};

use self::{
    vtable::{GetImplFlags},
};

/// The formatting mode for all std::fmt formatters.
///
/// For Display,"{}" is Default_ "{:#}" is Alternate
///
/// For Debug,"{:?}" is Default_ "{:#?}" is Alternate
///
/// etc.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub enum FormattingMode {
    Default_,
    Alternate,
}

//////////////////////////////////////////////////////////
