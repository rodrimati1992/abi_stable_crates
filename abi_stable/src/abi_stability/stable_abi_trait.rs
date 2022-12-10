//! Where the StableAbi trait is declared,as well as related types/traits.

use core_extensions::type_level_bool::{Boolean, False, True};
use std::{
    cell::{Cell, UnsafeCell},
    marker::{PhantomData, PhantomPinned},
    mem::ManuallyDrop,
    num::{NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize, Wrapping},
    pin::Pin,
    ptr::NonNull,
    sync::atomic::{AtomicBool, AtomicIsize, AtomicPtr, AtomicUsize},
};

use crate::{
    abi_stability::get_static_equivalent::GetStaticEquivalent_,
    reflection::ModReflMode,
    sabi_types::Constructor,
    std_types::{utypeid::UTypeId, RSlice},
    type_layout::{
        CompTLField, CompTLFields, DiscriminantRepr, GenericTLData, GenericTLEnum, ItemInfo,
        LifetimeRange, MonoTLData, MonoTLEnum, MonoTypeLayout, ReprAttr, StartLen, TLDiscriminants,
        TLPrimitive, TypeLayout,
    },
};

///////////////////////

/// Represents a type whose layout is stable.
///
/// This trait can be derived using
/// [`#[derive(StableAbi)]`](derive@crate::StableAbi).
///
/// # Safety
///
/// The layout specified in `LAYOUT` must be correct,
/// otherwise type checking when loading a dynamic library would be unsound,
/// and passing this into a dynamic library would be equivalent to transmuting it.
///
/// # Caveats
///
/// This trait cannot be directly implemented for functions that take lifetime parameters,
/// because of that, [`#[derive(StableAbi)]`](derive@crate::StableAbi)
/// detects the presence of `extern fn` types in type definitions.
pub unsafe trait StableAbi: GetStaticEquivalent_ {
    /// Whether this type has a single invalid bit-pattern.
    ///
    /// Possible values: [`True`]/[`False`]
    ///
    /// Some standard library types have a single value that is invalid for them eg:0,null.
    /// these types are the only ones which can be stored in a `Option<_>` that implements StableAbi.
    ///
    /// For an alternative to `Option<T>` for types where
    /// `IsNonZeroType = False`, you can use [`ROption`].
    ///
    /// Non-exhaustive list of std types that are NonZero:
    ///
    /// - `&T` (any T).
    ///
    /// - `&mut T` (any T).
    ///
    /// - `extern "C" fn()`.
    ///
    /// - `std::ptr::NonNull`
    ///
    /// - `std::num::NonZero*`
    ///
    /// [`True`]: crate::reexports::True
    /// [`False`]: crate::reexports::False
    /// [`ROption`]: crate::std_types::ROption
    type IsNonZeroType: Boolean;

    /// The layout of the type provided by implementors.
    const LAYOUT: &'static TypeLayout;

    /// `const`-equivalents of the associated types.
    const ABI_CONSTS: AbiConsts = AbiConsts {
        type_id: Constructor(crate::std_types::utypeid::new_utypeid::<Self::StaticEquivalent>),
        is_nonzero: <Self::IsNonZeroType as Boolean>::VALUE,
    };
}

/// A type that only has a stable layout when a `PrefixRef` to it is used.
///
/// Types that implement this trait usually have a `_Prefix` suffix.
///
/// # Safety
///
/// This trait can only be implemented by the `StableAbi` derive
/// on types that also use the `#[sabi(kind(Prefix))]` attribute,
/// implementing the trait for a macro generated type.
pub unsafe trait PrefixStableAbi: GetStaticEquivalent_ {
    /// Whether this type has a single invalid bit-pattern.
    type IsNonZeroType: Boolean;

    /// The layout of the type, provided by implementors.
    const LAYOUT: &'static TypeLayout;

    /// `const`-equivalents of the associated types.
    const ABI_CONSTS: AbiConsts = AbiConsts {
        type_id: Constructor(crate::std_types::utypeid::new_utypeid::<Self::StaticEquivalent>),
        is_nonzero: <Self::IsNonZeroType as Boolean>::VALUE,
    };
}

///////////////////////

/// Contains constants equivalent to the associated types in StableAbi.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
#[derive(StableAbi)]
pub struct AbiConsts {
    /// A function to get the unique identifier for some type
    pub type_id: Constructor<UTypeId>,

    /// Whether the type uses non-zero value optimization,
    /// if true then an `Option<Self>` implements StableAbi.
    pub is_nonzero: bool,
}

impl AbiConsts {
    /// Gets the `UTypeId` returned by the `type_id` field.
    #[inline]
    pub fn get_type_id(&self) -> UTypeId {
        self.type_id.get()
    }
}

