#![allow(dead_code)]

#[allow(unused_imports)]
use core_extensions::{matches, SelfOps};

use rand::{seq::SliceRandom, thread_rng};

use abi_stable::{
    abi_stability::abi_checking::{
        check_layout_compatibility_with_globals, AbiInstability, CheckingGlobals,
    },
    prefix_type::{PrefixTypeTrait, WithMetadata, __PrefixTypeMetadata},
    test_utils::{file_span, must_panic},
    type_layout::TypeLayout,
    type_level::bools::*,
    *,
};

fn custom_default<T>() -> T
where
    T: From<u8>,
{
    101.into()
}

mod prefix0 {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    // #[sabi(debug_print)]
    #[sabi(kind(Prefix))]
    pub struct Prefix {
        #[sabi(last_prefix_field)]
        pub field0: u8,
    }
}

mod prefix1 {
    use super::custom_default;
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    #[sabi(
        // debug_print,
        kind(Prefix),
        missing_field(with="custom_default::<_>"),
    )]
    pub struct Prefix {
        #[sabi(last_prefix_field)]
        pub field0: u8,
        pub field1: u16,
    }
}

mod prefix2 {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    #[sabi(kind(Prefix))]
    #[sabi(missing_field(default))]
    pub struct Prefix {
        #[sabi(last_prefix_field)]
        pub field0: u8,
        pub field1: u16,
        pub field2: u32,
    }
}

// Prefix types have to keep the same alignment when fields are added
mod prefix2_misaligned {
    #[repr(C, align(16))]
    #[derive(abi_stable::StableAbi)]
    // #[sabi(debug_print)]
    #[sabi(kind(Prefix))]
    pub struct Prefix {
        #[sabi(last_prefix_field)]
        pub field0: u8,
        pub field1: u16,
        pub field2: u32,
    }
}

mod prefix2_different_prefix {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    #[sabi(kind(Prefix))]
    pub struct Prefix {
        pub field0: u8,
        #[sabi(last_prefix_field)]
        pub field1: u16,
        pub field2: u32,
    }
}

mod prefix3 {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    #[sabi(kind(Prefix))]
    #[sabi(missing_field(panic))]
    pub struct Prefix {
        #[sabi(last_prefix_field)]
        pub field0: u8,
        pub field1: u16,
        pub field2: u32,
        pub field3: u64,
    }
}

/// Dereferences the TypeLayout of a `&T` to the layout of `T`
fn dereference_abi(abi: &'static TypeLayout) -> &'static TypeLayout {
    abi.phantom_fields().get(0).unwrap().layout()
}

static PREF_0: &'static TypeLayout = <prefix0::Prefix_Ref>::LAYOUT;
static PREF_1: &'static TypeLayout = <prefix1::Prefix_Ref>::LAYOUT;
static PREF_2: &'static TypeLayout = <prefix2::Prefix_Ref>::LAYOUT;
static PREF_3: &'static TypeLayout = <prefix3::Prefix_Ref>::LAYOUT;

fn new_list() -> Vec<&'static TypeLayout> {
    vec![PREF_0, PREF_1, PREF_2, PREF_3]
}

