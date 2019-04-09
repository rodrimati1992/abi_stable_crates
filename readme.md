A library is for doing Rust-to-Rust ffi,
with a focus on loading libraries to program startup.

This library allows moving loading of Rust libraries to runtime,
even if they were built with different Rust versions,


# Features

Currently this library has these features:

- ffi-safe equivalent of trait objects for any combination of a selection of traits.

- Provides ffi-safe alternatives to standard library types..

- Provides the `StableAbi` trait for asserting that types are ffi-safe.

- Checking at load-time that the types in the dynamic library have the expected layout,
    allowing for semver compatible changes while checking the layout of types.

- Provides the `StableAbi` derive macro to both assert that the type is ffi compatible,
    and to get the layout of the type at runtime to check that it is still compatible.

# Planned features

### 0.2

Adding direct support for immutable types that can add fields at the end in minor versions,
like vtables and library modules.
This will allow library evolution beyond adding more modules.

### Eventually

WASM support,with the same features as native dynamic libraries.


# Not-currently-panned features

Supporting library unloading,
since this requires building the entire library with the assumption that anything 
might get unloaded at any time.
If someone can make an argument that this is easy enough to add support for,it might be added.

# Examples

For **examples** of using `abi_stable` you can look at the abi_stable_example_* crates,
in the repository for this crate.

# Architecture


Users of this library are expected to follow this architecture:

Create an `interface crate`,
which declares all the functions and the datatypes passed to and returned by those functions,
as well as the `interface types`(types which implement InterfaceType).

Create an `implementation crate` that implements all the functions in the `interface crate`,
creates a `library getter function`,
and declares the `implementation types`(types which implement ImplType) for the `interface types`.

Creates a `user crate`,which declares the `interface crate` as a dependency,
passes the directory/folder of the `implementation crate` dynamic library to 
`<interface_crate::SomeLibType as ModuleTrait>::load_library` to get the library functions,
and then store them in a lazy_static or equivalent to access them afterwards.

For **examples** of this architecture you can look at the abi_stable_example_* crates,
in the repository for this crate.

# Known limitations

### Api evolution

In `0.1` ,this library doesn't allow adding functions in modules of the `implementation crate`
relative to the `interface crate` in minor versions,this will be fixed once `0.2` is released.

Until the `0.2` is released (at most at the end of May-2019),
you can add more modules instead of functions within the same module as a workaround.
