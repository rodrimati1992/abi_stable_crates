/*!
ffi-safe types that aren't wrappers for other types.
*/


mod constructor;
pub mod ignored_wrapper;
mod late_static_ref;
mod nul_str;
mod maybe_cmp;
pub mod move_ptr;
mod static_ref;
mod rref;
pub mod version;
pub mod rsmallbox;


pub use self::{
    constructor::{Constructor,ConstructorOrValue},
    ignored_wrapper::CmpIgnored,
    static_ref::StaticRef,
    nul_str::NulStr,
    maybe_cmp::MaybeCmp,
    move_ptr::MovePtr,
    rref::RRef,
    rsmallbox::RSmallBox,
    late_static_ref::LateStaticRef,
    version::{VersionNumber,VersionStrings,ParseVersionError},
};