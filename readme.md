[![Rust](https://github.com/rodrimati1992/abi_stable_crates/workflows/Rust/badge.svg)](https://github.com/rodrimati1992/abi_stable_crates/actions) [![Join the chat at https://gitter.im/abi_stable_crates/community](https://badges.gitter.im/abi_stable_crates/community.svg)](https://gitter.im/abi_stable_crates/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![](https://img.shields.io/crates/v/abi_stable.svg)][crates-io]
[![api-docs](https://docs.rs/abi_stable/badge.svg)][Documentation]

[crates-io]: https://crates.io/crates/abi_stable

[Documentation]: https://docs.rs/abi_stable

For Rust-to-Rust ffi,
with a focus on creating libraries loaded at program startup,
and with load-time type-checking.

This library allows defining Rust libraries that can be loaded at runtime. 
This isn't possible with the default (Rust) ABI and representation, since it's unstable.

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

# Changelog

The changelog is in the "Changelog.md" file.

# Example crates

For **example crates** using `abi_stable` you can look at the 
crates in the examples directory, in the repository for this crate.

To run the example crates you'll generally have to build the `*_impl` crate,
then run the `*_user` crate (all `*_user` crates should have a help message).

These are the example crates:

- 0 - modules and interface types:
    Demonstrates abi_stable "modules"(structs of function pointers),
    and interface types through a command line application with a dynamically linked backend.

- 1 - trait objects:
    Demonstrates ffi-safe trait objects (Generated using the [`sabi_trait`] attribute macro)
    by creating a minimal plugin system.

- 2 - nonexhaustive-enums:
    Demonstrates nonexhaustive-enums as parameters and return values,
    for an application that manages the catalogue of a shop.

<span id = "readme_example"></span>
# Example

This is a full example, which is located in `examples/readme_example`, demonstrating:

- `user crates`(defined in the Architecture section below).

- Ffi-safe trait objects, generated through the [`sabi_trait`] attribute macro.

- [`DynTrait`]:
An ffi-safe multi-trait object for a selection of traits,
which can also be downcast back into the concrete type.

- `interface crates`(defined in the Architecture section below).

- `ìmplementation crates`(defined in the Architecture section below).

### User crate

This user crate (also called "application crate") depends on the interface crate with:
```toml
[dependencies.readme_interface]
path = "../readme_interface" 
```
its Rust code is:
```rust
use abi_stable::std_types::RVec;

use readme_interface::{
    load_root_module_in_directory, AppenderBox, Appender_TO, BoxedInterface, ExampleLib_Ref,
};

fn main() {
    // The type annotation is for the reader
    let library: ExampleLib_Ref = load_root_module_in_directory("../../../target/debug".as_ref())
        .unwrap_or_else(|e| panic!("{}", e));

    {
        /////////////////////////////////////////////////////////////////////////////////
        //
        //       This block demonstrates `#[sabi_trait]` generated trait objects
        //
        ////////////////////////////////////////////////////////////////////////////////

        // The type annotation is for the reader
        let mut appender: AppenderBox<u32> = library.new_appender()();
        appender.push(100);
        appender.push(200);

        // The primary way to use the methods in the trait is through the inherent methods on
        // the ffi-safe trait object.
        Appender_TO::push(&mut appender, 300);
        appender.append(vec![500, 600].into());
        assert_eq!(
            appender.into_rvec(),
            RVec::from(vec![100, 200, 300, 500, 600])
        );
    }
    {
        ///////////////////////////////////////////////////////////////////////////////////
        //
        //  This block demonstrates the `DynTrait<>` trait object.
        //
        //  `DynTrait` is used here as a safe opaque type which can only be unwrapped back to
        //  the original type in the dynamic library that constructed the `DynTrait` itself.
        //
        ////////////////////////////////////////////////////////////////////////////////////

        // The type annotation is for the reader
        let mut unwrapped: BoxedInterface = library.new_boxed_interface()();

        library.append_string()(&mut unwrapped, "Hello".into());
        library.append_string()(&mut unwrapped, ", world!".into());

        assert_eq!(&*unwrapped.to_string(), "Hello, world!");
    }

    println!("success");
}

```
note: the implementation crate must be compiled before this is ran, otherwise you'll get a runtime error, because the library couldn't be loaded.

### Interface crate

```rust
use std::path::Path;

use abi_stable::{
    library::{LibraryError, RootModule},
    package_version_strings, sabi_trait,
    sabi_types::VersionStrings,
    std_types::{RBox, RString, RVec},
    DynTrait, StableAbi,
};

/// This struct is the root module,
/// which must be converted to `ExampleLib_Ref` to be passed through ffi.
///
/// The `#[sabi(kind(Prefix(prefix_ref = ExampleLib_Ref)))]`
/// attribute tells `StableAbi` to create an ffi-safe static reference type
/// for `ExampleLib` called `ExampleLib_Ref`.
///
/// The `#[sabi(missing_field(panic))]` attribute specifies that trying to
/// access a field that doesn't exist must panic with a message saying that
/// the field is inaccessible.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = ExampleLib_Ref)))]
#[sabi(missing_field(panic))]
pub struct ExampleLib {
    pub new_appender: extern "C" fn() -> AppenderBox<u32>,

    pub new_boxed_interface: extern "C" fn() -> BoxedInterface<'static>,

    /// The `#[sabi(last_prefix_field)]` attribute here means that this is the last
    /// field in this struct that was defined in the first compatible version of the library
    /// (0.1.0, 0.2.0, 0.3.0, 1.0.0, 2.0.0 ,etc),
    /// requiring new fields to always be added below preexisting ones.
    ///
    /// The `#[sabi(last_prefix_field)]` attribute would stay on this field until the
    /// library bumps its "major" version,
    /// at which point it would be moved to the last field at the time.
    ///
    #[sabi(last_prefix_field)]
    pub append_string: extern "C" fn(&mut BoxedInterface<'_>, RString),
}

/// The RootModule trait defines how to load the root module of a library.
impl RootModule for ExampleLib_Ref {
    abi_stable::declare_root_module_statics! {ExampleLib_Ref}

    const BASE_NAME: &'static str = "example_library";
    const NAME: &'static str = "example_library";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}

/// This loads the root from the library in the `directory` folder.
pub fn load_root_module_in_directory(directory: &Path) -> Result<ExampleLib_Ref, LibraryError> {
    ExampleLib_Ref::load_from_directory(directory)
}

//////////////////////////////////////////////////////////

/// `#[sabi_trait]` is how one creates an ffi-safe trait object from a trait definition.
///
/// In this case, the trait object is `Appender_TO<'lt, Pointer<()>, Element>`,where:
///
/// - `'lt`:
///     Is the lifetime bound of the type that constructed the trait object
///     (`'static` is the lifetime bound of objects that don't borrow anything).
///
/// - `Pointer<()>`:
///     Is any pointer that implements some abi_stable specific traits,
///     this pointer owns the value that implements `Appender`.
///
/// - `Element`:
///     This is the element type of the collection that we operate on.
///     This is a type parameter because it's a trait object,
///     which turn associated types into type parameters.
///
#[sabi_trait]
pub trait Appender {
    /// The element type of the collection.
    type Element;

    /// Appends one element at the end of the collection.    
    fn push(&mut self, value: Self::Element);

    /// Appends many elements at the end of the collection.    
    fn append(&mut self, vec: RVec<Self::Element>);

    /// Converts this collection into an `RVec`.
    ///
    /// As opposed to regular trait objects,
    /// it is possible to call by-value methods on trait objects generated by `#[sabi_trait]`.
    ///
    /// The `#[sabi(last_prefix_field)]` attribute here means that this is the last method
    /// that was defined in the first compatible version of the library
    /// (0.1.0, 0.2.0, 0.3.0, 1.0.0, 2.0.0 ,etc),
    /// requiring new methods to always be added below preexisting ones.
    ///
    /// The `#[sabi(last_prefix_field)]` attribute would stay on this method until the library
    /// bumps its "major" version,
    /// at which point it would be moved to the last method at the time.
    ///
    #[sabi(last_prefix_field)]
    fn into_rvec(self) -> RVec<Self::Element>;
}

/// A type alias for the Appender trait object.
///
/// `'static` here means that the trait object cannot contain any borrows.
pub type AppenderBox<T> = Appender_TO<'static, RBox<()>, T>;

// Impls of local traits for dependencies have to be implemented in
// the interface crate, because of the orphan rules.
//
// To avoid compiling more code than necessary,
// this impl is not compiled by default.
// it's enabled by the implementation crate but not the user crate.
#[cfg(feature = "impls")]
impl<T> Appender for RVec<T> {
    type Element = T;
    fn push(&mut self, value: Self::Element) {
        self.push(value);
    }
    fn append(&mut self, vec: RVec<Self::Element>) {
        self.extend(vec);
    }
    fn into_rvec(self) -> RVec<Self::Element> {
        self
    }
}

//////////////////////////////////////////////////////////

/// This type implements `ÌnterfaceType`
/// (because of the `#[sabi(impl_InterfaceType())]` helper attribute of `#[derive(StableAbi)]` ),
/// describing the traits required when constructing `DynTrait<_, TheInterface>`,
/// and are then implemented by it.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Sync, Send, Debug, Display))]
pub struct TheInterface;

/// An alias for the trait object used in this example
pub type BoxedInterface<'borr> = DynTrait<'borr, RBox<()>, TheInterface>;

```

### Implementation crate

This is the implementation crate, which is compiled as a cdylib (a dynamic library/shared object),
and loaded by the user crate at runtime.

The important bits of its Cargo.toml file are:
```toml
[lib]
name = "readme_library"
crate-type = ["cdylib",'rlib']

[dependencies.readme_interface]
path = "../readme_interface" 
features = ["impls"]
```
its Rust code is:

```rust
use std::fmt::{self, Display};

use readme_interface::{AppenderBox, Appender_TO, BoxedInterface, ExampleLib, ExampleLib_Ref};

use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    sabi_trait::prelude::TD_Opaque,
    std_types::{RString, RVec},
    DynTrait,
};

/// The function which exports the root module of the library.
///
/// The root module is exported inside a static of `LibHeader` type,
/// which has this extra metadata:
///
/// - The abi_stable version number used by the dynamic library.
///
/// - A constant describing the layout of the exported root module,and every type it references.
///
/// - A lazily initialized reference to the root module.
///
/// - The constructor function of the root module.
///
#[export_root_module]
pub fn get_library() -> ExampleLib_Ref {
    ExampleLib {
        new_appender,
        new_boxed_interface,
        append_string,
    }
    .leak_into_prefix()
}

/// `DynTrait<_, TheInterface>` is constructed from this type in this example
#[derive(Debug, Clone)]
pub struct StringBuilder {
    pub text: String,
    pub appended: Vec<RString>,
}

impl Display for StringBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.text, f)
    }
}

