/*!
Zero-sized types .
*/

use std::{cell::Cell,marker::PhantomData, rc::Rc};

use crate::{
    abi_stability::PrefixStableAbi,
    derive_macro_reexports::*,
    type_layout::{
        MonoTypeLayout, MonoTLData, ReprAttr, TypeLayout, GenericTLData,
    }
};


/////////////////

/// Marker type used to mark a type as being `Send + Sync`.
#[repr(C)]
#[derive(StableAbi)]
pub struct SyncSend;

/////////////////

/// Marker type used to mark a type as being `!Send + !Sync`.
#[repr(C)]
#[derive(StableAbi)]
pub struct UnsyncUnsend {
    _marker: UnsafeIgnoredType<Rc<()>>,
}

impl UnsyncUnsend {
    /// Constructs a `UnsyncUnsend`
    pub const NEW: Self = Self { _marker: UnsafeIgnoredType::NEW };
}

/////////////////

/// Marker type used to mark a type as being `Send + !Sync`.
#[repr(C)]
#[derive(StableAbi)]
pub struct UnsyncSend {
    _marker: UnsafeIgnoredType<Cell<()>>,
}

impl UnsyncSend {
    /// Constructs a `UnsyncSend`
    pub const NEW: Self = Self { _marker: UnsafeIgnoredType::NEW };
}

/////////////////

/// Marker type used to mark a type as being `!Send + Sync`.
#[repr(C)]
#[derive(StableAbi)]
pub struct SyncUnsend {
    _marker: UnsyncUnsend,
}

impl SyncUnsend {
    /// Constructs a `SyncUnsend`
    pub const NEW: Self = Self { _marker: UnsyncUnsend::NEW };
}

unsafe impl Sync for SyncUnsend{}

/////////////////

/// Zero-sized marker type used to signal that even though a type 
/// could implement `Copy` and `Clone`,
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
    _marker:NonOwningPhantom<T>,
}

//////////////////////////////////////////////////////////////


/// Used by pointers to vtables/modules to signal that the type has been erased.
///
#[repr(C)]
pub struct ErasedPrefix{
    _priv: [u8; 0],
}

unsafe impl GetStaticEquivalent_ for ErasedPrefix {
    type StaticEquivalent = ErasedPrefix;
}

unsafe impl PrefixStableAbi for ErasedPrefix {
    type IsNonZeroType = False;
    const LAYOUT: &'static TypeLayout = <ErasedObject as StableAbi>::LAYOUT;
}

//////////////////////////////////////////////////////////////



/**
MarkerType which ignores its type parameter in its [`StableAbi`] implementation.

# Safety

`Unsafe` is part of its name,
because users can violate memory safety
if they depend on the value of the type parameter passed to `UnsafeIgnoredType` for safety,
since the type parameter is ignored when type checking dynamic libraries.


[`StableAbi`]: ../trait.StableAbi.html

*/
#[repr(C)]
pub struct UnsafeIgnoredType<T:?Sized> {
    _priv: [u8; 0],
    _inner: PhantomData<T>,
}

impl<T:?Sized> UnsafeIgnoredType<T>{
    /// Constructs an `UnsafeIgnoredType`.
    pub const DEFAULT:Self=Self{
        _priv:[],
        _inner:PhantomData,
    };

    /// Constructs an `UnsafeIgnoredType`.
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
unsafe impl<T> StableAbi for UnsafeIgnoredType<T> {
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT:&MonoTypeLayout=&MonoTypeLayout::new(
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
            Self::ABI_CONSTS,
            GenericTLData::Struct,
        )
    };
}


//////////////////////////////////////////////////////////////

/// An ffi-safe equivalent of a `PhantomData<fn()->T>`
pub struct NonOwningPhantom<T:?Sized>{
    _priv: [u8;0],
    // The StableAbi layout for a `NonOwningPhantom<T>` is the same as `PhantomData<T>`,
    // the type of this field is purely for variance.
    _marker: PhantomData<extern "C" fn()->T>
}

impl<T:?Sized> NonOwningPhantom<T>{
    /// Constructs a `NonOwningPhantom`
    pub const DEFAULT:Self=Self{
        _priv:[],
        _marker:PhantomData,
    };

    /// Constructs a `NonOwningPhantom`
    pub const NEW:Self=Self{
        _priv:[],
        _marker:PhantomData,
    };
}

impl<T:?Sized> Copy for NonOwningPhantom<T>{}

impl<T:?Sized> Default for NonOwningPhantom<T>{
    #[inline(always)]
    fn default()->Self{
        Self::DEFAULT
    }
}

impl<T:?Sized> Clone for NonOwningPhantom<T>{
    #[inline(always)]
    fn clone(&self)->Self{
        *self
    }
}

unsafe impl<T:?Sized> GetStaticEquivalent_ for NonOwningPhantom<T> 
where
    PhantomData<T>:GetStaticEquivalent_
{
    type StaticEquivalent=GetStaticEquivalent<PhantomData<T>>;
}

unsafe impl<T:?Sized> StableAbi for NonOwningPhantom<T> 
where
    PhantomData<T>:StableAbi
{
    type IsNonZeroType = False;


    const LAYOUT: &'static TypeLayout = 
        <PhantomData<T> as StableAbi>::LAYOUT;
}