///////////////////////////////////////////////////////////////////////////////

/// Retrieves the TypeLayout of `T: StableAbi`,
pub extern "C" fn get_type_layout<T>() -> &'static TypeLayout
where
    T: StableAbi,
{
    T::LAYOUT
}

/// Retrieves the TypeLayout of `T: PrefixStableAbi`,
pub extern "C" fn get_prefix_field_type_layout<T>() -> &'static TypeLayout
where
    T: PrefixStableAbi,
{
    <T as PrefixStableAbi>::LAYOUT
}

#[doc(hidden)]
pub extern "C" fn __sabi_opaque_field_type_layout<T>() -> &'static TypeLayout
where
    T: StableAbi,
{
    <UnsafeOpaqueField<T> as StableAbi>::LAYOUT
}
#[doc(hidden)]
pub extern "C" fn __opaque_field_type_layout<T>() -> &'static TypeLayout {
    <UnsafeOpaqueField<T> as StableAbi>::LAYOUT
}

///////////////////////////////////////////////////////////////////////////////

/////////////////////////////////////////////////////////////////////////////
////                Implementations
/////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////

unsafe impl<T> GetStaticEquivalent_ for PhantomData<T>
where
    T: GetStaticEquivalent_,
{
    type StaticEquivalent = PhantomData<T::StaticEquivalent>;
}

unsafe impl<T> StableAbi for PhantomData<T>
where
    T: StableAbi,
{
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = {
        zst_assert!(Self);

        const MONO_TYPE_LAYOUT: &MonoTypeLayout = &MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("PhantomData"),
            ItemInfo::std_type_in(nulstr_trunc!("std::marker")),
            MonoTLData::EMPTY,
            tl_genparams!(;0;),
            ReprAttr::C,
            ModReflMode::Module,
            {
                const S: &[CompTLField] =
                    &[CompTLField::std_field(field0, LifetimeRange::EMPTY, 0)];
                RSlice::from_slice(S)
            },
        );

        make_shared_vars! {
            impl[T] PhantomData<T>
            where[T: StableAbi];

            let (mono_shared_vars,shared_vars)={
                strings={ field0:"0", },
                type_layouts=[T],
            };
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::ABI_CONSTS,
            GenericTLData::Struct,
        )
    };
}

macro_rules! phantomdata_tuples {
    (ignore; $($anything:tt)*)=>{ 1 };
    (
        $(($tuple_param:ident,$name_ident:ident=$name_str:literal))*
    )=>{
        unsafe impl<$($tuple_param,)*>
            GetStaticEquivalent_
        for PhantomData<($($tuple_param,)*)>
        where
            $($tuple_param:GetStaticEquivalent_,)*
        {
            type StaticEquivalent=PhantomData<($($tuple_param::StaticEquivalent,)*)>;
        }

        unsafe impl<$($tuple_param,)*>
            StableAbi
        for PhantomData<($($tuple_param,)*)>
        where
            $($tuple_param:StableAbi,)*
        {
            type IsNonZeroType = False;

            const LAYOUT: &'static TypeLayout = {
                zst_assert!(Self);

                const MONO_TYPE_LAYOUT:&MonoTypeLayout=&MonoTypeLayout::new(
                    *mono_shared_vars,
                    rstr!("PhantomData"),
                    ItemInfo::std_type_in(nulstr_trunc!("std::marker")),
                    MonoTLData::EMPTY,
                    tl_genparams!(;0..COUNT;),
                    ReprAttr::C,
                    ModReflMode::Module,
                    unsafe{
                        RSlice::from_raw_parts_with_lifetime(FIELDS,COUNT)
                    }
                );

                #[allow(unused_assignments)]
                const FIELDS:&'static [CompTLField;COUNT]={
                    let mut i=0;
                    $(
                        #[allow(non_snake_case)]
                        let $tuple_param=
                            CompTLField::std_field($name_ident,LifetimeRange::EMPTY,i);
                        i+=1;
                    )*
                    &[$($tuple_param,)*]
                };

                const COUNT:usize=$(phantomdata_tuples!(ignore;$tuple_param)+)* 0;

                make_shared_vars!{
                    impl[$($tuple_param,)*] PhantomData<($($tuple_param,)*)>
                    where[
                        $($tuple_param:StableAbi,)*
                    ];

                    let (mono_shared_vars,shared_vars)={
                        strings={ $($name_ident:$name_str,)* },
                        type_layouts=[$($tuple_param,)*],
                    };
                }

                &TypeLayout::from_std::<Self>(
                    shared_vars,
                    MONO_TYPE_LAYOUT,
                    Self::ABI_CONSTS,
                    GenericTLData::Struct,
                )
            };
        }
    }
}

