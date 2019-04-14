/*!
Where the StableAbi trait is declares,as well as related types/traits.
*/

use core_extensions::type_level_bool::{Boolean, False, True};
use std::{
    fmt,
    marker::PhantomData,
    num,
    pin::Pin,
    ptr::NonNull,
    sync::atomic::{AtomicBool, AtomicIsize, AtomicPtr, AtomicUsize},
};

use crate::std_types::{RNone, RSome, StaticSlice, StaticStr};

use super::{LifetimeIndex, RustPrimitive, TLData, TLField, TypeLayout, TypeLayoutParams};

///////////////////////

/**
A SharedStableAbi type whose layout does not change in minor versions.

There is a blanket impl of this trait for all `SharedStableAbi<Kind=ValueKind>` types.
*/
pub unsafe trait StableAbi:SharedStableAbi<Kind=ValueKind> {
    /**
Whether this type has a single invalid bit-pattern.

Possible values:True/False

Some standard library types have a single value that is invalid for them eg:0,null.
these types are the only ones which can be stored in a `Option<_>` that implements AbiStable.

An alternative for types where `IsNonZeroType=False`,you can use `abi_stable::ROption`.

Non-exhaustive list of std types that are NonZero:

- &T (any T).

- &mut T (any T).

- extern fn() : Any combination of StableAbi parameter/return types.

- std::ptr::NonNull

- std::num::NonZero* 

    */
    type IsNonZeroType: Boolean;


    /// The layout of the type provided by implementors.
    const LAYOUT: &'static TypeLayout;

    /// The layout of the type,derived from Self::LAYOUT and associated types.
    const ABI_INFO: &'static AbiInfoWrapper;
}


/**
Represents a type whose layout is stable.

This trait can be derived using `#[derive(StableAbi)]`.

# Safety

The layout of types implementing this trait must be stable across minor versions,

# Caveats

This trait cannot be directly implemented for functions that take lifetime parameters,
because of that,`#[derive(StableAbi)]` detects the presence of `extern fn` types 
in type definitions.

*/
pub unsafe trait SharedStableAbi {
    /**
Whether this type has a single invalid bit-pattern.

Possible values:True/False

Some standard library types have a single value that is invalid for them eg:0,null.
these types are the only ones which can be stored in a `Option<_>` that implements AbiStable.

An alternative for types where `IsNonZeroType=False`,you can use `abi_stable::ROption`.

Non-exhaustive list of std types that are NonZero:

- &T (any T).

- &mut T (any T).

- extern fn() : Any combination of StableAbi parameter/return types.

- std::ptr::NonNull

- std::num::NonZero* 

    */
    type IsNonZeroType: Boolean;

    /**
The kind of abi stability of this type,there are 2:

- ValueKind:The layout of this type does not change in minor versions.

- PrefixKind:
    A struct which can add fields in minor versions,
    only usable behind a shared reference,
    used to implement extensible vtables and modules.

    */
    type Kind:TypeKindTrait;

    /// The layout of the type provided by implementors.
    const LAYOUT: &'static TypeLayout;

    /// The layout of the type,derived from Self::LAYOUT and associated types.
    const ABI_INFO: &'static AbiInfoWrapper = {
        let info = AbiInfo {
            kind:<Self::Kind as TypeKindTrait>::VALUE,
            prefix_kind: <Self::Kind as TypeKindTrait>::IS_PREFIX,
            is_nonzero: <Self::IsNonZeroType as Boolean>::VALUE,
            layout: Self::LAYOUT,
        };

        &AbiInfoWrapper::new(info)
    };
}


impl<This> StableAbi for This
where This:SharedStableAbi<Kind=ValueKind>
{
    type IsNonZeroType=<This as SharedStableAbi>::IsNonZeroType;
    const LAYOUT: &'static TypeLayout=<This as SharedStableAbi>::LAYOUT;
    const ABI_INFO: &'static AbiInfoWrapper=<This as SharedStableAbi>::ABI_INFO;
}


///////////////////////


/// Wraps a correctly constructed AbiInfo.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct AbiInfoWrapper {
    inner: AbiInfo,
    _priv: (),
}

impl AbiInfoWrapper {
    const fn new(inner: AbiInfo) -> Self {
        Self { inner, _priv: () }
    }
    /// Unsafely constructs AbiInfoWrapper from any AbiInfo.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the layout is that of the datatype this AbiInfo represents,
    /// and that it stays consistent with it across time.
    pub const unsafe fn new_unchecked(inner: AbiInfo) -> Self {
        Self::new(inner)
    }
    /// Gets the wrapped AbiInfo.
    pub const fn get(&self) -> &AbiInfo {
        &self.inner
    }
}

