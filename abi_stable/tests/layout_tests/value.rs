#![allow(dead_code)]

use std::{marker::PhantomData, mem, num, ptr, sync::atomic};

#[allow(unused_imports)]
use core_extensions::matches;

use abi_stable::{
    abi_stability::abi_checking::{check_layout_compatibility, AbiInstability},
    external_types::{
        crossbeam_channel::{RReceiver, RSender},
        RMutex, ROnce, RRwLock,
    },
    marker_type::{NonOwningPhantom, UnsafeIgnoredType},
    std_types::*,
    type_layout::{TLData, Tag, TypeLayout},
    *,
};

use super::shared_types::{
    basic_enum, enum_extra_fields_b, extra_variant, gen_basic, gen_more_lts, gen_more_lts_b, mod_5,
    mod_7, swapped_fields_first,
};

pub(super) mod union_1a {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub union Union {
        x: u32,
    }
}

pub(super) mod union_1b {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub union Union {
        x_alt: u32,
    }
}

pub(super) mod union_2a {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub union Union {
        x: u32,
        y: u32,
    }
}

pub(super) mod union_2b {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub union Union {
        x: u32,
        y_alt: u32,
    }
}

pub(super) mod union_3 {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub union Union {
        x: u32,
        y: u32,
        w: u16,
    }
}

pub(super) mod union_4 {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub union Union {
        x: u32,
        y: u32,
        w: u16,
        h: u32,
    }
}

pub(super) mod regular {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Rectangle {
        x: u32,
        y: u32,
        w: u16,
        h: u32,
    }
}

pub(super) mod changed_name {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Rectangleiiiiii {
        x: u32,
        y: u32,
        w: u16,
        h: u32,
    }
}

pub(super) mod changed_field_name {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Rectangle {
        x: u32,
        y: u32,
        #[sabi(rename = w2)]
        w: u16,
        h: u32,
    }
}

pub(super) mod swapped_fields_last {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Rectangle {
        x: u32,
        y: u32,
        h: u32,
        w: u16,
    }
}

pub(super) mod removed_field_first {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Rectangle {
        y: u32,
        w: u16,
        h: u32,
    }
}

pub(super) mod removed_field_last {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Rectangle {
        x: u32,
        y: u32,
        w: u16,
    }
}

pub(super) mod removed_all_fields {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Rectangle {
        x: u32,
        y: u32,
        w: u16,
    }
}

pub(super) mod changed_type_first {
    use super::shadowed;

    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Rectangle {
        x: shadowed::u32,
        y: u32,
        w: u16,
        h: u32,
    }
}

pub(super) mod changed_type_last {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Rectangle {
        x: u32,
        y: u32,
        w: u16,
        h: u32,
    }
}

pub(super) mod changed_alignment {
    #[repr(C, align(16))]
    #[derive(abi_stable::StableAbi)]
    pub struct Rectangle {
        x: u32,
        y: u32,
        w: u16,
        h: u32,
    }
}

pub(super) mod built_in {
    pub use i32 as std_i32;
    pub use u32 as std_u32;
}

/// Types that have the same type-name and layout as built-in types.
#[allow(non_camel_case_types)]
pub(super) mod shadowed {
    use super::built_in::*;

    #[repr(transparent)]
    #[derive(abi_stable::StableAbi)]
    pub struct u32 {
        integer: std_i32,
    }

    #[repr(transparent)]
    #[derive(abi_stable::StableAbi)]
    pub struct i32 {
        integer: std_i32,
    }
}

fn assert_sane_type_layout(layout: &'static TypeLayout) {
    assert_equal_type_layout(layout, layout);
}

fn assert_equal_type_layout(interface: &'static TypeLayout, impl_: &'static TypeLayout) {
    assert_eq!(check_layout_compatibility(interface, impl_), Ok(()));
}

fn assert_different_type_layout(interface: &'static TypeLayout, impl_: &'static TypeLayout) {
    let res = check_layout_compatibility(interface, impl_);
    assert_ne!(
        res,
        Ok(()),
        "\n\nInterface:{:#?}\n\nimplementation:{:#?}",
        interface,
        impl_,
    );
}

