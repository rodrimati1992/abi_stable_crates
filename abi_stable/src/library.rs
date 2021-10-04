/*!
Traits and types related to loading an abi_stable dynamic library,
as well as functions/modules within.
*/

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
        lib_header_from_raw_library, ErasedRootModuleConsts, RootModule, RootModuleConsts,
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

/// Whether the ABI of a root module is checked.
#[repr(u8)]
#[derive(Debug, Copy, Clone, StableAbi)]
pub enum IsLayoutChecked {
    Yes(&'static TypeLayout),
    No,
}

impl IsLayoutChecked {
    pub fn into_option(self) -> Option<&'static TypeLayout> {
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
    #[doc(hidden)]
    #[inline]
    pub const fn _private_new() -> Self {
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
/// #[sabi(kind(Prefix(prefix_ref = "Module_Ref", prefix_fields = "Module_Prefix")))]
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
            static _ROOT_MOD_STATICS:$crate::library::RootModuleStatics<$this>=
                $crate::library::RootModuleStatics::_private_new();

            &_ROOT_MOD_STATICS
        }
    );
}

//////////////////////////////////////////////////////////////////////

#[deprecated(
    since = "0.10.3",
    note = "Use the ROOT_MODULE_LOADER_NAME constant instead"
)]
/// Gets the name of the static that contains the LibHeader of an abi_stable library.
pub fn mangled_root_module_loader_name() -> String {
    abi_stable_shared::mangled_root_module_loader_name()
}

abi_stable_derive::__const_mangled_root_module_loader_name! {}

/// The name of the `static` that contains the [`AbiHeader`] of an abi_stable library.
///
/// You can get a handle to that [`AbiHeader`] using
/// [abi_header_from_path](fn.abi_header_from_path.html) or
/// [abi_header_from_raw_library](fn.abi_header_from_raw_library.html).
///
/// If you need a nul-terminated string,
/// you can use [`ROOT_MODULE_LOADER_NAME_WITH_NUL`] instead.
///
/// [`LibHeader`]: ./struct.LibHeader.html
/// [`ROOT_MODULE_LOADER_NAME_WITH_NUL`]: ./constant.ROOT_MODULE_LOADER_NAME_WITH_NUL.html
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
    unsafe { NulStr::from_str(PRIV_MANGLED_ROOT_MODULE_LOADER_NAME_NUL) };

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
