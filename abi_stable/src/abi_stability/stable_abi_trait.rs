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
    std_types::utypeid::UTypeId,
    reflection::ModReflMode,
    type_layout::{
        LifetimeRange, 
        MonoTLData, GenericTLData, TypeLayout, MonoTypeLayout,
        ItemInfo,ReprAttr,TLPrimitive,TLDiscriminants,
        GenericTLEnum, MonoTLEnum, CompTLField, CompTLFields, StartLen, DiscriminantRepr,
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

- extern "C" fn().

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
pub type TypeLayoutCtor=Constructor<&'static TypeLayout>;

// pub unsafe trait GetTypeLayoutCtor<B> {


#[doc(hidden)]
pub struct GetTypeLayoutCtor<T>(T);

impl<T> GetTypeLayoutCtor<T>
where T: SharedStableAbi,
{
    pub const SHARED_STABLE_ABI:TypeLayoutCtor=Constructor (
        get_ssa_type_layout::<T>,
    );
}

impl<T> GetTypeLayoutCtor<T>
where T: StableAbi,
{
    pub const STABLE_ABI:TypeLayoutCtor=Constructor (
        get_type_layout::<T>,
    );

    pub const SABI_OPAQUE_FIELD:TypeLayoutCtor=Constructor (
        get_type_layout::<SabiUnsafeOpaqueField<T>>,
    );
}

impl<T> GetTypeLayoutCtor<T>{
    pub const OPAQUE_FIELD:TypeLayoutCtor=Constructor (
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

    const S_LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("PhantomData"),
            ItemInfo::std_type_in(nul_str!("std::marker")),
            MonoTLData::EMPTY,
            tl_genparams!(;0;),
            ReprAttr::C,
            ModReflMode::Module,
            rslice![CompTLField::std_field(field0,LifetimeRange::EMPTY,0)],
        );

        make_shared_vars!{
            let (mono_shared_vars,shared_vars)={
                strings={ field0:"0", },
                type_layouts_shared=[T],
            };
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::S_ABI_CONSTS,
            GenericTLData::Struct,
        )
    };


}


unsafe impl GetStaticEquivalent_ for () {
    type StaticEquivalent=();
}
unsafe impl SharedStableAbi for () {
    type Kind=ValueKind;
    type IsNonZeroType = False;

    const S_LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("()"),
            ItemInfo::primitive(),
            MonoTLData::EMPTY,
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

    const S_LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("&"),
            ItemInfo::primitive(),
            MonoTLData::Primitive(TLPrimitive::SharedRef),
            tl_genparams!('a;0;),
            ReprAttr::Primitive,
            ModReflMode::DelegateDeref{layout_index:0},
            rslice![CompTLField::std_field(field0,LifetimeRange::EMPTY,0)],
        );

