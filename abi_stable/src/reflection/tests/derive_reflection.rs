/*!
Tests the fields related to reflection generated in the StableAbi derive macro.
*/


use crate::{
    abi_stability::{
        type_layout::{TLData,FieldAccessor,TLField},
        SharedStableAbi,
    },
    reflection::ModReflMode,
    std_types::*,
};


#[repr(u8)]
#[derive(StableAbi)]
pub enum PubEnum {
    Variant0,
    Variant1(RString),
    Variant2{
        field0:u32,
    },
}


#[repr(C)]
#[derive(StableAbi)]
enum PrivEnum {
    Variant0,
    Variant1(RString),
    Variant2{
        field0:u32,
    },
}


#[repr(C)]
#[derive(StableAbi)]
//#[sabi(debug_print)]
pub struct RegularPubFields {
    pub field0: u8,
    pub field1: u8,
    #[sabi(refl(pub_getter="what_the"))]
    pub field2: u8,
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(module_reflection( Opaque ))]
pub struct RegularPubFieldsOpaque {
    pub field0: u8,
    pub field1: u8,
}



#[repr(C)]
#[derive(StableAbi)]
pub struct RegularMostPrivacies {
    pub field0: u8,
    pub(super)field1: u16,
    field2: u32,
}



#[repr(C)]
#[derive(StableAbi)]
pub struct RegularPriv {
    field0: u8,
    field1: u16,
    #[sabi(refl(pub_getter="hello"))]
    field2: u32,
}



#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    kind(Prefix(prefix_struct="PrefixPubFields")),
    missing_field(panic),
)]
pub struct PrefPubFieldsValue {
    #[sabi(last_prefix_field)]
    pub field0: u8,
    pub field1: u8,
    #[sabi(missing_field(option))]
    pub field2: u8,
    #[sabi(refl(pub_getter="hello"))]
    #[sabi(missing_field(default))]
    pub field3: u8,
    #[sabi(missing_field(panic))]
    pub field4: u8,
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    kind(Prefix(prefix_struct="PrefixPubFieldsOpaque")),
    missing_field(panic),
    module_reflection(Opaque),
)]
pub struct PrefixPubFieldsOpaqueValue {
    #[sabi(last_prefix_field)]
    pub field0: u8,
    #[sabi(missing_field(panic))]
    pub field1: u8,
}


#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_struct="PrefixMostPrivacies")))]
pub struct PrefMostPrivaciesValue {
    pub field0: u8,
    #[sabi(last_prefix_field)]
    pub field1: u8,
    pub field2: u8,
    pub field3: u8,
    pub(super)field4: u16,
    #[sabi(missing_field(default))]
    field5: u32,
}


#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_struct="PrefixPriv")))]
pub struct PrefPrivValue {
    field0: u8,
    pub(super) field1: u16,
    #[sabi(last_prefix_field)]
    #[sabi(missing_field(default))]
    pub(crate) field2: u32,
}


///////////////////////////////////////////////////////////////////////////////


fn check_fields(
    fields:&[TLField],
    accessors:&[FieldAccessor],
){
    for (field,expec_acc) in fields.iter().zip( accessors ) {
        assert_eq!(
            field.field_accessor,
            *expec_acc,
            "field:{}\nfields:{:#?}",
            field.name,
            fields
        );
    }
}


fn check_enum_accessors<T>(
    mod_refl_mode:ModReflMode,
    accessors:&[&[FieldAccessor]],
)where
    T:SharedStableAbi
{
    let layout=T::S_ABI_INFO.get().layout;

    let mut fields=match layout.data {
        TLData::Enum{fields,..}=>fields.get_fields(),
        x=>panic!("layout.data must be TLData::Struct{{..}}:\n{:#?}",x)
    };

    assert_eq!(layout.mod_refl_mode,mod_refl_mode);
    
    for expec_vari in accessors {
        let subfields=fields.by_ref().take(expec_vari.len()).collect::<Vec<_>>();
        check_fields(&subfields,expec_vari)
    }
}