#[repr(transparent)]
#[derive(abi_stable::StableAbi)]
pub struct UnsafeOF {
    #[sabi(unsafe_opaque_field)]
    opaque: Vec<u8>,
}

#[test]
fn unsafe_opaque_fields() {
    let layout = UnsafeOF::LAYOUT;

    let fields = match layout.data() {
        TLData::Struct { fields } => fields.iter().collect::<Vec<_>>(),
        _ => unreachable!(),
    };

    let field_0_ai = fields[0].layout();
    assert_eq!(field_0_ai.data(), TLData::Opaque);
    assert_eq!(field_0_ai.size(), mem::size_of::<Vec<u8>>());
    assert_eq!(field_0_ai.alignment(), mem::align_of::<Vec<u8>>());
}

#[cfg(not(miri))]
#[test]
fn same_different_abi_stability() {
    let must_be_equal = vec![
        regular::Rectangle::LAYOUT,
        swapped_fields_first::Rectangle::LAYOUT,
        swapped_fields_last::Rectangle::LAYOUT,
        removed_field_first::Rectangle::LAYOUT,
        removed_field_last::Rectangle::LAYOUT,
        removed_all_fields::Rectangle::LAYOUT,
        changed_type_first::Rectangle::LAYOUT,
        changed_type_last::Rectangle::LAYOUT,
        shadowed::u32::LAYOUT,
        shadowed::i32::LAYOUT,
    ];

    for this in must_be_equal {
        assert_sane_type_layout(this);
    }

    #[allow(unused_mut)]
    let mut list = vec![
        <&mut ()>::LAYOUT,
        <&mut i32>::LAYOUT,
        <&()>::LAYOUT,
        <&i32>::LAYOUT,
        <&'static &'static ()>::LAYOUT,
        <&'static mut &'static ()>::LAYOUT,
        <&'static &'static mut ()>::LAYOUT,
        <atomic::AtomicPtr<()>>::LAYOUT,
        <atomic::AtomicPtr<i32>>::LAYOUT,
        <*const ()>::LAYOUT,
        <*const i32>::LAYOUT,
        <*mut ()>::LAYOUT,
        <*mut i32>::LAYOUT,
        <[(); 0]>::LAYOUT,
        <[(); 1]>::LAYOUT,
        <[(); 2]>::LAYOUT,
        <[(); 3]>::LAYOUT,
        <[u32; 3]>::LAYOUT,
        <i32>::LAYOUT,
        <u32>::LAYOUT,
        <f32>::LAYOUT,
        <f64>::LAYOUT,
        <bool>::LAYOUT,
        <atomic::AtomicBool>::LAYOUT,
        <atomic::AtomicIsize>::LAYOUT,
        <atomic::AtomicUsize>::LAYOUT,
        <num::NonZeroU32>::LAYOUT,
        <num::NonZeroU16>::LAYOUT,
        <ptr::NonNull<()>>::LAYOUT,
        <ptr::NonNull<i32>>::LAYOUT,
        <RHashMap<RString, RString>>::LAYOUT,
        <RHashMap<RString, i32>>::LAYOUT,
        <RHashMap<i32, RString>>::LAYOUT,
        <RHashMap<i32, i32>>::LAYOUT,
        <RVec<()>>::LAYOUT,
        <RVec<i32>>::LAYOUT,
        <RSlice<'_, ()>>::LAYOUT,
        <RSlice<'_, i32>>::LAYOUT,
        <RSliceMut<'_, ()>>::LAYOUT,
        <RSliceMut<'_, i32>>::LAYOUT,
        <Option<&()>>::LAYOUT,
        <Option<&u32>>::LAYOUT,
        <Option<extern "C" fn()>>::LAYOUT,
        <ROption<()>>::LAYOUT,
        <ROption<u32>>::LAYOUT,
        <RCowVal<'_, u8>>::LAYOUT,
        <RCowVal<'_, u16>>::LAYOUT,
        <RCowStr<'_>>::LAYOUT,
        <RCowSlice<'_, u32>>::LAYOUT,
        <RCowSlice<'_, u64>>::LAYOUT,
        <RArc<()>>::LAYOUT,
        <RArc<u32>>::LAYOUT,
        <RBox<()>>::LAYOUT,
        <RBox<u32>>::LAYOUT,
        <RBoxError>::LAYOUT,
        <SendRBoxError>::LAYOUT,
        <UnsyncRBoxError>::LAYOUT,
        <RCmpOrdering>::LAYOUT,
        <PhantomData<()>>::LAYOUT,
        <PhantomData<(u8,)>>::LAYOUT,
        <PhantomData<(u8, u16)>>::LAYOUT,
        <PhantomData<(u8, u16, u32)>>::LAYOUT,
        <PhantomData<RString>>::LAYOUT,
        // NonOwningPhantom and PhantomData share the same type layout,
        // so the NonOwningPhantom types here have to be different
        <NonOwningPhantom<(RVec<()>,)>>::LAYOUT,
        <NonOwningPhantom<(RVec<()>, u16)>>::LAYOUT,
        <NonOwningPhantom<(RVec<()>, u16, u32)>>::LAYOUT,
        <NonOwningPhantom<RVec<u32>>>::LAYOUT,
        <RMutex<()>>::LAYOUT,
        <RMutex<RString>>::LAYOUT,
        <RRwLock<()>>::LAYOUT,
        <RRwLock<RString>>::LAYOUT,
        <RSender<()>>::LAYOUT,
        <RSender<RString>>::LAYOUT,
        <RReceiver<()>>::LAYOUT,
        <RReceiver<RString>>::LAYOUT,
        <ROnce>::LAYOUT,
        <mod_0::Mod>::LAYOUT,
        <mod_0b::Mod>::LAYOUT,
        <mod_1::Mod>::LAYOUT,
        <mod_2::Mod>::LAYOUT,
        <mod_3::Mod>::LAYOUT,
        <mod_4::Mod>::LAYOUT,
        <mod_5::Mod>::LAYOUT,
        <mod_6::Mod>::LAYOUT,
        <mod_6b::Mod>::LAYOUT,
        <mod_7::Mod>::LAYOUT,
        <union_1a::Union>::LAYOUT,
        <union_1b::Union>::LAYOUT,
        <union_2a::Union>::LAYOUT,
        <union_2b::Union>::LAYOUT,
        <union_3::Union>::LAYOUT,
        <union_4::Union>::LAYOUT,
        <enum_extra_fields_a::Enum>::LAYOUT,
        <enum_extra_fields_b::Enum>::LAYOUT,
        <gen_more_lts_b::Generics<'_>>::LAYOUT,
        <gen_more_lts_c::Generics<'_>>::LAYOUT,
        <gen_more_lts_d::Generics<'_>>::LAYOUT,
        <fn_safe::Fn>::LAYOUT,
        <fn_unsafe::Fn>::LAYOUT,
    ];

    #[cfg(not(feature = "no_fn_promotion"))]
    {
        use tagging_items::*;
        list.extend(vec![
            <Tagged<tagging_items::TAG_DEFAULT_1>>::LAYOUT,
            <Tagged<tagging_items::TAG_DEFAULT_2>>::LAYOUT,
            <Tagged<tagging_items::TAG_DEFAULT_3>>::LAYOUT,
            <Tagged<tagging_items::TAG_DEFAULT_4>>::LAYOUT,
            <Tagged<tagging_items::TAG_DEFAULT_5>>::LAYOUT,
            <Tagged<tagging_items::TAG_DEFAULT_6>>::LAYOUT,
        ]);
    }

    {
        list.extend(vec![
            <[u8; 0]>::LAYOUT,
            <[u8; 1]>::LAYOUT,
            <[u8; 31]>::LAYOUT,
            <[u8; 32]>::LAYOUT,
            <[u8; 41]>::LAYOUT,
            <[u8; 42]>::LAYOUT,
        ]);
    }

    let (_dur, ()) = core_extensions::measure_time::measure(|| {
        for (i, this) in list.iter().cloned().enumerate() {
            for (j, other) in list.iter().cloned().enumerate() {
                if i == j {
                    assert_equal_type_layout(this, other);
                } else {
                    assert_different_type_layout(this, other);
                }
            }
        }

        for this in [
            <UnsafeIgnoredType<()>>::LAYOUT,
            <UnsafeIgnoredType<RString>>::LAYOUT,
        ] {
            assert_equal_type_layout(<UnsafeIgnoredType<()>>::LAYOUT, this)
        }
    });

    // println!("taken {} to check all listed layouts", dur);
}

