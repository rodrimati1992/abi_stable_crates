/*!

A library is for doing Rust-to-Rust ffi,
with a focus on creating libraries loadable at program startup.

This library allows defining Rust libraries that can be loaded at runtime,
even if they were built with a different Rust version than the crate that depends on it,

# Features

Currently this library has these features:

- ffi-safe equivalent of trait objects for any combination of a selection of traits.

- Provides ffi-safe alternatives to standard library types..

- Provides the `StableAbi` trait for asserting that types are ffi-safe.

- Checking at load-time that the types in the dynamic library have the expected layout,
    allowing for semver compatible changes while checking the layout of types.

- Provides the `StableAbi` derive macro to both assert that the type is ffi compatible,
    and to get the layout of the type at runtime to check that it is still compatible.

# Documentation

The documentation is done entirely through modules,so that it can be read in `docs.rs/abi_stable`.

[Documentation for the `#[derive(StableAbi)]` derive macro
](./docs/stable_abi_trait/index.html)

[Documentation for the `#[mangle_library_getter]` attribute
](./docs/mangle_library_getter/index.html)

# Examples

For **examples** of using `abi_stable` you can look at the abi_stable_example_* crates,
in the repository for this crate.

# Glossary

`interface crate`:the crate that declares the public functions and types that 
are necessary to load the library at runtime.

`ìmplementation crate`:A crate that implements all the functions in the interface crate.

`user crate`:A crate that depends on an `interface crate` and 
loads 1 or more `ìmplementation crate`s for it.

`module`:refers to a struct of function pointers and other static values,
and implement the ModuleTrait trait.
These are declared in the `interface crate`,exported in the `implementation crate`,
and loaded in the `user crate`.

# Rust-to-Rust FFI guidelines.

Types must implement StableAbi to be safely passed through the FFI boundary,
which can be done using the StableAbi derive macro.

These are the 2 kinds of types passed through FFI:

- Value kind:
    The layout of types passed by value must not change in a minor version.
    This is the default kind when deriving StableAbi.

- Opaque kind:
    Types wrapped in `VirtualWrapper<SomePointer<OpaqueType<Interface>>>`,
    whose layout can change in any version of the library,
    and can only be unwrapped back to the original type in the dynamic library/binary 
    that created it.

### Declaring enums

Adding variants or fields to a variant is disallowed in minor versions.

To represent non-exhaustive enums without fields it is recommended using structs and associated constants so that it is not UB to keep adding field-less variants in minor versions.


*/

#![allow(unused_unsafe)]
#![warn(rust_2018_idioms)]

#[macro_use]
extern crate serde_derive;

#[macro_use(StableAbi)]
extern crate abi_stable_derive;

pub use abi_stable_derive::{StableAbi,mangle_library_getter};

#[macro_use]
mod impls;


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
pub mod ignored_wrapper;
pub mod marker_type;
pub mod opaque_type;
pub mod pointer_trait;
pub mod reexports;
pub mod std_types;
pub mod utils;
pub mod utypeid;
pub mod lazy_static_ref;
pub mod version;


pub mod docs{
    pub mod stable_abi_trait;
    pub mod mangle_library_getter;
}

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

#[doc(inline)]
pub use crate::{
    abi_stability::StableAbi,
    erased_types::VirtualWrapper,
    library::Library,
    opaque_type::{ErasedObject, OpaqueType},
    traits::{ImplType, InterfaceType},
};

