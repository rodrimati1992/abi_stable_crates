use super::*;

use crate::utils::leak_value;


/**
The root module of a dynamic library,
which may contain other modules,function pointers,and static references.

For an example of a type implementing this trait you can look 
at either the example in the readme for this crate,
or the `example/example_*_interface` crates  in this crates' repository .

*/
pub trait RootModule: Sized+SharedStableAbi+'static  {

    /// The name of the dynamic library,which is the same on all platforms.
    /// This is generally the name of the `implementation crate`.
    const BASE_NAME: &'static str;

    /// The name of the library used in error messages.
    const NAME: &'static str;

    /// The version number of the library of `Self:RootModule`.
    /// 
    /// Initialize this with ` package_version_strings!() `
    const VERSION_STRINGS: VersionStrings;

    /// All the constants of this trait and supertraits.
    ///
    /// It can safely be used as a proxy for the associated constants of this trait.
    const CONSTANTS:RootModuleConsts<Self>=RootModuleConsts{
        inner:ErasedRootModuleConsts{
            base_name:StaticStr::new(Self::BASE_NAME),
            name:StaticStr::new(Self::NAME),
            version_strings:Self::VERSION_STRINGS,
            abi_info:<&Self>::S_ABI_INFO,
            _priv:(),
        },
        _priv:PhantomData,
    };

    /// Gets the statics for Self.
    ///
    /// To define this associated function use:
    /// `abi_stable::declare_root_module_statics!{TypeOfSelf}`.
    /// Passing `Self` instead of `TypeOfSelf` won't work.
    ///
    fn root_module_statics()->&'static RootModuleStatics<Self>;

/**
Gets the root module,returning None if the module is not yet loaded.
*/
    #[inline]
    fn get_module()->Option<&'static Self>{
        Self::root_module_statics().root_mod.get()
    }

/**
Gets the RawLibrary of the module,
returning None if the dynamic library failed to load
(it doesn't exist or layout checking failed).

Note that if the root module is initialized using `Self::load_module_with`,
this will return None even though `Self::get_module` does not.

*/
    #[inline]
    fn get_raw_library()->Option<&'static RawLibrary>{
        Self::root_module_statics().raw_lib.get()
    }

    /// Returns the path the library would be loaded from,given a directory(folder).
    fn get_library_path(directory:&Path)-> PathBuf {
        let base_name=Self::BASE_NAME;
        RawLibrary::path_in_directory(directory, base_name,LibrarySuffix::NoSuffix)
    }

/**

Loads the root module,with a closure which either 
returns the root module or an error.

If the root module was already loaded,
this will return a reference to the already loaded root module,
without calling the closure.

*/
    fn load_module_with<F,E>(f:F)->Result<&'static Self,E>
    where
        F:FnOnce()->Result<&'static Self,E>
    {
        Self::root_module_statics().root_mod.try_init(f)
    }

/**
Loads this module from the path specified by `where_`,
first loading the dynamic library if it wasn't already loaded.

Once the root module is loaded,
this will return a reference to the already loaded root module.

# Warning

If this function is called within a dynamic library,
it must be called at or after the function that exports its root module is called.

**DO NOT** call this in the static initializer of a dynamic library,
since this library relies on setting up its global state before
calling the root module loader.

# Errors

This will return these errors:

- LibraryError::OpenError:
If the dynamic library itself could not be loaded.

- LibraryError::GetSymbolError:
If the root module was not exported.

- LibraryError::InvalidAbiHeader:
If the abi_stable version used by the library is not compatible.

- LibraryError::ParseVersionError:
If the version strings in the library can't be parsed as version numbers,
this can only happen if the version strings are manually constructed.

- LibraryError::IncompatibleVersionNumber:
If the version number of the library is incompatible.

- LibraryError::AbiInstability:
If the layout of the root module is not the expected one.

*/
    fn load_from(where_:LibraryPath<'_>) -> Result<&'static Self, LibraryError>{
        let statics=Self::root_module_statics();
        statics.root_mod.try_init(||{
            let raw_library=load_raw_library::<Self>(where_)?;
            let items = unsafe{ lib_header_from_raw_library(&raw_library)? };

            let root_mod=items.init_root_module::<Self>()?.initialization()?;

            // Important,If I don't leak the library after sucessfully loading the root module
            // it would cause any use of the module to be a use after free.
            let raw_lib=leak_value(raw_library);
            statics.raw_lib.init(|| raw_lib );

            Ok(root_mod)
        })
    }

/**

Loads this module from the directory specified by `where_`,
first loading the dynamic library if it wasn't already loaded.

Once the root module is loaded,
this will return a reference to the already loaded root module.

Warnings and Errors are detailed in `load_from`,

*/
    fn load_from_directory(where_:&Path) -> Result<&'static Self, LibraryError>{
        Self::load_from(LibraryPath::Directory(where_))
    }

