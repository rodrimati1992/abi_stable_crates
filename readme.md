[![Build Status](https://travis-ci.org/rodrimati1992/abi_stable_crates.svg?branch=master)](https://travis-ci.org/rodrimati1992/abi_stable_crates) [![Join the chat at https://gitter.im/abi_stable_crates/community](https://badges.gitter.im/abi_stable_crates/community.svg)](https://gitter.im/abi_stable_crates/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![](https://img.shields.io/crates/v/abi_stable.svg)][crates-io]

[crates-io]: https://crates.io/crates/abi_stable

[Documentation. This goes to 0.7 because that has working docs)](https://docs.rs/abi_stable/0.7)

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

- Features the `#[sabi_trait]` attribute,for creating ffi-safe trait objects.

- ffi-safe equivalent of trait objects for any combination of a selection of traits.

- Provides ffi-safe alternatives/wrappers for many standard library types,
    in the `std_types` module.

- Provides ffi-safe wrappers for some crates,in the `external_types` module.

- Provides the `StableAbi` trait for asserting that types are ffi-safe.

- Features for building extensible modules and vtables,without breaking ABI compatibility.

- Supports ffi-safe nonexhaustive enums,wrapped in `NonExhaustive<>`.

- Checking at load-time that the types in the dynamic library have the expected layout,
    allowing for semver compatible changes while checking the layout of types.

- Provides the `StableAbi` derive macro to both assert that the type is ffi compatible,
    and to get the layout of the type at load-time to check that it is still compatible.

# Changelog

The changelog is in the "Changelog.md" file.

# Example crates

For **example crates** using `abi_stable` you can look at the 
crates in the examples directory ,in the repository for this crate.

To run the examples generally you'll have to build the `*_impl` crate,
then run the `*_user` crate (all `*_user` crates should have a help message).

These are the example crates:

- 0 - modules and interface types:
    Demonstrates abi_stable "modules"(structs of function pointers),
    and interface types through a command line application with a dynamically linked backend.

- 1 - trait objects:
    Demonstrates ffi-safe trait objects (Generated using `#[sabi_trait]`)
    by creating a minimal plugin system.

- 2 - nonexhaustive-enums:
    Demonstrates nonexhaustive-enums as parameters and return values,
    for an application that manages the catalogue of a shop.

# Example

This is a full example,demonstrating:

- `user crates`(defined in the Architecture section bellow).

- `#[sabi_trait]` generated ffi-safe trait objects.

- `DynTrait<_>`:
    An ffi-safe multi-trait object for a selection of trait,
    which can also be unerased back into the concrete type.

- `interface crates`(defined in the Architecture section bellow).

- `ìmplementation crates`(defined in the Architecture section bellow).

Note that each section represents its own crate ,
with comments for how to turn them into 3 separate crates.

```rust

/////////////////////////////////////////////////////////////////////////////////
//
//                        Application (user crate) 
//
////////////////////////////////////////////////////////////////////////////////

use abi_stable::std_types::RVec;

use interface_crate::{
    AppenderBox,Appender_TO,
    ExampleLib,BoxedInterface,load_root_module_in_directory,
};

fn main(){
    // The type annotation is for the reader
    let library:&'static ExampleLib=
        load_root_module_in_directory("./target/debug".as_ref())
            .unwrap_or_else(|e| panic!("{}",e) );

    {
        /*/////////////////////////////////////////////////////////////////////////////////
        
        This block demonstrates `#[sabi_trait]` generated trait objects

        */////////////////////////////////////////////////////////////////////////////////

        // The type annotation is for the reader
        let mut appender:AppenderBox<u32>=library.new_appender()();
        appender.push(100);
        appender.push(200);

        // The primary way to use the methods in the trait is through the inherent methods on 
        // the ffi-safe trait object,
        // since `Trait` requires that the trait object implements the pointer traits for
        // the maximum mutability required by its methods.
        //
        // The inherent methods only require that those pointer traits are 
        // implemented on a per-method basis.
        Appender_TO::push(&mut appender,300);
        appender.append(vec![500,600].into());
        assert_eq!(
            appender.into_rvec(),
            RVec::from(vec![100,200,300,500,600]) 
        );
    }
    {
        /*/////////////////////////////////////////////////////////////////////////////////
        
        This block demonstrates the `DynTrait<>` trait object.
        
        `DynTrait` is used here as a safe opaque type which can only be unwrapped back to the 
        original type in the dynamic library that constructed the `DynTrait` itself.

        */////////////////////////////////////////////////////////////////////////////////

        // The type annotation is for the reader
        let mut unwrapped:BoxedInterface=
            library.new_boxed_interface()();

        library.append_string()(&mut unwrapped,"Hello".into());
        library.append_string()(&mut unwrapped,", world!".into());

        assert_eq!(
            &*unwrapped.to_string(),
            "Hello, world!",
        );
    }

    println!("success");
}


