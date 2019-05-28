/*!
StableAbi types that aren't wrappers for other types.
*/


pub mod static_ref;
pub mod move_ptr;

pub use self::{
    static_ref::StaticRef,
    move_ptr::MovePtr,
};