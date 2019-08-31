/*!
Where the StableAbi trait is declared,as well as related types/traits.
*/

use core_extensions::type_level_bool::{Boolean, False, True};
use std::{
    cell::{Cell,UnsafeCell},
    marker::{PhantomData,PhantomPinned},
    mem::ManuallyDrop,
    num::{NonZeroU8,NonZeroU16,NonZeroU32,NonZeroU64,NonZeroUsize,Wrapping},
    pin::Pin,
    ptr::NonNull,
    sync::atomic::{AtomicBool, AtomicIsize, AtomicPtr, AtomicUsize},
};

use crate::{
    abi_stability::get_static_equivalent::GetStaticEquivalent_,
    sabi_types::Constructor,
    std_types::{RNone, RSome, utypeid::UTypeId},
    reflection::ModReflMode,
    type_layout::{
        LifetimeIndex, TLData, TLField, TypeLayout, TypeLayoutParams,
        ItemInfo,ReprAttr,TLPrimitive,TLEnum,TLDiscriminants,IsExhaustive,
    },
};


///////////////////////

/**
Represents a type whose layout is stable.

This trait can indirectly be derived using `#[derive(StableAbi)]`
(There is a blanket impl of StableAbi for `SharedStableAbi<Kind=ValueKind>`,
    which `#[derive(StableAbi)]` implements.
).

There is a blanket impl of this trait for all `SharedStableAbi<Kind=ValueKind>` types.
*/
pub unsafe trait StableAbi:SharedStableAbi<Kind=ValueKind> {
    /// The layout of the type provided by implementors.
    const LAYOUT: &'static TypeLayout;

    const ABI_CONSTS: AbiConsts;
}


