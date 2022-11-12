//! types and traits related to abi stability.

#[doc(hidden)]
pub mod abi_checking;
mod const_generics;
pub mod extra_checks;
pub mod get_static_equivalent;
pub mod stable_abi_trait;

pub use self::{
    abi_checking::exported_check_layout_compatibility as check_layout_compatibility,
    const_generics::ConstGeneric,
    get_static_equivalent::{GetStaticEquivalent, GetStaticEquivalent_},
    stable_abi_trait::{AbiConsts, PrefixStableAbi, StableAbi},
};

#[doc(no_inline)]
pub use self::extra_checks::{ExtraChecks, TypeChecker};