#[cfg_attr(not(miri), test)]
fn prefixes_test() {
    let list = new_list();

    let mut rng = thread_rng();

    fn gen_elem_from(
        abi_wrapper: &'static TypeLayout,
    ) -> (&'static TypeLayout, __PrefixTypeMetadata) {
        let prefix = abi_wrapper
            .piped(dereference_abi)
            .piped(__PrefixTypeMetadata::new);
        (abi_wrapper, prefix)
    }

    let mut gen_generation = |skip_first: usize| {
        let mut ret = Vec::<(&'static TypeLayout, __PrefixTypeMetadata)>::new();
        for _ in 0..list.len() {
            let pushed = gen_elem_from(list.choose(&mut rng).unwrap().clone());
            ret.push(pushed);
        }
        let max_size = ret.iter().map(|(_, x)| x.fields.len()).max().unwrap();
        ret.extend(
            list.iter()
                .cloned()
                .skip(skip_first)
                .take(max_size)
                .map(gen_elem_from),
        );
        ret
    };

    for _ in 0..200 {
        let globals = CheckingGlobals::new();

        let t_list = gen_generation(0);
        let o_list = gen_generation(1);

        for ((this, t_prefix), (other, o_prefix)) in
            t_list.iter().cloned().zip(o_list.iter().cloned())
        {
            let prefix_type_map = globals.prefix_type_map.lock().unwrap();
            let value_len = prefix_type_map.value_len();
            let key_len = prefix_type_map.key_len();
            // Not dropping it here causes a deadlock inside
            // check_layout_compatibility_with_globals.
            drop(prefix_type_map);

            let res = check_layout_compatibility_with_globals(this, other, &globals);

            let prefix_type_map = globals.prefix_type_map.lock().unwrap();
            if t_prefix.fields.len() <= o_prefix.fields.len() {
                res.unwrap_or_else(|e| panic!("{:#?}", e));

                let deref_this = dereference_abi(this);
                let deref_other = dereference_abi(other);
                let t_id = deref_this.get_utypeid();
                let o_id = deref_other.get_utypeid();

                let t_map_prefix = prefix_type_map.get(&t_id);
                let o_map_prefix = prefix_type_map.get(&o_id);

                let t_map_prefix = t_map_prefix.unwrap();
                let o_map_prefix = o_map_prefix.unwrap();

                for pre in vec![o_prefix.clone(), t_map_prefix.clone(), o_map_prefix.clone()] {
                    assert_eq!(t_prefix.prefix_field_count, pre.prefix_field_count,);
                    for (l_field, r_field) in t_prefix.fields.iter().zip(pre.fields.iter()) {
                        assert_eq!(l_field, r_field);
                    }
                }

                assert!(t_prefix.fields.len() <= t_map_prefix.fields.len());
                assert!(o_prefix.fields.len() <= o_map_prefix.fields.len());

                assert_eq!(t_map_prefix as *const _, o_map_prefix as *const _);
            } else {
                assert_eq!(value_len, prefix_type_map.value_len());
                assert_eq!(key_len, prefix_type_map.key_len());

                let errs = res.unwrap_err().flatten_errors();
                assert!(errs
                    .iter()
                    .any(|err| matches!(err, AbiInstability::FieldCountMismatch { .. })),);
            }
        }

        let prefix_type_map = globals.prefix_type_map.lock().unwrap();

        let max_prefix = t_list
            .iter()
            .zip(o_list.iter())
            .map(|((_, l_prefix), (_, r_prefix))| (l_prefix.clone(), r_prefix.clone()))
            .filter(|(l, r)| l.fields.len() <= r.fields.len())
            .map(|(l, r)| __PrefixTypeMetadata::max(l, r))
            .max_by_key(|prefix| prefix.fields.len())
            .unwrap();

        // Asserting that the layout they all map to is the one with the most fields
        for this in list.iter().cloned() {
            let id = dereference_abi(this).get_utypeid();

            // The random sanpling did not include this type.
            let prefix = match prefix_type_map.get(&id) {
                Some(x) => x,
                None => continue,
            };

            for (l_field, r_field) in prefix.fields.iter().zip(max_prefix.fields.iter()) {
                assert_eq!(l_field, r_field);
            }
            assert_eq!(prefix.fields.len(), max_prefix.fields.len(),);
        }
    }
}

fn check_interface_impl_pair(
    globals: &CheckingGlobals,
    this: &'static TypeLayout,
    other: &'static TypeLayout,
) {
    let deref_this = dereference_abi(this);
    let deref_other = dereference_abi(other);

    let t_prefix = __PrefixTypeMetadata::new(deref_this);
    let o_prefix = __PrefixTypeMetadata::new(deref_other);

    if let Err(e) = check_layout_compatibility_with_globals(this, other, &globals) {
        if t_prefix.fields.len() <= o_prefix.fields.len() {
            panic!("{:#?}", e);
        } else {
            return;
        }
    }

    let prefix_type_map = globals.prefix_type_map.lock().unwrap();

    let t_id = deref_this.get_utypeid();
    let o_id = deref_other.get_utypeid();

    let t_map_prefix = prefix_type_map.get(&t_id);
    let o_map_prefix = prefix_type_map.get(&o_id);

    let t_map_prefix = t_map_prefix.unwrap();
    let o_map_prefix = o_map_prefix.unwrap();

    for pre in vec![o_prefix.clone(), t_map_prefix.clone(), o_map_prefix.clone()] {
        assert_eq!(t_prefix.prefix_field_count, pre.prefix_field_count,);
        for (field_i, (l_field, r_field)) in
            t_prefix.fields.iter().zip(pre.fields.iter()).enumerate()
        {
            if t_prefix.accessible_fields.is_accessible(field_i)
                && o_prefix.accessible_fields.is_accessible(field_i)
            {
                assert_eq!(
                    l_field, r_field,
                    "\nleft:{:#?}\n\nright:{:#?}\n",
                    l_field, r_field
                );
            }
        }
    }

    assert!(t_prefix.fields.len() <= t_map_prefix.fields.len());
    assert!(o_prefix.fields.len() <= o_map_prefix.fields.len());

    assert_eq!(t_map_prefix as *const _, o_map_prefix as *const _);
}

#[cfg_attr(not(miri), test)]
fn hierarchical_prefix_test() {
    let library_00 = PREF_2;
    let library_01 = PREF_1;
    let library_02 = PREF_0;

    let library_10 = PREF_2;
    let library_11 = PREF_2;
    let library_12 = PREF_3;

    let library_0 = PREF_0;
    let library_1 = PREF_1;

    let binary = PREF_0;

    let mut rng = thread_rng();

    for _ in 0..100 {
        let globals = CheckingGlobals::new();
        let mut checks = vec![
            (binary, library_0),
            (binary, library_1),
            (library_0, library_00),
            (library_0, library_01),
            (library_0, library_02),
            (library_1, library_10),
            (library_1, library_11),
            (library_1, library_12),
        ];
        checks.shuffle(&mut rng);

        for (this, other) in checks {
            check_interface_impl_pair(&globals, this, other);
        }
    }
}

#[cfg_attr(not(miri), test)]
fn prefix_is_same_alignment() {
    let globals = CheckingGlobals::new();
    let misaligned = <prefix2_misaligned::Prefix_Ref>::LAYOUT;

    for pref in vec![PREF_0, PREF_1] {
        let errs = check_layout_compatibility_with_globals(pref, misaligned, &globals)
            .unwrap_err()
            .flatten_errors();

        assert!(errs
            .iter()
            .any(|err| matches!(err, AbiInstability::Alignment { .. })));
    }
}

#[cfg_attr(not(miri), test)]
fn prefix_is_same_size() {
    let globals = CheckingGlobals::new();
    let list = new_list();

    for pref in list.iter().cloned() {
        let mismatched_prefix = <prefix2_different_prefix::Prefix_Ref>::LAYOUT;
        let errs = check_layout_compatibility_with_globals(pref, mismatched_prefix, &globals)
            .unwrap_err()
            .flatten_errors();

        assert!(errs
            .iter()
            .any(|err| matches!(err, AbiInstability::MismatchedPrefixSize { .. })));
    }
}

#[cfg_attr(not(miri), test)]
fn prefix_on_nonexistent_field() {
    pub const MOD_VAL: &WithMetadata<prefix0::Prefix> =
        &WithMetadata::new(PrefixTypeTrait::METADATA, prefix0::Prefix { field0: 1 });

    let prefix0 = MOD_VAL.static_as_prefix();

    {
        let value1: prefix1::Prefix_Ref = unsafe { std::mem::transmute(prefix0) };
        assert_eq!(value1.field0(), 1);
        assert_eq!(value1.field1(), custom_default::<u16>());
    }
    {
        let value2: prefix2::Prefix_Ref = unsafe { std::mem::transmute(prefix0) };
        assert_eq!(value2.field0(), 1);
        assert_eq!(value2.field1(), 0);
        assert_eq!(value2.field2(), 0);
    }
    {
        let value3: prefix3::Prefix_Ref = unsafe { std::mem::transmute(prefix0) };
        assert_eq!(value3.field0(), 1);
        must_panic(file_span!(), || value3.field1()).unwrap();
        must_panic(file_span!(), || value3.field2()).unwrap();
        must_panic(file_span!(), || value3.field3()).unwrap();
    }
}

/////////////////////////////////////////////////////////////////////////

pub trait EnabledFields {
    const ENABLE_FIELD_0: bool = true;
    const ENABLE_FIELD_1: bool = true;
    const ENABLE_FIELD_2: bool = true;
    const ENABLE_FIELD_3: bool = true;
}

impl<B0, B1, B2, B3> EnabledFields for (B0, B1, B2, B3)
where
    B0: Boolean,
    B1: Boolean,
    B2: Boolean,
    B3: Boolean,
{
    const ENABLE_FIELD_0: bool = <B0 as Boolean>::VALUE;
    const ENABLE_FIELD_1: bool = <B1 as Boolean>::VALUE;
    const ENABLE_FIELD_2: bool = <B2 as Boolean>::VALUE;
    const ENABLE_FIELD_3: bool = <B3 as Boolean>::VALUE;
}

macro_rules! declare_enabled_fields {
    (
        $struct:ident {
            $($trait_definition:tt)*
        }
    ) => {
        pub struct $struct;

        impl EnabledFields for $struct {
            $($trait_definition)*
        }
    }
}

declare_enabled_fields! {
    ACCESSIBLE_ALL {
        const ENABLE_FIELD_0:bool=true;
        const ENABLE_FIELD_1:bool=true;
        const ENABLE_FIELD_2:bool=true;
        const ENABLE_FIELD_3:bool=true;
    }
}

declare_enabled_fields! {
    ACCESSIBLE_ALL_EXCEPT_0 {
        const ENABLE_FIELD_0:bool=false;
        const ENABLE_FIELD_1:bool=true;
        const ENABLE_FIELD_2:bool=true;
        const ENABLE_FIELD_3:bool=true;
    }
}

declare_enabled_fields! {
    ACCESSIBLE_ALL_EXCEPT_1 {
        const ENABLE_FIELD_0:bool=true;
        const ENABLE_FIELD_1:bool=false;
        const ENABLE_FIELD_2:bool=true;
        const ENABLE_FIELD_3:bool=true;
    }
}

declare_enabled_fields! {
    ACCESSIBLE_ALL_EXCEPT_2 {
        const ENABLE_FIELD_0:bool=true;
        const ENABLE_FIELD_1:bool=true;
        const ENABLE_FIELD_2:bool=false;
        const ENABLE_FIELD_3:bool=true;
    }
}

declare_enabled_fields! {
    ACCESSIBLE_ALL_EXCEPT_3 {
        const ENABLE_FIELD_0:bool=true;
        const ENABLE_FIELD_1:bool=true;
        const ENABLE_FIELD_2:bool=true;
        const ENABLE_FIELD_3:bool=false;
    }
}

static COND_FIELD_0_ALL: &'static TypeLayout = <cond_fields_0::Prefix_Ref<ACCESSIBLE_ALL>>::LAYOUT;

static COND_FIELD_1_ALL: &'static TypeLayout = <cond_fields_1::Prefix_Ref<ACCESSIBLE_ALL>>::LAYOUT;

static COND_FIELD_2_ALL: &'static TypeLayout = <cond_fields_2::Prefix_Ref<ACCESSIBLE_ALL>>::LAYOUT;

static COND_FIELD_3_ALL: &'static TypeLayout = <cond_fields_3::Prefix_Ref<ACCESSIBLE_ALL>>::LAYOUT;

static COND_FIELD_0_EXCEPT_0: &'static TypeLayout =
    <cond_fields_0::Prefix_Ref<ACCESSIBLE_ALL_EXCEPT_0>>::LAYOUT;

static COND_FIELD_1_EXCEPT_0: &'static TypeLayout =
    <cond_fields_1::Prefix_Ref<ACCESSIBLE_ALL_EXCEPT_0>>::LAYOUT;

static COND_FIELD_2_EXCEPT_0: &'static TypeLayout =
    <cond_fields_2::Prefix_Ref<ACCESSIBLE_ALL_EXCEPT_0>>::LAYOUT;

static COND_FIELD_3_EXCEPT_0: &'static TypeLayout =
    <cond_fields_3::Prefix_Ref<ACCESSIBLE_ALL_EXCEPT_0>>::LAYOUT;

static COND_FIELD_1_EXCEPT_1: &'static TypeLayout =
    <cond_fields_1::Prefix_Ref<ACCESSIBLE_ALL_EXCEPT_1>>::LAYOUT;

static COND_FIELD_2_EXCEPT_1: &'static TypeLayout =
    <cond_fields_2::Prefix_Ref<ACCESSIBLE_ALL_EXCEPT_1>>::LAYOUT;

static COND_FIELD_3_EXCEPT_1: &'static TypeLayout =
    <cond_fields_3::Prefix_Ref<ACCESSIBLE_ALL_EXCEPT_1>>::LAYOUT;

static COND_FIELD_2_EXCEPT_2: &'static TypeLayout =
    <cond_fields_2::Prefix_Ref<ACCESSIBLE_ALL_EXCEPT_2>>::LAYOUT;

static COND_FIELD_3_EXCEPT_2: &'static TypeLayout =
    <cond_fields_3::Prefix_Ref<ACCESSIBLE_ALL_EXCEPT_2>>::LAYOUT;

static COND_FIELD_3_EXCEPT_3: &'static TypeLayout =
    <cond_fields_3::Prefix_Ref<ACCESSIBLE_ALL_EXCEPT_3>>::LAYOUT;

mod cond_fields_0 {
    use super::EnabledFields;
    use abi_stable::marker_type::UnsafeIgnoredType;
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    #[sabi(
        kind(Prefix),
        prefix_bound = "C:EnabledFields",
        unsafe_unconstrained(C)
    )]
    pub struct Prefix<C> {
        pub _marker: UnsafeIgnoredType<C>,
        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_0 ")]
        #[sabi(last_prefix_field)]
        pub field0: u8,
    }
}

