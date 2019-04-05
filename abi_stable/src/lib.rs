/*!

# Extern function writing guidelines.

When writing extern "C" functions,take this things into consideration:

- The layout of stack allocated structs must not change in a minor version.
    While a heap allocated struct,with private fields, can add fields in minor versions.

- A stack allocated struct must represent generic raw pointers and references
    using the CAbi datatype,so as to make casting function pointers valid.

- A stack allocated struct ought not use the generic parameters declared in an extern function,
    unless the layout can't change based on the generic arguments passed
    (ie:they are used only on pointers internally).
    This is to make type erasure possible.

### Declaring enums

Adding variants or fields to a variant is disallowed in minor versions.

To represent non-exhaustive enums it is recommended using structs and associated constants so that it is not UB to keep adding variants in minor versions.

There is no currently recommended way to represent non-exhaustive enums with fields.

### Casting function pointers

According to the C standard casting function pointers with incompatible types
is undefined behavior,even in the case of casting `*const T` parameters to `*const c_void`.

To prevent this UB,all types which contain raw pointers/references to generic types
must use the CAbi datatype,
or store it as a pointer to ErasedObject and define a method
to cast it back to the original type.


### CAbi datatype

This type wraps pointers for passing them through extern functions.

These are the currently supported conversions:

- CAbi<*const T> <-> *const T
- CAbi<*mut T> <-> *mut T
- CAbi<& T> <-> &T
- CAbi<&mut T> <-> &mut T

All pointer types declared in the abi_stable library don't need
their element type to be wrapped in a CAbi

# Current limitatioss

While this library can check that the layout of datatyoes passed through
ffi are compatible when the library is loaded,
it cannot currently check that auto-traits continue to be implemented by
the types in the dynamic library.

Once specialization lands,this library will add checks that types that implement
all built-in auto-traits continue to do so in future minor/patch versions of the same library,



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
