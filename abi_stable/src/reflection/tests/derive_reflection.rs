//! Tests the fields related to reflection generated in the StableAbi derive macro.

use crate::{
    abi_stability::{PrefixStableAbi, StableAbi},
    reflection::ModReflMode,
    std_types::*,
    type_layout::{FieldAccessor, TLData, TLField},
};

#[repr(u8)]
#[derive(StableAbi)]
pub enum PubEnum {
    Variant0,
    Variant1(RString),
    Variant2 { field0: u32 },
}

#[repr(C)]
#[derive(StableAbi)]
#[allow(dead_code)]
enum PrivEnum {
    Variant0,
    Variant1(RString),
    Variant2 { field0: u32 },
}

#[repr(C)]
#[derive(StableAbi)]
//#[sabi(debug_print)]
pub struct RegularPubFields {
    pub field0: u8,
    pub field1: u8,
    #[sabi(refl(pub_getter = what_the))]
    pub field2: u8,
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(module_reflection(Opaque))]
pub struct RegularPubFieldsOpaque {
    pub field0: u8,
    pub field1: u8,
}

#[repr(C)]
#[derive(StableAbi)]
pub struct RegularMostPrivacies {
    pub field0: u8,
    pub(super) field1: u16,
    field2: u32,
}

#[repr(C)]
#[derive(StableAbi)]
pub struct RegularPriv {
    field0: u8,
    field1: u16,
    #[sabi(refl(pub_getter = hello))]
    field2: u32,
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix), missing_field(panic))]
pub struct PrefixPubFields {
    #[sabi(last_prefix_field)]
    pub field0: u8,
    pub field1: u8,
    #[sabi(missing_field(option))]
    pub field2: u8,
    #[sabi(refl(pub_getter = hello))]
    #[sabi(missing_field(default))]
    pub field3: u8,
    #[sabi(missing_field(panic))]
    pub field4: u8,
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix), missing_field(panic), module_reflection(Opaque))]
pub struct PrefixPubFieldsOpaque {
    #[sabi(last_prefix_field)]
    pub field0: u8,
    #[sabi(missing_field(panic))]
    pub field1: u8,
}

#[allow(dead_code)]
mod some_prefixes {
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(kind(Prefix))]
    pub struct PrefMostPrivacies {
        pub field0: u8,
        #[sabi(last_prefix_field)]
        pub field1: u8,
        pub field2: u8,
        pub field3: u8,
        pub(super) field4: u16,
        #[sabi(missing_field(default))]
        field5: u32,
    }

    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(kind(Prefix))]
    pub struct PrefPriv {
        field0: u8,
        pub(super) field1: u16,
        #[sabi(last_prefix_field)]
        #[sabi(missing_field(default))]
        pub(crate) field2: u32,
    }
}

pub use self::some_prefixes::*;

///////////////////////////////////////////////////////////////////////////////

fn check_fields(fields: &[TLField], accessors: &[FieldAccessor]) {
    for (field, expec_acc) in fields.iter().zip(accessors) {
        assert_eq!(
            field.field_accessor(),
            *expec_acc,
            "field:{}\nfields:{:#?}",
            field.name(),
            fields
        );
    }
}

fn check_enum_accessors<T>(mod_refl_mode: ModReflMode, accessors: &[&[FieldAccessor]])
where
    T: StableAbi,
{
    let layout = T::LAYOUT;

    let mut fields = match layout.data() {
        TLData::Enum(enum_) => enum_.fields.iter(),
        x => panic!("layout.data must be TLData::Struct{{..}}:\n{:#?}", x),
    };

    assert_eq!(layout.mod_refl_mode(), mod_refl_mode);

    for expec_vari in accessors {
        let subfields = fields.by_ref().take(expec_vari.len()).collect::<Vec<_>>();
        check_fields(&subfields, expec_vari)
    }
}