mod cond_fields_1 {
    use super::EnabledFields;
    use abi_stable::marker_type::UnsafeIgnoredType;

    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    #[sabi(
        kind(Prefix),
        prefix_bound = "C:EnabledFields",
        unsafe_unconstrained(C)
    )]
    pub struct Prefix<C, T = u8, U = u16> {
        pub _marker: UnsafeIgnoredType<C>,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_0 ")]
        #[sabi(accessor_bound = "Copy")]
        #[sabi(last_prefix_field)]
        pub field0: T,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_1 ")]
        #[sabi(accessor_bound = "Copy")]
        pub field1: U,
    }
}

mod cond_fields_2 {
    use super::EnabledFields;
    use abi_stable::marker_type::UnsafeIgnoredType;
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    #[sabi(
        kind(Prefix),
        prefix_bound = "C:EnabledFields",
        unsafe_unconstrained(C)
    )]
    pub struct Prefix<C, T = u8, U = u16, V = u32> {
        pub _marker: UnsafeIgnoredType<C>,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_0 ")]
        #[sabi(last_prefix_field)]
        #[sabi(accessor_bound = "Copy")]
        pub field0: T,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_1 ")]
        #[sabi(accessor_bound = "Copy")]
        pub field1: U,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_2 ")]
        #[sabi(accessor_bound = "Copy")]
        pub field2: V,
    }
}

