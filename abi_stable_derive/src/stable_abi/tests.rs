use crate::{
    derive_stable_abi_from_str as derive_sabi,
    test_framework::Tests,
};

use abi_stable_shared::{file_span,test_utils::{must_panic}};


/// For testing that adding #[repr(C)] makes the derive macro not panic.
const RECTANGLE_DEF_REPR:&str=r##"
    pub struct Rectangle {
        x:u32,
        y:u32,
        w:u16,
        h:u32,
    }
"##;




#[test]
fn test_cases(){
    Tests::load("stable_abi").run_test(derive_sabi);
}

#[test]
fn check_struct_repr_attrs(){

    let rect_def=RECTANGLE_DEF_REPR;

    must_panic(file_span!(),|| derive_sabi(rect_def).unwrap() ).expect("TEST BUG");
    
    let invalid_reprs=vec![
        "Rust",
        "u8",
        "i8",
        "u16",
        "i16",
        "u32",
        "i32",
        "u64",
        "i64",
        "usize",
        "isize",
    ];

    for invalid_repr in invalid_reprs {
        must_panic(file_span!(),||{
            let with_repr_rust=format!(
                "#[repr({repr})]\n{struct_def}",
                repr=invalid_repr,
                struct_def=rect_def,
            );
            derive_sabi(&with_repr_rust).unwrap();
        }).unwrap_or_else(|_|{
            panic!("invalid_repr={}",invalid_repr);
        });
    }


    derive_sabi(&format!("#[repr(C)]\n{}",rect_def)).unwrap();
    derive_sabi(&format!("#[repr(transparent)]\n{}",rect_def)).unwrap();
}