/// Describes the abi of some type.
///
/// # Safety
///
/// You must ensure that it describes the actual abi of the type.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct AbiInfo {
    pub kind:PrefixKind,
    pub prefix_kind: bool,
    /// Whether the type uses non-zero value optimization,
    /// if true then an Option<Self> implements StableAbi.
    pub is_nonzero: bool,
    pub layout: &'static TypeLayout,
}

///////////////////////////////////////////////////////////////////////////////


pub enum TypeKind{
    /// A value whose layout must not change in minor versions
    Value,
    /// A struct whose fields can be extended in minor versions,
    /// but only behind a shared reference,
    /// used to implement vtables and modules.
    Prefix,
}
pub trait TypeKindTrait:Sealed{
    const VALUE:TypeKind;
    const IS_PREFIX:bool;
}
pub struct ValueKind;
pub struct PrefixKind;


mod sealed{
    pub trait Sealed{}
}

impl sealed::Sealed for ValueKind{}
impl sealed::Sealed for PrefixKind{}

impl TypeKindTrait for ValueKind {
    const VALUE:TypeKind=TypeKind::Value;
    const IS_PREFIX:bool=false;
}

impl TypeKindTrait for PrefixKind {
    const VALUE:TypeKind=TypeKind::Prefix;
    const IS_PREFIX:bool=true;
}

///////////////////////////////////////////////////////////////////////////////

/// Gets for the AbiInfo of some type,wraps an `extern "C" fn() -> &'static AbiInfo`.
#[derive(Copy, Clone)]
#[repr(transparent)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct GetAbiInfo {
    abi_info: extern "C" fn() -> &'static AbiInfo,
}

impl fmt::Debug for GetAbiInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.get(), f)
    }
}

impl GetAbiInfo {
    /// Gets the `&'static AbiInfo` of some type.
    pub fn get(self) -> &'static AbiInfo {
        (self.abi_info)()
    }
}

/// Constructs the GetAbiInfo for Self.
///
/// # Safety
///
/// Implementors must make sure that the AbiInfo actually describes the layout of the type.
pub unsafe trait MakeGetAbiInfo<B> {
    const CONST: GetAbiInfo;
}

unsafe impl<T> MakeGetAbiInfo<StableAbi_Bound> for T
where
    T: StableAbi,
{
    const CONST: GetAbiInfo = GetAbiInfo {
        abi_info: get_abi_info::<T>,
    };
}

unsafe impl<T> MakeGetAbiInfo<UnsafeOpaqueField_Bound> for T {
    const CONST: GetAbiInfo = GetAbiInfo {
        abi_info: get_abi_info::<UnsafeOpaqueField<T>>,
    };
}

#[allow(non_camel_case_types)]
pub struct StableAbi_Bound;
#[allow(non_camel_case_types)]
pub struct UnsafeOpaqueField_Bound;

/// Retrieves the AbiInfo of `T`,
pub extern "C" fn get_abi_info<T>() -> &'static AbiInfo
where
    T: StableAbi,
{
    T::ABI_INFO.get()
}


///////////////////////////////////////////////////////////////////////////////

/////////////////////////////////////////////////////////////////////////////
////                Implementations
/////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////

unsafe impl<T> SharedStableAbi for PhantomData<T> {
    type Kind=ValueKind;
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_phantom::<Self>(
        "PhantomData",
        RNone,
        TLData::Primitive,
        tl_genparams!(;;),
        &[],
    );
}
unsafe impl SharedStableAbi for () {
    type Kind=ValueKind;
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout =
        &TypeLayout::from_std_lib::<Self>("()", TLData::Primitive, tl_genparams!(;;));
}

/////////////