#[cfg(miri)]
#[test]
fn same_different_abi_stability() {
    let l0 = <&()>::LAYOUT;
    let l1 = <&i32>::LAYOUT;
    let l2 = <mod_4::Mod>::LAYOUT;
    let l3 = <mod_5::Mod>::LAYOUT;
    let l6 = <ROption<()>>::LAYOUT;
    let l7 = <ROption<u32>>::LAYOUT;

    assert_equal_type_layout(l0, l0);
    assert_different_type_layout(l0, l1);
    assert_different_type_layout(l0, l2);

    assert_equal_type_layout(l2, l2);
    assert_different_type_layout(l3, l2);

    #[cfg(not(feature = "no_fn_promotion"))]
    {
        use tagging_items::*;
        let l4 = <Tagged<TAG_DEFAULT_5>>::LAYOUT;
        let l5 = <Tagged<TAG_DEFAULT_6>>::LAYOUT;

        assert_different_type_layout(l2, l4);

        assert_equal_type_layout(l4, l4);
        assert_different_type_layout(l4, l5);
        assert_different_type_layout(l4, l2);

        assert_equal_type_layout(l6, l6);
        assert_different_type_layout(l6, l7);
        assert_different_type_layout(l6, l4);
    }
}

#[cfg_attr(not(miri), test)]
fn compare_references() {
    let list = vec![
        <&mut ()>::LAYOUT,
        <&mut i32>::LAYOUT,
        <&()>::LAYOUT,
        <&i32>::LAYOUT,
        <&'static &'static ()>::LAYOUT,
        <&'static mut &'static ()>::LAYOUT,
        <&'static &'static mut ()>::LAYOUT,
    ];

    let (_dur, ()) = core_extensions::measure_time::measure(|| {
        for (i, this) in list.iter().cloned().enumerate() {
            for (j, other) in list.iter().cloned().enumerate() {
                if i == j {
                    assert_equal_type_layout(this, other);
                } else {
                    assert_different_type_layout(this, other);
                }
            }
        }

        for this in [
            <UnsafeIgnoredType<()>>::LAYOUT,
            <UnsafeIgnoredType<RString>>::LAYOUT,
        ] {
            assert_equal_type_layout(<UnsafeIgnoredType<()>>::LAYOUT, this)
        }
    });
}

