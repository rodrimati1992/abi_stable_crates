/*!
Zero-sized types .
*/

use std::{cell::Cell,marker::PhantomData, rc::Rc};

use crate::{
    derive_macro_reexports::*, 
    std_types::RNone,
};



/// Marker type used to mark a type as being Send+Sync.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct SyncSend;

/// Marker type used to mark a type as being !Send+!Sync.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct UnsyncUnsend {
    _marker: UnsafeIgnoredType<Rc<()>>,
}

/// Marker type used to mark a type as being Send+!Sync.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct UnsyncSend {
    _marker: UnsafeIgnoredType<Cell<()>>,
}


/// Zero-sized marker type used to signal that even though a type 
/// could implement Copy and Clone,
/// it is semantically an error to do so.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct NotCopyNotClone;



/// A Zero-sized type used by `DynTrait<Pointer<()>,T>`.
///
/// If this did not wrap `T`,
/// we could pretend to have a `T` even though we don't.
///
/// This type intentionally does not implement any traits.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct ZeroSized<T> {
    _priv: [u8; 0],
    _inner: PhantomData<T>,
}

/// Used by vtables/pointers to signal that the type has been erased.
///
/// Also,because `()` implements InterfaceType,
/// `DynTrait<Pointer<ErasedObject>>`
/// can be created by calling `DynTrait::from_any_ptr(ptr,())`.
///
/// Do note that `()` is an `InterfaceType<Send=True,Sync=True>`,
/// which requires that `ptr` implements `Send+Sync`
///
pub type ErasedObject = ZeroSized<()>;



#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct ErasedRef<'a>{
    _marker:PhantomData<&'a ()>
}



/**
MarkerType which ignores its type parameter in its StableAbi implementation.

# Safety

`Unsafe` is part of its name,
because users could inadvertently violate memory safety
if they depend on the value of the type parameter passed to `UnsafeIgnoredType` for safety,
since the other side could choose any other type parameter.

*/
#[repr(C)]
pub struct UnsafeIgnoredType<T:?Sized> {
    _priv: [u8; 0],
    _inner: PhantomData<T>,
}

impl<T:?Sized> UnsafeIgnoredType<T>{
    pub const DEFAULT:Self=Self{
        _priv:[],
        _inner:PhantomData,
    };
}

impl<T:?Sized> Copy for UnsafeIgnoredType<T>{}

impl<T:?Sized> Default for UnsafeIgnoredType<T>{
    fn default()->Self{
        Self::DEFAULT
    }
}

impl<T:?Sized> Clone for UnsafeIgnoredType<T>{
    fn clone(&self)->Self{
        *self
    }
}


unsafe impl<T> SharedStableAbi for UnsafeIgnoredType<T> {
    type IsNonZeroType = False;
    type Kind=ValueKind;
    type StaticEquivalent=();

    const S_LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_phantom::<Self>(
        "UnsafeIgnoredType",
        RNone,
        TLData::Primitive,
        tl_genparams!(;;),
        &[]
    );
}
