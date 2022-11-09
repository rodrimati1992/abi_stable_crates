use super::*;

use crate::{prefix_type::PrefixRefTrait, utils::leak_value};

/// The root module of a dynamic library,
/// which may contain other modules,function pointers,and static references.
///
///
/// # Examples
///
/// For a more in-context example of a type implementing this trait you can look
/// at either the example in the readme for this crate,
/// or the `example/example_*_interface` crates  in this crates' repository .
///
/// ### Basic
///
/// ```rust
/// use abi_stable::{library::RootModule, sabi_types::VersionStrings, StableAbi};
///
/// #[repr(C)]
/// #[derive(StableAbi)]
/// #[sabi(kind(Prefix(prefix_ref = Module_Ref, prefix_fields = Module_Prefix)))]
/// pub struct Module {
///     pub first: u8,
///     // The `#[sabi(last_prefix_field)]` attribute here means that this is
///     // the last field in this module that was defined in the
///     // first compatible version of the library,
///     #[sabi(last_prefix_field)]
///     pub second: u16,
///     pub third: u32,
/// }
/// impl RootModule for Module_Ref {
///     abi_stable::declare_root_module_statics! {Module_Ref}
///     const BASE_NAME: &'static str = "example_root_module";
///     const NAME: &'static str = "example_root_module";
///     const VERSION_STRINGS: VersionStrings = abi_stable::package_version_strings!();
/// }
///