#[cfg(test)]
fn different_zeroness() {
    const ZEROABLE_ABI: &TypeLayout = &{ <&()>::LAYOUT._set_is_nonzero(false) };

    let non_zero = <&()>::LAYOUT;

    assert!(non_zero.is_nonzero());
    assert!(!ZEROABLE_ABI.is_nonzero());

    let errs = check_layout_compatibility(non_zero, ZEROABLE_ABI)
        .unwrap_err()
        .flatten_errors();
    assert!(errs
        .iter()
        .any(|err| matches!(err, AbiInstability::NonZeroness { .. })));
}

#[test]
fn different_name() {
    let regular = regular::Rectangle::LAYOUT;
    let other = changed_name::Rectangleiiiiii::LAYOUT;
    let errs = check_layout_compatibility(regular, other)
        .unwrap_err()
        .flatten_errors();
    assert!(errs
        .iter()
        .any(|err| matches!(err, AbiInstability::Name { .. })));
}

#[test]
fn different_field_name() {
    let regular = regular::Rectangle::LAYOUT;
    let other = changed_field_name::Rectangle::LAYOUT;

    let fields = match other.data() {
        TLData::Struct { fields } => fields.iter().collect::<Vec<_>>(),
        _ => unreachable!(),
    };

    assert_eq!(fields[0].name(), "x");
    assert_eq!(fields[1].name(), "y");
    assert_eq!(fields[2].name(), "w2");

    let errs = check_layout_compatibility(regular, other)
        .unwrap_err()
        .flatten_errors();
    assert!(errs
        .iter()
        .any(|err| matches!(err, AbiInstability::UnexpectedField { .. })));
}

