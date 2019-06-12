use super::*;

use crate::{
    utils::{Constructor,ConstructorOrValue},
};

/// Used to check the layout of modules returned by module-loading functions
/// exported by dynamic libraries.
#[repr(C)]
#[derive(StableAbi,Clone)]
pub struct LibHeader {
    header:AbiHeader,
    root_mod_consts:ErasedRootModuleConsts,
    init_globals_with:InitGlobalsWith,
    module:ConstructorOrValue<&'static ErasedObject>
}

impl LibHeader {
    /// Constructs a LibHeader from the root module loader.
    pub const unsafe fn from_constructor<M>(
        constructor:Constructor<&'static ErasedObject>,
        root_mod_consts:RootModuleConsts<M>,
    )->Self
    {
        Self {
            header:AbiHeader::VALUE,
            root_mod_consts:root_mod_consts.erased(),
            init_globals_with: INIT_GLOBALS_WITH,
            module:ConstructorOrValue::Constructor(constructor),
        }
    }

    /// Constructs a LibHeader from the module.
    pub fn from_module<T>(value:&'static T)->Self
    where
        T: RootModule,
    {
        let value=unsafe{ transmute_reference::<T,ErasedObject>(value) };
        Self {
            header:AbiHeader::VALUE,
            root_mod_consts: T::CONSTANTS.erased(),
            init_globals_with: INIT_GLOBALS_WITH,
            module:ConstructorOrValue::Value(value),
        }
    }

    pub fn root_mod_consts(&self)->&ErasedRootModuleConsts{
        &self.root_mod_consts
    }

    /// The version string of the library the module is being loaded from.
    pub fn version_strings(&self)->VersionStrings{
        self.root_mod_consts.version_strings()
    }

    /// Gets the layout of the root module.
    pub fn layout(&self)->&'static AbiInfoWrapper{
        self.root_mod_consts.abi_info()
    }

    pub(super) fn initialize_library_globals(&self,globals:&'static Globals){
        (self.init_globals_with.0)(globals);
    }


    fn check_version<M>(&self)->Result<(),LibraryError>
    where
        M:RootModule
    {
        let expected_version = M::VERSION_STRINGS
            .piped(VersionNumber::new)?;

        let actual_version = self.version_strings().piped(VersionNumber::new)?;

        if expected_version.major != actual_version.major || 
            (expected_version.major==0) && expected_version.minor > actual_version.minor
        {
            return Err(LibraryError::IncompatibleVersionNumber {
                library_name: M::NAME,
                expected_version,
                actual_version,
            });
        }
        Ok(())
    }


    /**
Checks that the library is compatible,returning the root module on success.

It checks that these are compatible:

- The version number of the library

- The layout of the root module.

# Warning

If this function is called within a dynamic library,
it must be called at or after the function that exports its root module is called.

**DO NOT** call this in the static initializer of a dynamic library,
since this library relies on setting up its global state before
calling the root module loader.

# Errors

This will return these errors:

- LibraryError::ParseVersionError:
If the version strings in the library can't be parsed as version numbers,
this can only happen if the version strings are manually constructed.

- LibraryError::IncompatibleVersionNumber:
If the version number of the library is incompatible.

- LibraryError::AbiInstability:
If the layout of the root module is not the expected one.



    */
    pub fn init_root_module<M>(&mut self)-> Result<&'static M, LibraryError>
    where
        M: RootModule
    {
        self.check_version::<M>()?;
        self.check_layout::<M>()
    }



    /**
Checks that the version number of the library is compatible,
returning the root module on success.

This function transmutes the root module type,
without checking that the layout is compatible first.

# Warning

If this function is called within a dynamic library,
it must be called at or after the function that exports its root module is called.

**DO NOT** call this in the static initializer of a dynamic library,
since this library relies on setting up its global state before
calling the root module loader.

# Safety

The caller must ensure that `M` has the expected layout.

# Errors

This will return these errors:

- LibraryError::ParseVersionError:
If the version strings in the library can't be parsed as version numbers,
this can only happen if the version strings are manually constructed.

- LibraryError::IncompatibleVersionNumber:
If the version number of the library is incompatible.

    */
    pub unsafe fn init_root_module_with_unchecked_layout<M>(
        &mut self
    )-> Result<&'static M, LibraryError>
    where
        M: RootModule
    {
        self.check_version::<M>()?;
        Ok(self.unchecked_layout())
    }


    /// Gets the root module,first 
    /// checking that the layout of the `M` from the dynamic library is 
    /// compatible with the expected layout.
    pub fn check_layout<M>(&mut self) -> Result<&'static M, LibraryError>
    where
        M: RootModule,
    {

        // Using this instead of
        // crate::abi_stability::abi_checking::check_layout_compatibility
        // so that if this is called in a dynamic-library that loads 
        // another dynamic-library,
        // it uses the layout checker of the executable,
        // ensuring a globally unique view of the layout of types.
        //
        // This might also reduce the code in the library,
        // because it doesn't have to compile the layout checker for every library.
        (globals::initialized_globals().layout_checking)
            (<&M>::S_ABI_INFO, self.root_mod_consts.abi_info())
            .into_result()
            .map_err(LibraryError::AbiInstability)?;
        
        atomic::compiler_fence(atomic::Ordering::SeqCst);
        
        let ret=unsafe{ 
            transmute_reference::<ErasedObject,M>(self.module.get())
        };
        Ok(ret)
    }


/**
Gets the root module without checking that the layout of `M` is the expected one.
This is effectively a transmute.

This is useful if a user keeps a cache of which dynamic libraries 
have been checked for layout compatibility.

# Safety

The caller must ensure that `M` has the expected layout.

*/
    pub unsafe fn unchecked_layout<M>(&mut self)->&'static M{
        transmute_reference::<ErasedObject,M>(self.module.get())
    }
}

//////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(StableAbi,Copy,Clone)]
struct InitGlobalsWith(pub extern fn(&'static Globals));

const INIT_GLOBALS_WITH:InitGlobalsWith=
    InitGlobalsWith(crate::globals::initialize_globals_with);


//////////////////////////////////////////////////////////////////////

/**
Represents the abi_stable version used by a compiled dynamic library,
which if incompatible would produce a `LibraryError::InvalidAbiHeader`
*/
#[repr(C)]
#[derive(Debug,StableAbi,Copy,Clone)]
pub struct AbiHeader{
    /// A magic string used to check that this is actually abi_stable.
    pub magic_string:[u8;32],
    /// The major abi version of abi_stable
    pub abi_major:u32,
    /// The minor abi version of abi_stable
    pub abi_minor:u32,
    _priv:(),
}


impl AbiHeader{
    /// The value of the AbiHeader stored in dynamic libraries that use this 
    /// version of abi_stable
    pub const VALUE:AbiHeader=AbiHeader{
        magic_string:*b"abi stable library for Rust     ",
        abi_major:0,
        abi_minor:4,
        _priv:(),
    };
}



impl AbiHeader{
    /// Checks whether this AbiHeader is compatible with `other`.
    pub fn is_compatible(&self,other:&Self)->bool{
        self.magic_string == other.magic_string&&
        self.abi_major    == other.abi_major   &&
        ( self.abi_major!=0 || self.abi_minor==other.abi_minor )
    }
}
