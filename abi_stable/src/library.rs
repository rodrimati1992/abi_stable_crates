/*!
Traits and types related to loading an abi_stable dynamic library,
as well as functions/modules within.
*/

use std::{
    convert::Infallible,
    mem,
    path::{Path,PathBuf},
    sync::atomic,
};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use libloading::{
    Library as LibLoadingLibrary,
    Symbol as LLSymbol,
};

pub use abi_stable_shared::mangled_root_module_loader_name;



use crate::{
    abi_stability::stable_abi_trait::StableAbi,
    globals::{self,Globals},
    marker_type::ErasedPrefix,
    prefix_type::{PrefixRef, PrefixRefTrait},
    type_layout::TypeLayout,
    sabi_types::{ LateStaticRef, VersionNumber, VersionStrings },
    std_types::{RResult, RStr},
    utils::{transmute_reference},
};


pub mod c_abi_testing;
pub mod development_utils;
mod lib_header;
mod errors;
mod root_mod_trait;
mod raw_library;


#[doc(no_inline)]
pub use self::c_abi_testing::{CAbiTestingFns,C_ABI_TESTING_FNS};

pub use self::{
    errors::{IntoRootModuleResult, LibraryError, RootModuleError},
    lib_header::{AbiHeader,LibHeader},
    root_mod_trait::{
        RootModule,
        lib_header_from_raw_library,
        lib_header_from_path,
        abi_header_from_raw_library,
        abi_header_from_path,
        RootModuleConsts,
        ErasedRootModuleConsts,
    },
    raw_library::RawLibrary,
};


///////////////////////////////////////////////////////////////////////////////


/// What naming convention to expect when loading a library from a directory.
#[derive(Debug,Copy,Clone,PartialEq,Eq,Ord,PartialOrd,Hash)]
pub enum LibrarySuffix{
    /// Loads a dynamic library at `<folder>/<name>.extension`
    NoSuffix,
    
    /// Loads a dynamic library at `<folder>/<name>-<pointer_size>.<extension>`
    Suffix,
}


//////////////////////////////////////////////////////////////////////

/// The path a library is loaded from.
#[derive(Debug,Copy,Clone,PartialEq,Eq,Ord,PartialOrd,Hash)]
pub enum LibraryPath<'a>{
    /// The full path to the dynamic library.
    FullPath(&'a Path),
    /// The path to the directory that contains the dynamic library.
    Directory(&'a Path),
}

//////////////////////////////////////////////////////////////////////


/// Whether the ABI of a root module is checked.
#[repr(u8)]
#[derive(Debug,Copy,Clone,StableAbi)]
pub enum IsLayoutChecked{
    Yes(&'static TypeLayout),
    No
}


impl IsLayoutChecked{
    pub fn into_option(self)->Option<&'static TypeLayout>{
        match self {
            IsLayoutChecked::Yes(x)=>Some(x),
            IsLayoutChecked::No    =>None,
        }
    }
}


//////////////////////////////////////////////////////////////////////

/// The return type of the function that the
/// `#[export_root_module]` attribute outputs.
pub type RootModuleResult = RResult<PrefixRef<ErasedPrefix>, RootModuleError>;

//////////////////////////////////////////////////////////////////////


/// The static variables declared for some `RootModule` implementor.
#[doc(hidden)]
pub struct RootModuleStatics<M>{
    root_mod:LateStaticRef<M>,
    raw_lib:LateStaticRef<&'static RawLibrary>,
}

impl<M> RootModuleStatics<M>{
    #[doc(hidden)]
    #[inline]
    pub const fn _private_new()->Self{
        Self{
            root_mod:LateStaticRef::new(),
            raw_lib:LateStaticRef::new(),
        }
    }
}


/// Implements the `RootModule::root_module_statics` associated function.
///
/// To define the associated function use:
/// `abi_stable::declare_root_module_statics!{TypeOfSelf}`.
/// Passing `Self` instead of `TypeOfSelf` won't work.
#[macro_export]
macro_rules! declare_root_module_statics {
    ( ( $($stuff:tt)* ) ) => (
        $carte::declare_root_module_statics!{$($stuff)*}
    );
    ( $this:ty ) => (
        #[inline]
        fn root_module_statics()->&'static $crate::library::RootModuleStatics<$this>{
            static _ROOT_MOD_STATICS:$crate::library::RootModuleStatics<$this>=
                $crate::library::RootModuleStatics::_private_new();

            &_ROOT_MOD_STATICS
        }
    );
    ( Self ) => (
        compile_error!("Don't use `Self`, write the full type name")
    );
}

//////////////////////////////////////////////////////////////////////


#[doc(hidden)]
pub fn __call_root_module_loader<T>(function: fn()->T ) -> RootModuleResult
where
    T: IntoRootModuleResult
{
    type TheResult = Result<PrefixRef<ErasedPrefix>, RootModuleError>;
    let res = ::std::panic::catch_unwind(||-> TheResult {
        let ret: T::Module = function().into_root_module_result()?;

        let _= <T::Module as RootModule>::load_module_with(|| Ok::<_,Infallible>(ret) );
        unsafe{
            ret.to_prefix_ref()
                .cast::<ErasedPrefix>()
                .piped(Ok)
        }
    });
    // We turn an unwinding panic into an error value
    let flattened: TheResult = res.unwrap_or(Err(RootModuleError::Unwound));
    RootModuleResult::from(flattened)
}