#[test]
fn swapped_fields() {
    let regular = regular::Rectangle::LAYOUT;
    let first = swapped_fields_first::Rectangle::LAYOUT;
    let last = swapped_fields_first::Rectangle::LAYOUT;

    for other in [first, last] {
        let errs = check_layout_compatibility(regular, other)
            .unwrap_err()
            .flatten_errors();
        assert!(errs
            .iter()
            .any(|x| matches!(x, AbiInstability::UnexpectedField { .. })))
    }
}

#[test]
fn removed_fields() {
    let regular = regular::Rectangle::LAYOUT;
    let list = [
        removed_field_first::Rectangle::LAYOUT,
        removed_field_last::Rectangle::LAYOUT,
        removed_all_fields::Rectangle::LAYOUT,
    ];

    for other in list {
        let errs = check_layout_compatibility(regular, other)
            .unwrap_err()
            .flatten_errors();
        let mut found_field_count_mismatch = false;
        let mut is_size_mismatch = false;
        for err in errs {
            match err {
                AbiInstability::FieldCountMismatch { .. } => {
                    found_field_count_mismatch = true;
                }
                AbiInstability::Size { .. } => {
                    is_size_mismatch = true;
                }
                _ => {}
            }
        }
        assert!(found_field_count_mismatch);
        assert!(is_size_mismatch);
    }
}

#[cfg(test)]
fn different_alignment() {
    let regular = regular::Rectangle::LAYOUT;
    let other = changed_alignment::Rectangle::LAYOUT;
    let errs = check_layout_compatibility(regular, other)
        .unwrap_err()
        .flatten_errors();

    let found_alignment_mismatch = errs
        .iter()
        .any(|err| matches!(err, AbiInstability::Alignment { .. }));

    assert!(found_alignment_mismatch);
}

//////////////////////////////////////////////////////////
//// Generics
//////////////////////////////////////////////////////////

pub(super) mod gen_more_lts_c {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Generics<'a> {
        x: &'static (),
        y: &'a &'static &'static &'static &'static &'static &'static (),
    }
}

pub(super) mod gen_more_lts_d {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Generics<'a> {
        x: &'static (),
        y: &'a &'a &'a &'a &'static &'static &'static (),
    }
}

pub(super) mod gen_more_tys {
    use super::{PhantomData, Tuple2};
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Generics<T: 'static, U> {
        x: &'static T,
        y: &'static T,
        _marker: PhantomData<Tuple2<T, U>>,
    }
}

// pub(super) mod gen_more_consts{
// For when const-generics are usable
// #[repr(C)]
// #[derive(abi_stable::StableAbi)]
//// pub struct ExtraConstParam<T,const LEN:usize> {
//     x:T,
//     y:T,
//     _marker:PhantomData<(T,[u8;LEN])>,
// }
// }

#[cfg(test)]
fn different_generics() {
    let regular = gen_basic::Generics::<()>::LAYOUT;

    {
        let list = vec![
            gen_more_lts::Generics::<()>::LAYOUT,
            // gen_more_tys::Generics::<(), ()>::LAYOUT,
        ];

        for other in list {
            let errs = check_layout_compatibility(regular, other)
                .unwrap_err()
                .flatten_errors();
            assert!(errs
                .iter()
                .any(|err| matches!(err, AbiInstability::GenericParamCount { .. })));
        }
    }

    {
        let list = vec![gen_more_lts::Generics::<()>::LAYOUT];

        for other in list {
            let errs = check_layout_compatibility(regular, other)
                .unwrap_err()
                .flatten_errors();
            assert!(errs
                .iter()
                .any(|err| matches!(err, AbiInstability::FieldLifetimeMismatch { .. })));
        }
    }
}

//////////////////////////////////////////////////////////
////    Function pointers
//////////////////////////////////////////////////////////

pub(super) mod fn_safe {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Fn(extern "C" fn(u8));
}
pub(super) mod fn_unsafe {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Fn(unsafe extern "C" fn(u8));
}

