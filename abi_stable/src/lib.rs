/*!

For Rust-to-Rust ffi,
with a focus on creating libraries loaded at program startup,
and with load-time type-checking.

This library allows defining Rust libraries that can be loaded at runtime,
even if they were built with a different Rust version than the crate that depends on it.

These are some usecases for this library:

- Converting a Rust dependency tree from compiling statically into a single binary,
    into one binary (and potentially) many dynamic libraries,
    allowing separate re-compilation on changes.

- Creating a plugin system (without support for unloading).

# Features

Currently this library has these features:

- Features the [`sabi_trait`] attribute macro, for creating ffi-safe trait objects.

- Ffi-safe equivalent of some trait objects with [`DynTrait`].

- Provides ffi-safe alternatives/wrappers for many standard library types,
    in the [`std_types`] module.

- Provides ffi-safe wrappers for some types defined in external crates,
    in the [`external_types`] module.

- Provides the [`StableAbi`] trait for asserting that types are ffi-safe.

- The [prefix types] feature for building extensible modules and vtables,
without breaking ABI compatibility.

- Supports ffi-safe [nonexhaustive enums], wrapped in [`NonExhaustive`].

- Checking at load-time that the types in the dynamic library have the expected layout,
    allowing for semver compatible changes while checking the layout of types.

- Provides the [`StableAbi` derive] macro
    to both assert that the type is ffi compatible,
    and to get the layout of the type at load-time to check that it is still compatible.

# Examples

For **examples** of using `abi_stable` you can look at [the readme example],
or for the crates in the examples directory in the repository for this crate.
This crate also has examples in the docs for most features.

To run the example crates you'll generally have to build the `*_impl` crate,
then run the `*_user` crate (all `*_user` crates should have a help message and a readme.md).


# Minimum Rust version

This crate support Rust back to 1.61.0

You can manually enable support for Rust past 1.61.0 with the `rust_*_*` cargo features.

# Crate Features

These are default cargo features that enable optional crates :

- "channels":
    Depends on `crossbeam-channel`,
    wrapping channels from it for ffi in `abi_stable::external_types::crossbeam_channel` .

- "serde_json":
    Depends on `serde_json`,
    providing ffi-safe equivalents of
    `&serde_json::value::RawValue` and `Box<serde_json::value::RawValue>`,
    in `abi_stable::external_types::serde_json` .


To disable the default features use:
```text
[dependencies.abi_stable]
version = "<current_version>"
default-features = false
features = [  ]
```
enabling the features you need in the `features` array.

### Manually enabled

These are crate features to manually enable support for newer language features:

- "rust_1_64": Turns many functions for converting types to slices into const fns.

- "rust_latest_stable":
Enables the "rust_1_*" features for all the stable releases.

# Glossary

`interface crate`: the crate that declares the public functions, types, and traits that
are necessary to load a library at runtime.

`ìmplementation crate`: A crate that implements all the functions in a interface crate.

`user crate`: A crate that depends on an `interface crate` and
loads 1 or more `ìmplementation crate`s for it.

`module`: refers to a struct of function pointers and other static values.
The root module of a library implements the [`RootModule`] trait.
These are declared in the `interface crate`,exported in the `implementation crate`,
and loaded in the `user crate`.

# Rust-to-Rust FFI types.

Types must implement [`StableAbi`] to be safely passed through the FFI boundary,
which can be done using the [`StableAbi` derive] macro.

For how to evolve dynamically loaded libraries you can look at the [library_evolution] module.

These are the kinds of types passed through FFI:

- Value kind:<br>
    This is the default kind when deriving StableAbi.
    The layout of these types must not change in a minor versions.

- [Nonexhaustive enums] :<br>
    Enums wrapped inside [`NonExhaustive`],
    which can add variants in minor versions of the library.

- [Trait objects] :<br>
    Trait object-like types generated using the [`sabi_trait`] attribute macro,
    which erase the type of the value they wrap,implements the methods of the trait,
    and can only be unwrapped back to the original type in the dynamic library/binary
    that created it.

- Opaque kind:<br>
    Types wrapped in [`DynTrait`],
    whose layout can change in any version of the library,
    and can only be unwrapped back to the original type in the dynamic library/binary
    that created it.

- [Prefix types] :<br>
    Types only accessible through some custom pointer types,
    most commonly vtables and modules,
    which can be extended in minor versions while staying ABI compatible,
    by adding fields at the end.

# Extra documentation

- [Unsafe code guidelines] :<br>
    Describes how to write unsafe code ,relating to this library.

- [Troubleshooting] :<br>
    Some problems and their solutions.

# Macros (derive and attribute)

- [`sabi_trait`] attribute macro:<br>
    For generating ffi-safe trait objects.

- [`StableAbi` derive] :<br>
    For asserting abi-stability of a type,
    and obtaining the layout of the type at runtime.

- [Nonexhaustive enums] :<br>
    Details for how to declare nonexhaustive enums.

- [Prefix types] \(using the StableAbi derive macro):<br>
    The way that *vtables* and *modules* are implemented,
    allowing extending them in minor versions of a library.

[`std_types`]: ./std_types/index.html
[`external_types`]: ./external_types/index.html
[prefix types]: ./docs/prefix_types/index.html
[Prefix types]: ./docs/prefix_types/index.html
[nonexhaustive enums]: ./docs/sabi_nonexhaustive/index.html
[Nonexhaustive enums]: ./docs/sabi_nonexhaustive/index.html
[library_evolution]: ./docs/library_evolution/index.html
[`NonExhaustive`]: ./nonexhaustive_enum/struct.NonExhaustive.html

[the readme example]:
https://github.com/rodrimati1992/abi_stable_crates/blob/master/readme.md#readme_example

[`RootModule`]: ./library/trait.RootModule.html
[`StableAbi`]: ./abi_stability/stable_abi_trait/trait.StableAbi.html
[`sabi_trait`]: ./attr.sabi_trait.html
[Trait objects]: ./attr.sabi_trait.html
[`StableAbi` derive]: ./derive.StableAbi.html
[`DynTrait`]: ./struct.DynTrait.html
[Troubleshooting]: ./docs/troubleshooting/index.html
[Unsafe code guidelines]: ./docs/unsafe_code_guidelines/index.html

*/

