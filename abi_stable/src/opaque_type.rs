use std::marker::PhantomData;

use crate::{reexports::*, std_types::RNone};

#[repr(C)]
/// This type intentionally does not implement any traits.
pub struct OpaqueType<T> {
    _priv: [u8; 0],
    _inner: PhantomData<T>,
}

/// InterfaceType type used by vtables to signal that the type has been erased.
///
/// # C ABI Safety
///
/// When creating generic types which use pointers to some generic `T`,
/// it is recommended that you store a pointer to `ErasedObject` in the struct,
/// and cast the pointer back to `T` when doing anything with it.
///
/// This is a protection against compiler backends that treat
/// casting `extern fn(*const T)` to `extern fn(*const ErasedObject)`
/// as being Undefined Behavior.
/// Most compilers in practice don't treat this cast as invalid.
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