fn check_struct_accessors<T>(mod_refl_mode: ModReflMode, accessors: &[FieldAccessor])
where
    T: StableAbi,
{
    let layout = T::LAYOUT;

    let fields = match layout.data() {
        TLData::Struct { fields } => fields.to_vec(),
        x => panic!("layout.data must be TLData::Struct{{..}}:\n{:#?}", x),
    };

    assert_eq!(layout.mod_refl_mode(), mod_refl_mode);

    check_fields(&fields, accessors);
}

fn check_prefix_accessors<T>(mod_refl_mode: ModReflMode, accessors: &[FieldAccessor])
where
    T: PrefixStableAbi,
{
    let layout = T::LAYOUT;

    let fields = match layout.data() {
        TLData::PrefixType(prefix) => prefix.fields.to_vec(),
        x => panic!("layout.data must be TLData::Struct{{..}}:\n{:#?}", x),
    };

    assert_eq!(layout.mod_refl_mode(), mod_refl_mode);

    check_fields(&fields, accessors);
}

///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_pub_enum() {
    check_enum_accessors::<PubEnum>(
        ModReflMode::Module,
        &[&[], &[FieldAccessor::Direct], &[FieldAccessor::Direct]],
    );
}

#[test]
fn test_priv_enum() {
    check_enum_accessors::<PrivEnum>(
        ModReflMode::Opaque,
        &[&[], &[FieldAccessor::Opaque], &[FieldAccessor::Opaque]],
    );
}

///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_regular_pub_fields() {
    check_struct_accessors::<RegularPubFields>(
        ModReflMode::Module,
        &[FieldAccessor::Direct, FieldAccessor::Direct, {
            const FA: FieldAccessor = FieldAccessor::method_named(rstr!("what_the"));
            FA
        }],
    );
}

#[test]
fn test_regular_pub_fields_opaque() {
    check_struct_accessors::<RegularPubFieldsOpaque>(
        ModReflMode::Opaque,
        &[FieldAccessor::Opaque, FieldAccessor::Opaque],
    );
}

#[test]
fn test_regular_most_privacies() {
    check_struct_accessors::<RegularMostPrivacies>(
        ModReflMode::Module,
        &[
            FieldAccessor::Direct,
            FieldAccessor::Opaque,
            FieldAccessor::Opaque,
        ],
    );
}

#[test]
fn test_regular_priv() {
    check_struct_accessors::<RegularPriv>(
        ModReflMode::Opaque,
        &[FieldAccessor::Opaque, FieldAccessor::Opaque, {
            const FA: FieldAccessor = FieldAccessor::method_named(rstr!("hello"));
            FA
        }],
    );
}

///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_prefix_pub_fields() {
    check_prefix_accessors::<PrefixPubFields_Prefix>(
        ModReflMode::Module,
        &[
            FieldAccessor::Method,
            FieldAccessor::Method,
            FieldAccessor::MethodOption,
            {
                const FA: FieldAccessor = FieldAccessor::method_named(rstr!("hello"));
                FA
            },
            FieldAccessor::Method,
        ],
    );
}

#[test]
fn test_prefix_pub_fields_opaque() {
    check_prefix_accessors::<PrefixPubFieldsOpaque_Prefix>(
        ModReflMode::Opaque,
        &[FieldAccessor::Opaque, FieldAccessor::Opaque],
    );
}

#[test]
fn test_prefix_most_privacies() {
    check_prefix_accessors::<PrefMostPrivacies_Prefix>(
        ModReflMode::Module,
        &[
            FieldAccessor::Method,
            FieldAccessor::Method,
            FieldAccessor::MethodOption,
            FieldAccessor::MethodOption,
            FieldAccessor::Opaque,
            FieldAccessor::Opaque,
        ],
    );
}

#[test]
fn test_prefix_priv() {
    check_prefix_accessors::<PrefPriv_Prefix>(
        ModReflMode::Opaque,
        &[
            FieldAccessor::Opaque,
            FieldAccessor::Opaque,
            FieldAccessor::Opaque,
        ],
    );
}
