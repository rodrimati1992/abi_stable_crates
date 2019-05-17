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

    /// Returns the path the library would be loaded from.
    fn get_library_path(where_:LibraryPath<'_>)-> PathBuf {
        match where_ {
            LibraryPath::Directory(directory)=>{
                let base_name=Self::BASE_NAME;
                RawLibrary::path_in_directory(directory, base_name,LibrarySuffix::NoSuffix)
            }
            LibraryPath::FullPath(full_path)=>  
                full_path.to_owned(),
        }
    }

    /// Loads this module from the path specified by `where_`,
    /// first loading the dynamic library if it wasn't already loaded.
    fn load_from_library(where_:LibraryPath<'_>) -> Result<&'static Self, LibraryError>{
        let raw_library=load_raw_library::<Self>(where_)?;
        let items = unsafe{ lib_header_from_raw_library(&raw_library)? };

        let globals=globals::initialized_globals();
        
        // This has to run before anything else.    
        items.initialize_library_globals(globals);

        let expected_version = Self::VERSION_STRINGS
            .piped(VersionNumber::new)?;
        let actual_version = items.version_strings().piped(VersionNumber::new)?;

        if expected_version.major != actual_version.major || 
            (expected_version.major==0) && expected_version.minor > actual_version.minor
        {
            return Err(LibraryError::IncompatibleVersionNumber {
                library_name: Self::NAME,
                expected_version,
                actual_version,
            });
        }

        let root_mod=items.check_layout::<Self>()?
            .initialization()?;

        // Important,If I don't leak the library after sucessfully loading the root module
        // it would cause any use of the module to be a use after free.
        mem::forget(raw_library);

        Ok(root_mod)

    }

    /// Returns the layout of the root module of the library at the specified path.
    fn layout_of_library(where_:LibraryPath<'_>)->Result<&'static AbiInfo,LibraryError>{
        let raw_lib=load_raw_library::<Self>(where_)?;

        let library_getter=unsafe{ lib_header_from_raw_library(&raw_lib)? };

        let layout=library_getter.layout();

        // Important,If I don't leak the library after sucessfully loading the root module
        // it would cause any use of the module to be a use after free.
        mem::forget(raw_lib);

        Ok(layout)

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
    let path=M::get_library_path(where_);
    RawLibrary::load_at(&path)
}


/// Gets the LibHeader of a library.
///
/// # Safety
///
/// The LibHeader is implicitly tied to the lifetime of the library,
/// it will contain dangling `'static` references if the library is dropped. 
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

        Ok(*transmute_reference::<AbiHeader,LibHeader>(header))
    }
}


/// Gets the LibHeader of the library at the path.
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

