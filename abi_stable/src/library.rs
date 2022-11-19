//! Traits and types related to loading an abi_stable dynamic library,
//! as well as functions/modules within.
//!
//! # Loading the root module
//!
//! When you use the [`RootModule`]`::load_from*` associated functions,
//! the root module of a library is loaded in this order:
//! 1. A [`RawLibrary`] is loaded
//! (The library is leaked so that the root module loader can
//! do anything incompatible with library unloading.)
//! 2. An [`AbiHeaderRef`] handle to the static that contains the root module is obtained.
//! 3. The [`AbiHeaderRef`] checks that the abi_stable version used by that library is
//! compatible with the loader's, upgrading to a [`&'static LibHeader`] on success.
//! 4. The [`LibHeader`] checks that the layout of the types in the root module
//! (and everything it references) are compatible with the loader's
//! 5. The [root module](./trait.RootModule.html)
//! is loaded using the function from the loaded library
//! that was annotated with [`#[export_root_module]`](../attr.export_root_module.html).
//! 6. [`RootModule::initialize`] is called on the root module.
//!
//! All steps can return errors.
//!
//! [`RawLibrary`]: ./struct.RawLibrary.html
//! [`AbiHeaderRef`]: ./struct.AbiHeaderRef.html
//! [`RootModule`]: ./trait.RootModule.html
//! [`RootModule::initialize`]: ./trait.RootModule.html#method.initialization
//! [`&'static LibHeader`]: ./struct.LibHeader.html

use std::{
    convert::Infallible,
    mem,
    path::{Path, PathBuf},
    sync::atomic,
};

#[allow(unused_imports)]
use core_extensions::SelfOps;

use libloading::{Library as LibLoadingLibrary, Symbol as LLSymbol};

use crate::{
    abi_stability::stable_abi_trait::StableAbi,
    globals::{self, Globals},
    marker_type::ErasedPrefix,
    prefix_type::{PrefixRef, PrefixRefTrait},
    sabi_types::{LateStaticRef, NulStr, VersionNumber, VersionStrings},
    std_types::{RResult, RStr},
    type_layout::TypeLayout,
};

pub mod c_abi_testing;
pub mod development_utils;
mod errors;
mod lib_header;

#[cfg(test)]
mod library_tests;

mod raw_library;
mod root_mod_trait;

#[doc(no_inline)]
pub use self::c_abi_testing::{CAbiTestingFns, C_ABI_TESTING_FNS};

pub use self::{
    errors::{IntoRootModuleResult, LibraryError, RootModuleError},
    lib_header::{AbiHeader, AbiHeaderRef, LibHeader},
    raw_library::RawLibrary,
    root_mod_trait::{
        abi_header_from_path, abi_header_from_raw_library, lib_header_from_path,
        lib_header_from_raw_library, RootModule, RootModuleConsts,
    },
};

///////////////////////////////////////////////////////////////////////////////

/// What naming convention to expect when loading a library from a directory.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum LibrarySuffix {
    /// Loads a dynamic library at `<folder>/<name>.extension`
    NoSuffix,

    /// Loads a dynamic library at `<folder>/<name>-<pointer_size>.<extension>`
    Suffix,
}

//////////////////////////////////////////////////////////////////////