/*
fn main(){
    for i in 1..=16{
        println!("phantomdata_tuples!{{");
        for j in 0..i{
            println!("    (T{0},p{0}=\"{0}\")",j);
        }
        println!("}}")
    }
}
*/

phantomdata_tuples! {
    (T0,p0="0")
}
phantomdata_tuples! {
    (T0,p0="0")
    (T1,p1="1")
}
phantomdata_tuples! {
    (T0,p0="0")
    (T1,p1="1")
    (T2,p2="2")
}
phantomdata_tuples! {
    (T0,p0="0")
    (T1,p1="1")
    (T2,p2="2")
    (T3,p3="3")
}
phantomdata_tuples! {
    (T0,p0="0")
    (T1,p1="1")
    (T2,p2="2")
    (T3,p3="3")
    (T4,p4="4")
}
phantomdata_tuples! {
    (T0,p0="0")
    (T1,p1="1")
    (T2,p2="2")
    (T3,p3="3")
    (T4,p4="4")
    (T5,p5="5")
}
phantomdata_tuples! {
    (T0,p0="0")
    (T1,p1="1")
    (T2,p2="2")
    (T3,p3="3")
    (T4,p4="4")
    (T5,p5="5")
    (T6,p6="6")
}
phantomdata_tuples! {
    (T0,p0="0")
    (T1,p1="1")
    (T2,p2="2")
    (T3,p3="3")
    (T4,p4="4")
    (T5,p5="5")
    (T6,p6="6")
    (T7,p7="7")
}
phantomdata_tuples! {
    (T0,p0="0")
    (T1,p1="1")
    (T2,p2="2")
    (T3,p3="3")
    (T4,p4="4")
    (T5,p5="5")
    (T6,p6="6")
    (T7,p7="7")
    (T8,p8="8")
}
phantomdata_tuples! {
    (T0,p0="0")
    (T1,p1="1")
    (T2,p2="2")
    (T3,p3="3")
    (T4,p4="4")
    (T5,p5="5")
    (T6,p6="6")
    (T7,p7="7")
    (T8,p8="8")
    (T9,p9="9")
}
phantomdata_tuples! {
    (T0,p0="0")
    (T1,p1="1")
    (T2,p2="2")
    (T3,p3="3")
    (T4,p4="4")
    (T5,p5="5")
    (T6,p6="6")
    (T7,p7="7")
    (T8,p8="8")
    (T9,p9="9")
    (T10,p10="10")
}
phantomdata_tuples! {
    (T0,p0="0")
    (T1,p1="1")
    (T2,p2="2")
    (T3,p3="3")
    (T4,p4="4")
    (T5,p5="5")
    (T6,p6="6")
    (T7,p7="7")
    (T8,p8="8")
    (T9,p9="9")
    (T10,p10="10")
    (T11,p11="11")
}
phantomdata_tuples! {
    (T0,p0="0")
    (T1,p1="1")
    (T2,p2="2")
    (T3,p3="3")
    (T4,p4="4")
    (T5,p5="5")
    (T6,p6="6")
    (T7,p7="7")
    (T8,p8="8")
    (T9,p9="9")
    (T10,p10="10")
    (T11,p11="11")
    (T12,p12="12")
}
phantomdata_tuples! {
    (T0,p0="0")
    (T1,p1="1")
    (T2,p2="2")
    (T3,p3="3")
    (T4,p4="4")
    (T5,p5="5")
    (T6,p6="6")
    (T7,p7="7")
    (T8,p8="8")
    (T9,p9="9")
    (T10,p10="10")
    (T11,p11="11")
    (T12,p12="12")
    (T13,p13="13")
}
phantomdata_tuples! {
    (T0,p0="0")
    (T1,p1="1")
    (T2,p2="2")
    (T3,p3="3")
    (T4,p4="4")
    (T5,p5="5")
    (T6,p6="6")
    (T7,p7="7")
    (T8,p8="8")
    (T9,p9="9")
    (T10,p10="10")
    (T11,p11="11")
    (T12,p12="12")
    (T13,p13="13")
    (T14,p14="14")
}
phantomdata_tuples! {
    (T0,p0="0")
    (T1,p1="1")
    (T2,p2="2")
    (T3,p3="3")
    (T4,p4="4")
    (T5,p5="5")
    (T6,p6="6")
    (T7,p7="7")
    (T8,p8="8")
    (T9,p9="9")
    (T10,p10="10")
    (T11,p11="11")
    (T12,p12="12")
    (T13,p13="13")
    (T14,p14="14")
    (T15,p15="15")
}

