use super::*;


/// The root module of a dynamic library,
/// which may contain other modules,function pointers,and static references.
///
/// For an example of a type implementing this trait you can look 
/// for the `example/example_*_interface` crates  in this crates' repository .
pub trait RootModule: Sized+SharedStableAbi  {

    /// The base name of the dynamic library,which is the same on all platforms.
    /// This is generally the name of the `implementation crate`.
    const BASE_NAME: &'static str;

    /// The name of the library used in error messages.
    const NAME: &'static str;

    /// The version number of this library.
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

    /// Returns the path the library would be loaded from,given a directory(folder).
    fn get_library_path(directory:&Path)-> PathBuf {
        let base_name=Self::BASE_NAME;
        RawLibrary::path_in_directory(directory, base_name,LibrarySuffix::NoSuffix)
    }

/**
Loads this module from the path specified by `where_`,
first loading the dynamic library if it wasn't already loaded.

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
    fn load_from_path(where_:LibraryPath<'_>) -> Result<&'static Self, LibraryError>{

        let raw_library=load_raw_library::<Self>(where_)?;
        let mut items = unsafe{ lib_header_from_raw_library(&raw_library)? };

        let root_mod=items.init_root_module::<Self>()?.initialization()?;

        // Important,If I don't leak the library after sucessfully loading the root module
        // it would cause any use of the module to be a use after free.
        mem::forget(raw_library);

        Ok(root_mod)
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
)->Result< LibHeader , LibraryError>
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

        Ok(lib_header.clone())
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
pub fn lib_header_from_path(path:&Path)->Result< LibHeader , LibraryError> {
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