/// The path a library is loaded from.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum LibraryPath<'a> {
    /// The full path to the dynamic library.
    FullPath(&'a Path),
    /// The path to the directory that contains the dynamic library.
    Directory(&'a Path),
}

//////////////////////////////////////////////////////////////////////

/// Tells [`LibHeader::from_constructor`] whether to
/// include the layout of the root module for checking it when loaded.
pub enum CheckTypeLayout {
    /// Include the layout of the root module
    Yes,
    /// Exclude the layout of the root module
    No,
}

//////////////////////////////////////////////////////////////////////

/// Whether the ABI of a root module is checked.
#[repr(u8)]
#[derive(Debug, Copy, Clone, StableAbi)]
pub enum IsLayoutChecked {
    /// The ABI is checked
    Yes(&'static TypeLayout),
    /// The ABI is not checked
    No,
}

impl IsLayoutChecked {
    /// Converts this into an `Option`.
    ///
    /// `Ỳes` corresponds to `Some`, and `No` corresponds to `None`.
    pub const fn into_option(self) -> Option<&'static TypeLayout> {
        match self {
            IsLayoutChecked::Yes(x) => Some(x),
            IsLayoutChecked::No => None,
        }
    }
}

//////////////////////////////////////////////////////////////////////

/// The return type of the function that the
/// [`#[export_root_module]`](../attr.export_root_module.html) attribute outputs.
pub type RootModuleResult = RResult<PrefixRef<ErasedPrefix>, RootModuleError>;

//////////////////////////////////////////////////////////////////////

/// The static variables declared for some [`RootModule`] implementor.
/// [`RootModule`]: ./trait.RootModule.html
#[doc(hidden)]
pub struct RootModuleStatics<M> {
    root_mod: LateStaticRef<M>,
    raw_lib: LateStaticRef<&'static RawLibrary>,
}

impl<M> RootModuleStatics<M> {
    ///
    /// # Safety
    ///
    /// This must only be called from the `abi_stable::declare_root_module_statics` macro.
    #[doc(hidden)]
    #[inline]
    pub const unsafe fn __private_new() -> Self {
        Self {
            root_mod: LateStaticRef::new(),
            raw_lib: LateStaticRef::new(),
        }
    }
}

/// Implements the [`RootModule::root_module_statics`] associated function.
///
/// To define the associated function use:
/// `abi_stable::declare_root_module_statics!{TypeOfSelf}`.
/// Passing `Self` instead of `TypeOfSelf` won't work.
///
/// # Example
///
/// ```rust
/// use abi_stable::{
///     library::RootModule,
///     sabi_types::VersionStrings,
///     StableAbi,
/// };
///
/// #[repr(C)]
/// #[derive(StableAbi)]
/// #[sabi(kind(Prefix(prefix_ref = Module_Ref, prefix_fields = Module_Prefix)))]
/// pub struct Module{
///     pub first: u8,
///     #[sabi(last_prefix_field)]
///     pub second: u16,
///     pub third: u32,
/// }
/// impl RootModule for Module_Ref {
///     abi_stable::declare_root_module_statics!{Module_Ref}
///     const BASE_NAME: &'static str = "example_root_module";
///     const NAME: &'static str = "example_root_module";
///     const VERSION_STRINGS: VersionStrings = abi_stable::package_version_strings!();
/// }
///
/// # fn main(){}
/// ```
///
#[cfg_attr(
    doctest,
    doc = r###"

```rust
struct Foo;
impl Foo {
    abi_stable::declare_root_module_statics!{Foo}
}
```

```rust
struct Foo;
impl Foo {
    abi_stable::declare_root_module_statics!{(Foo)}
}
```

```compile_fail
struct Foo;
impl Foo {
    abi_stable::declare_root_module_statics!{Self}
}
```

```compile_fail
struct Foo;
impl Foo {
    abi_stable::declare_root_module_statics!{(Self)}
}
```

```compile_fail
struct Foo;
impl Foo {
    abi_stable::declare_root_module_statics!{((Self))}
}
```

"###
)]
/// [`RootModule::root_module_statics`]:
/// ./library/trait.RootModule.html#tymethod.root_module_statics
#[macro_export]
macro_rules! declare_root_module_statics {
    ( ( $($stuff:tt)* ) ) => (
        $crate::declare_root_module_statics!{$($stuff)*}
    );
    ( Self ) => (
        compile_error!{"Don't use `Self`, write the full type name"}
    );
    ( $this:ty ) => (
        #[inline]
        fn root_module_statics()->&'static $crate::library::RootModuleStatics<$this>{
            static _ROOT_MOD_STATICS:$crate::library::RootModuleStatics<$this>= unsafe{
                $crate::library::RootModuleStatics::__private_new()
            };

            &_ROOT_MOD_STATICS
        }
    );
}

//////////////////////////////////////////////////////////////////////

abi_stable_derive::__const_mangled_root_module_loader_name! {}

/// The name of the `static` that contains the [`LibHeader`] of an abi_stable library.
///
/// There's also these alternatives to this constant:
/// - [`ROOT_MODULE_LOADER_NAME_WITH_NUL`]: this constant concatenated with `"\0"`
/// - [`ROOT_MODULE_LOADER_NAME_NULSTR`]: a [`NulStr`] equivalent of this constant
///
/// [`LibHeader`]: ./struct.LibHeader.html
/// [`AbiHeaderRef`]: ./struct.AbiHeaderRef.html
/// [`AbiHeaderRef::upgrade`]: ./struct.AbiHeaderRef.html#method.upgrade
/// [`ROOT_MODULE_LOADER_NAME_WITH_NUL`]: ./constant.ROOT_MODULE_LOADER_NAME_WITH_NUL.html
/// [`ROOT_MODULE_LOADER_NAME_NULSTR`]: ./constant.ROOT_MODULE_LOADER_NAME_NULSTR.html
/// [`NulStr`]: ../sabi_types/struct.NulStr.html
pub const ROOT_MODULE_LOADER_NAME: &str = PRIV_MANGLED_ROOT_MODULE_LOADER_NAME;

/// A nul-terminated equivalent of [`ROOT_MODULE_LOADER_NAME`].
///
/// [`ROOT_MODULE_LOADER_NAME`]: ./constant.ROOT_MODULE_LOADER_NAME.html
pub const ROOT_MODULE_LOADER_NAME_WITH_NUL: &str = PRIV_MANGLED_ROOT_MODULE_LOADER_NAME_NUL;

/// A [`NulStr`] equivalent of [`ROOT_MODULE_LOADER_NAME`].
///
/// [`ROOT_MODULE_LOADER_NAME`]: ./constant.ROOT_MODULE_LOADER_NAME.html
/// [`NulStr`]: ../sabi_types/struct.NulStr.html
pub const ROOT_MODULE_LOADER_NAME_NULSTR: NulStr<'_> =
    NulStr::from_str(PRIV_MANGLED_ROOT_MODULE_LOADER_NAME_NUL);

//////////////////////////////////////////////////////////////////////

#[doc(hidden)]
pub fn __call_root_module_loader<T>(function: fn() -> T) -> RootModuleResult
where
    T: IntoRootModuleResult,
{
    type TheResult = Result<PrefixRef<ErasedPrefix>, RootModuleError>;
    let res = ::std::panic::catch_unwind(|| -> TheResult {
        let ret: T::Module = function().into_root_module_result()?;

        let _ = <T::Module as RootModule>::load_module_with(|| Ok::<_, Infallible>(ret));
        unsafe { ret.to_prefix_ref().cast::<ErasedPrefix>().piped(Ok) }
    });
    // We turn an unwinding panic into an error value
    let flattened: TheResult = res.unwrap_or(Err(RootModuleError::Unwound));
    RootModuleResult::from(flattened)
}
