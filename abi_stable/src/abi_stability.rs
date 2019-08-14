/*!
types and traits related to abi stability.
*/

pub(crate) mod abi_checking;
pub mod get_static_equivalent;
pub mod stable_abi_trait;


mod layout_tests;

pub use self::{
    abi_checking::exported_check_layout_compatibility as check_layout_compatibility,
    get_static_equivalent::{GetStaticEquivalent_,GetStaticEquivalent},
    stable_abi_trait::{StableAbi,SharedStableAbi,AbiConsts},
};
