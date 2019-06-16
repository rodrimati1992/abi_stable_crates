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

- Features the `#[sabi_trait]` attribute,for creating ffi-safe trait objects.

- ffi-safe equivalent of trait objects for any combination of a selection of traits.

- Provides ffi-safe alternatives/wrappers for many standard library types,
    in the `std_types` module.

- Provides ffi-safe wrappers for some crates,in the `external_types` module.

- Provides the `StableAbi` trait for asserting that types are ffi-safe.

- Features for building extensible modules and vtables,without breaking ABI compatibility.

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

# Example

This is a full example,demonstrating:

- `user crates`(defined in the Architecture section bellow).

- `#[sabi_trait]` generated ffi-safe trait objects.

- `DynTrait<_>`:the ffi-safe trait object(with downcasting).

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
    AppenderType,Appender_Methods,Appender,
    ExampleLib,BoxedInterface,load_root_module_in_directory,
};

fn main(){
    // The type annotation is for the reader
    let library:&'static ExampleLib=
        load_root_module_in_directory("./target/debug".as_ref())
            .unwrap_or_else(|e| panic!("{}",e) );

    {// Trait object

        // The type annotation is for the reader
        let mut appender:AppenderType<u32>=library.new_appender()();
        appender.push_(100);
        appender.push_(200);

        // `TraitName_Methods` is the primary way to call methods on ffi-safe trait objects,
        // since `Trait` requires that the trait object implements the pointer traits for
        // the maximum mutability required by its methods.
        //
        // TraitName_Methods only require that those pointer traits are 
        // implemented on a per-method basis.
        //
        // Each method in TraitName_Methods appends '_' to 
        // the name of each method(relative to Trait).
        Appender_Methods::push_(&mut appender,300);
        appender.append_(vec![500,600].into());
        assert_eq!(
            appender.into_rvec_(),
            RVec::from(vec![100,200,300,500,600]) 
        );
    }
    {// Erased type (using DynTrait)

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
    impl_InterfaceType,
    library::{LibraryError,RootModule},
    package_version_strings,
    std_types::{RBox,RString,RVec},
    type_level::bools::*,
    sabi_types::VersionStrings,
};



#[repr(C)]
#[derive(StableAbi)] 
#[sabi(kind(Prefix(prefix_struct="ExampleLib")))]
#[sabi(missing_field(panic))]
pub struct ExampleLibVal {
    pub new_appender:extern "C" fn()->AppenderType<u32>,

    pub new_boxed_interface: extern "C" fn()->BoxedInterface<'static>,
    
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


#[sabi_trait]
pub trait Appender{
    type Element;
    fn push(&mut self,value:Self::Element);
    fn append(&mut self,vec:RVec<Self::Element>);
    fn into_rvec(self)->RVec<Self::Element>;
}

/// A type alias for the Appender trait object.
pub type AppenderType<T>=Appender_TO<'static,RBox<()>,T>;



/*

/// This for the case where this example is copied into the 3 crates.
/// 
/// This loads the root from the library in the `directory` folder.
///
pub fn load_root_module_in_directory(directory:&Path) -> Result<&'static ExampleLib,LibraryError> {
    ExampleLib::load_from_directory(directory)
}
*/

/// This is for the case where this example is copied into a single crate
pub fn load_root_module_in_directory(_:&Path) -> Result<&'static ExampleLib,LibraryError> {
    ExampleLib::load_module_with(|| Ok(super::implementation::get_library()) )
}

//////////////////////////////////////////////////////////



#[repr(C)]
#[derive(StableAbi)]
pub struct TheInterface;

// The `impl_InterfaceType` macro emulates default associated types.
impl_InterfaceType!{
    /// Each associated type represents a trait,
    /// which is required of types when ẁrapped in a 
    /// `DynTrait<Pointer<()>,TheInterface>`,
    /// as well as is usable in that `DynTrait<_>`.
    ///
    /// A trait is required (and becomes usable in `DynTrait<_>`) 
    /// when the associated type is `True`,not required when it is `False`.
    ///
    impl InterfaceType for TheInterface {
        type Debug = True;

        type Display = True;

        //////////////////////////////////////////////////////////
        //  some defaulted associated types (there may be more) //
        //////////////////////////////////////////////////////////

        // Changing this to require/unrequire in minor versions,is an abi breaking change.
        // type Send=True;

        // Changing this to require/unrequire in minor versions,is an abi breaking change.
        // type Sync=True;

        // type Iterator=False;
        
        // type DoubleEndedIterator=False;
        
        // type Clone=False;

        // type Default=False;

        // type Serialize=False;

        // type Eq=False;

        // type PartialEq=False;

        // type Ord=False;

        // type PartialOrd=False;

        // type Hash=False;

        // type Deserialize=False;

        // type FmtWrite=False;
        
        // type IoWrite=False;
        
        // type IoSeek=False;
        
        // type IoRead=False;

        // type IoBufRead=False;
    }
}


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
    AppenderType,
    Appender_from_value,
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
    extern_fn_panic_handling,
    impl_get_type_info,
    prefix_type::PrefixTypeTrait,
    sabi_trait::prelude::TU_Opaque,
    std_types::{RString,RVec},
};


/// The function which exports the root module of the library.
#[export_root_module]
pub extern "C" fn get_library() -> &'static ExampleLib {
    ExampleLibVal{
        new_appender,
        new_boxed_interface,
        append_string,
    }.leak_into_prefix()
}


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


pub extern "C" fn new_appender()->AppenderType<u32>{
    Appender_from_value::<_,TU_Opaque>(RVec::new())
}


/// Constructs a BoxedInterface.
extern fn new_boxed_interface()->BoxedInterface<'static>{
    extern_fn_panic_handling!{
        DynTrait::from_value(StringBuilder{
            text:"".into(),
            appended:vec![],
        })
    }
}


/// Appends a string to the erased `StringBuilderType`.
extern fn append_string(wrapped:&mut BoxedInterface<'_>,string:RString){
    extern_fn_panic_handling!{
        wrapped
            .sabi_as_unerased_mut::<StringBuilder>() // Returns `Result<&mut StringBuilder,_>`
            .unwrap() // Returns `&mut StringBuilder`
            .append_string(string);
    }
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

### 0.6

Ffi-safe non-exhaustive enums with fields,allowing for libraries to add variants to enums without breaking backwards compatibility (API or ABI).

### Eventually

WASM support,with the same features as native dynamic libraries,
once WASM supports dynamic linking.

# Not-currently-planned features

Supporting library unloading,
since this requires building the entire library with the assumption that anything 
might get unloaded at any time.

# Architecture


This is a way that users can structure their libraries to allow for dynamic linking
(:

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

- Optionally:declares ìnterface types,types which implement InterfaceType,
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


# Known limitations

### Extensible enums with fields

You can't add variants to enums with fields in the `interface crate` in minor versions.

This will be part of the 0.6 release.

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

To disable the default features use:
```
[dependencies.abi_stable]
version="<current_version>"
default-features=false
features=[  ]
```
enabling the features you need in the `features` array.

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
