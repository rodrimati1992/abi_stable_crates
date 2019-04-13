[![Build Status](https://travis-ci.org/rodrimati1992/abi_stable_crates.svg?branch=master)](https://travis-ci.org/rodrimati1992/abi_stable_crates)

A library is for doing Rust-to-Rust ffi,
with a focus on creating libraries loadable at program startup.

This library allows defining Rust libraries that can be loaded at runtime,
even if they were built with a different Rust version than the crate that depends on it.


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

# Planned features

### 0.2

Adding support for vtables/modules that can add fields at the end in minor versions,
this will allow library evolution beyond adding more modules.

### Eventually

WASM support,with the same features as native dynamic libraries,
once WASM supports dynamic linking.



# Not-currently-planned features

Supporting library unloading,
since this requires building the entire library with the assumption that anything 
might get unloaded at any time.
If someone can make an argument that this is easy enough to add support for,it might be added.


# Architecture


Users of this library are expected to follow this architecture:

Create an `interface crate`,
which declares all the functions and the datatypes passed to and returned by those functions,
as well as the `interface types`(types which implement InterfaceType,used by `VirtualWrapper<_>`).

Create an `implementation crate` that implements all the functions in the `interface crate`,
creates a `library getter function`,
and declares the `implementation types`
(types which implement ImplType,wrapped in `VirtualWrapper<_>` to use in ffi) 
for the `interface types`.

Creates a `user crate`,which declares the `interface crate` as a dependency,
passes the directory/folder of the `implementation crate` dynamic library to 
`<interface_crate::SomeLibType as ModuleTrait>::load_library` to get the library functions,
and then store them in a lazy_static or equivalent to access them afterwards.

For **examples** of this architecture you can look at the abi_stable_example_* crates,
in the repository for this crate.

# Known limitations

### Api evolution

This library doesn't currently allow adding functions in modules of the `implementation crate`
relative to the `interface crate` in minor versions,this will be fixed once `0.2` is released.

Until the `0.2` is released (at most at the end of May-2019),
you can add more modules instead of functions-within-the-same-module as a workaround.

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

This crate support Rust back to 1.33,
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