/**
Represents a type whose layout is stable.

This trait can be derived using ``.

# Safety

The layout specified in `S_LAYOUT` must be correct,
otherwise type checking when loading a dynamic library would be unsound,
and passing this into a dynamic library would be equivalent to transmuting it.

# Caveats

This trait cannot be directly implemented for functions that take lifetime parameters,
because of that,`#[derive(StableAbi)]` detects the presence of `extern fn` types 
in type definitions.

*/
pub unsafe trait SharedStableAbi:GetStaticEquivalent_ {
    /**
Whether this type has a single invalid bit-pattern.

Possible values:True/False

Some standard library types have a single value that is invalid for them eg:0,null.
these types are the only ones which can be stored in a `Option<_>` that implements AbiStable.

An alternative for types where `IsNonZeroType=False`,you can use `abi_stable::ROption`.

Non-exhaustive list of std types that are NonZero:

- &T (any T).

- &mut T (any T).

- extern fn().

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
    const S_LAYOUT: &'static TypeLayout;

    const S_ABI_CONSTS: AbiConsts=AbiConsts {
        kind:<Self::Kind as TypeKindTrait>::VALUE,
        type_id:Constructor(
            crate::std_types::utypeid::new_utypeid::<Self::StaticEquivalent> 
        ),
        is_nonzero: <Self::IsNonZeroType as Boolean>::VALUE,
    };
}


unsafe impl<This> StableAbi for This
where 
    This:SharedStableAbi<Kind=ValueKind>,
{
    const LAYOUT: &'static TypeLayout=<This as SharedStableAbi>::S_LAYOUT;

    const ABI_CONSTS: AbiConsts=<This as SharedStableAbi>::S_ABI_CONSTS;
}


///////////////////////

/// Contains constants equivalent to the associated types in SharedStableAbi.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
#[derive(StableAbi)]
pub struct AbiConsts {
    /// 
    pub kind:TypeKind,
    
    /// Equivalent to the UTypeId returned by the function in Constructor.
    pub type_id:Constructor<UTypeId>,
    
    /// Whether the type uses non-zero value optimization,
    /// if true then an Option<Self> implements StableAbi.
    pub is_nonzero: bool,
}

impl AbiConsts{
    #[inline]
    pub fn get_type_id(&self)->UTypeId{
        self.type_id.get()
    }
}

///////////////////////////////////////////////////////////////////////////////

/// The abi_stable kind of a type.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq,Eq,Ord,PartialOrd,Hash,StableAbi)]
pub enum TypeKind{
    /// A value whose layout must not change in minor versions
    Value,
    /// A struct whose fields can be extended in minor versions,
    /// but only behind a shared reference,
    /// used to implement vtables and modules.
    Prefix,
}


/// For marker types that represent variants of TypeKind.
pub trait TypeKindTrait:sealed::Sealed{
    /// The equivalent TypeKind of this type.
    const VALUE:TypeKind;
    /// Whether this is a prefix-kind
    const IS_PREFIX:bool;
}

/// The kind of a regular value,the default kind.
pub struct ValueKind;

/// The kind of a prefix-type,vtables and modules.
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

/// Gets for the TypeLayout of some type,wraps an `extern "C" fn() -> &'static TypeLayout`.
pub type GetTypeLayout=Constructor<&'static TypeLayout>;

// pub unsafe trait GetTypeLayoutCtor<B> {


#[doc(hidden)]
pub struct GetTypeLayoutCtor<T>(T);

impl<T> GetTypeLayoutCtor<T>
where T: SharedStableAbi,
{
    pub const SHARED_STABLE_ABI:GetTypeLayout=Constructor (
        get_ssa_type_layout::<T>,
    );
}

impl<T> GetTypeLayoutCtor<T>
where T: StableAbi,
{
    pub const STABLE_ABI:GetTypeLayout=Constructor (
        get_type_layout::<T>,
    );

    pub const SABI_OPAQUE_FIELD:GetTypeLayout=Constructor (
        get_type_layout::<SabiUnsafeOpaqueField<T>>,
    );
}

impl<T> GetTypeLayoutCtor<T>{
    pub const OPAQUE_FIELD:GetTypeLayout=Constructor (
        get_type_layout::<UnsafeOpaqueField<T>>,
    );
}

/// Retrieves the TypeLayout of `T:StableAbi`,
pub extern "C" fn get_type_layout<T>() -> &'static TypeLayout
where
    T: StableAbi,
{
    T::LAYOUT
}

/// Retrieves the TypeLayout of `T:SharedStableAbi`,
pub extern "C" fn get_ssa_type_layout<T>() -> &'static TypeLayout
where
    T: SharedStableAbi,
{
    T::S_LAYOUT
}


///////////////////////////////////////////////////////////////////////////////

/////////////////////////////////////////////////////////////////////////////
////                Implementations
/////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////

unsafe impl<T> GetStaticEquivalent_ for PhantomData<T> 
where T:GetStaticEquivalent_
{
    type StaticEquivalent=PhantomData<T::StaticEquivalent>;
}

unsafe impl<T> SharedStableAbi for PhantomData<T> 
where T:StableAbi
{
    type Kind=ValueKind;
    type IsNonZeroType = False;

    const S_LAYOUT: &'static TypeLayout = &TypeLayout::from_std_full::<Self>(
        Self::S_ABI_CONSTS,
        "PhantomData",
        RNone,
        ItemInfo::std_type_in("std::marker"),
        TLData::EMPTY,
        ReprAttr::c(),
        tl_genparams!(;;),
        rslice![TLField::new("0",rslice![],GetTypeLayoutCtor::<T>::STABLE_ABI,)],
    );
}


unsafe impl GetStaticEquivalent_ for () {
    type StaticEquivalent=();
}
unsafe impl SharedStableAbi for () {
    type Kind=ValueKind;
    type IsNonZeroType = False;

    const S_LAYOUT: &'static TypeLayout =
        &TypeLayout::from_std::<Self>(
            Self::S_ABI_CONSTS,
            "()", 
            TLData::EMPTY,
            ReprAttr::c(),
            ItemInfo::primitive(), 
            tl_genparams!(;;)
        );
}


/////////////


unsafe impl<'a, T> GetStaticEquivalent_ for &'a T
where
    T: 'a + GetStaticEquivalent_,
{
    type StaticEquivalent=&'static T::StaticEquivalent;
}

// Does not allow ?Sized types because the DST fat pointer does not have a stable layout.
unsafe impl<'a, T> SharedStableAbi for &'a T
where
    T: 'a + SharedStableAbi,
{
    type Kind=ValueKind;
    type IsNonZeroType = True;

    const S_LAYOUT: &'static TypeLayout = &TypeLayout::from_std_full::<Self>(
        Self::S_ABI_CONSTS,
        "&",
        RSome(TLPrimitive::SharedRef),
        ItemInfo::primitive(),
        TLData::Primitive(TLPrimitive::SharedRef),
        ReprAttr::Primitive,
        tl_genparams!('a;T;),
        rslice![TLField::new(
            "0",
            rslice![LifetimeIndex::Param(0)],
            GetTypeLayoutCtor::<T>::SHARED_STABLE_ABI,
        )],
    ).set_mod_refl_mode(ModReflMode::DelegateDeref{phantom_field_index:0});
}


unsafe impl<'a, T> GetStaticEquivalent_ for &'a mut T
where
    T: 'a + GetStaticEquivalent_,
{
    type StaticEquivalent=&'static mut T::StaticEquivalent;
}

// Does not allow ?Sized types because the DST fat pointer does not have a stable layout.
unsafe impl<'a, T> SharedStableAbi for &'a mut T
where
    T: 'a + StableAbi,
{
    type Kind=ValueKind;
    type IsNonZeroType = True;

    const S_LAYOUT: &'static TypeLayout = &TypeLayout::from_std_full::<Self>(
        Self::S_ABI_CONSTS,
        "&mut",
        RSome(TLPrimitive::MutRef),
        ItemInfo::primitive(),
        TLData::Primitive(TLPrimitive::MutRef),
        ReprAttr::Primitive,
        tl_genparams!('a;T;),
        rslice![TLField::new(
            "0",
            rslice![LifetimeIndex::Param(0)],
            GetTypeLayoutCtor::<T>::STABLE_ABI,
        )],
    );
}


unsafe impl<T> GetStaticEquivalent_ for NonNull<T>
where
    T: GetStaticEquivalent_,
{
    type StaticEquivalent=NonNull<T::StaticEquivalent>;
}

// Does not allow ?Sized types because the DST fat pointer does not have a stable layout.
unsafe impl<T> SharedStableAbi for NonNull<T>
where
    T: StableAbi,
{
    type Kind=ValueKind;
    type IsNonZeroType = True;

    const S_LAYOUT: &'static TypeLayout = &TypeLayout::from_std_full::<Self>(
        Self::S_ABI_CONSTS,
        "NonNull",
        RNone,
        ItemInfo::std_type_in("std::ptr"),
        TLData::struct_(rslice![
            TLField::new(
                "0",
                rslice![],
                GetTypeLayoutCtor::<*mut T>::STABLE_ABI,
            )
        ]),
        ReprAttr::Transparent,
        tl_genparams!(;T;),
        rslice![],
    );
}


unsafe impl<T> GetStaticEquivalent_ for AtomicPtr<T>
where
    T: GetStaticEquivalent_,
{
    type StaticEquivalent=AtomicPtr<T::StaticEquivalent>;
}

unsafe impl<T> SharedStableAbi for AtomicPtr<T>
where
    T: StableAbi,
{
    type Kind=ValueKind;
    type IsNonZeroType = False;

    const S_LAYOUT: &'static TypeLayout = &TypeLayout::from_std_full::<Self>(
        Self::S_ABI_CONSTS,
        "AtomicPtr",
        RNone,
        ItemInfo::std_type_in("std::sync::atomic"),
        TLData::struct_(rslice![
            TLField::new(
                "0",
                rslice![],
                GetTypeLayoutCtor::<*mut T>::STABLE_ABI,
            )
        ]),
        ReprAttr::Transparent,
        tl_genparams!(;T;),
        rslice![],
    );
}

unsafe impl<T> GetStaticEquivalent_ for *const T
where
    T: GetStaticEquivalent_,
{
    type StaticEquivalent=*const T::StaticEquivalent;
}
// Does not allow ?Sized types because the DST fat pointer does not have a stable layout.
unsafe impl<T> SharedStableAbi for *const T
where
    T: SharedStableAbi,
{
    type Kind=ValueKind;
    type IsNonZeroType = False;

    const S_LAYOUT: &'static TypeLayout = &TypeLayout::from_std_full::<Self>(
        Self::S_ABI_CONSTS,
        "*const",
        RSome(TLPrimitive::ConstPtr),
        ItemInfo::primitive(),
        TLData::Primitive(TLPrimitive::ConstPtr),
        ReprAttr::Primitive,
        tl_genparams!(;T;),
        rslice![TLField::new(
            "0",
            rslice![],
            GetTypeLayoutCtor::<T>::SHARED_STABLE_ABI,
        )],
    );
}


unsafe impl<T> GetStaticEquivalent_ for *mut T
where
    T: GetStaticEquivalent_,
{
    type StaticEquivalent=*mut T::StaticEquivalent;
}
// Does not allow ?Sized types because the DST fat pointer does not have a stable layout.
unsafe impl<T> SharedStableAbi for *mut T
where
    T: StableAbi,
{
    type Kind=ValueKind;
    type IsNonZeroType = False;

    const S_LAYOUT: &'static TypeLayout = &TypeLayout::from_std_full::<Self>(
        Self::S_ABI_CONSTS,
        "*mut",
        RSome(TLPrimitive::MutPtr),
        ItemInfo::primitive(),
        TLData::Primitive(TLPrimitive::MutPtr),
        ReprAttr::Primitive,
        tl_genparams!(;T;),
        rslice![TLField::new(
            "0",
            rslice![],
            GetTypeLayoutCtor::<T>::STABLE_ABI,
        )],
    );
}

/////////////

macro_rules! impl_stable_abi_array {
    ($($size:expr),*)=>{
        $(
            unsafe impl<T> GetStaticEquivalent_ for [T;$size]
            where T:GetStaticEquivalent_
            {
                type StaticEquivalent=[T::StaticEquivalent;$size];
            }

            unsafe impl<T> SharedStableAbi for [T;$size]
            where T:StableAbi
            {
                type Kind=ValueKind;
                type IsNonZeroType=False;

                const S_LAYOUT:&'static TypeLayout=&TypeLayout::from_std_full::<Self>(
                    Self::S_ABI_CONSTS,
                    "array",
                    RSome(TLPrimitive::Array{len:$size}),
                    ItemInfo::primitive(),
                    TLData::Primitive(TLPrimitive::Array{len:$size}),
                    ReprAttr::Primitive,
                    tl_genparams!(;T;$size),
                    rslice![
                        TLField::new(
                            "element", 
                            rslice![],
                            GetTypeLayoutCtor::<T>::SHARED_STABLE_ABI
                        )
                    ],
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

unsafe impl<T> GetStaticEquivalent_ for Option<T>
where
    T: GetStaticEquivalent_,
{
    type StaticEquivalent=Option<T::StaticEquivalent>;
}
/// Implementing abi stability for Option<T> is fine if
/// T is a NonZero primitive type.
unsafe impl<T> SharedStableAbi for Option<T>
where
    T: StableAbi<IsNonZeroType = True>,
{
    type Kind=ValueKind;
    type IsNonZeroType = False;

    const S_LAYOUT: &'static TypeLayout = &TypeLayout::from_std_full::<Self>(
        Self::S_ABI_CONSTS,
        "Option",
        RNone,
        ItemInfo::primitive(),
        TLData::Enum(&TLEnum::new(
            "Some;None;",
            IsExhaustive::exhaustive(),
            rslice![
                TLField::new(
                    "0",
                    rslice![],
                    GetTypeLayoutCtor::<T>::STABLE_ABI,
                )
            ],
            TLDiscriminants::U8(rslice![0,1]),
            rslice![1,0]
        )),
        ReprAttr::OptionNonZero,
        tl_genparams!(;T;),
        rslice![],
    );
}

/////////////


macro_rules! impl_for_primitive_ints {
    (
        $( ($zeroable:ty,$tl_primitive:expr) ,)*
    ) => (
        $(
            unsafe impl GetStaticEquivalent_ for $zeroable {
                type StaticEquivalent=Self;
            }
            unsafe impl SharedStableAbi for $zeroable {
                type Kind=ValueKind;
                type IsNonZeroType=False;

                const S_LAYOUT:&'static TypeLayout=&TypeLayout::from_std_full::<Self>(
                    Self::S_ABI_CONSTS,
                    stringify!($zeroable),
                    RSome($tl_primitive),
                    ItemInfo::primitive(),
                    TLData::Primitive($tl_primitive),
                    ReprAttr::Primitive,
                    tl_genparams!(;;),
                    rslice![],
                );
            }
        )*
    )
}

impl_for_primitive_ints!{
    (u8   ,TLPrimitive::U8),
    (i8   ,TLPrimitive::I8),
    (u16  ,TLPrimitive::U16),
    (i16  ,TLPrimitive::I16),
    (u32  ,TLPrimitive::U32),
    (i32  ,TLPrimitive::I32),
    (u64  ,TLPrimitive::U64),
    (i64  ,TLPrimitive::I64),
    (usize,TLPrimitive::Usize),
    (isize,TLPrimitive::Isize),
    (bool ,TLPrimitive::Bool),
}


macro_rules! impl_for_concrete {
    (
        type IsNonZeroType=$zeroness:ty;
        [
            $( ($this:ty,$prim_repr:ty,$in_mod:expr) ,)*
        ]
    ) => (
        $(
            unsafe impl GetStaticEquivalent_ for $this {
                type StaticEquivalent=Self;
            }
            unsafe impl SharedStableAbi for $this {
                type Kind=ValueKind;
                type IsNonZeroType=$zeroness;

                const S_LAYOUT:&'static TypeLayout=&TypeLayout::from_std::<Self>(
                    Self::S_ABI_CONSTS,
                    stringify!($this),
                    TLData::struct_(rslice![
                        TLField::new(
                            "0",
                            rslice![],
                            GetTypeLayoutCtor::<$prim_repr>::STABLE_ABI,
                        )
                    ]),
                    ReprAttr::Transparent,
                    ItemInfo::std_type_in($in_mod),
                    tl_genparams!(;;),
                );
            }
        )*
    )
}



impl_for_concrete! {
    type IsNonZeroType=False;
    [
        (AtomicBool ,bool,"std::sync::atomic"),
        (AtomicIsize,isize,"std::sync::atomic"),
        (AtomicUsize,usize,"std::sync::atomic"),
    ]
}

impl_for_concrete! {
    type IsNonZeroType=True;
    [
        (NonZeroU8   ,u8,"std::num"),
        (NonZeroU16  ,u16,"std::num"),
        (NonZeroU32  ,u32,"std::num"),
        (NonZeroU64  ,u64,"std::num"),
        (NonZeroUsize,usize,"std::num"),
    ]
}
/////////////


mod rust_1_34_impls{
    use super::*;
    use std::sync::atomic::*;
    use core::num::*;

    impl_for_concrete! {
        type IsNonZeroType=False;
        [
            (AtomicI8 ,i8,"std::sync::atomic"),
            (AtomicI16,i16,"std::sync::atomic"),
            (AtomicI32,i32,"std::sync::atomic"),
            (AtomicI64,i64,"std::sync::atomic"),
            (AtomicU8 ,u8,"std::sync::atomic"),
            (AtomicU16,u16,"std::sync::atomic"),
            (AtomicU32,u32,"std::sync::atomic"),
            (AtomicU64,u64,"std::sync::atomic"),
        ]
    }

    impl_for_concrete! {
        type IsNonZeroType=True;
        [
            (NonZeroI8   ,i8,"core::num"),
            (NonZeroI16  ,i16,"core::num"),
            (NonZeroI32  ,i32,"core::num"),
            (NonZeroI64  ,i64,"core::num"),
            (NonZeroIsize,isize,"core::num"),
        ]
    }
}


#[cfg(any(rust_1_36,feature="rust_1_36"))]
mod rust_1_36_impls{
    use super::*;
    use std::mem::MaybeUninit;

    unsafe impl<T> GetStaticEquivalent_ for MaybeUninit<T>
    where
        T:GetStaticEquivalent_
    {
        type StaticEquivalent=MaybeUninit<T::StaticEquivalent>;
    }
    unsafe impl<T> SharedStableAbi for MaybeUninit<T>
    where
        T:StableAbi
    {
        type Kind=ValueKind;

        // MaybeUninit blocks layout optimizations.
        type IsNonZeroType = False;

        const S_LAYOUT: &'static TypeLayout = &TypeLayout::from_std::<Self>(
            Self::S_ABI_CONSTS,
            stringify!($this),
            TLData::struct_(rslice![
                TLField::new(
                    "value",
                    rslice![],
                    GetTypeLayoutCtor::<T>::STABLE_ABI,
                )
            ]),
            // Using `ReprAttr::Transparent` so that if I add C header file translation
            // it will be translated to just `T`.
            ReprAttr::Transparent,
            ItemInfo::std_type_in("std::mem"),
            tl_genparams!(;;),
        );
    }
}



/////////////

macro_rules! impl_stableabi_for_repr_transparent {
    (
        $type_constr:ident
        $(where[ $($where_clause:tt)* ])* ,
        $item_info:expr
    ) => (
        unsafe impl<P> GetStaticEquivalent_ for $type_constr<P>
        where
            P: GetStaticEquivalent_,
            $($($where_clause)*)*
        {
            type StaticEquivalent=$type_constr<P::StaticEquivalent>;
        }
        unsafe impl<P> SharedStableAbi for $type_constr<P>
        where
            P: StableAbi,
            $($($where_clause)*)*
        {
            type Kind=ValueKind;
            type IsNonZeroType = P::IsNonZeroType;

            const S_LAYOUT: &'static TypeLayout = &TypeLayout::from_std::<Self>(
                Self::S_ABI_CONSTS,
                stringify!($type_constr),
                TLData::struct_(rslice![
                    TLField::new("0",rslice![],GetTypeLayoutCtor::<P>::STABLE_ABI,)
                ]),
                ReprAttr::Transparent,
                $item_info,
                tl_genparams!(;P;),
            );
        }
    )
}


impl_stableabi_for_repr_transparent!{ Wrapping ,ItemInfo::std_type_in("std::num") }
impl_stableabi_for_repr_transparent!{ Pin         ,ItemInfo::std_type_in("std::pin") }
impl_stableabi_for_repr_transparent!{ ManuallyDrop,ItemInfo::std_type_in("std::mem") }
impl_stableabi_for_repr_transparent!{ Cell        ,ItemInfo::std_type_in("std::cell") }
impl_stableabi_for_repr_transparent!{ UnsafeCell  ,ItemInfo::std_type_in("std::cell") }

/////////////

macro_rules! impl_stableabi_for_unit_struct {
    (
        $type_constr:ident,
        $item_info:expr
    ) => (
        unsafe impl GetStaticEquivalent_ for $type_constr{
            type StaticEquivalent=$type_constr;
        }
        unsafe impl SharedStableAbi for $type_constr{
            type Kind=ValueKind;
            type IsNonZeroType = False;

            const S_LAYOUT: &'static TypeLayout = &TypeLayout::from_std::<Self>(
                Self::S_ABI_CONSTS,
                stringify!($type_constr),
                TLData::EMPTY,
                ReprAttr::c(),
                $item_info,
                tl_genparams!(;;),
            );
        }
    )
}


impl_stableabi_for_unit_struct!{ PhantomPinned,ItemInfo::std_type_in("std::marker") }

/////////////


unsafe impl GetStaticEquivalent_ for core_extensions::Void {
    type StaticEquivalent=Self;
}
unsafe impl SharedStableAbi for core_extensions::Void {
    type Kind=ValueKind;
    type IsNonZeroType = False;


    const S_LAYOUT: &'static TypeLayout =
        &TypeLayout::from_params::<Self>(TypeLayoutParams {
            abi_consts:Self::S_ABI_CONSTS,
            name: "Void",
            item_info:ItemInfo::package_and_mod("core_extensions;0.0.0","core_extensions"),
            data: TLData::Enum(&TLEnum::new(
                "",
                IsExhaustive::exhaustive(),
                rslice![],
                TLDiscriminants::U8(rslice![]),
                rslice![]
            )),
            generics: tl_genparams!(;;),
        });
}



/////////////

/// The layout of `extern fn()` and `unsafe extern fn()`
macro_rules! empty_extern_fn_layout{
    ($this:ty) => (
        &TypeLayout::from_params::<extern "C" fn()>(TypeLayoutParams {
            abi_consts:Self::S_ABI_CONSTS,
            name: "AFunctionPointer",
            item_info:make_item_info!(),
            data: TLData::struct_(rslice![]),
            generics: tl_genparams!(;;),
        })
    )
}


/// This is the only function type that implements StableAbi
/// so as to make it more obvious that functions involving lifetimes
/// cannot implement this trait directly (because of higher ranked trait bounds).
unsafe impl GetStaticEquivalent_ for extern "C" fn() {
    type StaticEquivalent=Self;
}
unsafe impl SharedStableAbi for extern "C" fn() {
    type Kind=ValueKind;
    type IsNonZeroType = True;

    const S_LAYOUT: &'static TypeLayout = empty_extern_fn_layout!(Self);
}

/// This is the only function type that implements StableAbi
/// so as to make it more obvious that functions involving lifetimes
/// cannot implement this trait directly (because of higher ranked trait bounds).
unsafe impl GetStaticEquivalent_ for unsafe extern "C" fn() {
    type StaticEquivalent=Self;
}
unsafe impl SharedStableAbi for unsafe extern "C" fn() {
    type Kind=ValueKind;
    type IsNonZeroType = True;

    const S_LAYOUT: &'static TypeLayout = empty_extern_fn_layout!(Self);
}


/// The GetTypeLayout of an `unsafe extern fn()`
pub const UNSAFE_EXTERN_FN_ABI_INFO:GetTypeLayout=
    GetTypeLayoutCtor::<unsafe extern fn()>::STABLE_ABI;

/// The GetTypeLayout of an `extern fn()`
pub const EXTERN_FN_ABI_INFO:GetTypeLayout=
    GetTypeLayoutCtor::<extern fn()>::STABLE_ABI;


/////////////

/// Allows one to create the TypeLayout/TypeLayout for any type `T`,
/// by pretending that it is a primitive type.
/// 
/// Used by the StableAbi derive macro by fields marker as `#[sabi(unsafe_opaque_field)]`.
/// 
/// # Safety
/// 
/// You must ensure that the layout of `T` is compatible through other means.
#[repr(transparent)]
pub struct UnsafeOpaqueField<T>(T);

unsafe impl<T> GetStaticEquivalent_ for UnsafeOpaqueField<T> {
    /// it is fine to use `()` because this type is treated as opaque anyway.
    type StaticEquivalent=();
}
unsafe impl<T> SharedStableAbi for UnsafeOpaqueField<T> {
    type Kind=ValueKind;
    type IsNonZeroType = False;

    const S_LAYOUT: &'static TypeLayout = &TypeLayout::from_params::<Self>(TypeLayoutParams {
        abi_consts:Self::S_ABI_CONSTS,
        name: "OpaqueField",
        item_info:make_item_info!(),
        data: TLData::Opaque,
        generics: tl_genparams!(;;),
    });
}

/////////////

/// Allows one to ensure that a `T` implements `StableAbi` without storing it's layout,
/// by pretending that it is a primitive type.
/// 
/// Used by the StableAbi derive macro by fields marker as `#[sabi(unsafe_sabi_opaque_field)]`.
/// 
/// # Safety
/// 
/// You must ensure that the layout of `T` is compatible through other means.
#[repr(transparent)]
pub struct SabiUnsafeOpaqueField<T>(T);

unsafe impl<T> GetStaticEquivalent_ for SabiUnsafeOpaqueField<T> {
    /// it is fine to use `()` because this type is treated as opaque anyway.
    type StaticEquivalent=();
}
unsafe impl<T> SharedStableAbi for SabiUnsafeOpaqueField<T> 
where
    T:StableAbi
{
    type Kind=ValueKind;
    type IsNonZeroType = False;

    const S_LAYOUT: &'static TypeLayout = &TypeLayout::from_params::<Self>(TypeLayoutParams {
        abi_consts:Self::S_ABI_CONSTS,
        name: "SabiOpaqueField",
        item_info:make_item_info!(),
        data: TLData::Opaque,
        generics: tl_genparams!(;;),
    });
}

/////////////
