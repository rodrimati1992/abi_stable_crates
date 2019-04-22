/*!
types and traits related to abi stability.
*/

#[macro_use]
pub mod type_layout;
pub(crate) mod abi_checking;
pub mod stable_abi_trait;

#[cfg(test)]
mod layout_tests;

pub use self::{
    stable_abi_trait::{
        AbiInfo, AbiInfoWrapper, StableAbi,
        SharedStableAbi,
    },

};

use self::{
    stable_abi_trait::{
        GetAbiInfo,
    },
    type_layout::{
        LifetimeIndex, TLData, TLDataDiscriminant, TLEnumVariant, TLField,
        TLFieldAndType, TypeLayout, TypeLayoutParams, FullType,
    },
};

use self::type_layout::RustPrimitive;
