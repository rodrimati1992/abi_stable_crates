[![Build Status](https://travis-ci.org/rodrimati1992/abi_stable_crates.svg?branch=master)](https://travis-ci.org/rodrimati1992/abi_stable_crates)

For Rust-to-Rust ffi,
with a focus on creating libraries loaded at program startup,
type-checked at load-time.

This library allows defining Rust libraries that can be loaded at runtime,
even if they were built with a different Rust version than the crate that depends on it.

These are some usecases for this library:
    
- Converting a Rust dependency tree from compiling statically into a single binary,
    into a binary and dynamic libraries,
    allowing separate re-compilation on changes.

- Creating a plugin system (without support for unloading).
    
# Features

Currently this library has these features:

- ffi-safe equivalent of trait objects for any combination of a selection of traits.

- Provides ffi-safe alternatives to many standard library types..

- Provides the `StableAbi` trait for asserting that types are ffi-safe.

- Features for building extensible modules and vtables,without breaking ABI compatibility.

- Checking at load-time that the types in the dynamic library have the expected layout,
    allowing for semver compatible changes while checking the layout of types.

- Provides the `StableAbi` derive macro to both assert that the type is ffi compatible,
    and to get the layout of the type at load-time to check that it is still compatible.

# Examples

For **examples** of using `abi_stable` you can look at the crates in the examples directory ,
in the repository for this crate.

To run the examples generally you'll have to build the `*_impl` crate,
then run the `*_user` crate (all `*_user` crates should have a help message).


# Safety

This library ensures that the loaded libraries are safe to use through these mechanisms:

- Types are recursively checked when the dynamic library is loaded,
    before any function can be called.

- The name of the function which exports the root module of the library is mangled
    to prevent mixing of incompatible abi_stable versions.
    Each `0.y.0` version and `x.0.0` version of abi_stable defines its own ABI 
    which is incompatible with previous versions.

Note that this library assumes that dynamic libraries come from a benign source,
these checks are done purely to detect programming errors.

# Planned features

### Eventually

WASM support,with the same features as native dynamic libraries,
once WASM supports dynamic linking.

# Not-currently-planned features

Supporting library unloading,
since this requires building the entire library with the assumption that anything 
might get unloaded at any time.

# Architecture


Users of this library are expected to follow this architecture:

### Interface crate

A crate which declares:

- The root module (a structs of function pointers/other modules),
    which implements the `RootModule` trait,
    exported from the dynamic library.

- All the sub-modules of the root module.

- All the public types passed to and returned by the functions.

- Optionally:declares ìnterface types,types which implement InterfaceType,
    used to specify the traits usable in the VirtualWrapper ffi-safe trait object .


### Implementation crate

The crate compiled as a dynamic library that:

- Implements all the functions declared in the `interface crate`.

- Declares a function to export the root module,
    using the `export_sabi_module` attribute to export the module.

- Optionally:create types which implement `ImplType<Iterface= Interface >`,
    where `Interface` is some interface type from the interface crate,
    so as to be able to use wrap it in `VirtualWrapper`s of that interface.

### User crate

A crate that that declares the `ìnterface crate` as a dependency,
and loads the pre-compiled `implementation crate` dynamic library from some path.


# Known limitations

### Enums with fields

You can't add variants to enums with fields in the `interface crate` in minor versions.

Adding a variant to an enum with fields in a minor version is a breaking change,
since `implementation crates` have to be usable with  previous minor versions of 
the `interface crate`.
If the `implementation crate` returns an enum in the interface it cannot add variants because 
it would break users of previous `interface crates` at runtime.

Using unions to solve this does not currently work since they don't work with non-Copy types,
and I'd rather have a complete solution.

Here is the relevant rfcs for unions with Drop types:
https://github.com/rust-lang/rfcs/blob/master/text/2514-union-initialization-and-drop.md

# Minumum Rust version

This crate support Rust back to 1.34
(previously 1.33,but had to abandon it because of an impossible to 
avoid internal compiler error related to associated types as types of associated constants),
using a build script to automatically enable features from newer versions.

# Cargo Features

If it becomes possible to disable build scripts,
you can manually enable support for Rust past 1.33 features with the `rust_*_*` cargo features.

# License

abi_stable is licensed under either of

    Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
    MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in abi_stable by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