//////////////////////////////////////////////////////////
////    Enums
//////////////////////////////////////////////////////////

pub(super) mod enum_extra_fields_a {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub enum Enum {
        Variant0,
        Variant1 { a: u32, b: u32 },
    }
}

pub(super) mod misnamed_variant {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub enum Enum {
        Variant000000000,
        Variant1 { a: u32 },
    }
}

#[cfg(test)]
fn variant_mismatch() {
    let regular = basic_enum::Enum::LAYOUT;

    {
        let other = misnamed_variant::Enum::LAYOUT;
        let errs = check_layout_compatibility(regular, other)
            .unwrap_err()
            .flatten_errors();
        assert!(errs
            .iter()
            .any(|err| matches!(err, AbiInstability::UnexpectedVariant { .. })));
    }

    {
        let other = extra_variant::Enum::LAYOUT;
        let errs = check_layout_compatibility(regular, other)
            .unwrap_err()
            .flatten_errors();
        assert!(errs
            .iter()
            .any(|err| matches!(err, AbiInstability::TooManyVariants { .. })));
    }
}

//////////////////////////////////////////////////////////////////////////////
///  Modules,with function pointers

pub(super) mod mod_0 {
    use abi_stable::std_types::RString;

    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Mod {
        pub function_0: extern "C" fn(&mut u32, u64) -> RString,
        pub function_1: extern "C" fn() -> RString,
    }
}

pub(super) mod mod_0b {
    use abi_stable::std_types::RString;

    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Mod {
        pub function_0: extern "C" fn(&mut u32, u64, ()) -> RString,
        pub function_1: extern "C" fn() -> RString,
    }
}

pub(super) mod mod_1 {
    use abi_stable::std_types::RString;
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Mod {
        pub function_0: extern "C" fn(&mut u32, u64, RString),
        pub function_1: extern "C" fn(RString),
    }
}

pub(super) mod mod_2 {
    use abi_stable::std_types::RString;
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Mod {
        pub function_0: extern "C" fn(&mut u32, u64, RString) -> RString,
        pub function_1: extern "C" fn(),
    }
}

pub(super) mod mod_3 {
    use abi_stable::std_types::RString;
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Mod {
        pub function_0: extern "C" fn(&mut u32, u64, RString),
        pub function_1: extern "C" fn() -> RString,
    }
}

pub(super) mod mod_4 {
    use abi_stable::std_types::RString;
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Mod {
        pub function_0: extern "C" fn() -> RString,
        pub function_1: extern "C" fn(&mut u32, u64, RString),
    }
}

pub(super) mod mod_6 {
    use abi_stable::std_types::RString;
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Mod {
        pub function_0: extern "C" fn() -> RString,
    }
}

// Changing only the return type
pub(super) mod mod_6b {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Mod {
        pub function_0: extern "C" fn() -> u32,
    }
}

//////////////////////////////////////////////////////////////////////////////
////            Tagged values
//////////////////////////////////////////////////////////////////////////////