// Prefix types have to keep the same alignment when fields are added
mod cond_fields_2_misaligned {
    use super::EnabledFields;
    use abi_stable::marker_type::UnsafeIgnoredType;
    #[repr(C, align(16))]
    #[derive(abi_stable::StableAbi)]
    #[sabi(
        kind(Prefix),
        prefix_bound = "C:EnabledFields",
        unsafe_unconstrained(C)
    )]
    pub struct Prefix<C> {
        pub _marker: UnsafeIgnoredType<C>,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_0 ")]
        #[sabi(last_prefix_field)]
        pub field0: u8,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_1 ")]
        pub field1: u16,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_2 ")]
        pub field2: u32,
    }
}

mod cond_fields_2_different_prefix {
    use super::EnabledFields;
    use abi_stable::marker_type::UnsafeIgnoredType;
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    #[sabi(
        kind(Prefix),
        prefix_bound = "C:EnabledFields",
        unsafe_unconstrained(C)
    )]
    pub struct Prefix<C, T = u8, U = u16, V = u32> {
        pub _marker: UnsafeIgnoredType<C>,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_0 ")]
        #[sabi(accessor_bound = "Copy")]
        pub field0: T,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_1 ")]
        #[sabi(last_prefix_field)]
        #[sabi(accessor_bound = "Copy")]
        pub field1: U,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_2 ")]
        #[sabi(accessor_bound = "Copy")]
        pub field2: V,
    }
}