/// # fn main(){}
/// ```
pub trait RootModule: Sized + StableAbi + PrefixRefTrait + 'static {
    /// The name of the dynamic library,which is the same on all platforms.
    /// This is generally the name of the `implementation crate`.
    const BASE_NAME: &'static str;

    /// The name of the library used in error messages.
    const NAME: &'static str;

    /// The version number of the library that this is a root module of.
    ///
    /// Initialize this with
    /// [`package_version_strings!()`](../macro.package_version_strings.html)
    const VERSION_STRINGS: VersionStrings;

    /// All the constants of this trait and supertraits.
    ///
    /// It can safely be used as a proxy for the associated constants of this trait.
    const CONSTANTS: RootModuleConsts = RootModuleConsts {
        base_name: RStr::from_str(Self::BASE_NAME),
        name: RStr::from_str(Self::NAME),
        version_strings: Self::VERSION_STRINGS,
        layout: IsLayoutChecked::Yes(<Self as StableAbi>::LAYOUT),
        c_abi_testing_fns: crate::library::c_abi_testing::C_ABI_TESTING_FNS,
        _priv: (),
    };

    /// Like `Self::CONSTANTS`,
    /// except without including the type layout constant for the root module.
    const CONSTANTS_NO_ABI_INFO: RootModuleConsts = RootModuleConsts {
        layout: IsLayoutChecked::No,
        ..Self::CONSTANTS
    };

    /// Gets the statics for Self.
    ///
    /// To define this associated function use:
    /// [`abi_stable::declare_root_module_statics!{TypeOfSelf}`
    /// ](../macro.declare_root_module_statics.html).
    /// Passing `Self` instead of `TypeOfSelf` won't work.
    ///
    fn root_module_statics() -> &'static RootModuleStatics<Self>;

    /// Gets the root module,returning None if the module is not yet loaded.
    #[inline]
    fn get_module() -> Option<Self> {
        Self::root_module_statics().root_mod.get()
    }

    /// Gets the RawLibrary of the module,
    /// returning None if the dynamic library failed to load
    /// (it doesn't exist or layout checking failed).
    ///
    /// Note that if the root module is initialized using `Self::load_module_with`,
    /// this will return None even though `Self::get_module` does not.
    ///
    #[inline]
    fn get_raw_library() -> Option<&'static RawLibrary> {
        Self::root_module_statics().raw_lib.get()
    }

    /// Returns the path the library would be loaded from,given a directory(folder).
    fn get_library_path(directory: &Path) -> PathBuf {
        let base_name = Self::BASE_NAME;
        RawLibrary::path_in_directory(directory, base_name, LibrarySuffix::NoSuffix)
    }

    /// Loads the root module,with a closure which either
    /// returns the root module or an error.
    ///
    /// If the root module was already loaded,
    /// this will return the already loaded root module,
    /// without calling the closure.
    fn load_module_with<F, E>(f: F) -> Result<Self, E>
    where
        F: FnOnce() -> Result<Self, E>,
    {
        Self::root_module_statics().root_mod.try_init(f)
    }

    /// Loads this module from the path specified by `where_`,
    /// first loading the dynamic library if it wasn't already loaded.
    ///
    /// Once the root module is loaded,
    /// this will return the already loaded root module.
    ///
    /// # Warning
    ///
    /// If this function is called within a dynamic library,
    /// it must be called either within the root module loader function or
    /// after that function has been called.
    ///
    /// **DO NOT** call this in the static initializer of a dynamic library,
    /// since this library relies on setting up its global state before
    /// calling the root module loader.
    ///
    /// # Errors
    ///
    /// This will return these errors:
    ///
    /// - `LibraryError::OpenError`:
    /// If the dynamic library itself could not be loaded.
    ///
    /// - `LibraryError::GetSymbolError`:
    /// If the root module was not exported.
    ///
    /// - `LibraryError::InvalidAbiHeader`:
    /// If the abi_stable version used by the library is not compatible.
    ///
    /// - `LibraryError::ParseVersionError`:
    /// If the version strings in the library can't be parsed as version numbers,
    /// this can only happen if the version strings are manually constructed.
    ///
    /// - `LibraryError::IncompatibleVersionNumber`:
    /// If the version number of the library is incompatible.
    ///
    /// - `LibraryError::AbiInstability`:
    /// If the layout of the root module is not the expected one.
    ///
    /// - `LibraryError::RootModule` :
    /// If the root module initializer returned an error or panicked.
    ///
    fn load_from(where_: LibraryPath<'_>) -> Result<Self, LibraryError> {
        let statics = Self::root_module_statics();
        statics.root_mod.try_init(|| {
            let lib = statics.raw_lib.try_init(|| -> Result<_, LibraryError> {
                let raw_library = load_raw_library::<Self>(where_)?;

                // if the library isn't leaked
                // it would cause any use of the module to be a use after free.
                //
                // By leaking the library
                // this allows the root module loader to do anything that'd prevent
                // sound library unloading.
                Ok(leak_value(raw_library))
            })?;
            let items = unsafe { lib_header_from_raw_library(lib)? };

            items.ensure_layout::<Self>()?;

            // safety: the layout was checked in the code above,
            unsafe {
                items
                    .init_root_module_with_unchecked_layout::<Self>()?
                    .initialization()
            }
        })
    }

    /// Loads this module from the directory specified by `where_`,
    /// first loading the dynamic library if it wasn't already loaded.
    ///
    /// Once the root module is loaded,
    /// this will return the already loaded root module.
    ///
    /// Warnings and Errors are detailed in [`load_from`](#method.load_from),
    ///
    fn load_from_directory(where_: &Path) -> Result<Self, LibraryError> {
        Self::load_from(LibraryPath::Directory(where_))
    }

    /// Loads this module from the file at `path_`,
    /// first loading the dynamic library if it wasn't already loaded.
    ///
    /// Once the root module is loaded,
    /// this will return the already loaded root module.
    ///
    /// Warnings and Errors are detailed in [`load_from`](#method.load_from),
    ///
    fn load_from_file(path_: &Path) -> Result<Self, LibraryError> {
        Self::load_from(LibraryPath::FullPath(path_))
    }

    /// Defines behavior that happens once the module is loaded.
    ///
    /// This is ran in the `RootModule::load*` associated functions
    /// after the root module has succesfully been loaded.
    ///
    /// The default implementation does nothing.
    fn initialization(self) -> Result<Self, LibraryError> {
        Ok(self)
    }
}

/// Loads the raw library at `where_`
fn load_raw_library<M>(where_: LibraryPath<'_>) -> Result<RawLibrary, LibraryError>
where
    M: RootModule,
{
    let path = match where_ {
        LibraryPath::Directory(directory) => M::get_library_path(directory),
        LibraryPath::FullPath(full_path) => full_path.to_owned(),
    };
    RawLibrary::load_at(&path)
}

/// Gets the LibHeader of a library.
///
/// # Errors
///
/// This will return these errors:
///
/// - `LibraryError::GetSymbolError`:
/// If the root module was not exported.
///
/// - `LibraryError::InvalidAbiHeader`:
/// If the abi_stable used by the library is not compatible.
///
/// # Safety
///
/// The LibHeader is implicitly tied to the lifetime of the library,
/// it will contain dangling `'static` references if the library is dropped before it does.
///
///
pub unsafe fn lib_header_from_raw_library(
    raw_library: &RawLibrary,
) -> Result<&'static LibHeader, LibraryError> {
    unsafe { abi_header_from_raw_library(raw_library)?.upgrade() }
}

