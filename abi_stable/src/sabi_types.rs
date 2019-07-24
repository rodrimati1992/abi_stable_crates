/*!
ffi-safe types that aren't wrappers for other types.
*/


pub mod ignored_wrapper;
mod late_static_ref;
mod maybe_cmp;
pub mod move_ptr;
mod return_value_equality;
mod static_ref;
pub mod version;
pub mod rsmallbox;


pub use self::{
    ignored_wrapper::CmpIgnored,
    static_ref::StaticRef,
    maybe_cmp::MaybeCmp,
    move_ptr::MovePtr,
    return_value_equality::ReturnValueEquality,
    rsmallbox::RSmallBox,
    late_static_ref::LateStaticRef,
    version::{VersionNumber,VersionStrings,ParseVersionError},
};