unsafe impl GetStaticEquivalent_ for () {
    type StaticEquivalent = ();
}
unsafe impl StableAbi for () {
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT: &MonoTypeLayout = &MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("()"),
            ItemInfo::primitive(),
            MonoTLData::EMPTY,
            tl_genparams!(;;),
            ReprAttr::C,
            ModReflMode::Module,
            RSlice::EMPTY,
        );

        make_shared_vars! {
            impl[] ();

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

/////////////

unsafe impl<'a, T> GetStaticEquivalent_ for &'a T
where
    T: 'a + GetStaticEquivalent_,
{
    type StaticEquivalent = &'static T::StaticEquivalent;
}

// Does not allow ?Sized types because the DST fat pointer does not have a stable layout.
unsafe impl<'a, T> StableAbi for &'a T
where
    T: 'a + StableAbi,
{
    type IsNonZeroType = True;

    const LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT: &MonoTypeLayout = &MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("&"),
            ItemInfo::primitive(),
            MonoTLData::Primitive(TLPrimitive::SharedRef),
            tl_genparams!('a;0;),
            ReprAttr::Primitive,
            ModReflMode::DelegateDeref { layout_index: 0 },
            {
                const S: &[CompTLField] =
                    &[CompTLField::std_field(field0, LifetimeRange::EMPTY, 0)];
                RSlice::from_slice(S)
            },
        );

        make_shared_vars! {
            impl['a, T] &'a T
            where[ T: 'a + StableAbi];

            let (mono_shared_vars,shared_vars)={
                strings={ field0:"0", },
                type_layouts=[T],
            };
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::ABI_CONSTS,
            GenericTLData::Primitive,
        )
    };
}

unsafe impl<'a, T> GetStaticEquivalent_ for &'a mut T
where
    T: 'a + GetStaticEquivalent_,
{
    type StaticEquivalent = &'static mut T::StaticEquivalent;
}

// Does not allow ?Sized types because the DST fat pointer does not have a stable layout.
unsafe impl<'a, T> StableAbi for &'a mut T
where
    T: 'a + StableAbi,
{
    type IsNonZeroType = True;

    const LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT: &MonoTypeLayout = &MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("&mut"),
            ItemInfo::primitive(),
            MonoTLData::Primitive(TLPrimitive::MutRef),
            tl_genparams!('a;0;),
            ReprAttr::Primitive,
            ModReflMode::DelegateDeref { layout_index: 0 },
            {
                const S: &[CompTLField] =
                    &[CompTLField::std_field(field0, LifetimeRange::EMPTY, 0)];
                RSlice::from_slice(S)
            },
        );

        make_shared_vars! {
            impl['a, T] &'a mut T
            where[ T: 'a + StableAbi];

            let (mono_shared_vars,shared_vars)={
                strings={ field0:"0", },
                type_layouts=[T],
            };
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::ABI_CONSTS,
            GenericTLData::Primitive,
        )
    };
}

unsafe impl<T> GetStaticEquivalent_ for NonNull<T>
where
    T: GetStaticEquivalent_,
{
    type StaticEquivalent = NonNull<T::StaticEquivalent>;
}

// Does not allow ?Sized types because the DST fat pointer does not have a stable layout.
unsafe impl<T> StableAbi for NonNull<T>
where
    T: StableAbi,
{
    type IsNonZeroType = True;

    const LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT: &MonoTypeLayout = &MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("NonNull"),
            ItemInfo::std_type_in(nulstr_trunc!("std::ptr")),
            {
                const S: &[CompTLField] =
                    &[CompTLField::std_field(field0, LifetimeRange::EMPTY, 1)];
                MonoTLData::struct_(RSlice::from_slice(S))
            },
            tl_genparams!(;0;),
            ReprAttr::Transparent,
            ModReflMode::Module,
            RSlice::EMPTY,
        );

        make_shared_vars! {
            impl[T] NonNull<T>
            where[ T: StableAbi];

            let (mono_shared_vars,shared_vars)={
                strings={ field0:"0", },
                type_layouts=[T,*const T],
            };
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::ABI_CONSTS,
            GenericTLData::Struct,
        )
    };
}

unsafe impl<T> GetStaticEquivalent_ for AtomicPtr<T>
where
    T: GetStaticEquivalent_,
{
    type StaticEquivalent = AtomicPtr<T::StaticEquivalent>;
}

unsafe impl<T> StableAbi for AtomicPtr<T>
where
    T: StableAbi,
{
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT: &MonoTypeLayout = &MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("AtomicPtr"),
            ItemInfo::std_type_in(nulstr_trunc!("std::sync::atomic")),
            {
                const S: &[CompTLField] =
                    &[CompTLField::std_field(field0, LifetimeRange::EMPTY, 1)];
                MonoTLData::struct_(RSlice::from_slice(S))
            },
            tl_genparams!(;0;),
            ReprAttr::Transparent,
            ModReflMode::Module,
            RSlice::EMPTY,
        );

        make_shared_vars! {
            impl[T] AtomicPtr<T>
            where[T: StableAbi];

            let (mono_shared_vars,shared_vars)={
                strings={ field0:"0", },
                type_layouts=[T,*mut T],
            };
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::ABI_CONSTS,
            GenericTLData::Struct,
        )
    };
}