mod cond_fields_3 {
    use super::EnabledFields;
    use abi_stable::marker_type::UnsafeIgnoredType;
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    #[sabi(
        // debug_print,
        kind(Prefix),
        prefix_bound="C:EnabledFields",
        unsafe_unconstrained(C),
    )]
    pub struct Prefix<C, T = u8, U = u16, V = u32, W = u64> {
        pub _marker: UnsafeIgnoredType<(C, T, U, V, W)>,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_0 ")]
        #[sabi(last_prefix_field)]
        #[sabi(accessor_bound = "Copy")]
        pub field0: T,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_1 ")]
        #[sabi(accessor_bound = "Copy")]
        pub field1: U,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_2 ")]
        #[sabi(accessor_bound = "Copy")]
        pub field2: V,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_3 ")]
        #[sabi(accessor_bound = "Copy")]
        pub field3: W,
    }
}

mod cond_fields_3_uncond_prefix {
    use super::EnabledFields;
    use abi_stable::marker_type::UnsafeIgnoredType;
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    #[sabi(
        // debug_print,
        kind(Prefix),
        prefix_bound="C:EnabledFields",
        unsafe_unconstrained(C),
    )]
    pub struct Prefix<C, T = u8, U = u16, V = u32, W = u64> {
        pub _marker: UnsafeIgnoredType<(C, T, U, V, W)>,

        #[sabi(last_prefix_field)]
        #[sabi(accessor_bound = "Copy")]
        pub field0: T,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_1 ")]
        #[sabi(accessor_bound = "Copy")]
        pub field1: U,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_2 ")]
        #[sabi(accessor_bound = "Copy")]
        pub field2: V,

        #[sabi(accessible_if = " <C as EnabledFields>::ENABLE_FIELD_3 ")]
        #[sabi(accessor_bound = "Copy")]
        pub field3: W,
    }
}

