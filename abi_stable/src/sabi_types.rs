/*!
StableAbi types that aren't wrappers for other types.
*/


pub mod ignored_wrapper;
pub mod late_static_ref;
pub mod maybe_cmp;
pub mod move_ptr;
pub mod return_value_equality;
pub mod static_ref;
pub mod version;


pub use self::{
    ignored_wrapper::CmpIgnored,
    static_ref::StaticRef,
    maybe_cmp::MaybeCmp,
    move_ptr::MovePtr,
    return_value_equality::ReturnValueEquality,
    late_static_ref::LateStaticRef,
    version::{VersionNumber,VersionStrings,ParseVersionError},
};