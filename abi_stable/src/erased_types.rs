/*!
Types and traits related to type erasure.
*/

use std::{
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    ops::Deref,
};


use serde::{Serialize, Serializer};

use crate::{
    traits::{IntoReprC, IntoReprRust},
    std_types::{
        RBoxError, RCmpOrdering, RCow, RErr, ROk, ROption,RResult, RString,RStr,
        RSlice, RSliceMut
    },
    type_level::{
        //option::{Some_,None_,SomeTrait}
        bools::{Boolean, False, True},
    },
};

pub(crate)mod c_functions;

/// `impl InterfaceType`s used in examples.
pub mod interfaces;

pub mod trait_objects;

pub mod type_info;

pub(crate) mod iterator;

pub mod dyn_trait;

pub(crate) mod vtable;

pub mod traits;



pub use self::{
    dyn_trait::{DynTrait, DynTraitBound},
    vtable::{ GetVtable,InterfaceBound},
    traits::{
        ImplType, InterfaceType, SerializeImplType, DeserializeOwnedInterface,
        DeserializeBorrowedInterface,IteratorItem,
    },
    type_info::TypeInfo,
};


/// The formatting mode for all std::fmt formatters.
///
/// For Display,"{}" is Default_ "{:#}" is Alternate
///
/// For Debug,"{:?}" is Default_ "{:#?}" is Alternate
///
/// etc.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, StableAbi)]
pub enum FormattingMode {
    Default_,
    Alternate,
}