/////////////////////////////////////////////////////////////////////////////////
//
//                      Interface crate
//
//////////////////////////////////////////////////////////////////////////////////

mod interface_crate{

use std::path::Path;

use abi_stable::{
    InterfaceType,
    StableAbi,
    DynTrait,
    sabi_trait,
    library::{LibraryError,RootModule},
    package_version_strings,
    std_types::{RBox,RString,RVec},
    type_level::bools::*,
    sabi_types::VersionStrings,
};


/**
This struct is the root module,
which must be converted to `&ExampleLib` to be passed through ffi.

The `#[sabi(kind(Prefix(prefix_struct="ExampleLib")))]` 
attribute specifies that the ffi-safe 
equivalent of `ExampleLibVal` is called `ExampleLib`.

The `#[sabi(missing_field(panic))]` attribute specifies that trying to 
access a field that doesn't exist
will panic with a message saying that the field is inaccessible.


*/
#[repr(C)]
#[derive(StableAbi)] 
#[sabi(kind(Prefix(prefix_struct="ExampleLib")))]
#[sabi(missing_field(panic))]
pub struct ExampleLibVal {
    pub new_appender:extern "C" fn()->AppenderBox<u32>,

    pub new_boxed_interface: extern "C" fn()->BoxedInterface<'static>,

/**
The `#[sabi(last_prefix_field)]` attribute here means that this is the last field in this struct
that was defined in the first compatible version of the library
(0.1.0, 0.2.0, 0.3.0, 1.0.0, 2.0.0 ,etc),
requiring new fields to always be added bellow preexisting ones.

The `#[sabi(last_prefix_field)]` attribute would stay on this field until the library 
bumps its "major" version,
at which point it would be moved to the last field at the time.

*/
    #[sabi(last_prefix_field)]
    pub append_string:extern "C" fn(&mut BoxedInterface<'_>,RString),
}


/// The RootModule trait defines how to load the root module of a library.
impl RootModule for ExampleLib {

    abi_stable::declare_root_module_statics!{ExampleLib}

    const BASE_NAME: &'static str = "example_library";
    const NAME: &'static str = "example_library";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}

/**

`#[sabi_trait]` is how one creates an ffi-safe trait object from a trait definition.

In this case the trait object is `Appender_TO<'lt,Pointer<()>,Element>`,where:

- `'lt`:
    Is the lifetime bound of the type that constructed the trait object
    (`'static` is the lifetime bound of objects that don't borrow anything).

- `Pointer<()>`:
    Is any pointer that implements some abi_stable specific traits,
    this pointer owns the value that implements `Appender`.

- `Element`:
    This is the element type of the collection that we operate on.

*/
#[sabi_trait]
pub trait Appender{
    /// The element type of the collection.
    type Element;

    /// Appends one element at the end of the collection.    
    fn push(&mut self,value:Self::Element);
    