// `improper_ctypes` is way too noisy of a lint,
// every single warning was a false positive.
// the true positives are caught by the StableAbi trait.
#![allow(improper_ctypes)]
#![allow(improper_ctypes_definitions)]
#![allow(non_camel_case_types)]
#![deny(unused_must_use)]
#![warn(rust_2018_idioms)]
#![allow(clippy::needless_doctest_main)]
#![allow(clippy::bool_assert_comparison)]
#![allow(clippy::zero_prefixed_literal)]
#![allow(clippy::type_complexity)]
#![allow(clippy::ptr_offset_with_cast)]
#![allow(clippy::assertions_on_constants)]
#![deny(missing_docs)]
#![deny(clippy::missing_safety_doc)]
// #![deny(clippy::missing_const_for_fn)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(feature = "docsrs", feature(doc_cfg))]

#[macro_use]
extern crate serde_derive;

#[macro_use(StableAbi)]
extern crate abi_stable_derive;

extern crate self as abi_stable;

include! {"./proc_macro_reexports/get_static_equivalent.rs"}
include! {"./proc_macro_reexports/export_root_module.rs"}
include! {"./proc_macro_reexports/sabi_extern_fn.rs"}
include! {"./proc_macro_reexports/sabi_trait_attribute.rs"}
include! {"./proc_macro_reexports/stable_abi_derive.rs"}

#[doc(no_inline)]
pub use abi_stable::sabi_types::{RMut, RRef};

use abi_stable_derive::impl_InterfaceType;

#[doc(hidden)]
pub use abi_stable_derive::get_root_module_static;

#[macro_use]
mod impls;

#[macro_use]
mod internal_macros;