unsafe impl<T> GetStaticEquivalent_ for *const T
where
    T: GetStaticEquivalent_,
{
    type StaticEquivalent = *const T::StaticEquivalent;
}
// Does not allow ?Sized types because the DST fat pointer does not have a stable layout.
unsafe impl<T> StableAbi for *const T
where
    T: StableAbi,
{
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT: &MonoTypeLayout = &MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("*const"),
            ItemInfo::primitive(),
            MonoTLData::Primitive(TLPrimitive::ConstPtr),
            tl_genparams!(;0;),
            ReprAttr::Primitive,
            ModReflMode::Module,
            {
                const S: &[CompTLField] =
                    &[CompTLField::std_field(field0, LifetimeRange::EMPTY, 0)];
                RSlice::from_slice(S)
            },
        );

        make_shared_vars! {
            impl[T] *const T
            where[T: StableAbi];

            let (mono_shared_vars,shared_vars)={
                strings={ field0:"0", },
                type_layouts=[T],
            };
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::ABI_CONSTS,
            GenericTLData::Primitive,
        )
    };
}

unsafe impl<T> GetStaticEquivalent_ for *mut T
where
    T: GetStaticEquivalent_,
{
    type StaticEquivalent = *mut T::StaticEquivalent;
}
// Does not allow ?Sized types because the DST fat pointer does not have a stable layout.
unsafe impl<T> StableAbi for *mut T
where
    T: StableAbi,
{
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT: &MonoTypeLayout = &MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("*mut"),
            ItemInfo::primitive(),
            MonoTLData::Primitive(TLPrimitive::MutPtr),
            tl_genparams!(;0;),
            ReprAttr::Primitive,
            ModReflMode::Module,
            {
                const S: &[CompTLField] =
                    &[CompTLField::std_field(field0, LifetimeRange::EMPTY, 0)];
                RSlice::from_slice(S)
            },
        );

        make_shared_vars! {
            impl[T] *mut T
            where[T: StableAbi];

            let (mono_shared_vars,shared_vars)={
                strings={ field0:"0", },
                type_layouts=[T],
            };
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::ABI_CONSTS,
            GenericTLData::Primitive,
        )
    };
}

/////////////

macro_rules! impl_stable_abi_array {
    () => {
        unsafe impl<T, const N: usize> GetStaticEquivalent_ for [T; N]
        where
            T: GetStaticEquivalent_,
        {
            type StaticEquivalent = [T::StaticEquivalent; N];
        }

        unsafe impl<T, const N: usize> StableAbi for [T; N]
        where
            T: StableAbi,
        {
            type IsNonZeroType = False;

            const LAYOUT: &'static TypeLayout = {
                // Used to get constants for [T; N]  where T doesn't matter
                struct ArrayMonoConsts<const N: usize>;

                impl<const N: usize> ArrayMonoConsts<N> {
                    const MONO_TYPE_LAYOUT: &'static MonoTypeLayout = &MonoTypeLayout::new(
                        *mono_shared_vars,
                        rstr!("array"),
                        ItemInfo::primitive(),
                        MonoTLData::Primitive(TLPrimitive::Array),
                        tl_genparams!(;0;0),
                        ReprAttr::Primitive,
                        ModReflMode::Module,
                        {
                            const S: &[CompTLField] =
                                &[CompTLField::std_field(field0, LifetimeRange::EMPTY, 0)];
                            RSlice::from_slice(S)
                        },
                    );
                }

                make_shared_vars! {
                    impl[T, const N: usize] [T; N]
                    where[T: StableAbi];

                    let (mono_shared_vars,shared_vars)={
                        strings={ field0:"element", },
                        type_layouts=[T],
                        constant=[usize => N],
                    };
                }

                &TypeLayout::from_std::<Self>(
                    shared_vars,
                    ArrayMonoConsts::<N>::MONO_TYPE_LAYOUT,
                    Self::ABI_CONSTS,
                    GenericTLData::Primitive,
                )
            };
        }
    };
}

impl_stable_abi_array! {}

/////////////