/// Gets the AbiHeaderRef of a library.
///
/// # Errors
///
/// This will return these errors:
///
/// - `LibraryError::GetSymbolError`:
/// If the root module was not exported.
///
/// # Safety
///
/// The AbiHeaderRef is implicitly tied to the lifetime of the library,
/// it will contain dangling `'static` references if the library is dropped before it does.
///
///
pub unsafe fn abi_header_from_raw_library(
    raw_library: &RawLibrary,
) -> Result<AbiHeaderRef, LibraryError> {
    let mangled = ROOT_MODULE_LOADER_NAME_WITH_NUL;
    let header: AbiHeaderRef = unsafe { *raw_library.get::<AbiHeaderRef>(mangled.as_bytes())? };

    Ok(header)
}

/// Gets the LibHeader of the library at the path.
///
/// This leaks the underlying dynamic library,
/// if you need to do this without leaking you'll need to use
/// `lib_header_from_raw_library` instead.
///
/// # Errors
///
/// This will return these errors:
///
/// - `LibraryError::OpenError`:
/// If the dynamic library itself could not be loaded.
///
/// - `LibraryError::GetSymbolError`:
/// If the root module was not exported.
///
/// - `LibraryError::InvalidAbiHeader`:
/// If the abi_stable version used by the library is not compatible.
///
///
pub fn lib_header_from_path(path: &Path) -> Result<&'static LibHeader, LibraryError> {
    let raw_lib = RawLibrary::load_at(path)?;

    let library_getter = unsafe { lib_header_from_raw_library(&raw_lib)? };

    mem::forget(raw_lib);

    Ok(library_getter)
}

/// Gets the AbiHeaderRef of the library at the path.
///
/// This leaks the underlying dynamic library,
/// if you need to do this without leaking you'll need to use
/// `lib_header_from_raw_library` instead.
///
/// # Errors
///
/// This will return these errors:
///
/// - `LibraryError::OpenError`:
/// If the dynamic library itself could not be loaded.
///
/// - `LibraryError::GetSymbolError`:
/// If the root module was not exported.
///
///
pub fn abi_header_from_path(path: &Path) -> Result<AbiHeaderRef, LibraryError> {
    let raw_lib = RawLibrary::load_at(path)?;

    let library_getter = unsafe { abi_header_from_raw_library(&raw_lib)? };

    mem::forget(raw_lib);

    Ok(library_getter)
}

//////////////////////////////////////////////////////////////////////

macro_rules! declare_root_module_consts {
    (
        fields=[
            $(
                $(#[$field_meta:meta])*
                method_docs=$method_docs:expr,
                $field:ident : $field_ty:ty
            ),* $(,)*
        ]
    ) => (
        /// All the constants of the [`RootModule`] trait for some erased type.
        ///
        /// [`RootModule`]: ./trait.RootModule.html
        #[repr(C)]
        #[derive(StableAbi,Copy,Clone)]
        pub struct RootModuleConsts{
            $(
                $(#[$field_meta])*
                $field : $field_ty,
            )*
            _priv:(),
        }

        impl RootModuleConsts{
            $(
                #[doc=$method_docs]
                pub const fn $field(&self)->$field_ty{
                    self.$field
                }
            )*
        }

    )
}

declare_root_module_consts! {
    fields=[
        method_docs="
         The name of the dynamic library,which is the same on all platforms.
         This is generally the name of the implementation crate.",
        base_name: RStr<'static>,

        method_docs="The name of the library used in error messages.",
        name: RStr<'static>,

        method_docs="The version number of the library this was created from.",
        version_strings: VersionStrings,

        method_docs="The (optional) type layout constant of the root module.",
        layout: IsLayoutChecked,

        method_docs="\
         Functions used to test that the C abi is the same in both the library 
         and the loader\
        ",
        c_abi_testing_fns:&'static CAbiTestingFns,
    ]
}