#[cfg(not(feature = "no_fn_promotion"))]
mod tagging_items {
    use super::*;
    use core_extensions::matches;

    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    #[sabi(
        not_stableabi(M),
        bound(M: ToTagConst),
        tag = <M as ToTagConst>::TAG,
    )]
    pub struct Tagged<M>(UnsafeIgnoredType<M>);

    pub trait ToTagConst {
        const TAG: Tag;
    }

    macro_rules! declare_tags {
        (
            $(const $marker_ty:ident = $tag:expr;)*
        ) => (
            $(
                #[repr(C)]
                #[derive(GetStaticEquivalent)]
                pub struct $marker_ty;

                impl ToTagConst for $marker_ty {
                    const TAG:Tag=$tag;
                }
            )*
        )
    }

    declare_tags! {
        const TAG_DEFAULT_0=Tag::null();
        const TAG_DEFAULT_1=Tag::bool_(false);
        const TAG_DEFAULT_2=Tag::int(0);
        const TAG_DEFAULT_3=Tag::uint(0);
        const TAG_DEFAULT_4=Tag::str("");
        const TAG_DEFAULT_5=Tag::arr(RSlice::EMPTY);
        const TAG_DEFAULT_6=Tag::set(RSlice::EMPTY);

        const TAG_EMPTY_SET=Tag::set(RSlice::EMPTY);

        const TAG_SET_A0=Tag::set(rslice![
            Tag::str("Sync"),
        ]);
        const TAG_SET_A1=Tag::set(rslice![
            Tag::str("Send"),
        ]);
        const TAG_SET_A2=Tag::set(rslice![
            Tag::str("Copy"),
        ]);
        const TAG_SET_A3=Tag::set(rslice![
            Tag::str("Clone"),
        ]);
        const TAG_SET_B0=Tag::set(rslice![
            Tag::str("Send"),
            Tag::str("Sync"),
        ]);
        const TAG_SET_B1=Tag::set(rslice![
            Tag::str("Copy"),
            Tag::str("Clone"),
        ]);

        const TAG_SET_C0=Tag::set(rslice![
            Tag::str("Send"),
            Tag::str("Sync"),
            Tag::str("Copy"),
            Tag::str("Clone"),
        ]);

        const TAG_SET_C1=Tag::set(rslice![
            Tag::str("Debug"),
            Tag::str("Display"),
        ]);
    }

    trait TaggedExt {
        const GET_AI: &'static TypeLayout;
    }

    impl<T> TaggedExt for T
    where
        Tagged<T>: StableAbi,
    {
        const GET_AI: &'static TypeLayout = <Tagged<T> as StableAbi>::LAYOUT;
    }

    #[cfg(not(miri))]
    #[test]
    fn test_tag_subsets() {
        let valid_subsets = vec![
            vec![
                TAG_EMPTY_SET::GET_AI,
                TAG_SET_A0::GET_AI,
                TAG_SET_B0::GET_AI,
                TAG_SET_C0::GET_AI,
            ],
            vec![
                TAG_EMPTY_SET::GET_AI,
                TAG_SET_A1::GET_AI,
                TAG_SET_B0::GET_AI,
                TAG_SET_C0::GET_AI,
            ],
            vec![
                TAG_EMPTY_SET::GET_AI,
                TAG_SET_A2::GET_AI,
                TAG_SET_B1::GET_AI,
                TAG_SET_C0::GET_AI,
            ],
            vec![
                TAG_EMPTY_SET::GET_AI,
                TAG_SET_A3::GET_AI,
                TAG_SET_B1::GET_AI,
                TAG_SET_C0::GET_AI,
            ],
        ];

        for subset in &valid_subsets {
            for (l_i, l_abi) in subset.iter().enumerate() {
                for (r_i, r_abi) in subset.iter().enumerate() {
                    let res = check_layout_compatibility(l_abi, r_abi);

                    if l_i <= r_i {
                        assert_eq!(res, Ok(()));
                    } else {
                        let errs = res.unwrap_err().flatten_errors();
                        assert!(errs
                            .iter()
                            .any(|err| matches!(err, AbiInstability::TagError { .. })));
                    }
                }
            }
        }
    }

    #[cfg(not(miri))]
    #[test]
    fn test_tag_incompatible() {
        let incompatible_sets = vec![
            vec![
                TAG_SET_A0::GET_AI,
                TAG_SET_A1::GET_AI,
                TAG_SET_A2::GET_AI,
                TAG_SET_A3::GET_AI,
                TAG_SET_C1::GET_AI,
            ],
            vec![TAG_SET_B0::GET_AI, TAG_SET_B1::GET_AI],
            vec![TAG_SET_C0::GET_AI, TAG_SET_C1::GET_AI],
        ];

        for subset in &incompatible_sets {
            for (l_i, l_abi) in subset.iter().enumerate() {
                for (r_i, r_abi) in subset.iter().enumerate() {
                    let res = check_layout_compatibility(l_abi, r_abi);

                    if l_i == r_i {
                        assert_eq!(res, Ok(()));
                    } else {
                        let errs = res.unwrap_err().flatten_errors();
                        assert!(errs
                            .iter()
                            .any(|err| matches!(err, AbiInstability::TagError { .. })));
                    }
                }
            }
        }
    }
}