#[macro_use]
mod macros;

#[cfg(test)]
#[macro_use]
mod test_macros;

#[allow(missing_docs)]
#[cfg(feature = "testing")]
#[macro_use]
pub mod test_utils;

#[cfg(test)]
mod misc_tests;

#[macro_use]
pub mod utils;

#[macro_use]
pub mod const_utils;

#[macro_use]
pub mod traits;

pub mod for_examples;

#[macro_use]
pub mod abi_stability;
#[macro_use]
pub mod erased_types;
pub mod external_types;
#[macro_use]
pub mod library;
pub mod inline_storage;
pub mod marker_type;
mod multikey_map;
pub mod nonexhaustive_enum;
pub mod pointer_trait;
pub mod prefix_type;
pub mod type_layout;

#[doc(hidden)]
pub mod derive_macro_reexports;

// `pmr` is what I call "private" reexport for macros in newer crates.
#[doc(hidden)]
pub use self::derive_macro_reexports as pmr;

pub mod sabi_types;
pub mod std_types;

pub mod reflection;
pub mod type_level;

pub mod docs;

pub mod sabi_trait;

/// The header used to identify the version number of abi_stable
/// that a dynamic libraries uses.
pub static LIB_HEADER: library::AbiHeader = library::AbiHeader::VALUE;

/// Miscelaneous items re-exported from core_extensions.
pub mod reexports {
    pub use core_extensions::{
        type_level_bool::{False, True},
        utils::transmute_ignore_size,
        SelfOps,
    };
}

#[doc(hidden)]
pub const ABI_STABLE_VERSION: sabi_types::VersionStrings = package_version_strings!();

/*
I am using this static as the `identity` of this dynamic library/executable,
this assumes that private static variables don't get merged between
Rust dynamic libraries that have a different global allocator.

If the address of this is the same among dynamic libraries that have *different*
allocators,please create an issue for this.
*/
use std::sync::atomic::AtomicUsize;
static EXECUTABLE_IDENTITY: AtomicUsize = AtomicUsize::new(1);

#[doc(inline)]
pub use crate::{
    abi_stability::StableAbi,
    erased_types::{dyn_trait::DynTrait, InterfaceType},
};

#[doc(hidden)]
pub mod globals {
    use crate::{
        abi_stability::abi_checking::check_layout_compatibility_for_ffi,
        sabi_types::LateStaticRef,
        std_types::{RBoxError, RResult},
        type_layout::TypeLayout,
        utils::leak_value,
    };

    #[repr(C)]
    #[derive(StableAbi)]
    // #[sabi(debug_print)]
    pub struct Globals {
        pub layout_checking:
            extern "C" fn(&'static TypeLayout, &'static TypeLayout) -> RResult<(), RBoxError>,
    }

    impl Globals {
        pub fn new() -> &'static Self {
            leak_value(Globals {
                layout_checking: check_layout_compatibility_for_ffi,
            })
        }
    }

    pub(crate) static GLOBALS: LateStaticRef<&Globals> = LateStaticRef::new();

    #[inline(never)]
    pub fn initialized_globals() -> &'static Globals {
        GLOBALS.init(Globals::new)
    }

    #[inline(never)]
    pub extern "C" fn initialize_globals_with(globs: &'static Globals) {
        GLOBALS.init(|| globs);
    }
}

#[cfg(all(test, not(feature = "testing")))]
compile_error! { "tests must be run with the \"testing\" feature" }

#[cfg(miri)]
extern "Rust" {
    /// Miri-provided extern function to mark the block `ptr` points to as a "root"
    /// for some static memory. This memory and everything reachable by it is not
    /// considered leaking even if it still exists when the program terminates.
    ///
    /// `ptr` has to point to the beginning of an allocated block.
    fn miri_static_root(ptr: *const u8);
}

// Removed readme testing, now the readme code is tested with actual crates.
// #[cfg(doctest)]
// #[doc = include_str!("../../readme.md")]
// pub struct ReadmeTest;