unsafe impl<T> GetStaticEquivalent_ for Option<T>
where
    T: GetStaticEquivalent_,
{
    type StaticEquivalent = Option<T::StaticEquivalent>;
}
/// Implementing abi stability for `Option<T>` is fine if
/// T is a NonZero primitive type.
unsafe impl<T> StableAbi for Option<T>
where
    T: StableAbi<IsNonZeroType = True>,
{
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT: &MonoTypeLayout = &MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("Option"),
            ItemInfo::std_type_in(nulstr_trunc!("std::option")),
            MonoTLData::Enum(MonoTLEnum::new(variant_names, rslice![1, 0], {
                const S: &[CompTLField] =
                    &[CompTLField::std_field(field0, LifetimeRange::EMPTY, 0)];
                CompTLFields::from_fields(RSlice::from_slice(S))
            })),
            tl_genparams!(;0;),
            ReprAttr::OptionNonZero,
            ModReflMode::Module,
            RSlice::EMPTY,
        );

        make_shared_vars! {
            impl[T] Option<T>
            where [ T: StableAbi<IsNonZeroType = True>, ];

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
            Self::ABI_CONSTS,
            GenericTLData::Enum(GenericTLEnum::exhaustive(TLDiscriminants::from_u8_slice(
                rslice![0, 1],
            ))),
        )
    };
}

/////////////

macro_rules! impl_for_primitive_ints {
    (
        $( ($type:ty,$type_name:literal,$tl_primitive:expr) ,)*
    ) => (
        $(
            unsafe impl GetStaticEquivalent_ for $type {
                type StaticEquivalent=Self;
            }
            unsafe impl StableAbi for $type {
                type IsNonZeroType=False;

                const LAYOUT: &'static TypeLayout = {
                    const MONO_TYPE_LAYOUT:&MonoTypeLayout=&MonoTypeLayout::new(
                        *mono_shared_vars,
                        rstr!($type_name),
                        ItemInfo::primitive(),
                        MonoTLData::Primitive($tl_primitive),
                        tl_genparams!(;;),
                        ReprAttr::Primitive,
                        ModReflMode::Module,
                        RSlice::EMPTY,
                    );

                    make_shared_vars!{
                        impl[] $type;

                        let (mono_shared_vars,shared_vars)={
                            type_layouts=[],
                        };
                    }

                    &TypeLayout::from_std::<Self>(
                        shared_vars,
                        MONO_TYPE_LAYOUT,
                        Self::ABI_CONSTS,
                        GenericTLData::Primitive,
                    )
                };
            }
        )*
    )
}

