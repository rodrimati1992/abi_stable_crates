/*!
Zero-sized types .
*/

use std::{cell::Cell, marker::PhantomData, rc::Rc};

use crate::{
    abi_stability::PrefixStableAbi,
    derive_macro_reexports::*,
    type_layout::{GenericTLData, MonoTLData, MonoTypeLayout, ReprAttr, TypeLayout},
};

#[macro_use]
mod stable_abi_impls;

/////////////////

/// Marker type used to mark a type as being `Send + Sync`.
#[repr(C)]
#[derive(StableAbi)]
pub struct SyncSend;

const _: () = zst_assert! {SyncSend};

/////////////////

/// Marker type used to mark a type as being `!Send + !Sync`.
pub struct UnsyncUnsend {
    _marker: UnsafeIgnoredType<Rc<()>>,
}

monomorphic_marker_type! {UnsyncUnsend, UnsafeIgnoredType<Rc<()>>}

impl UnsyncUnsend {
    /// Constructs a `UnsyncUnsend`
    pub const NEW: Self = Self {
        _marker: UnsafeIgnoredType::NEW,
    };
}

/////////////////

/// Marker type used to mark a type as being `Send + !Sync`.
pub struct UnsyncSend {
    _marker: UnsafeIgnoredType<Cell<()>>,
}

monomorphic_marker_type! {UnsyncSend, UnsafeIgnoredType<Cell<()>>}

impl UnsyncSend {
    /// Constructs a `UnsyncSend`
    pub const NEW: Self = Self {
        _marker: UnsafeIgnoredType::NEW,
    };
}

/////////////////

/// Marker type used to mark a type as being `!Send + Sync`.
// #[sabi(debug_print)]
pub struct SyncUnsend {
    _marker: UnsyncUnsend,
}

monomorphic_marker_type! {SyncUnsend, UnsyncUnsend}

impl SyncUnsend {
    /// Constructs a `SyncUnsend`
    pub const NEW: Self = Self {
        _marker: UnsyncUnsend::NEW,
    };
}

unsafe impl Sync for SyncUnsend {}

/////////////////

/// Zero-sized marker type used to signal that even though a type
/// could implement `Copy` and `Clone`,
/// it is semantically an error to do so.
#[repr(C)]
#[derive(StableAbi)]
// #[sabi(debug_print)]
pub struct NotCopyNotClone;

const _: () = zst_assert! {NotCopyNotClone};

//////////////////////////////////////////////////////////////

/// Used by vtables/pointers to signal that the type has been erased.
///
pub struct ErasedObject<T = ()> {
    _marker: NonOwningPhantom<T>,
}

// Delete this once the abi_stable version number isn't 0.9
//
const _ITEM_INFO_CONST_ERASEDOBJECT: abi_stable::type_layout::ItemInfo =
    abi_stable::make_item_info!();
const _SHARED_VARS_STRINGS_ERASEDOBJECT: ::abi_stable::std_types::RStr<'static> =
    abi_stable::std_types::RStr::from_str("_priv;_marker;");
