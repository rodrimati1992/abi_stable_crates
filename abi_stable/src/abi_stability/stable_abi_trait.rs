use core_extensions::{
    type_level_bool::{Boolean, False, True},
    
};
use std::{
    fmt,
    marker::PhantomData,
    num,
    pin::Pin,
    ptr::NonNull,
    sync::atomic::{AtomicBool, AtomicIsize, AtomicPtr, AtomicUsize},
};

use crate::{
    RNone, RSome, StaticSlice, StaticStr,
};

use super::{
    LifetimeIndex, RustPrimitive, TLData,TLField, TypeLayout,
    TypeLayoutParams,
};


/// Like StableAbi,has a stable layout but must be accessed through a shared reference.
pub unsafe trait SharedStableAbi {
    /// Whether this type has a single (always the same one) invalid bit-pattern.
    ///
    /// Possible values:True/False
    type IsNonZeroType: Boolean;

    const LAYOUT: &'static TypeLayout;
    const ABI_INFO: &'static AbiInfoWrapper = {
        let info = AbiInfo {
            prefix_kind: true,
            is_nonzero: <Self::IsNonZeroType as Boolean>::VALUE,
            layout: Self::LAYOUT,
        };

        &AbiInfoWrapper::new(info)
    };
}

///////////////////////

/// Represents a type whose layout is stable.
///
/// # Safety
///
/// The layout of types implementing this trait must be stable across minor versions,
///
/// # Caveats
///
/// This trait cannot be currently implemented for functions that take lifetime parameters,
/// even if their parameters and return types implement StableAbi.
/// To mitigate this `#[derive(StableAbi)]` specially supports `extern fn` types 
/// (except through type aliases).
pub unsafe trait StableAbi {
    /// Whether this type has 0 as an invalid bit-pattern.
    ///
    /// Possible values:True/False
    type IsNonZeroType: Boolean;

    const LAYOUT: &'static TypeLayout;

    const ABI_INFO: &'static AbiInfoWrapper = {
        let info = AbiInfo {
            prefix_kind: false,
            is_nonzero: <Self::IsNonZeroType as Boolean>::VALUE,
            layout: Self::LAYOUT,
        };

        &AbiInfoWrapper::new(info)
    };
}

///////////////////////

unsafe impl<This> SharedStableAbi for This
where
    This: StableAbi,
{
    type IsNonZeroType = This::IsNonZeroType;
    const LAYOUT: &'static TypeLayout = This::LAYOUT;
    const ABI_INFO: &'static AbiInfoWrapper = This::ABI_INFO;
}

///////////////////////


/// Wrapper type for AbiInfo that requires the 
/// correct construction of AbiInfo to construct it.
#[derive(Debug, Copy, Clone,PartialEq)]
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
    ///
    pub const unsafe fn new_unchecked(inner: AbiInfo) -> Self {
        Self::new(inner)
    }
    pub const fn get(&self) -> &AbiInfo {
        &self.inner
    }
}


/// Describes the abi of some type.
/// 
/// # Safety
/// 
/// If this is manually constructed,you must ensure that it describes the actual abi of the type.
#[derive(Debug, Copy, Clone,PartialEq)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct AbiInfo {
    pub prefix_kind: bool,
    pub is_nonzero: bool,
    pub layout: &'static TypeLayout,
}

///////////////////////////////////////////////////////////////////////////////

/// Getter for the AbiInfo of some type.
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
    pub fn get(self) -> &'static AbiInfo {
        (self.abi_info)()
    }
}

/// Constructs the GetAbiInfo for Self.
///
/// # Safety
///
/// Implementors must make sure that the AbiInfo actually describes the layout of the type.
pub unsafe trait MakeGetAbiInfo<B>{
    const CONST:GetAbiInfo;
}

unsafe impl<T> MakeGetAbiInfo<StableAbi_Bound> for T
where T: StableAbi,
{
    const CONST: GetAbiInfo = GetAbiInfo {
        abi_info: get_abi_info::<T>
    };
}

unsafe impl<T> MakeGetAbiInfo<SharedStableAbi_Bound> for T
where
    T: SharedStableAbi,
{
    const CONST: GetAbiInfo = GetAbiInfo {
        abi_info: get_shared_abi_info::<T>,
    };
}

