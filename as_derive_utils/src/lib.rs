#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::needless_late_init)]

#[doc(hidden)]
#[macro_use]
pub mod macros;

#[doc(hidden)]
pub mod gen_params_in;

#[doc(hidden)]
pub mod to_token_fn;

#[doc(hidden)]
pub mod datastructure;

#[doc(hidden)]
pub mod utils;

#[doc(hidden)]
pub mod parse_utils;

#[doc(hidden)]
pub use crate::to_token_fn::ToTokenFnMut;

#[cfg(feature = "testing")]
#[doc(hidden)]
pub mod test_framework;
