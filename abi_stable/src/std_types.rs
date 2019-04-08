pub mod arc;
pub mod boxed;
pub mod cmp_ordering;
pub mod cow;
pub mod option;
pub mod range;
pub mod result;
pub mod slice_mut;
pub mod slices;
pub mod static_slice;
pub mod static_str;
pub mod std_error;
pub mod std_io;
pub mod str;
pub mod string;
pub mod time;
pub mod tuple;
pub mod vec;

pub use self::{
    arc::RArc,
    boxed::RBox,
    cmp_ordering::RCmpOrdering,
    cow::RCow,
    option::{RNone, ROption, RSome},
    result::{RErr, ROk, RResult},
    slice_mut::RSliceMut,
    slices::RSlice,
    std_error::{RBoxError, UnsyncRBoxError},
    std_io::{RIoError, RIoErrorKind},
    str::RStr,
    string::RString,
    time::RDuration,
    tuple::{Tuple2, Tuple3, Tuple4},
    vec::RVec,
    static_str::StaticStr,
    static_slice::StaticSlice,
};