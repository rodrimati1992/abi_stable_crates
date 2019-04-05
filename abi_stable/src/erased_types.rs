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
    traits::{ImplType, InterfaceType, SerializeImplType},
    CAbi, IntoReprC, IntoReprRust, OpaqueType, RBoxError, RCmpOrdering, RCow, RErr, ROk, ROption,
    RResult, RSlice, RString,
};

pub mod c_functions;
pub mod trait_objects;
pub mod virtual_wrapper;
pub mod vtable;

pub use self::{
    virtual_wrapper::{VirtualWrapper, VirtualWrapperTrait},
    vtable::{trait_selector, GetImplFlags, GetVtable},
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