impl_for_primitive_ints! {
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
    (f32 ,"f32" ,TLPrimitive::F32),
    (f64 ,"f64" ,TLPrimitive::F64),
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
            unsafe impl StableAbi for $this {
                type IsNonZeroType=$zeroness;

                const LAYOUT: &'static TypeLayout = {
                    const MONO_TYPE_LAYOUT:&MonoTypeLayout=&MonoTypeLayout::new(
                        *mono_shared_vars,
                        rstr!($this_name),
                        ItemInfo::std_type_in(nulstr_trunc!($in_mod)),
                        {
                            const S: &[CompTLField] = &[
                                CompTLField::std_field(field0,LifetimeRange::EMPTY,0),
                            ];
                            MonoTLData::struct_(RSlice::from_slice(S))
                        },
                        tl_genparams!(;;),
                        ReprAttr::Transparent,
                        ModReflMode::Module,
                        RSlice::EMPTY,
                    );

                    make_shared_vars!{
                        impl[] $this;

                        let (mono_shared_vars,shared_vars)={
                            strings={ field0:"0" },
                            type_layouts=[$prim_repr],
                        };
                    }

                    &TypeLayout::from_std::<Self>(
                        shared_vars,
                        MONO_TYPE_LAYOUT,
                        Self::ABI_CONSTS,
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

mod rust_1_34_impls {
    use super::*;
    use core::num::*;
    use std::sync::atomic::*;

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

mod rust_1_36_impls {
    use super::*;
    use std::mem::MaybeUninit;

    unsafe impl<T> GetStaticEquivalent_ for MaybeUninit<T>
    where
        T: GetStaticEquivalent_,
    {
        type StaticEquivalent = MaybeUninit<T::StaticEquivalent>;
    }
    unsafe impl<T> StableAbi for MaybeUninit<T>
    where
        T: StableAbi,
    {
        // MaybeUninit blocks layout optimizations.
        type IsNonZeroType = False;

        const LAYOUT: &'static TypeLayout = {
            const MONO_TYPE_LAYOUT: &MonoTypeLayout = &MonoTypeLayout::new(
                *mono_shared_vars,
                rstr!("MaybeUninit"),
                ItemInfo::std_type_in(nulstr_trunc!("std::mem")),
                {
                    const S: &[CompTLField] =
                        &[CompTLField::std_field(field0, LifetimeRange::EMPTY, 0)];
                    MonoTLData::struct_(RSlice::from_slice(S))
                },
                tl_genparams!(;0;),
                // Using `ReprAttr::Transparent` so that if I add C header file translation
                // it will be translated to just `T`.
                ReprAttr::Transparent,
                ModReflMode::Opaque,
                RSlice::EMPTY,
            );

            make_shared_vars! {
                impl[T] MaybeUninit<T>
                where [T: StableAbi];

                let (mono_shared_vars,shared_vars)={
                    strings={ field0:"value" },
                    type_layouts=[T],
                };
            }

            &TypeLayout::from_std::<Self>(
                shared_vars,
                MONO_TYPE_LAYOUT,
                Self::ABI_CONSTS,
                GenericTLData::Struct,
            )
        };
    }
}

/////////////

macro_rules! impl_sabi_for_newtype {
    (@trans transparent)=>{ P::IsNonZeroType };
    (@trans C)=>{ False };
    (
        $type_constr:ident
        $(where[ $($where_clause:tt)* ])* ,
        $transparency:ident,
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
        unsafe impl<P> StableAbi for $type_constr<P>
        where
            P: StableAbi,
            $($($where_clause)*)*
        {
            type IsNonZeroType = impl_sabi_for_newtype!(@trans $transparency);

            const LAYOUT: &'static TypeLayout = {
                const MONO_TYPE_LAYOUT:&MonoTypeLayout=&MonoTypeLayout::new(
                    *mono_shared_vars,
                    rstr!($type_name),
                    ItemInfo::std_type_in(nulstr_trunc!($mod_path)),
                    {
                        const S: &[CompTLField] = &[
                            CompTLField::std_field(field0,LifetimeRange::EMPTY,0),
                        ];
                        MonoTLData::struct_(RSlice::from_slice(S))
                    },
                    tl_genparams!(;0;),
                    ReprAttr::Transparent,
                    ModReflMode::Module,
                    RSlice::EMPTY,
                );

                make_shared_vars!{
                    impl[P]  $type_constr<P>
                    where [
                        P: StableAbi,
                        $($($where_clause)*)*
                    ];

                    let (mono_shared_vars,shared_vars)={
                        strings={ field0:"0" },
                        type_layouts=[P],
                    };
                }

                &TypeLayout::from_std::<Self>(
                    shared_vars,
                    MONO_TYPE_LAYOUT,
                    Self::ABI_CONSTS,
                    GenericTLData::Struct,
                )
            };
        }
    )
}

impl_sabi_for_newtype! { Wrapping    ,transparent,"Wrapping"    ,"std::num" }
impl_sabi_for_newtype! { Pin         ,transparent,"Pin"         ,"std::pin" }
impl_sabi_for_newtype! { ManuallyDrop,transparent,"ManuallyDrop","std::mem" }

impl_sabi_for_newtype! { Cell        ,C,"Cell"        ,"std::cell" }
impl_sabi_for_newtype! { UnsafeCell  ,C,"UnsafeCell"  ,"std::cell" }

/////////////

macro_rules! impl_stableabi_for_unit_struct {
    (
        $type_constr:ident,
        $type_name:literal,
        $item_info:expr
    ) => {
        unsafe impl GetStaticEquivalent_ for $type_constr {
            type StaticEquivalent = $type_constr;
        }
        unsafe impl StableAbi for $type_constr {
            type IsNonZeroType = False;

            const LAYOUT: &'static TypeLayout = {
                const MONO_TYPE_LAYOUT: &MonoTypeLayout = &MonoTypeLayout::new(
                    *mono_shared_vars,
                    rstr!($type_name),
                    $item_info,
                    MonoTLData::struct_(RSlice::EMPTY),
                    tl_genparams!(;;),
                    ReprAttr::C,
                    ModReflMode::Module,
                    RSlice::EMPTY,
                );

                make_shared_vars! {
                    impl[] $type_constr;

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
    };
}

impl_stableabi_for_unit_struct! {
    PhantomPinned,"PhantomPinned",ItemInfo::std_type_in(nulstr_trunc!("std::marker"))
}

/////////////

unsafe impl GetStaticEquivalent_ for ::core::ffi::c_void {
    type StaticEquivalent = ::core::ffi::c_void;
}
unsafe impl StableAbi for ::core::ffi::c_void {
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT: &MonoTypeLayout = &MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("c_void"),
            ItemInfo::std_type_in(nulstr_trunc!("std::ffi")),
            MonoTLData::EMPTY,
            tl_genparams!(;;),
            ReprAttr::C,
            ModReflMode::Module,
            RSlice::EMPTY,
        );

        make_shared_vars! {
            impl[] ::core::ffi::c_void;

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

/////////////

unsafe impl GetStaticEquivalent_ for core_extensions::Void {
    type StaticEquivalent = Self;
}
unsafe impl StableAbi for core_extensions::Void {
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT: &MonoTypeLayout = &MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("Void"),
            ItemInfo::package_and_mod("core_extensions;0.0.0", nulstr_trunc!("core_extensions")),
            MonoTLData::Enum(MonoTLEnum::new(
                StartLen::EMPTY,
                RSlice::EMPTY,
                CompTLFields::EMPTY,
            )),
            tl_genparams!(;;),
            ReprAttr::Int(DiscriminantRepr::U8),
            ModReflMode::Module,
            RSlice::EMPTY,
        );

        make_shared_vars! {
            impl[] core_extensions::Void;

            let (mono_shared_vars,shared_vars)={};
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::ABI_CONSTS,
            GenericTLData::Enum(GenericTLEnum::exhaustive(TLDiscriminants::from_u8_slice(
                RSlice::EMPTY,
            ))),
        )
    };
}

/////////////

/// The layout of `extern "C" fn()` and `unsafe extern "C" fn()`
macro_rules! empty_extern_fn_layout {
    ($this:ty) => {{
        make_shared_vars! {
            impl[] $this;

            let (mono_shared_vars,shared_vars)={};
        }
        const MONO_TL_EXTERN_FN: &'static MonoTypeLayout = &MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("AFunctionPointer"),
            make_item_info!(),
            MonoTLData::Opaque,
        tl_genparams!(;;),
            ReprAttr::C,
            ModReflMode::Opaque,
            RSlice::EMPTY,
        );

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TL_EXTERN_FN,
            Self::ABI_CONSTS,
            GenericTLData::Opaque,
        )
    }};
}

/// This is the only function type that implements StableAbi
/// so as to make it more obvious that functions involving lifetimes
/// cannot implement this trait directly (because of higher ranked trait bounds).
unsafe impl GetStaticEquivalent_ for extern "C" fn() {
    type StaticEquivalent = Self;
}
unsafe impl StableAbi for extern "C" fn() {
    type IsNonZeroType = True;

    const LAYOUT: &'static TypeLayout = empty_extern_fn_layout!(extern "C" fn());
}

/// This is the only function type that implements StableAbi
/// so as to make it more obvious that functions involving lifetimes
/// cannot implement this trait directly (because of higher ranked trait bounds).
unsafe impl GetStaticEquivalent_ for unsafe extern "C" fn() {
    type StaticEquivalent = Self;
}
unsafe impl StableAbi for unsafe extern "C" fn() {
    type IsNonZeroType = True;

    const LAYOUT: &'static TypeLayout = empty_extern_fn_layout!(unsafe extern "C" fn());
}

/// A function that returns the TypeLayout of an `unsafe extern "C" fn()`
#[doc(hidden)]
pub const UNSAFE_EXTERN_FN_LAYOUT: extern "C" fn() -> &'static TypeLayout =
    get_type_layout::<unsafe extern "C" fn()>;

/// A function that returns the TypeLayout of an `extern "C" fn()`
#[doc(hidden)]
pub const EXTERN_FN_LAYOUT: extern "C" fn() -> &'static TypeLayout =
    get_type_layout::<extern "C" fn()>;

/////////////

/// Allows one to create the `TypeLayout` for any type `T`,
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
    type StaticEquivalent = ();
}
unsafe impl<T> StableAbi for UnsafeOpaqueField<T> {
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT: &MonoTypeLayout = &MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("OpaqueField"),
            make_item_info!(),
            MonoTLData::Opaque,
            tl_genparams!(;;),
            ReprAttr::C,
            ModReflMode::Module,
            RSlice::EMPTY,
        );

        make_shared_vars! {
            impl[T] UnsafeOpaqueField<T>;

            let (mono_shared_vars,shared_vars)={};
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::ABI_CONSTS,
            GenericTLData::Opaque,
        )
    };
}

/// Allows one to ensure that a `T` implements `StableAbi`,
/// while storing an opaque layout instead of `<T as StableAbi>::LAYOUT`.
///
/// Used by the `StableAbi` derive macro by fields marker as `#[sabi(unsafe_sabi_opaque_field)]`.
///
/// # Safety
///
/// You must ensure that the layout of `T` is compatible through other means.
#[repr(transparent)]
pub struct SabiUnsafeOpaqueField<T>(T);

unsafe impl<T> GetStaticEquivalent_ for SabiUnsafeOpaqueField<T> {
    /// it is fine to use `()` because this type is treated as opaque anyway.
    type StaticEquivalent = ();
}
unsafe impl<T> StableAbi for SabiUnsafeOpaqueField<T>
where
    T: StableAbi,
{
    type IsNonZeroType = False;

    const LAYOUT: &'static TypeLayout = { <UnsafeOpaqueField<T>>::LAYOUT };
}

/////////////