fn check_struct_accessors<T>(
    mod_refl_mode:ModReflMode,
    accessors:&[FieldAccessor],
)where
    T:SharedStableAbi
{
    let layout=T::S_ABI_INFO.get().layout;

    let fields=match layout.data {
        TLData::Struct{fields}=>fields.get_field_vec(),
        x=>panic!("layout.data must be TLData::Struct{{..}}:\n{:#?}",x)
    };

    assert_eq!(layout.mod_refl_mode,mod_refl_mode);

    check_fields(&fields,accessors);
}


fn check_prefix_accessors<T>(
    mod_refl_mode:ModReflMode,
    accessors:&[FieldAccessor],
)where
    T:SharedStableAbi
{
    let layout=T::S_ABI_INFO.get().layout;

    let fields=match &layout.data {
        TLData::PrefixType(prefix)=>prefix.fields.get_field_vec(),
        x=>panic!("layout.data must be TLData::Struct{{..}}:\n{:#?}",x)
    };

    assert_eq!(layout.mod_refl_mode,mod_refl_mode);

    check_fields(&fields,accessors);
}

///////////////////////////////////////////////////////////////////////////////


#[test]
fn test_pub_enum(){
    check_enum_accessors::<PubEnum>(
        ModReflMode::Module,
        &[
            &[],
            &[FieldAccessor::Direct],
            &[FieldAccessor::Direct],
        ]
    );
}


#[test]
fn test_priv_enum(){
    check_enum_accessors::<PrivEnum>(
        ModReflMode::Opaque,
        &[
            &[],
            &[FieldAccessor::Opaque],
            &[FieldAccessor::Opaque],
        ]
    );
}



///////////////////////////////////////////////////////////////////////////////


#[test]
fn test_regular_pub_fields(){
    check_struct_accessors::<RegularPubFields>(
        ModReflMode::Module,
        &[
            FieldAccessor::Direct,
            FieldAccessor::Direct,
            {const FA:FieldAccessor=FieldAccessor::method_named(&StaticStr::new("what_the"));FA},
        ]
    );
}


#[test]
fn test_regular_pub_fields_opaque(){
    check_struct_accessors::<RegularPubFieldsOpaque>(
        ModReflMode::Opaque,
        &[
            FieldAccessor::Opaque,
            FieldAccessor::Opaque,
        ]
    );
}


#[test]
fn test_regular_most_privacies(){
    check_struct_accessors::<RegularMostPrivacies>(
        ModReflMode::Module,
        &[
            FieldAccessor::Direct,
            FieldAccessor::Opaque,
            FieldAccessor::Opaque,
        ]
    );
}


#[test]
fn test_regular_priv(){
    check_struct_accessors::<RegularPriv>(
        ModReflMode::Opaque,
        &[
            FieldAccessor::Opaque,
            FieldAccessor::Opaque,
            {const FA:FieldAccessor=FieldAccessor::method_named(&StaticStr::new("hello"));FA},
        ]
    );
}


///////////////////////////////////////////////////////////////////////////////


#[test]
fn test_prefix_pub_fields(){
    check_prefix_accessors::<PrefixPubFields>(
        ModReflMode::Module,
        &[
            FieldAccessor::Method{name:None},
            FieldAccessor::Method{name:None},
            FieldAccessor::MethodOption,
            {const FA:FieldAccessor=FieldAccessor::method_named(&StaticStr::new("hello"));FA},
            FieldAccessor::Method{name:None},
        ]
    );
}


#[test]
fn test_prefix_pub_fields_opaque(){
    check_prefix_accessors::<PrefixPubFieldsOpaque>(
        ModReflMode::Opaque,
        &[
            FieldAccessor::Opaque,
            FieldAccessor::Opaque,
        ]
    );
}


#[test]
fn test_prefix_most_privacies(){
    check_prefix_accessors::<PrefixMostPrivacies>(
        ModReflMode::Module,
        &[
            FieldAccessor::Method{name:None},
            FieldAccessor::Method{name:None},
            FieldAccessor::MethodOption,
            FieldAccessor::MethodOption,
            FieldAccessor::Opaque,
            FieldAccessor::Opaque,
        ]
    );
}


#[test]
fn test_prefix_priv(){
    check_prefix_accessors::<PrefixPriv>(
        ModReflMode::Opaque,
        &[
            FieldAccessor::Opaque,
            FieldAccessor::Opaque,
            FieldAccessor::Opaque,
        ]
    );
}


