/*!
ffi-safe types that aren't wrappers for other types.
*/

mod constructor;
mod ignored_wrapper;
mod late_static_ref;
mod maybe_cmp;
mod move_ptr;
mod nul_str;
mod rmut;
mod rref;
pub mod rsmallbox;
mod static_ref;
pub mod version;

pub use self::{
    constructor::{Constructor, ConstructorOrValue},
    ignored_wrapper::CmpIgnored,
    late_static_ref::LateStaticRef,
    maybe_cmp::MaybeCmp,
    move_ptr::MovePtr,
    nul_str::NulStr,
    rmut::RMut,
    rref::RRef,
    rsmallbox::RSmallBox,
    static_ref::StaticRef,
    version::{ParseVersionError, VersionNumber, VersionStrings},
};
