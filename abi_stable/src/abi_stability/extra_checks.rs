/*

The untested prototype of a way for users to add extra layout checks.

The main reasons this is not an accessible module are:

- I haven't figured out what to do with respect to merging 
    TagVariant::Ignored in the tag of a TypeLayout,
    in the case that someone wants to load multiple 
    dynamic libraries with the same interface,
    which have to be compatible to be loaded.



*/

use crate::{
    erased_types::trait_objects::DebugDisplayObject,
};

/**
Extra layout checks,passed by the user in either:

- the StableAbi derive macro:
    in the `#[sabi(extra_checks=" funtion_name::<type_params> ")]` attribute.

- the `extra_checks` field of `TypeLayoutParams`,

*/
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq,StableAbi)]
pub struct ExtraChecks{
    pub func:extern "C" fn(ExtraChecksParams)->RResult<(),ExtraAbiErrors>
}

#[repr(C)]
#[derive(Debug, PartialEq,Eq,StableAbi)]
pub struct ExtraChecksParams<'tag>{
    interface    :&'static AbiInfo,
    interface_tag:&'tag CheckableTag,
    impl_   :&'static AbiInfo,
    impl_tag:&'tag CheckableTag,
}



/// All the errors returned by `ExtraChecks.func`,as well as how serious they are.
#[repr(C)]
#[derive(Debug, PartialEq,Eq,StableAbi)]
pub struct ExtraAbiErrors{
    pub errors:RVec<ExtraAbiError>,
}


/// An error returned by `ExtraChecks.func`.
#[repr(C)]
#[derive(Debug, PartialEq,Eq,StableAbi)]
pub struct ExtraAbiError{
    pub name:RString,
    pub description:RString,
    pub error:DebugDisplayObject,
    pub seriousness:ErrorSeriousness,
}


/// How serious an error returned from `ExtraChecks.func`,
/// determining whether layout checking immediately returns an error or
/// collects a few more errors.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq,Eq,StableAbi)]
pub enum ErrorSeriousness{
    /// An error that stops layout checking immediately.
    Fatal,

    /**
An error that doesn't prevent more checks from happening.
Eg:If this error comes from checking a field,
    it would not stop checking of the other fields.
    */
    NonFatal,
}


