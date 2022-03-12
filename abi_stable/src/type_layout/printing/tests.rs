use super::RECURSIVE_INDICATOR;

use crate::{
    abi_stability::stable_abi_trait::get_type_layout, sabi_types::Constructor, std_types::RSlice,
    test_utils::AlwaysDisplay, type_layout::TypeLayout, StableAbi,
};

mod display {
    use super::*;

    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(phantom_const_param = STRUCT_0_LAYS)]
    pub(super) struct Struct0;

    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(phantom_const_param = STRUCT_1_LAYS)]
    pub(super) struct Struct1;

    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(phantom_const_param = STRUCT_2_LAYS)]
    pub(super) struct Struct2;

    const STRUCT_0_LAYS: RSlice<'static, AlwaysDisplay<Constructor<&'static TypeLayout>>> =
        rslice![];

    const STRUCT_1_LAYS: RSlice<'static, AlwaysDisplay<Constructor<&'static TypeLayout>>> =
        rslice![AlwaysDisplay(Constructor(get_type_layout::<Struct1>)),];

    const STRUCT_2_LAYS: RSlice<'static, AlwaysDisplay<Constructor<&'static TypeLayout>>> = rslice![
        AlwaysDisplay(Constructor(get_type_layout::<Struct2>)),
        AlwaysDisplay(Constructor(get_type_layout::<Struct2>)),
    ];
}

#[test]
fn recursive_display() {
    let list = vec![
        <display::Struct0 as StableAbi>::LAYOUT,
        <display::Struct1 as StableAbi>::LAYOUT,
        <display::Struct2 as StableAbi>::LAYOUT,
    ];

    for (i, layout) in list.into_iter().enumerate() {
        let matches = layout.to_string().matches(RECURSIVE_INDICATOR).count();
        assert_eq!(matches, i);
        {
            let full_type = layout.full_type();
            let matches = full_type.to_string().matches(RECURSIVE_INDICATOR).count();
            assert_eq!(matches, i, "\n{}\n", full_type);
            let name_matches = full_type.to_string().matches("align").count();
            assert_eq!(name_matches, i, "\n{}\n", full_type);
        }
    }
}

mod debug {
    use super::*;

    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(phantom_const_param = STRUCT_0_LAYS)]
    pub(super) struct Struct0;

    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(phantom_const_param = STRUCT_1_LAYS)]
    pub(super) struct Struct1;

    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(phantom_const_param = STRUCT_2_LAYS)]
    pub(super) struct Struct2;

    const STRUCT_0_LAYS: RSlice<'static, Constructor<&'static TypeLayout>> = rslice![];

    const STRUCT_1_LAYS: RSlice<'static, Constructor<&'static TypeLayout>> =
        rslice![Constructor(get_type_layout::<Struct1>),];

    const STRUCT_2_LAYS: RSlice<'static, Constructor<&'static TypeLayout>> = rslice![
        Constructor(get_type_layout::<Struct2>),
        Constructor(get_type_layout::<Struct2>),
    ];
}

#[test]
fn recursive_debug() {
    let list = vec![
        <debug::Struct0 as StableAbi>::LAYOUT,
        <debug::Struct1 as StableAbi>::LAYOUT,
        <debug::Struct2 as StableAbi>::LAYOUT,
    ];

    for (i, layout) in list.into_iter().enumerate() {
        {
            let formatted = format!("{:#?}", layout);
            let matches = formatted.matches(RECURSIVE_INDICATOR).count();
            assert_eq!(matches, i * 2, "\n{}\n", formatted);
        }
        {
            let full_type = layout.full_type();
            let formatted = format!("{:#?}", full_type);
            let matches = formatted.matches(RECURSIVE_INDICATOR).count();
            assert_eq!(matches, i, "\n{}\n", formatted);
            let name_matches = formatted.to_string().matches("align").count();
            assert_eq!(name_matches, i, "\n{}\n", formatted);
        }
    }
}
