/*!
StableAbi types that aren't wrappers for other types.
*/


pub mod static_ref;
pub mod move_ptr;
pub mod maybe_cmp;

pub use self::{
    static_ref::StaticRef,
    maybe_cmp::MaybeCmp,
    move_ptr::MovePtr,
};