unsafe impl<T> MakeGetAbiInfo<UnsafeOpaqueField_Bound> for T{
    const CONST: GetAbiInfo = GetAbiInfo {
        abi_info: get_abi_info::<UnsafeOpaqueField<T>>,
    };
}

#[allow(non_camel_case_types)]
pub struct StableAbi_Bound;
#[allow(non_camel_case_types)]
pub struct SharedStableAbi_Bound;
#[allow(non_camel_case_types)]
pub struct UnsafeOpaqueField_Bound;




/// Retrieves the AbiInfo of `T`,
pub extern "C" fn get_abi_info<T>() -> &'static AbiInfo
where
    T: StableAbi,
{
    T::ABI_INFO.get()
}

pub extern "C" fn get_shared_abi_info<T>() -> &'static AbiInfo
where
    T: SharedStableAbi,
{
    T::ABI_INFO.get()
}

///////////////////////////////////////////////////////////////////////////////

/////////////////////////////////////////////////////////////////////////////
////                Implementations
/////////////////////////////////////////////////////////////////////////////


///////////////////////////////////////////////////////////////////////////////

// By default PhantomData has to reference an StableAbi type.
unsafe impl<T> StableAbi for PhantomData<T> {
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_phantom::<Self>(
        "PhantomData",
        RNone,
        TLData::Primitive,
        tl_genparams!(;;),
        &[],
    );
}
unsafe impl StableAbi for () {
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout =
        &TypeLayout::from_std_lib::<Self>("()", TLData::Primitive, tl_genparams!(;;));
}

/////////////