impl StringBuilder {
    /// Appends the string at the end.
    pub fn append_string(&mut self, string: RString) {
        self.text.push_str(&string);
        self.appended.push(string);
    }
}

#[sabi_extern_fn]
pub fn new_appender() -> AppenderBox<u32> {
    // What `TD_Opaque` does here is specify that the trait object cannot be downcasted,
    // disallowing the `Appender_TO` from being unwrapped back into an `RVec<u32>`
    // when the `trait_object.obj.*_downcast_*()` methods are used.
    //
    // To be able to unwrap a `#[sabi_trait]` trait object back into the type it
    // was constructed with, you must:
    //
    // - Have a type that implements `std::anu::Any`
    // (it requires that the type doesn't borrow anything).
    //
    // - Pass `TD_CanDowncast` instead of `TD_Opaque` to
    // `Appender_TO::{from_const, from_value,from_ptr}`.
    //
    // - Unerase the trait object back into the original type with
    //     `trait_object.obj.downcast_into::<RVec<u32>>().unwrap()`
    //     (or the other downcasting methods).
    //
    // Downcasting a trait object will fail in any of these conditions:
    //
    // - It wasn't constructed in the same dynamic library.
    //
    // - It's not the same type.
    //
    // - It was constructed with `TD_Opaque`.
    //
    Appender_TO::from_value(RVec::new(), TD_Opaque)
}