// Does not allow ?Sized types because the DST fat pointer does not have a stable layout.
unsafe impl<'a, T> SharedStableAbi for &'a T
where
    T: 'a + SharedStableAbi,
{
    type Kind=ValueKind;
    type IsNonZeroType = True;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_phantom::<Self>(
        "&",
        RSome(RustPrimitive::Reference),
        TLData::Primitive,
        tl_genparams!('a;T;),
        &[TLField::new(
            "0",
            &[LifetimeIndex::Param(0)],
            <T as MakeGetAbiInfo<StableAbi_Bound>>::CONST,
        )],
    );
}

// Does not allow ?Sized types because the DST fat pointer does not have a stable layout.
unsafe impl<'a, T> SharedStableAbi for &'a mut T
where
    T: 'a + StableAbi,
{
    type Kind=ValueKind;
    type IsNonZeroType = True;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_phantom::<Self>(
        "&mut",
        RSome(RustPrimitive::MutReference),
        TLData::Primitive,
        tl_genparams!('a;T;),
        &[TLField::new(
            "0",
            &[LifetimeIndex::Param(0)],
            <T as MakeGetAbiInfo<StableAbi_Bound>>::CONST,
        )],
    );
}

// Does not allow ?Sized types because the DST fat pointer does not have a stable layout.
unsafe impl<T> SharedStableAbi for NonNull<T>
where
    T: StableAbi,
{
    type Kind=ValueKind;
    type IsNonZeroType = True;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_phantom::<Self>(
        "NonNull",
        RNone,
        TLData::Primitive,
        tl_genparams!(;T;),
        &[TLField::new(
            "0",
            &[],
            <T as MakeGetAbiInfo<StableAbi_Bound>>::CONST,
        )],
    );
}

unsafe impl<T> SharedStableAbi for AtomicPtr<T>
where
    T: StableAbi,
{
    type Kind=ValueKind;
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_phantom::<Self>(
        "AtomicPtr",
        RNone,
        TLData::Primitive,
        tl_genparams!(;T;),
        &[TLField::new(
            "0",
            &[],
            <T as MakeGetAbiInfo<StableAbi_Bound>>::CONST,
        )],
    );
}

// Does not allow ?Sized types because the DST fat pointer does not have a stable layout.
unsafe impl<T> SharedStableAbi for *const T
where
    T: StableAbi,
{
    type Kind=ValueKind;
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_phantom::<Self>(
        "*const",
        RSome(RustPrimitive::ConstPtr),
        TLData::Primitive,
        tl_genparams!(;T;),
        &[TLField::new(
            "0",
            &[],
            <T as MakeGetAbiInfo<StableAbi_Bound>>::CONST,
        )],
    );
}

// Does not allow ?Sized types because the DST fat pointer does not have a stable layout.
unsafe impl<T> SharedStableAbi for *mut T
where
    T: StableAbi,
{
    type Kind=ValueKind;
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_phantom::<Self>(
        "*mut",
        RSome(RustPrimitive::MutPtr),
        TLData::Primitive,
        tl_genparams!(;T;),
        &[TLField::new(
            "0",
            &[],
            <T as MakeGetAbiInfo<StableAbi_Bound>>::CONST,
        )],
    );
}

/////////////

macro_rules! impl_stable_abi_array {
    ($($size:expr),*)=>{
        $(
            unsafe impl<T> SharedStableAbi for [T;$size]
            where T:StableAbi
            {
                type Kind=ValueKind;
                type IsNonZeroType=False;

                const LAYOUT:&'static TypeLayout=&TypeLayout::from_std_lib_phantom::<Self>(
                    stringify!(concat!("[_;",stringify!($size),"]")),
                    RSome(RustPrimitive::Array),
                    TLData::Primitive,
                    tl_genparams!(;T;$size),
                    &[TLField::new("0", &[], <T as MakeGetAbiInfo<StableAbi_Bound>>::CONST)],
                );
            }
        )*
    }
}

impl_stable_abi_array! {
    00,01,02,03,04,05,06,07,08,09,
    10,11,12,13,14,15,16,17,18,19,
    20,21,22,23,24,25,26,27,28,29,
    30,31,32
}

/////////////

/// Implementing abi stability for Option<T> is fine if
/// T is a NonZero primitive type.
unsafe impl<T> SharedStableAbi for Option<T>
where
    T: StableAbi<IsNonZeroType = True>,
{
    type Kind=ValueKind;
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_phantom::<Self>(
        "Option",
        RNone,
        TLData::Primitive,
        tl_genparams!(;T;),
        &[TLField::new(
            "0",
            &[],
            <T as MakeGetAbiInfo<StableAbi_Bound>>::CONST,
        )],
    );
}

/////////////

macro_rules! impl_for_concrete {
    (
        zeroable=[$( $zeroable:ty ,)*]
        nonzero=[ $( $nonzero:ty ,)* ]
    ) => (
        $(
            unsafe impl SharedStableAbi for $zeroable {
                type Kind=ValueKind;
                type IsNonZeroType=False;

                const LAYOUT:&'static TypeLayout=&TypeLayout::from_std_lib::<Self>(
                    stringify!($zeroable),
                    TLData::Primitive,
                    tl_genparams!(;;),
                );
            }
        )*

        $(
            unsafe impl SharedStableAbi for $nonzero {
                type Kind=ValueKind;
                type IsNonZeroType=True;

                const LAYOUT:&'static TypeLayout=&TypeLayout::from_std_lib::<Self>(
                    stringify!($nonzero),
                    TLData::Primitive,
                    tl_genparams!(;;),
                );
            }
        )*
    )
}

