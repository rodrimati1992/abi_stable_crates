//! Contains many ffi-safe equivalents of standard library types.
//! The vast majority of them can be converted to and from std equivalents.
//!
//! For ffi-safe equivalents/wrappers of types outside the standard library go to
//! the [external_types module](../external_types/index.html)

pub(crate) mod arc;
pub(crate) mod boxed;
pub(crate) mod cmp_ordering;
pub mod cow;
pub mod map;
pub(crate) mod option;
pub(crate) mod range;
pub(crate) mod result;
pub(crate) mod slice_mut;
pub(crate) mod slices;
pub(crate) mod std_error;
pub(crate) mod std_io;
pub(crate) mod str;
pub mod string;
pub(crate) mod time;
pub(crate) mod tuple;
pub mod utypeid;
pub mod vec;

/// Some types from the `std::sync` module have ffi-safe equivalents in
/// `abi_stable::external_types`.
///
/// The `sync::{Mutex,RwLock,Once}` equivalents are declared in
/// `abi_stable::external_types::parking_lot`
///
/// The `mpsc` equivalents are declared in
/// `abi_stable::external_types::crossbeam_channel`,
/// this is enabled by default with the `channels`/`crossbeam-channel` cargo feature.
pub mod sync {}

#[doc(inline)]
pub use self::{
    arc::RArc,
    boxed::RBox,
    cmp_ordering::RCmpOrdering,
    cow::{RCow, RCowSlice, RCowStr, RCowVal},
    map::RHashMap,
    option::{RNone, ROption, RSome},
    result::{RErr, ROk, RResult},
    slice_mut::RSliceMut,
    slices::RSlice,
    std_error::{RBoxError, RBoxError_, SendRBoxError, UnsyncRBoxError},
    std_io::{RIoError, RIoErrorKind, RSeekFrom},
    str::RStr,
    string::RString,
    time::RDuration,
    tuple::{Tuple1, Tuple2, Tuple3, Tuple4},
    utypeid::UTypeId,
    vec::RVec,
};