/// Constructs a BoxedInterface.
#[sabi_extern_fn]
fn new_boxed_interface() -> BoxedInterface<'static> {
    DynTrait::from_value(StringBuilder {
        text: "".into(),
        appended: vec![],
    })
}

/// Appends a string to the erased `StringBuilder`.
#[sabi_extern_fn]
fn append_string(wrapped: &mut BoxedInterface<'_>, string: RString) {
    wrapped
        .downcast_as_mut::<StringBuilder>() // Returns `Result<&mut StringBuilder, _>`
        .unwrap() // Returns `&mut StringBuilder`
        .append_string(string);
}

```


# Safety

This library ensures that the loaded libraries are safe to use through these mechanisms:

- The abi_stable ABI of the library is checked,
    Each `0.y.0` version and `x.0.0` version of abi_stable defines its own ABI 
    which is incompatible with previous versions.

- Types are recursively checked when the dynamic library is loaded,
    before any function can be called.

Note that this library assumes that dynamic libraries come from a benign source,
these checks are done purely to detect programming errors.

# Planned features

None right now.

# Non-features (extremely unlikely to be added)

Supporting library unloading,
since this requires building the entire library with the assumption that anything 
might get unloaded at any time.

# Architecture


This is a way that users can structure their libraries to allow for dynamic linking.

For how to evolve dynamically loaded libraries loaded using the safe API in abi_stable 
[look here](https://docs.rs/abi_stable/*/abi_stable/docs/library_evolution/index.html).

### Interface crate

A crate which declares:

- The root module (a struct of function pointers/other modules),
    which implements the [`RootModule`] trait,
    exported from the dynamic library.

- All the sub-modules of the root module.

- All the public types passed to and returned by the functions.

- Optionally:
    declare the ffi-safe traits with the [`sabi_trait`] attribute,
    used as trait objects in the public interface.

- Optionally:
    declares ìnterface types,types which implement [`InterfaceType`],
    used to specify the traits usable in the [`DynTrait`] ffi-safe trait object .


### Implementation crate

The crate compiled as a dynamic library that:

- Implements all the functions declared in the `interface crate`.

- Declares a function to export the root module,
    using the [`export_root_module`] attribute to export the module.

- Optionally:
    Implement traits that were annotated with the [`sabi_trait`] attribute,
    constructing their trait objects exposed in the public API.

### User crate

A crate that that declares the `ìnterface crate` as a dependency,
and loads the pre-compiled `implementation crate` dynamic library from some path.

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
```toml
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