#[cfg_attr(not(miri), test)]
fn prefix_cond_field_test() {
    let mut rng = thread_rng();

    use abi_stable::type_level::bools::{False as F, True as T};

    use self::cond_fields_2::Prefix_Ref as Prefix2;
    use self::cond_fields_3::Prefix_Ref as Prefix3;
    use self::cond_fields_3_uncond_prefix::Prefix_Ref as Prefix3UncondPrefix;

    type au32 = [u32; 1];
    type ai32 = [i32; 1];

    let mut valid_lists = vec![
        vec![
            <Prefix3<(F, F, F, F), ai32, ai32, ai32, ai32>>::LAYOUT,
            <Prefix3<(T, F, F, F), i32, ai32, ai32, ai32>>::LAYOUT,
            <Prefix3<(T, T, F, F), i32, i32, ai32, ai32>>::LAYOUT,
            <Prefix3<(T, T, T, F), i32, i32, i32, ai32>>::LAYOUT,
            <Prefix3<(T, T, T, T), i32, i32, i32, i32>>::LAYOUT,
            <Prefix3<(T, T, T, T), i32, i32, i32, i32>>::LAYOUT,
        ],
        vec![
            <Prefix3<(T, T, T, T), i32, i32, i32, i32>>::LAYOUT,
            <Prefix3<(F, T, T, T), ai32, i32, i32, i32>>::LAYOUT,
            <Prefix3<(F, F, T, T), ai32, ai32, i32, i32>>::LAYOUT,
            <Prefix3<(F, F, F, T), ai32, ai32, ai32, i32>>::LAYOUT,
            <Prefix3<(F, F, F, F), ai32, ai32, ai32, ai32>>::LAYOUT,
            <Prefix3<(T, T, T, T), i32, i32, i32, i32>>::LAYOUT,
            <Prefix3<(T, T, T, T), i32, i32, i32, i32>>::LAYOUT,
        ],
        vec![
            <Prefix2<(F, T, F, F), au32, i32, au32>>::LAYOUT,
            <Prefix3<(T, F, T, T), i32, au32, i32, i32>>::LAYOUT,
            <Prefix3<(T, T, T, T), i32, i32, i32, i32>>::LAYOUT,
            <Prefix3<(T, T, T, T), i32, i32, i32, i32>>::LAYOUT,
        ],
    ];

    let invalid = vec![
        (
            <Prefix3<(T, F, T, F), i32, au32, i32, i32>>::LAYOUT,
            <Prefix2<(F, T, F, F), au32, i32, au32>>::LAYOUT,
        ),
        (
            <Prefix2<(F, T, F, F), au32, i32, au32>>::LAYOUT,
            <Prefix3<(T, T, T, T), i32, au32, i32, i32>>::LAYOUT,
        ),
        (
            <Prefix3<(T, T, T, T), i32, i32, i32, i32>>::LAYOUT,
            <Prefix3<(T, T, T, T), i32, au32, i32, i32>>::LAYOUT,
        ),
        (
            <Prefix3<(T, T, T, T), i32, i32, au32, i32>>::LAYOUT,
            <Prefix3<(F, F, T, F), i32, i32, i32, i32>>::LAYOUT,
        ),
        (
            <Prefix3UncondPrefix<(T, T, T, T), i32, i32, au32, i32>>::LAYOUT,
            <Prefix3<(T, T, T, T), i32, i32, au32, i32>>::LAYOUT,
        ),
    ];

    for valid_list in &valid_lists {
        let globals = CheckingGlobals::new();
        for window in valid_list.windows(2) {
            check_interface_impl_pair(&globals, window[0], window[1]);
        }
    }

    for _ in 0..50 {
        for valid_list in &mut valid_lists {
            let globals = CheckingGlobals::new();
            valid_list.shuffle(&mut rng);

            for window in valid_list.windows(2) {
                check_interface_impl_pair(&globals, window[0], window[1]);
            }
        }
    }

    const CHECKED_ERRORS: usize = 3;

    let mut err_counts = vec![0; CHECKED_ERRORS];

    {
        let mut inc_on_err = |conds: [bool; CHECKED_ERRORS]| -> bool {
            let mut any = false;
            for (i, &cond) in conds.iter().enumerate() {
                any = any || cond;
                err_counts[i] += cond as usize;
            }
            any
        };

        let globals = CheckingGlobals::new();
        for (interf, impl_) in invalid {
            let errs = check_layout_compatibility_with_globals(interf, impl_, &globals)
                .unwrap_err()
                .flatten_errors();

            assert!(errs.iter().any(|e| {
                inc_on_err([
                    matches!(e, AbiInstability::FieldCountMismatch { .. }),
                    matches!(e, AbiInstability::Name { .. }),
                    matches!(e, AbiInstability::MismatchedPrefixConditionality { .. }),
                ])
            }),);
        }
    }

    assert!(
        err_counts.iter().all(|&x| x != 0),
        "err_counts:{:#?}",
        err_counts,
    );
}

