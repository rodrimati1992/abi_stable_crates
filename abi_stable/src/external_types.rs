/*!
Ffi wrapper for types defined outside the standard library.

The modules here are named after the crates whose types are being wrapped.


*/

#[cfg(feature="crossbeam-channel")]
pub mod crossbeam_channel;

#[cfg(not(feature="crossbeam-channel"))]
/// This is disabled,
/// enable the "channels" feature to get ffi-safe wrappers for crossbeam channels.
pub mod crossbeam_channel{}


pub mod parking_lot;

#[cfg(feature="serde_json")]
pub mod serde_json;


pub use self::{
    parking_lot::{RMutex,RRwLock,ROnce},
};
    

#[cfg(feature="serde_json")]
pub use self::serde_json::{RawValueRef,RawValueBox};

