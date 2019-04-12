/*!
types and traits related to abi stability.
*/

#[macro_use]
pub mod type_layout;
pub mod abi_checking;
pub mod stable_abi_trait;

pub use self::{
    abi_checking::{check_abi_stability, check_abi_stability_for},
    stable_abi_trait::{
        AbiInfo, AbiInfoWrapper, StableAbi,
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
