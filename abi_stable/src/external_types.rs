//! Ffi wrapper for types defined outside the standard library.
//!
//! The modules here are named after the crates whose types are being wrapped.
//!

#[cfg(feature = "crossbeam-channel")]
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "channels")))]
pub mod crossbeam_channel;

pub mod parking_lot;

#[cfg(feature = "serde_json")]
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "serde_json")))]
pub mod serde_json;

pub use self::parking_lot::{RMutex, ROnce, RRwLock};

#[cfg(feature = "serde_json")]
pub use self::serde_json::{RawValueBox, RawValueRef};