        make_shared_vars!{
            let (mono_shared_vars,shared_vars)={
                strings={ field0:"0", },
                type_layouts_shared=[T],
            };
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::S_ABI_CONSTS,
            GenericTLData::Primitive,
        )
    };
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

    const S_LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("&mut"),
            ItemInfo::primitive(),
            MonoTLData::Primitive(TLPrimitive::MutRef),
            tl_genparams!('a;0;),
            ReprAttr::Primitive,
            ModReflMode::DelegateDeref{layout_index:0},
            rslice![CompTLField::std_field(field0,LifetimeRange::EMPTY,0)],
        );

        make_shared_vars!{
            let (mono_shared_vars,shared_vars)={
                strings={ field0:"0", },
                type_layouts_shared=[T],
            };
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::S_ABI_CONSTS,
            GenericTLData::Primitive,
        )
    };
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

    const S_LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("NonNull"),
            ItemInfo::std_type_in(nul_str!("std::ptr")),
            MonoTLData::struct_(rslice![ 
                CompTLField::std_field(field0,LifetimeRange::EMPTY,1) 
            ]),
            tl_genparams!(;0;),
            ReprAttr::Transparent,
            ModReflMode::Module,
            rslice![],
        );

        make_shared_vars!{
            let (mono_shared_vars,shared_vars)={
                strings={ field0:"0", },
                type_layouts=[T,*const T],
            };
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::S_ABI_CONSTS,
            GenericTLData::Struct,
        )
    };
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

    const S_LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("AtomicPtr"),
            ItemInfo::std_type_in(nul_str!("std::sync::atomic")),
            MonoTLData::struct_(rslice![ 
                CompTLField::std_field(field0,LifetimeRange::EMPTY,1) 
            ]),
            tl_genparams!(;0;),
            ReprAttr::Transparent,
            ModReflMode::Module,
            rslice![],
        );

        make_shared_vars!{
            let (mono_shared_vars,shared_vars)={
                strings={ field0:"0", },
                type_layouts=[T,*mut T],
            };
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::S_ABI_CONSTS,
            GenericTLData::Struct,
        )
    };
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

    const S_LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("*const"),
            ItemInfo::primitive(),
            MonoTLData::Primitive(TLPrimitive::ConstPtr),
            tl_genparams!(;0;),
            ReprAttr::Primitive,
            ModReflMode::Module,
            rslice![ CompTLField::std_field(field0,LifetimeRange::EMPTY,0) ],
        );

        make_shared_vars!{
            let (mono_shared_vars,shared_vars)={
                strings={ field0:"0", },
                type_layouts_shared=[T],
            };
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::S_ABI_CONSTS,
            GenericTLData::Primitive,
        )
    };
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

    const S_LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("*mut"),
            ItemInfo::primitive(),
            MonoTLData::Primitive(TLPrimitive::MutPtr),
            tl_genparams!(;0;),
            ReprAttr::Primitive,
            ModReflMode::Module,
            rslice![ CompTLField::std_field(field0,LifetimeRange::EMPTY,0) ],
        );

        make_shared_vars!{
            let (mono_shared_vars,shared_vars)={
                strings={ field0:"0", },
                type_layouts=[T],
            };
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::S_ABI_CONSTS,
            GenericTLData::Primitive,
        )
    };
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

                const S_LAYOUT: &'static TypeLayout = {
                    const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
                        *mono_shared_vars,
                        rstr!("array"),
                        ItemInfo::primitive(),
                        MonoTLData::Primitive(TLPrimitive::Array{len:$size}),
                        tl_genparams!(;0;0),
                        ReprAttr::Primitive,
                        ModReflMode::Module,
                        rslice![ CompTLField::std_field(field0,LifetimeRange::EMPTY,0) ],
                    );

                    make_shared_vars!{
                        let (mono_shared_vars,shared_vars)={
                            strings={ field0:"element", },
                            type_layouts=[T],
                            constants=[$size],
                        };
                    }

                    &TypeLayout::from_std::<Self>(
                        shared_vars,
                        MONO_TYPE_LAYOUT,
                        Self::S_ABI_CONSTS,
                        GenericTLData::Primitive,
                    )
                };
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


    const S_LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("Option"),
            ItemInfo::std_type_in(nul_str!("std::option")),
            MonoTLData::Enum(MonoTLEnum::new(
                variant_names,
                rslice![1,0],
                CompTLFields::from_fields(rslice![
                    CompTLField::std_field(field0,LifetimeRange::EMPTY,0),
                ])
            )),
            tl_genparams!(;0;),
            ReprAttr::OptionNonZero,
            ModReflMode::Module,
            rslice![],
        );

        make_shared_vars!{
            let (mono_shared_vars,shared_vars)={
                strings={
                    variant_names:"Some;None;",
                    field0:"0", 
                },
                type_layouts=[T],
            };
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::S_ABI_CONSTS,
            GenericTLData::Enum(GenericTLEnum::exhaustive(
                TLDiscriminants::from_u8_slice(rslice![0,1])
            )),
        )
    };
}

/////////////


macro_rules! impl_for_primitive_ints {
    (
        $( ($zeroable:ty,$zeroable_name:literal,$tl_primitive:expr) ,)*
    ) => (
        $(
            unsafe impl GetStaticEquivalent_ for $zeroable {
                type StaticEquivalent=Self;
            }
            unsafe impl SharedStableAbi for $zeroable {
                type Kind=ValueKind;
                type IsNonZeroType=False;

                const S_LAYOUT: &'static TypeLayout = {
                    const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
                        *mono_shared_vars,
                        rstr!($zeroable_name),
                        ItemInfo::primitive(),
                        MonoTLData::Primitive($tl_primitive),
                        tl_genparams!(;;),
                        ReprAttr::Primitive,
                        ModReflMode::Module,
                        rslice![],
                    );

                    make_shared_vars!{
                        let (mono_shared_vars,shared_vars)={
                            type_layouts=[],
                        };
                    }

                    &TypeLayout::from_std::<Self>(
                        shared_vars,
                        MONO_TYPE_LAYOUT,
                        Self::S_ABI_CONSTS,
                        GenericTLData::Primitive,
                    )
                };
            }
        )*
    )
}

impl_for_primitive_ints!{
    (u8   ,"u8"   ,TLPrimitive::U8),
    (i8   ,"i8"   ,TLPrimitive::I8),
    (u16  ,"u16"  ,TLPrimitive::U16),
    (i16  ,"i16"  ,TLPrimitive::I16),
    (u32  ,"u32"  ,TLPrimitive::U32),
    (i32  ,"i32"  ,TLPrimitive::I32),
    (u64  ,"u64"  ,TLPrimitive::U64),
    (i64  ,"i64"  ,TLPrimitive::I64),
    (usize,"usize",TLPrimitive::Usize),
    (isize,"isize",TLPrimitive::Isize),
    (bool ,"bool" ,TLPrimitive::Bool),
}