# Tools

Here are some tools,all of which are in the "tools" directory(folder).

### sabi_extract

A program to extract a variety of information from an abi_stable dynamic library.

# License

abi_stable is licensed under either of

```text
    Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
    MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
```

at your option.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in abi_stable by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.



[`std_types`]: https://docs.rs/abi_stable/*/abi_stable/std_types/index.html
[`external_types`]: https://docs.rs/abi_stable/*/abi_stable/external_types/index.html
[prefix types]: https://docs.rs/abi_stable/*/abi_stable/docs/prefix_types/index.html
[Prefix types]: https://docs.rs/abi_stable/*/abi_stable/docs/prefix_types/index.html
[nonexhaustive enums]: https://docs.rs/abi_stable/*/abi_stable/docs/sabi_nonexhaustive/index.html
[Nonexhaustive enums]: https://docs.rs/abi_stable/*/abi_stable/docs/sabi_nonexhaustive/index.html
[library_evolution]: https://docs.rs/abi_stable/*/abi_stable/docs/library_evolution/index.html
[`NonExhaustive`]: https://docs.rs/abi_stable/*/abi_stable/nonexhaustive_enum/struct.NonExhaustive.html
[the readme]: https://github.com/rodrimati1992/abi_stable_crates/blob/master/readme.md
[`RootModule`]: https://docs.rs/abi_stable/*/abi_stable/library/trait.RootModule.html
[`StableAbi`]: https://docs.rs/abi_stable/*/abi_stable/abi_stability/stable_abi_trait/trait.StableAbi.html
[`sabi_trait`]: https://docs.rs/abi_stable/*/abi_stable/attr.sabi_trait.html
[Trait objects]: https://docs.rs/abi_stable/*/abi_stable/attr.sabi_trait.html
[`StableAbi` derive]: https://docs.rs/abi_stable/*/abi_stable/derive.StableAbi.html
[`DynTrait`]: https://docs.rs/abi_stable/*/abi_stable/struct.DynTrait.html
[Troubleshooting]: https://docs.rs/abi_stable/*/abi_stable/docs/troubleshooting/index.html
[Unsafe code guidelines]: https://docs.rs/abi_stable/*/abi_stable/docs/unsafe_code_guidelines/index.html

[`InterfaceType`]: https://docs.rs/abi_stable/*/abi_stable/trait.InterfaceType.html
[`export_root_module`]: https://docs.rs/abi_stable/*/abi_stable/attr.export_root_module.html