// Does not allow ?Sized types because the DST fat pointer does not have a fixed layout.
unsafe impl<'a, T> StableAbi for &'a T
where
    T: 'a + SharedStableAbi,
{
    type IsNonZeroType = True;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_phantom::<Self>(
        "&",
        RSome(RustPrimitive::Reference),
        TLData::Primitive,
        tl_genparams!('a;T;),
        &[TLField::new(
            "0",
            &[LifetimeIndex::Param(0)],
            <T as MakeGetAbiInfo<SharedStableAbi_Bound>>::CONST,
        )],
    );
}

unsafe impl<'a, T> StableAbi for &'a mut T
where
    T: 'a + StableAbi,
{
    type IsNonZeroType = True;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_phantom::<Self>(
        "&mut",
        RSome(RustPrimitive::MutReference),
        TLData::Primitive,
        tl_genparams!('a;T;),
        &[TLField::new(
            "0",
            &[LifetimeIndex::Param(0)],
            <T as MakeGetAbiInfo<SharedStableAbi_Bound>>::CONST,
        )],
    );
}

unsafe impl<T> StableAbi for NonNull<T>
where
    T: StableAbi,
{
    type IsNonZeroType = True;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_phantom::<Self>(
        "NonNull",
        RNone,
        TLData::Primitive,
        tl_genparams!(;T;),
        &[TLField::new("0", &[], <T as MakeGetAbiInfo<SharedStableAbi_Bound>>::CONST)],
    );
}

unsafe impl<T> StableAbi for AtomicPtr<T>
where
    T: StableAbi,
{
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_phantom::<Self>(
        "AtomicPtr",
        RNone,
        TLData::Primitive,
        tl_genparams!(;T;),
        &[TLField::new("0", &[], <T as MakeGetAbiInfo<SharedStableAbi_Bound>>::CONST)],
    );
}

unsafe impl<T> StableAbi for *const T
where
    T: SharedStableAbi,
{
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_phantom::<Self>(
        "*const",
        RSome(RustPrimitive::ConstPtr),
        TLData::Primitive,
        tl_genparams!(;T;),
        &[TLField::new("0", &[], <T as MakeGetAbiInfo<SharedStableAbi_Bound>>::CONST)],
    );
}

unsafe impl<T> StableAbi for *mut T
where
    T: StableAbi,
{
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_phantom::<Self>(
        "*mut",
        RSome(RustPrimitive::MutPtr),
        TLData::Primitive,
        tl_genparams!(;T;),
        &[TLField::new("0", &[], <T as MakeGetAbiInfo<SharedStableAbi_Bound>>::CONST)],
    );
}

/////////////

macro_rules! impl_stable_abi_array {
    ($($size:expr),*)=>{
        $(
            unsafe impl<T> StableAbi for [T;$size]
            where T:StableAbi
            {
                type IsNonZeroType=False;

                const LAYOUT:&'static TypeLayout=&TypeLayout::from_std_lib_phantom::<Self>(
                    stringify!(concat!("[_;",stringify!($size),"]")),
                    RSome(RustPrimitive::Array),
                    TLData::Primitive,
                    tl_genparams!(;T;$size),
                    &[TLField::new("0", &[], <T as MakeGetAbiInfo<SharedStableAbi_Bound>>::CONST)],
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
unsafe impl<T> StableAbi for Option<T>
where
    T: StableAbi<IsNonZeroType = True>,
{
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib_phantom::<Self>(
        "Option",
        RNone,
        TLData::Primitive,
        tl_genparams!(;T;),
        &[TLField::new("0", &[], <T as MakeGetAbiInfo<StableAbi_Bound>>::CONST)],
    );
}

/////////////

macro_rules! impl_for_concrete {
    (
        zeroable=[$( $zeroable:ty ,)*]
        nonzero=[ $( $nonzero:ty ,)* ]
    ) => (
        $(
            unsafe impl StableAbi for $zeroable {
                type IsNonZeroType=False;

                const LAYOUT:&'static TypeLayout=&TypeLayout::from_std_lib::<Self>(
                    stringify!($zeroable),
                    TLData::Primitive,
                    tl_genparams!(;;),
                );
            }
        )*

        $(
            unsafe impl StableAbi for $nonzero {
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
        u128,i128,
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
        num::NonZeroU128,
        num::NonZeroUsize,
    ]
}

/////////////

unsafe impl<N> StableAbi for num::Wrapping<N>
where
    N: StableAbi,
{
    type IsNonZeroType = N::IsNonZeroType;

    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib::<Self>(
        "num::Wrapping",
        TLData::ReprTransparent(N::ABI_INFO.get()),
        tl_genparams!(;N;),
    );
}

/////////////

unsafe impl<P> StableAbi for Pin<P>
where
    P: StableAbi,
{
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
unsafe impl StableAbi for extern "C" fn() {
    type IsNonZeroType = True;

    const LAYOUT: &'static TypeLayout = EMPTY_EXTERN_FN_LAYOUT;
}

/// This is the only function type that implements StableAbi
/// so as to make it more obvious that functions involving lifetimes
/// cannot implement this trait directly (because of higher ranked trait bounds).
unsafe impl StableAbi for unsafe extern "C" fn() {
    type IsNonZeroType = True;

    const LAYOUT: &'static TypeLayout = EMPTY_EXTERN_FN_LAYOUT;
}


const EMPTY_EXTERN_FN_LAYOUT: &'static TypeLayout = 
    &TypeLayout::from_params::<extern fn()>(TypeLayoutParams {
        name: "AFunctionPointer",
        package: env!("CARGO_PKG_NAME"),
        package_version: crate::version::VersionStrings {
            major: StaticStr::new(env!("CARGO_PKG_VERSION_MAJOR")),
            minor: StaticStr::new(env!("CARGO_PKG_VERSION_MINOR")),
            patch: StaticStr::new(env!("CARGO_PKG_VERSION_PATCH")),
        },
        data: TLData::Struct {
            fields: StaticSlice::new( &[] ),
        },
        generics: tl_genparams!(;;),
        phantom_fields: &[],
    });



/////////////

/// An unsafe type,which allows treating a field as though it were a primitive type.
#[repr(transparent)]
struct UnsafeOpaqueField<T>(T);

unsafe impl<T> StableAbi for UnsafeOpaqueField<T> {
    type IsNonZeroType = False;
    const LAYOUT: &'static TypeLayout = &TypeLayout::from_params::<Self>(TypeLayoutParams {
        name: "OpaqueField",
        package: env!("CARGO_PKG_NAME"),
        package_version: crate::version::VersionStrings {
            major: StaticStr::new(env!("CARGO_PKG_VERSION_MAJOR")),
            minor: StaticStr::new(env!("CARGO_PKG_VERSION_MINOR")),
            patch: StaticStr::new(env!("CARGO_PKG_VERSION_PATCH")),
        },
        data: TLData::Primitive,
        generics: tl_genparams!(;;),
        phantom_fields: &[],
    });
}

/////////////