macro_rules! impl_for_concrete {
    (
        type IsNonZeroType=$zeroness:ty;
        [
            $( ($this:ty,$this_name:literal,$prim_repr:ty,$in_mod:expr) ,)*
        ]
    ) => (
        $(
            unsafe impl GetStaticEquivalent_ for $this {
                type StaticEquivalent=Self;
            }
            unsafe impl SharedStableAbi for $this {
                type Kind=ValueKind;
                type IsNonZeroType=$zeroness;

                const S_LAYOUT: &'static TypeLayout = {
                    const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
                        *mono_shared_vars,
                        rstr!($this_name),
                        ItemInfo::std_type_in(nul_str!($in_mod)),
                        MonoTLData::struct_(rslice![ 
                            CompTLField::std_field(field0,LifetimeRange::EMPTY,0) 
                        ]),
                        tl_genparams!(;;),
                        ReprAttr::Transparent,
                        ModReflMode::Module,
                        rslice![],
                    );

                    make_shared_vars!{
                        let (mono_shared_vars,shared_vars)={
                            strings={ field0:"0" },
                            type_layouts=[$prim_repr],
                        };
                    }

                    &TypeLayout::from_std::<Self>(
                        shared_vars,
                        MONO_TYPE_LAYOUT,
                        Self::S_ABI_CONSTS,
                        GenericTLData::Struct,
                    )
                };
            }
        )*
    )
}



impl_for_concrete! {
    type IsNonZeroType=False;
    [
        (AtomicBool ,"AtomicBool" ,bool,"std::sync::atomic"),
        (AtomicIsize,"AtomicIsize",isize,"std::sync::atomic"),
        (AtomicUsize,"AtomicUsize",usize,"std::sync::atomic"),
    ]
}

impl_for_concrete! {
    type IsNonZeroType=True;
    [
        (NonZeroU8   ,"NonZeroU8"   ,u8,"std::num"),
        (NonZeroU16  ,"NonZeroU16"  ,u16,"std::num"),
        (NonZeroU32  ,"NonZeroU32"  ,u32,"std::num"),
        (NonZeroU64  ,"NonZeroU64"  ,u64,"std::num"),
        (NonZeroUsize,"NonZeroUsize",usize,"std::num"),
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
            (AtomicI8 ,"AtomicI8" ,i8,"std::sync::atomic"),
            (AtomicI16,"AtomicI16",i16,"std::sync::atomic"),
            (AtomicI32,"AtomicI32",i32,"std::sync::atomic"),
            (AtomicI64,"AtomicI64",i64,"std::sync::atomic"),
            (AtomicU8 ,"AtomicU8" ,u8,"std::sync::atomic"),
            (AtomicU16,"AtomicU16",u16,"std::sync::atomic"),
            (AtomicU32,"AtomicU32",u32,"std::sync::atomic"),
            (AtomicU64,"AtomicU64",u64,"std::sync::atomic"),
        ]
    }

    impl_for_concrete! {
        type IsNonZeroType=True;
        [
            (NonZeroI8   ,"NonZeroI8"   ,i8,"core::num"),
            (NonZeroI16  ,"NonZeroI16"  ,i16,"core::num"),
            (NonZeroI32  ,"NonZeroI32"  ,i32,"core::num"),
            (NonZeroI64  ,"NonZeroI64"  ,i64,"core::num"),
            (NonZeroIsize,"NonZeroIsize",isize,"core::num"),
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


        const S_LAYOUT: &'static TypeLayout = {
            const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
                *mono_shared_vars,
                rstr!("MaybeUninit"),
                ItemInfo::std_type_in(nul_str!("std::mem")),
                MonoTLData::struct_(rslice![ 
                    CompTLField::std_field(field0,LifetimeRange::EMPTY,0),
                ]),
                tl_genparams!(;0;),
                // Using `ReprAttr::Transparent` so that if I add C header file translation
                // it will be translated to just `T`.
                ReprAttr::Transparent,
                ModReflMode::Opaque,
                rslice![],
            );

            make_shared_vars!{
                let (mono_shared_vars,shared_vars)={
                    strings={ field0:"value" },
                    type_layouts=[T],
                };
            }

            &TypeLayout::from_std::<Self>(
                shared_vars,
                MONO_TYPE_LAYOUT,
                Self::S_ABI_CONSTS,
                GenericTLData::Struct,
            )
        };
    }
}