    /// Appends many elements at the end of the collection.    
    fn append(&mut self,vec:RVec<Self::Element>);

/**
Converts this collection into an `RVec`.

As opposed to regular trait objects (as of Rust 1.36),
it is possible to call by-value methods on trait objects generated by `#[sabi_trait]`.

The `#[sabi(last_prefix_field)]` attribute here means that this is the last method 
that was defined in the first compatible version of the library
(0.1.0, 0.2.0, 0.3.0, 1.0.0, 2.0.0 ,etc),
requiring new methods to always be added bellow preexisting ones.

The `#[sabi(last_prefix_field)]` attribute would stay on this method until the library 
bumps its "major" version,
at which point it would be moved to the last method at the time.

*/
    #[sabi(last_prefix_field)]
    fn into_rvec(self)->RVec<Self::Element>;
}

/// A type alias for the Appender trait object.
///
/// `'static` here means that the trait object cannot contain any borrows.
pub type AppenderBox<T>=Appender_TO<'static,RBox<()>,T>;



/*

/// This loads the root from the library in the `directory` folder.
///
/// This for the case where this example is copied into the 3 crates.
/// 
pub fn load_root_module_in_directory(directory:&Path) -> Result<&'static ExampleLib,LibraryError> {
    ExampleLib::load_from_directory(directory)
}
*/

/// This loads the root module
///
/// This is for the case where this example is copied into a single crate
pub fn load_root_module_in_directory(_:&Path) -> Result<&'static ExampleLib,LibraryError> {
    ExampleLib::load_module_with(|| Ok(super::implementation::get_library()) )
}

//////////////////////////////////////////////////////////


/// This types implement `ÌnterfaceType`
/// (because of the `#[sabi(impl_InterfaceType())]` helper attribute of `#[derive(StableAbi)]` ),
/// describing the traits required when constructing `DynTrait<_,TheInterface>`,
/// and are then implemented by it.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Sync,Send,Debug,Display))]
pub struct TheInterface;


/// An alias for the trait object used in this example
pub type BoxedInterface<'borr>=DynTrait<'borr,RBox<()>,TheInterface>;

}



/////////////////////////////////////////////////////////////////////////////////
//
//                            Implementation crate
//
// This is generally done in a separate crate than the interface.
//
//////////////////////////////////////////////////////////////////////////////////
//
// If you copy paste this into its own crate use this setting in the 
// Cargo.toml file.
//
// ```
// [lib]
// name = "example_library"
// crate-type = ["cdylib",'rlib']
// ```
//
//
//////////////////////////////////////////////////////////////////////////////////

mod implementation {

use std::fmt::{self,Display};


// Comment this out if this is on its own crate
use super::{interface_crate};

use interface_crate::{
    Appender,
    AppenderBox,
    Appender_TO,
    BoxedInterface,
    ExampleLib,
    ExampleLibVal,
    TheInterface,
};

use abi_stable::{
    ImplType,
    DynTrait,
    erased_types::TypeInfo,
    export_root_module,
    sabi_extern_fn,
    impl_get_type_info,
    prefix_type::PrefixTypeTrait,
    sabi_trait::prelude::TU_Opaque,
    std_types::{RString,RVec},
};


/**
The function which exports the root module of the library.

The root module is exported inside a static of `LibHeader` type,
which has this extra metadata:

- The abi_stable version number used by the dynamic library.

- A constant describing the layout of the exported root module,and every type it references.

- A lazily initialized reference to the root module.

- The constructor function of the root module.


*/
#[export_root_module]
pub fn get_library() -> &'static ExampleLib {
    ExampleLibVal{
        new_appender,
        new_boxed_interface,
        append_string,
    }.leak_into_prefix()
}

/**
This is the `implementation crate` dual of `TheInterface`.

A `DynTrait<_,TheInterface>` is expected to only be constructed from a `StringBuilder`.

*/
#[derive(Debug,Clone)]
pub struct StringBuilder{
    pub text:String,
    pub appended:Vec<RString>,
}

///
/// Defines this as an `implementation type`,
/// this trait is mostly for improving error messages when unerasing the DynTrait.
///
impl ImplType for StringBuilder {
    type Interface = TheInterface;

    const INFO: &'static TypeInfo=impl_get_type_info! { StringBuilder };
}

impl Display for StringBuilder{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        fmt::Display::fmt(&self.text,f)
    }
}

impl StringBuilder{
    /// Appends the string at the end.
    pub fn append_string(&mut self,string:RString){
        self.text.push_str(&string);
        self.appended.push(string);
    }
}

#[sabi_extern_fn]
pub fn new_appender()->AppenderBox<u32>{
    /*
    What `TU_Opaque` does here is specify that the trait object cannot be unerased,
    disallowing the `Appender_TO` from being unwrapped back into an `RVec<u32>`
    using the `trait_object.obj.*_unerased_*()` methods.
    
    To be able to unwrap a `#[sabi_trait]` trait object back into the type it 
    was constructed with,you must:

    - Have a type that implements `Any` (it requires that the type doesn't borrow anything).

    - Pass `TU_Unerasable` instead of `TU_Opaque` to Appender_TO::{from_value,from_ptr}.

    - Unerase the trait object back into the original type with
        `trait_object.obj.into_unerased_impltype::<RVec<u32>>().unwrap()` 
        (or the other unerasure methods).

    Unerasing a trait object will fail in any of these conditions:

    - It wasn't constructed in the same dynamic library.
    
    - It's not the same type.

    - It was constructed with `TU_Opaque`.

    */
    Appender_TO::from_value(RVec::new(),TU_Opaque)
}