#[allow(non_upper_case_globals)]
mod _sabi_erasedobject {
    use super::*;
    pub(super) use :: abi_stable;
    #[allow(unused_imports)]
    pub(super) use ::abi_stable::derive_macro_reexports::{self as __sabi_re, renamed::*};
    pub struct _static_ErasedObject<T: ?Sized>(extern "C" fn(&T));
    unsafe impl<T> __GetStaticEquivalent_ for ErasedObject<T>
    where
        T: __StableAbi,
    {
        type StaticEquivalent = _static_ErasedObject<__GetStaticEquivalent<T>>;
    }
    #[doc(hidden)]
    pub(super) const _MONO_LAYOUT_ErasedObject: &'static __sabi_re::MonoTypeLayout =
        &__sabi_re::MonoTypeLayout::from_derive(__sabi_re::_private_MonoTypeLayoutDerive {
            name: abi_stable::std_types::RStr::from_str("ErasedObject"),
            item_info: _ITEM_INFO_CONST_ERASEDOBJECT,
            data: __sabi_re::MonoTLData::derive_struct(__CompTLFields::new(
                abi_stable::std_types::RSlice::from_slice(&[
                    562949953748992u64,
                    1688849860722694u64,
                ]),
                None,
            )),
            generics: abi_stable ::tl_genparams !(
                ;
                __StartLen :: new(0u16, 0u16) ;
                __StartLen :: new(0u16, 0u16)
            ),
            mod_refl_mode: __ModReflMode::Opaque,
            repr_attr: __ReprAttr::C,
            phantom_fields: abi_stable::std_types::RSlice::from_slice(&[]),
            shared_vars: abi_stable::type_layout::MonoSharedVars::new(
                _SHARED_VARS_STRINGS_ERASEDOBJECT,
                abi_stable::std_types::RSlice::from_slice(&[]),
            ),
        });
    impl<T> ErasedObject<T>
    where
        T: __StableAbi,
    {
        const __SABI_CONST_PARAMS_A: &'static [&'static __sabi_re::ConstGenericErasureHack<
            dyn ::std::marker::Send,
        >] = &[];
        const __SABI_CONST_PARAMS_B: &'static [__ConstGeneric] = &[];
        const __SABI_SHARED_VARS: &'static __sabi_re::SharedVars =
            &abi_stable::type_layout::SharedVars::new(
                _MONO_LAYOUT_ErasedObject.shared_vars_static(),
                abi_stable::_sabi_type_layouts!([u8; 0], NonOwningPhantom<T>,),
                __sabi_re::RSlice::from_slice(Self::__SABI_CONST_PARAMS_B),
            );
    }
    unsafe impl<T> __sabi_re::StableAbi for ErasedObject<T>
    where
        T: __StableAbi,
    {
        type IsNonZeroType = __sabi_re::False;
        const LAYOUT: &'static __sabi_re::TypeLayout = {
            zst_assert! {Self}

            &__sabi_re::TypeLayout::from_derive::<Self>(__sabi_re::_private_TypeLayoutDerive {
                shared_vars: Self::__SABI_SHARED_VARS,
                mono: _MONO_LAYOUT_ErasedObject,
                abi_consts: Self::ABI_CONSTS,
                data: __sabi_re::GenericTLData::Struct,
                tag: None,
                extra_checks: None,
            })
        };
    }
}

//////////////////////////////////////////////////////////////

/// Used by pointers to vtables/modules to signal that the type has been erased.
///
pub struct ErasedPrefix {
    _priv: PhantomData<u8>,
}

const _: () = zst_assert!(ErasedPrefix);

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
pub struct UnsafeIgnoredType<T: ?Sized> {
    _inner: PhantomData<T>,
}

impl<T: ?Sized> UnsafeIgnoredType<T> {
    /// Constructs an `UnsafeIgnoredType`.
    pub const DEFAULT: Self = Self {
        _inner: PhantomData,
    };

    /// Constructs an `UnsafeIgnoredType`.
    pub const NEW: Self = Self {
        _inner: PhantomData,
    };
}

impl<T: ?Sized> Copy for UnsafeIgnoredType<T> {}

impl<T: ?Sized> Default for UnsafeIgnoredType<T> {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl<T: ?Sized> Clone for UnsafeIgnoredType<T> {
    fn clone(&self) -> Self {
        *self
    }
}

unsafe impl<T> GetStaticEquivalent_ for UnsafeIgnoredType<T> {
    type StaticEquivalent = ();
}
unsafe impl<T> StableAbi for UnsafeIgnoredType<T> {
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT: &MonoTypeLayout = &MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("UnsafeIgnoredType"),
            make_item_info!(),
            MonoTLData::struct_(rslice![]),
            tl_genparams!(;;),
            ReprAttr::C,
            ModReflMode::Module,
            rslice![],
        );

        make_shared_vars! {
            impl[T] UnsafeIgnoredType<T>;

            let (mono_shared_vars,shared_vars)={};
        }

        zst_assert!(Self);

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
pub struct NonOwningPhantom<T: ?Sized> {
    // The StableAbi layout for a `NonOwningPhantom<T>` is the same as `PhantomData<T>`,
    // the type of this field is purely for variance.
    _marker: PhantomData<extern "C" fn() -> T>,
}

impl<T: ?Sized> NonOwningPhantom<T> {
    /// Constructs a `NonOwningPhantom`
    pub const DEFAULT: Self = Self {
        _marker: PhantomData,
    };

    /// Constructs a `NonOwningPhantom`
    pub const NEW: Self = Self {
        _marker: PhantomData,
    };
}

impl<T: ?Sized> Copy for NonOwningPhantom<T> {}

impl<T: ?Sized> Default for NonOwningPhantom<T> {
    #[inline(always)]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl<T: ?Sized> Clone for NonOwningPhantom<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

unsafe impl<T: ?Sized> GetStaticEquivalent_ for NonOwningPhantom<T>
where
    PhantomData<T>: GetStaticEquivalent_,
{
    type StaticEquivalent = GetStaticEquivalent<PhantomData<T>>;
}

unsafe impl<T: ?Sized> StableAbi for NonOwningPhantom<T>
where
    PhantomData<T>: StableAbi,
{
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = {
        zst_assert!(Self);
        <PhantomData<T> as StableAbi>::LAYOUT
    };
}