/////////////

macro_rules! impl_sabi_for_transparent {
    (
        $type_constr:ident
        $(where[ $($where_clause:tt)* ])* ,
        $type_name:literal,
        $mod_path:expr
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

            const S_LAYOUT: &'static TypeLayout = {
                const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
                    *mono_shared_vars,
                    rstr!($type_name),
                    ItemInfo::std_type_in(nul_str!($mod_path)),
                    MonoTLData::struct_(rslice![ 
                        CompTLField::std_field(field0,LifetimeRange::EMPTY,0) 
                    ]),
                    tl_genparams!(;0;),
                    ReprAttr::Transparent,
                    ModReflMode::Module,
                    rslice![],
                );

                make_shared_vars!{
                    let (mono_shared_vars,shared_vars)={
                        strings={ field0:"0" },
                        type_layouts=[P],
                    };
                }

                &TypeLayout::from_std::<Self>(
                    shared_vars,
                    MONO_TYPE_LAYOUT,
                    Self::S_ABI_CONSTS,
                    GenericTLData::Struct,
                )
            };
        }
    )
}


impl_sabi_for_transparent!{ Wrapping    ,"Wrapping"    ,"std::num" }
impl_sabi_for_transparent!{ Pin         ,"Pin"         ,"std::pin" }
impl_sabi_for_transparent!{ ManuallyDrop,"ManuallyDrop","std::mem" }
impl_sabi_for_transparent!{ Cell        ,"Cell"        ,"std::cell" }
impl_sabi_for_transparent!{ UnsafeCell  ,"UnsafeCell"  ,"std::cell" }

/////////////

macro_rules! impl_stableabi_for_unit_struct {
    (
        $type_constr:ident,
        $type_name:literal,
        $item_info:expr
    ) => (
        unsafe impl GetStaticEquivalent_ for $type_constr{
            type StaticEquivalent=$type_constr;
        }
        unsafe impl SharedStableAbi for $type_constr{
            type Kind=ValueKind;
            type IsNonZeroType = False;

            const S_LAYOUT: &'static TypeLayout = {
                const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
                    *mono_shared_vars,
                    rstr!($type_name),
                    $item_info,
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
    )
}


impl_stableabi_for_unit_struct!{ 
    PhantomPinned,"PhantomPinned",ItemInfo::std_type_in(nul_str!("std::marker"))
}

/////////////


unsafe impl GetStaticEquivalent_ for core_extensions::Void {
    type StaticEquivalent=Self;
}
unsafe impl SharedStableAbi for core_extensions::Void {
    type Kind=ValueKind;
    type IsNonZeroType = False;

    const S_LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("Void"),
            ItemInfo::package_and_mod(
                "core_extensions;0.0.0",
                nul_str!("core_extensions"),
            ),
            MonoTLData::Enum(MonoTLEnum::new(StartLen::EMPTY,rslice![],CompTLFields::EMPTY)),
            tl_genparams!(;;),
            ReprAttr::Int(DiscriminantRepr::U8),
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
            GenericTLData::Enum(GenericTLEnum::exhaustive(
                TLDiscriminants::from_u8_slice(rslice![])
            )),
        )
    };
}



/////////////




/// The layout of `extern "C" fn()` and `unsafe extern "C" fn()`
macro_rules! empty_extern_fn_layout{
    ($this:ty) => ({
        make_shared_vars!{
            let (mono_shared_vars,shared_vars)={};
        }
        const MONO_TL_EXTERN_FN:&'static MonoTypeLayout=&MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("AFunctionPointer"),
            make_item_info!(),
            MonoTLData::Opaque,
            tl_genparams!(;;),
            ReprAttr::C,
            ModReflMode::Opaque,
            rslice![],
        );

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TL_EXTERN_FN,
            Self::S_ABI_CONSTS,
            GenericTLData::Opaque,
        )
    })
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


/// The TypeLayoutCtor of an `unsafe extern "C" fn()`
pub const UNSAFE_EXTERN_FN_LAYOUT:TypeLayoutCtor=
    GetTypeLayoutCtor::<unsafe extern "C" fn()>::STABLE_ABI;

/// The TypeLayoutCtor of an `extern "C" fn()`
pub const EXTERN_FN_LAYOUT:TypeLayoutCtor=
    GetTypeLayoutCtor::<extern "C" fn()>::STABLE_ABI;


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

    const S_LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("OpaqueField"),
            make_item_info!(),
            MonoTLData::Opaque,
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
            GenericTLData::Opaque,
        )
    };
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

    const S_LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("SabiOpaqueField"),
            make_item_info!(),
            MonoTLData::Opaque,
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
            GenericTLData::Opaque,
        )
    };
}

/////////////