/// Constructs a BoxedInterface.
#[sabi_extern_fn]
fn new_boxed_interface()->BoxedInterface<'static>{
    DynTrait::from_value(StringBuilder{
        text:"".into(),
        appended:vec![],
    })
}


/// Appends a string to the erased `StringBuilderType`.
#[sabi_extern_fn]
fn append_string(wrapped:&mut BoxedInterface<'_>,string:RString){
    wrapped
        .as_unerased_mut_impltype::<StringBuilder>() // Returns `Result<&mut StringBuilder,_>`
        .unwrap() // Returns `&mut StringBuilder`
        .append_string(string);
}


impl<T> Appender for RVec<T>{
    type Element=T;
    fn push(&mut self,value:Self::Element){
        self.push(value);
    }
    fn append(&mut self,vec:RVec<Self::Element>){
        self.extend(vec);
    }
    fn into_rvec(self)->RVec<Self::Element>{
        self
    }
}



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

### Eventually

WASM support,with the same features as native dynamic libraries,
once WASM supports dynamic linking.

# Not-currently-planned features

Supporting library unloading,
since this requires building the entire library with the assumption that anything 
might get unloaded at any time.

# Architecture


This is a way that users can structure their libraries to allow for dynamic linking.

For how to evolve dynamically loaded libraries loaded using the safe API in abi_stable 
[look here](https://docs.rs/abi_stable/*/abi_stable/docs/library_evolution/index.html).

### Interface crate

A crate which declares:

- The root module (a structs of function pointers/other modules),
    which implements the `RootModule` trait,
    exported from the dynamic library.

- All the sub-modules of the root module.

- All the public types passed to and returned by the functions.

- Optionally:
    declare the ffi-safe traits with `#[sabi_trait]`,
    used as trait objects in the public interface.

- Optionally:
    declares ìnterface types,types which implement InterfaceType,
    used to specify the traits usable in the DynTrait ffi-safe trait object .


### Implementation crate

The crate compiled as a dynamic library that:

- Implements all the functions declared in the `interface crate`.

- Declares a function to export the root module,
    using the `export_root_module` attribute to export the module.

- Optionally:
    Implement traits that were annotated with `#[sabi_trait]`,
    constructing their trait objects exposed in the public API.

- Optionally:create types which implement `ImplType<Iterface= Interface >`,
    where `Interface` is some interface type from the interface crate,
    so as to be able to use wrap it in `DynTrait`s of that interface.

### User crate

A crate that that declares the `ìnterface crate` as a dependency,
and loads the pre-compiled `implementation crate` dynamic library from some path.

# Minimum Rust version

This crate support Rust back to 1.34
(previously 1.33,but had to abandon it because of an impossible to 
avoid internal compiler error related to associated types as types of associated constants),
using a build script to automatically enable features from newer versions.

# Cargo Features

If it becomes possible to disable build scripts,
you can manually enable support for Rust past 1.34 features with the `rust_*_*` cargo features.

These are default cargo features that enable optional crates :

- "channels":
    Depends on `crossbeam-channel`,
    wrapping channels from it for ffi in abi_stable::external_types::crossbeam_channel .

- "serde_json":
    Depends on `serde_json`,
    providing ffi-safe equivalents of 
    `&serde_json::value::RawValue` and `Box<serde_json::value::RawValue>`.


To disable the default features use:
```
[dependencies.abi_stable]
version="<current_version>"
default-features=false
features=[  ]
```
enabling the features you need in the `features` array.


### Manually enabled

These are features to manually enabled support for newer language features,
required until this library is updated to automatically detect them,
every one of which has a `nightly_*` equivalent.

Features:

- `const_params`:Enables items in abi_stable that use const generics.

### Nightly features

The `all_nightly` feature enables all the `nightly_*` equivalents of the 
manually enabled features.

Every `nightly_*` feature enables both support from abi_stable,
as well as the nightly feature flag in the compiler.

# Tools

Here are some tools,all of which are in the "tools" directory(folder).

### sabi_extract

A program to extract a variety of information from an abi_stable dynamic library.

# License

abi_stable is licensed under either of

    Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
    MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in abi_stable by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
