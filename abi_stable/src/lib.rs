/*!

This crate is for doing Rust-to-Rust ffi,
with a focus on loading libraries to program startup.

Currently this library has these features:

- ffi-safe equivalent of trait objects for any combination of a selection of traits.

- Provides ffi-safe alternatives to standard library types..

- Provides the `StableAbi` trait for asserting that types are ffi-safe.

- Checking at load-time that the types in the dynamic library have the expected layout,
    allowing for semver compatible changes while checking the layout of types.

- Provides the `StableAbi` derive macro to both assert that the type is ffi compatible,
    and to store the layout of the type in a constant.


# Rust-to-Rust FFI guidelines.

Types must implement StableAbi to be safely passed through the FFI boundary,
which can be done using the StableAbi derive macro.

These are the 3 kinds of types passed through FFI:

- Value kind:
    The layout of types passed by value must not change in a minor version.

- Prefix kind: 
    Types where `<Type as StableAbi>::ABI_INFO.is_prefix==true`,
    passed by reference and can increase in size so long as they only add fields at the end,
    as well as not change their alignment in newer versions.

- Opaque kind:
    Types wrapped in `VirtualWrapper<SomePointer<OpaqueType<Interface>>>`,
    whose layout can change in any version of the library,
    and can only be unwrapped back to the original type in the dynamic library/binary 
    that created it.

### Declaring enums

Adding variants or fields to a variant is disallowed in minor versions.

To represent non-exhaustive enums without fields it is recommended using structs and associated constants so that it is not UB to keep adding field-less variants in minor versions.

# Current limitatioss

While this library can check that the layout of datatypes passed through
ffi are compatible when the library is loaded,
it cannot currently check that auto-traits continue to be implemented by
the types in the dynamic library.

Once specialization is in beta,this library will add checks that types that implement
all built-in auto-traits continue to do so in future minor/patch versions of the same library.



*/

#![allow(unused_unsafe)]
#![warn(rust_2018_idioms)]

#[macro_use]
extern crate serde_derive;

#[macro_use(StableAbi)]
extern crate abi_stable_derive;

pub use abi_stable_derive::StableAbi;

#[macro_use]
mod impls;

#[macro_use]
pub mod static_str;

#[macro_use]
pub mod static_slice;

#[macro_use]
mod macros;

#[cfg(test)]
#[macro_use]
mod test_utils;

#[macro_use]
pub mod type_info;

#[macro_use]
pub mod traits;

#[macro_use]
pub mod abi_stability;
pub mod cabi_type;
pub mod erased_types;
// pub mod immovable_wrapper;
pub mod library;
pub mod marker_type;
pub mod opaque_type;
pub mod pointer_trait;
pub mod reexports;
pub mod std_types;
pub mod utils;
pub mod utypeid;
pub mod version;

#[cfg(test)]
#[macro_use]
pub mod test_macros;
#[cfg(test)]
pub mod layout_tests;

#[doc(hidden)]
pub mod abi_stable {
    pub use crate::*;
}

// Using an AtomicUsize so that it doesn't get put in read-only memory.
use std::sync::atomic::AtomicUsize;
static EXECUTABLE_IDENTITY: AtomicUsize = AtomicUsize::new(1);

//pub use shared_traits::with_lifetime::WithLifetime;

pub use crate::{
    cabi_type::CAbi,
    erased_types::VirtualWrapper,
    // immovable_wrapper::Immov,
    library::Library,
    opaque_type::{ErasedObject, OpaqueType},
    std_types::{
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
    },
    traits::{
        DeserializeImplType, ImplType, InterfaceType, IntoReprC, IntoReprRust, SerializeImplType,
    },
};

#[doc(hidden)]
pub use crate::{static_slice::StaticSlice, static_str::StaticStr};
