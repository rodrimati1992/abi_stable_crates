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

# Example crates

For **example crates** using `abi_stable` you can look at the 
crates in the examples directory ,in the repository for this crate.

To run the examples generally you'll have to build the `*_impl` crate,
then run the `*_user` crate (all `*_user` crates should have a help message).

# Example

This is a full example,demonstrating:

- `user crates`(defined in the Architecture section bellow).

- `DynTrait<_>`:the ffi-safe trait object(with downcasting).

- `interface crates`(defined in the Architecture section bellow).

- `ìmplementation crates`(defined in the Architecture section bellow).


```rust

/////////////////////////////////////////////////////////////////////////////////
//
//                        Application (user crate) 
//
////////////////////////////////////////////////////////////////////////////////


use interface_crate::{ExampleLib,BoxedInterface,load_root_module_in};

fn main(){
    // The type annotation is for the reader
    let library:&'static ExampleLib=
        load_root_module_in("./target/debug".as_ref())
            .unwrap_or_else(|e| panic!("{}",e) );

    // The type annotation is for the reader
    let mut unwrapped:BoxedInterface=
        library.new_boxed_interface()();

    library.append_string()(&mut unwrapped,"Hello".into());
    library.append_string()(&mut unwrapped,", world!".into());

    assert_eq!(
        &*unwrapped.to_string(),
        "Hello, world!",
    );

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
    impl_InterfaceType,
    lazy_static_ref::LazyStaticRef,
    library::{Library,LibraryError,RootModule},
    package_version_strings,
    std_types::{RBox,RString},
    type_level::bools::*,
    version::VersionStrings,
};



#[repr(C)]
#[derive(StableAbi)]
pub struct TheInterface;

// The `impl_InterfaceType` macro emulates default associated types.
impl_InterfaceType!{
    /// Each associated type represents a trait,
    /// will is required of types when ẁrapped in a 
    /// `DynTrait<Pointer<()>,TheInterface>`,
    /// as well as be usable in that `DynTrait<_>`.
    ///
    /// A trait is required (and becomes usable in the `DynTrait`) 
    /// when the associated type is `True`,not required when it is `False`.
    ///
    impl InterfaceType for TheInterface {
        type Debug = True;

        type Display = True;

        //////////////////////////////////////////////////////////
        //  some defaulted associated types (there may be more) //
        //////////////////////////////////////////////////////////
        
        // type Clone=False;

        // type Default=False;

        // type Serialize=False;

        // type Eq=False;

        // type PartialEq=False;

        // type Ord=False;

        // type PartialOrd=False;

        // type Hash=False;

        // type Deserialize=False;
    }
}


/// An alias for the trait object used in this example
pub type BoxedInterface<'borr>=DynTrait<'borr,RBox<()>,TheInterface>;


#[repr(C)]
#[derive(StableAbi)] 
#[sabi(kind(Prefix(prefix_struct="ExampleLib")))]
#[sabi(missing_field(panic))]
pub struct ExampleLibVal {
    pub new_boxed_interface: extern "C" fn()->BoxedInterface<'static>,
    
    #[sabi(last_prefix_field)]
    pub append_string:extern "C" fn(&mut BoxedInterface<'_>,RString),
}


/// The RootModule trait defines how to load the root module of a library.
impl RootModule for ExampleLib {
    fn raw_library_ref()->&'static LazyStaticRef<Library>{
        static RAW_LIB:LazyStaticRef<Library>=LazyStaticRef::new();
        &RAW_LIB
    }

    const BASE_NAME: &'static str = "example_library";
    const NAME: &'static str = "example_library";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
    const LOADER_FN: &'static str = "get_library";
}

/// A global handle to the root module of the library.
///
/// To get the module call `ROOTMOD.get()`,
/// which returns None if the module is not yet loaded.
pub static ROOTMOD:LazyStaticRef<ExampleLib>=LazyStaticRef::new();

/*

/// This for the case where this example is copied into the 3 crates.
/// 
/// This loads the root from the library in the `directory` folder.
/// 
/// Failing (with an Err(_)) in these conditions:
///
/// - The library is not there.
///
/// - The module loader is not there,most likely because the abi is incompatible.
///
/// - The layout-checker detects a type error.
///
pub fn load_root_module_in(directory:&Path) -> Result<&'static ExampleLib,LibraryError> {
    ROOTMOD.try_init(||{
        ExampleLib::load_from_library_in(directory)
    })
}
*/

/// This is for the case where this example is copied into a single crate
pub fn load_root_module_in(_directory:&Path) -> Result<&'static ExampleLib,LibraryError> {
    ROOTMOD.try_init(||{
        super::implementation::get_library()
            .check_layout()
    })
}

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
    BoxedInterface,
    ExampleLib,
    ExampleLibVal,
    TheInterface,
};

use abi_stable::{
    ImplType,
    DynTrait,
    erased_types::TypeInfo,
    export_sabi_module,
    extern_fn_panic_handling,
    impl_get_type_info,
    library::{WithLayout},
    std_types::RString,
};


/// The function which exports the root module of the library.
#[export_sabi_module]
pub extern "C" fn get_library() -> WithLayout<ExampleLib> {
    WithLayout::new(ExampleLibVal{
        new_boxed_interface,
        append_string,
    })
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


// Constructs a BoxedInterface.
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
            .as_unerased_mut::<StringBuilder>() // Returns `Result<&mut StringBuilder,_>`
            .unwrap() // Returns `&mut StringBuilder`
            .append_string(string);
    }
}
}



```



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
    used to specify the traits usable in the DynTrait ffi-safe trait object .


### Implementation crate

The crate compiled as a dynamic library that:

- Implements all the functions declared in the `interface crate`.

- Declares a function to export the root module,
    using the `export_sabi_module` attribute to export the module.

- Optionally:create types which implement `ImplType<Iterface= Interface >`,
    where `Interface` is some interface type from the interface crate,
    so as to be able to use wrap it in `DynTrait`s of that interface.

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
