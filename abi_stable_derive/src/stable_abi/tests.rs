use crate::derive_stable_abi_from_str as derive_sabi;

use abi_stable_shared::test_utils::must_panic;

use as_derive_utils::test_framework::Tests;

/// For testing that adding #[repr(C)] makes the derive macro not panic.
const RECTANGLE_DEF_REPR: &str = r##"
    pub struct Rectangle {
        x:u32,
        y:u32,
        w:u16,
        h:u32,
    }
"##;

#[test]
fn test_cases() {
    Tests::load("stable_abi").run_test(derive_sabi);
}

#[test]
fn check_struct_repr_attrs() {
    let rect_def = RECTANGLE_DEF_REPR;

    must_panic(|| derive_sabi(rect_def).unwrap()).expect("TEST BUG");

    let invalid_reprs = vec![
        "Rust", "u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64", "usize", "isize",
    ];

    for invalid_repr in invalid_reprs {
        let with_repr_rust = format!(
            "#[repr({repr})]\n{struct_def}",
            repr = invalid_repr,
            struct_def = rect_def,
        );
        assert!(derive_sabi(&with_repr_rust).is_err())
    }

    derive_sabi(&format!("#[repr(C)]\n{}", rect_def)).unwrap();
    derive_sabi(&format!("#[repr(transparent)]\n{}", rect_def)).unwrap();
}
