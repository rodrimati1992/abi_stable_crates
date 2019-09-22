/*!
Zero-sized types .
*/

use std::{cell::Cell,marker::PhantomData, rc::Rc};

use crate::{
    derive_macro_reexports::*,
    type_layout::{
        MonoTypeLayout, MonoTLData, ReprAttr, TypeLayout, GenericTLData,
    }
};



/// Marker type used to mark a type as being Send+Sync.
#[repr(C)]
#[derive(StableAbi)]
pub struct SyncSend;

/// Marker type used to mark a type as being !Send+!Sync.
#[repr(C)]
#[derive(StableAbi)]
pub struct UnsyncUnsend {
    _marker: UnsafeIgnoredType<Rc<()>>,
}

/// Marker type used to mark a type as being Send+!Sync.
#[repr(C)]
#[derive(StableAbi)]
pub struct UnsyncSend {
    _marker: UnsafeIgnoredType<Cell<()>>,
}


/// Marker type used to mark a type as being !Send+Sync.
#[repr(C)]
#[derive(StableAbi)]
pub struct SyncUnsend {
    _marker: UnsyncUnsend,
}

unsafe impl Sync for SyncUnsend{}


/// Zero-sized marker type used to signal that even though a type 
/// could implement Copy and Clone,
/// it is semantically an error to do so.
#[repr(C)]
#[derive(StableAbi)]
pub struct NotCopyNotClone;



/// Used by vtables/pointers to signal that the type has been erased.
///
#[repr(C)]
#[derive(StableAbi)]
pub struct ErasedObject<T=()>{
    _priv: [u8; 0],
    _marker:PhantomData<extern "C" fn()->T>,
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

    pub const NEW:Self=Self{
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



unsafe impl<T> GetStaticEquivalent_ for UnsafeIgnoredType<T> {
    type StaticEquivalent=();
}
unsafe impl<T> SharedStableAbi for UnsafeIgnoredType<T> {
    type IsNonZeroType = False;
    type Kind=ValueKind;


    const S_LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("UnsafeIgnoredType"),
            make_item_info!(),
            MonoTLData::struct_(rslice![]),
            tl_genparams!(;;),
            ReprAttr::C,
            ModReflMode::Module,
            rslice![],
        );

        make_shared_vars!{
            let (mono_shared_vars,shared_vars)={};
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::S_ABI_CONSTS,
            GenericTLData::Struct,
        )
    };
}
