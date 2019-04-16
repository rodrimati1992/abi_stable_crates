/*!

For Rust-to-Rust ffi,with a focus on creating libraries loaded at program startup.

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


#[doc(inline)]
pub use abi_stable_derive::StableAbi;

#[doc(inline)]
pub use abi_stable_derive::{
    mangle_library_getter,
    export_sabi_module,
};

#[macro_use]
mod impls;


#[macro_use]
mod macros;

#[cfg(test)]
#[macro_use]
mod test_utils;

#[macro_use]
pub mod traits;

#[macro_use]
pub mod abi_stability;
// pub mod cabi_type;
pub mod erased_types;
// pub mod immovable_wrapper;
pub mod library;
pub mod ignored_wrapper;
pub mod marker_type;
pub mod pointer_trait;

#[doc(hidden)]
pub mod return_value_equality;

#[doc(hidden)]
pub mod derive_macro_reexports;
pub mod std_types;


pub mod utils;
pub mod lazy_static_ref;
pub mod version;

#[cfg(test)]
#[macro_use]
mod test_macros;
#[cfg(test)]
mod layout_tests;


/**
Type-level booleans.

This is a re-export from `core_extensions::type_level_bool`,
so as to allow glob imports (`abi_stable::type_level_bool::*`)
without worrying about importing too many items.
*/
pub mod type_level_bool{
    #[doc(inline)]
    pub use core_extensions::type_level_bool::{True,False,Boolean};
}

/// Miscelaneous items re-exported from core_extensions.
pub mod reexports{
    pub use core_extensions::SelfOps;
}

/*
I am using this static as the `identity` of this dynamic library/executable,
this assumes that private static variables don't get merged between 
Rust dynamic libraries that have a different global allocator.

If the address of this is the same among dynamic libraries that have *different* 
allocators,please create an issue for this.
*/
use std::sync::atomic::AtomicUsize;
static EXECUTABLE_IDENTITY: AtomicUsize = AtomicUsize::new(1);

use crate::abi_stability::stable_abi_trait::SharedStableAbi;

#[doc(inline)]
pub use crate::{
    abi_stability::StableAbi,
    erased_types::{VirtualWrapper,ImplType, InterfaceType},
    library::Library,
    marker_type::{ErasedObject, OpaqueType},
};

