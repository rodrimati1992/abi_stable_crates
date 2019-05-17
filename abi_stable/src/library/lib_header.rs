use super::*;

use crate::{
    utils::{Constructor,ConstructorOrValue},
};

/// Used to check the layout of modules returned by module-loading functions
/// exported by dynamic libraries.
#[repr(C)]
#[derive(StableAbi,Copy,Clone)]
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
    pub fn layout(&self)->&'static AbiInfo{
        self.root_mod_consts.abi_info().get()
    }

    pub fn initialize_library_globals(&self,globals:&'static Globals){
        (self.init_globals_with.0)(globals);
    }

    /// Checks that the layout of the `T` from the dynamic library is 
    /// compatible with the caller's .
    pub fn check_layout<T>(mut self) -> Result<&'static T, LibraryError>
    where
        T: RootModule,
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
            (<&T>::S_ABI_INFO, self.root_mod_consts.abi_info())
            .into_result()
            .map_err(LibraryError::AbiInstability)?;
        
        atomic::compiler_fence(atomic::Ordering::SeqCst);
        
        let ret=unsafe{ 
            transmute_reference::<ErasedObject,T>(self.module.get())
        };
        Ok(ret)
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
    pub magic_string:[u8;32],
    pub abi_major:u32,
    pub abi_minor:u32,
    _priv:(),
}


impl AbiHeader{
    pub const VALUE:AbiHeader=AbiHeader{
        magic_string:*b"abi stable library for Rust     ",
        abi_major:0,
        abi_minor:4,
        _priv:(),
    };
}



impl AbiHeader{
    pub fn is_compatible(&self,other:&Self)->bool{
        self.magic_string == other.magic_string&&
        self.abi_major    == other.abi_major   &&
        ( self.abi_major!=0 || self.abi_minor==other.abi_minor )
    }
}
