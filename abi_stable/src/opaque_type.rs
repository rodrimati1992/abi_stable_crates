use std::marker::PhantomData;

use crate::{reexports::*, std_types::RNone};

#[repr(C)]
/// A Zero-sized type used by `VirtualWrapper<Pointer<OpaqueType<T>>>`.
///
/// If this did not wrap `T`,
/// we could pretend to have a `T` even though we don't.
///
/// Casting the pointer type to point to this type is safe,
/// because the pointer is required to be castable to point to different types,
/// so long as a reference to one is valid for the other.
///
/// This type intentionally does not implement any traits.
pub struct OpaqueType<T> {
    _priv: [u8; 0],
    _inner: PhantomData<T>,
}

/// Used by vtables/pointers to signal that the type has been erased.
///
/// # Safety
///
/// This is unsafe to use,
/// because it allows passing a pointer of a different type to a vtable function.
///
pub type ErasedObject = OpaqueType<()>;

unsafe impl<T> StableAbi for OpaqueType<T> {
    type IsNonZeroType = False;
    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_primitive::<Self>(
        "OpaqueType",
        RNone,
        TLData::Primitive,
        tl_genparams!(;;),
    );
}
