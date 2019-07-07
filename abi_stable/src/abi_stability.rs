/*!
types and traits related to abi stability.
*/

pub(crate) mod abi_checking;
pub mod stable_abi_trait;


mod layout_tests;

pub use self::{
    abi_checking::exported_check_layout_compatibility as check_layout_compatibility,
    stable_abi_trait::{
        AbiInfo, AbiInfoWrapper, StableAbi,
        SharedStableAbi,
    },
};