#[cfg_attr(not(miri), test)]
fn hierarchical_prefix_cond_field_test() {
    let mut rng = thread_rng();

    let fields_0 = vec![COND_FIELD_0_ALL, COND_FIELD_0_EXCEPT_0];

    let fields_1 = vec![
        COND_FIELD_1_ALL,
        COND_FIELD_1_EXCEPT_0,
        COND_FIELD_1_EXCEPT_1,
    ];

    let fields_2 = vec![
        COND_FIELD_2_ALL,
        COND_FIELD_2_EXCEPT_0,
        COND_FIELD_2_EXCEPT_1,
        COND_FIELD_2_EXCEPT_2,
    ];

    let fields_3 = vec![
        COND_FIELD_3_ALL,
        COND_FIELD_3_EXCEPT_0,
        COND_FIELD_3_EXCEPT_1,
        COND_FIELD_3_EXCEPT_2,
        COND_FIELD_3_EXCEPT_3,
    ];

    for _ in 0..500 {
        let globals = CheckingGlobals::new();

        let library_00 = fields_2.choose(&mut rng).unwrap().clone();
        let library_01 = fields_1.choose(&mut rng).unwrap().clone();
        let library_02 = fields_0.choose(&mut rng).unwrap().clone();

        let library_10 = fields_2.choose(&mut rng).unwrap().clone();
        let library_11 = fields_2.choose(&mut rng).unwrap().clone();
        let library_12 = fields_3.choose(&mut rng).unwrap().clone();

        let library_0 = fields_0.choose(&mut rng).unwrap().clone();
        let library_1 = fields_1.choose(&mut rng).unwrap().clone();

        let binary = fields_0.choose(&mut rng).unwrap().clone();

        let mut checks = vec![
            (binary, library_0),
            (binary, library_1),
            (library_0, library_00),
            (library_0, library_01),
            (library_0, library_02),
            (library_1, library_10),
            (library_1, library_11),
            (library_1, library_12),
        ];
        checks.shuffle(&mut rng);

        for (this, other) in checks {
            check_interface_impl_pair(&globals, this, other);
        }
    }
}