/**

Loads this module from the file at `path_`,
first loading the dynamic library if it wasn't already loaded.

Once the root module is loaded,
this will return a reference to the already loaded root module.

Warnings and Errors are detailed in `load_from`,

*/
    fn load_from_file(path_:&Path) -> Result<&'static Self, LibraryError>{
        Self::load_from(LibraryPath::FullPath(path_))
    }

    /// Defines behavior that happens once the module is loaded.
    ///
    /// The default implementation does nothing.
    fn initialization(self: &'static Self) -> Result<&'static Self, LibraryError> {
        Ok(self)
    }
}


/// Loads the raw library at `where_`
fn load_raw_library<M>(where_:LibraryPath<'_>) -> Result<RawLibrary, LibraryError>
where
    M:RootModule
{
    let path=match where_ {
        LibraryPath::Directory(directory)=>{
            M::get_library_path(&directory)
        }
        LibraryPath::FullPath(full_path)=>  
            full_path.to_owned(),
    };
    RawLibrary::load_at(&path)
}

/**
Gets the LibHeader of a library.

# Errors

This will return these errors:

- LibraryError::GetSymbolError:
If the root module was not exported.

- LibraryError::InvalidAbiHeader:
If the abi_stable used by the library is not compatible.

# Safety

The LibHeader is implicitly tied to the lifetime of the library,
it will contain dangling `'static` references if the library is dropped before it does.

*/
pub unsafe fn lib_header_from_raw_library(
    raw_library:&RawLibrary
)->Result< &'static LibHeader , LibraryError>
{
    unsafe{
        let mut mangled=mangled_root_module_loader_name();
        mangled.push('\0');
        let library_getter=
            raw_library.get::<&'static AbiHeader>(mangled.as_bytes())?;

        let header:&'static AbiHeader= *library_getter;

        if !header.is_compatible(&AbiHeader::VALUE) {
            return Err(LibraryError::InvalidAbiHeader(*header))
        }

        let lib_header=transmute_reference::<AbiHeader,LibHeader>(header);
        
        let globals=globals::initialized_globals();
        
        // This has to run before anything else.
        lib_header.initialize_library_globals(globals);

        Ok(lib_header)
    }
}


/**
Gets the LibHeader of the library at the path.

This leaks the underlying dynamic library,
if you need to do this without leaking you'll need to use
`lib_header_from_raw_library` instead.

# Errors

This will return these errors:

- LibraryError::OpenError:
If the dynamic library itself could not be loaded.

- LibraryError::GetSymbolError:
If the root module was not exported.

- LibraryError::InvalidAbiHeader:
If the abi_stable version used by the library is not compatible.

*/
pub fn lib_header_from_path(path:&Path)->Result< &'static LibHeader , LibraryError> {
    let raw_lib=RawLibrary::load_at(path)?;

    let library_getter=unsafe{ lib_header_from_raw_library(&raw_lib)? };

    mem::forget(raw_lib);

    Ok(library_getter)

}

//////////////////////////////////////////////////////////////////////


macro_rules! declare_root_module_consts {
    (
        fields=[
            $(
                $(#[$field_meta:meta])*
                $field:ident : $field_ty:ty
            ),* $(,)*
        ]
    ) => (
        /// Encapsulates all the important constants of `RootModule` for `M`,
        /// used mostly to construct a `LibHeader` with `LibHeader::from_constructor`.
        #[repr(C)]
        #[derive(StableAbi,Copy,Clone)]
        pub struct RootModuleConsts<M>{
            inner:ErasedRootModuleConsts,
            _priv:PhantomData<extern fn()->M>,
        }


        /// Encapsulates all the important constants of `RootModule` for some erased type.
        #[repr(C)]
        #[derive(StableAbi,Copy,Clone)]
        pub struct ErasedRootModuleConsts{
            $(
                $(#[$field_meta])*
                $field : $field_ty,
            )*
            _priv:(),
        }


        impl<M> RootModuleConsts<M>{
            pub const fn erased(&self)->ErasedRootModuleConsts{
                self.inner
            }
            $(
                pub const fn $field(&self)->$field_ty{
                    self.inner.$field
                }
            )*
        }

        impl ErasedRootModuleConsts{
            $(
                pub const fn $field(&self)->$field_ty{
                    self.$field
                }
            )*
        }

    )
}


declare_root_module_consts!{
    fields=[
        base_name: StaticStr,
        name: StaticStr,
        version_strings: VersionStrings,
        abi_info: &'static AbiInfoWrapper,
    ]
}

