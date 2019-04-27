/*!
Zero-sized types .
*/

use std::{marker::PhantomData, rc::Rc};

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


/// Zero-sized marker type used to signal that even though a type 
/// could implement Copy and Clone,
/// it is semantically an error to do so.
pub struct NotCopyNotClone;



/// A Zero-sized type used by `VirtualWrapper<Pointer<ZeroSized<T>>>`.
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
/// `VirtualWrapper<Pointer<ErasedObject>>`
/// can be created by calling `VirtualWrapper::from_any_ptr(ptr,())`.
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
pub struct UnsafeIgnoredType<T> {
    _priv: [u8; 0],
    _inner: PhantomData<T>,
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