impl_for_concrete! {
    zeroable=[
        u8,i8,
        u16,i16,
        u32,i32,
        u64,i64,
        usize,isize,
        bool,
        AtomicBool,
        AtomicIsize,
        AtomicUsize,
    ]

    nonzero=[
        num::NonZeroU8,
        num::NonZeroU16,
        num::NonZeroU32,
        num::NonZeroU64,
        num::NonZeroUsize,
    ]
}
/////////////


#[cfg(any(rust_1_34,feature="rust_1_34"))]
mod rust_1_34_impls{
    use super::*;
    use std::sync::atomic;
    use core::num as core_num;

    impl_for_concrete! {
        zeroable=[
            atomic::AtomicI16,
            atomic::AtomicI32,
            atomic::AtomicI64,
            atomic::AtomicI8,
            atomic::AtomicU16,
            atomic::AtomicU32,
            atomic::AtomicU64,
            atomic::AtomicU8,
        ]
        nonzero=[
            core_num::NonZeroI8,
            core_num::NonZeroI16,
            core_num::NonZeroI32,
            core_num::NonZeroI64,
            core_num::NonZeroIsize,
        ]
    }
}

/////////////

unsafe impl<N> SharedStableAbi for num::Wrapping<N>
where
    N: StableAbi,
{
    type Kind=ValueKind;
    type IsNonZeroType = N::IsNonZeroType;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib::<Self>(
        "num::Wrapping",
        TLData::ReprTransparent(N::ABI_INFO.get()),
        tl_genparams!(;N;),
    );
}

/////////////

unsafe impl<P> SharedStableAbi for Pin<P>
where
    P: StableAbi,
{
    type Kind=ValueKind;
    type IsNonZeroType = P::IsNonZeroType;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib::<Self>(
        "Pin",
        TLData::ReprTransparent(P::ABI_INFO.get()),
        tl_genparams!(;P;),
    );
}

/////////////

/// This is the only function type that implements StableAbi
/// so as to make it more obvious that functions involving lifetimes
/// cannot implement this trait directly (because of higher ranked trait bounds).
unsafe impl SharedStableAbi for extern "C" fn() {
    type Kind=ValueKind;
    type IsNonZeroType = True;

    const LAYOUT: &'static TypeLayout = EMPTY_EXTERN_FN_LAYOUT;
}

/// This is the only function type that implements StableAbi
/// so as to make it more obvious that functions involving lifetimes
/// cannot implement this trait directly (because of higher ranked trait bounds).
unsafe impl SharedStableAbi for unsafe extern "C" fn() {
    type Kind=ValueKind;
    type IsNonZeroType = True;

    const LAYOUT: &'static TypeLayout = EMPTY_EXTERN_FN_LAYOUT;
}

/// The layout of `extern fn()` and `unsafe extern fn()`
const EMPTY_EXTERN_FN_LAYOUT: &'static TypeLayout =
    &TypeLayout::from_params::<extern "C" fn()>(TypeLayoutParams {
        name: "AFunctionPointer",
        package: env!("CARGO_PKG_NAME"),
        package_version: crate::version::VersionStrings {
            major: StaticStr::new(env!("CARGO_PKG_VERSION_MAJOR")),
            minor: StaticStr::new(env!("CARGO_PKG_VERSION_MINOR")),
            patch: StaticStr::new(env!("CARGO_PKG_VERSION_PATCH")),
        },
        file:"<unavailable>",
        line:0,
        data: TLData::Struct {
            fields: StaticSlice::new(&[]),
        },
        generics: tl_genparams!(;;),
        phantom_fields: &[],
    });

/////////////

/// Allows one to create the TypeLayout/AbiInfoWrapper for any type `T`,
/// by pretending that it is a primitive type.
/// 
/// Used by the StableAbi derive macro by fields marker as `#[sabi(unsafe_opaque_field)]`.
/// 
/// # Safety
/// 
/// You must ensure that the layout of `T` is compatible through other means.
#[repr(transparent)]
pub struct UnsafeOpaqueField<T>(T);

unsafe impl<T> SharedStableAbi for UnsafeOpaqueField<T> {
    type Kind=ValueKind;
    type IsNonZeroType = False;
    const LAYOUT: &'static TypeLayout = &TypeLayout::from_params::<Self>(TypeLayoutParams {
        name: "OpaqueField",
        package: env!("CARGO_PKG_NAME"),
        package_version: crate::version::VersionStrings {
            major: StaticStr::new(env!("CARGO_PKG_VERSION_MAJOR")),
            minor: StaticStr::new(env!("CARGO_PKG_VERSION_MINOR")),
            patch: StaticStr::new(env!("CARGO_PKG_VERSION_PATCH")),
        },
        file:"<unavailable>",
        line:0,
        data: TLData::Primitive,
        generics: tl_genparams!(;;),
        phantom_fields: &[],
    });
}

/////////////