#[test]
fn prefix_on_conditional_fields() {
    use abi_stable::{
        marker_type::UnsafeIgnoredType,
        type_level::bools::{False as F, True as T},
    };

    type Prefix1_Ref<AF> = cond_fields_1::Prefix_Ref<AF, i8, i32>;
    type Prefix2_Ref<AF> = cond_fields_2::Prefix_Ref<AF, i8, i32, i32>;
    type Prefix3_Ref<AF> = cond_fields_3::Prefix_Ref<AF, i8, i32, i32, i32>;
    type Prefix3UncondPrefix_Ref<AF> =
        cond_fields_3_uncond_prefix::Prefix_Ref<AF, i8, i32, i32, i32>;

    {
        // Casting Prefix0 to Prefix1 with different field accessibilities
        pub const MOD_VAL: &WithMetadata<cond_fields_0::Prefix<(T, T, T, T)>> = &WithMetadata::new(
            PrefixTypeTrait::METADATA,
            cond_fields_0::Prefix {
                _marker: UnsafeIgnoredType::DEFAULT,
                field0: 1,
            },
        );

        let prefix0 = MOD_VAL.static_as_prefix();

        {
            // The field cannot be accessed even though it was initialized.
            let value: Prefix1_Ref<(F, F, F, F)> = unsafe { std::mem::transmute(prefix0) };

            assert_eq!(value.field0(), None);
            assert_eq!(value.field1(), None);
        }
        {
            // The first field can be accessed.
            let value: Prefix1_Ref<(T, F, F, F)> = unsafe { std::mem::transmute(prefix0) };

            assert_eq!(value.field0(), Some(1));
            assert_eq!(value.field1(), None);
        }
    }

    pub const MOD_VAL_P3: &WithMetadata<cond_fields_3::Prefix<(T, T, T, T), i8, i32, i32, i32>> =
        &WithMetadata::new(
            PrefixTypeTrait::METADATA,
            cond_fields_3::Prefix {
                _marker: UnsafeIgnoredType::DEFAULT,
                field0: 1,
                field1: 3,
                field2: 7,
                field3: 12,
            },
        );

    let prefix3 = MOD_VAL_P3.static_as_prefix();

    {
        // Casting Prefix3 to Prefix2 with different field accessibilities
        {
            let value: Prefix2_Ref<(F, F, F, F)> = unsafe { std::mem::transmute(prefix3) };

            assert_eq!(value.field0(), None);
            assert_eq!(value.field1(), None);
            assert_eq!(value.field2(), None);
        }
        {
            let value: Prefix2_Ref<(T, F, F, F)> = unsafe { std::mem::transmute(prefix3) };

            assert_eq!(value.field0(), Some(1));
            assert_eq!(value.field1(), None);
            assert_eq!(value.field2(), None);
        }
        {
            let value: Prefix2_Ref<(F, T, F, F)> = unsafe { std::mem::transmute(prefix3) };

            assert_eq!(value.field0(), None);
            assert_eq!(value.field1(), Some(3));
            assert_eq!(value.field2(), None);
        }
        {
            let value: Prefix2_Ref<(F, F, T, F)> = unsafe { std::mem::transmute(prefix3) };

            assert_eq!(value.field0(), None);
            assert_eq!(value.field1(), None);
            assert_eq!(value.field2(), Some(7));
        }
        {
            let value: Prefix2_Ref<(T, T, T, T)> = unsafe { std::mem::transmute(prefix3) };

            assert_eq!(value.field0(), Some(1));
            assert_eq!(value.field1(), Some(3));
            assert_eq!(value.field2(), Some(7));
        }
    }

    {
        // Casting Prefix3 to Prefix3UncondPrefix with different field accessibilities
        {
            let value: Prefix3UncondPrefix_Ref<(F, F, F, F)> =
                unsafe { std::mem::transmute(prefix3) };

            assert_eq!(value.field0(), 1);
            assert_eq!(value.field1(), None);
            assert_eq!(value.field2(), None);
            assert_eq!(value.field2(), None);
        }
        {
            let value: Prefix3UncondPrefix_Ref<(F, F, T, F)> =
                unsafe { std::mem::transmute(prefix3) };

            assert_eq!(value.field0(), 1);
            assert_eq!(value.field1(), None);
            assert_eq!(value.field2(), Some(7));
            assert_eq!(value.field3(), None);
        }
        {
            let value: Prefix3UncondPrefix_Ref<(T, T, T, T)> =
                unsafe { std::mem::transmute(prefix3) };

            assert_eq!(value.field0(), 1);
            assert_eq!(value.field1(), Some(3));
            assert_eq!(value.field2(), Some(7));
            assert_eq!(value.field3(), Some(12));
        }
    }
}
