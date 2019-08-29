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
fn test_discriminant_gen_code(){
    let list=vec![
        ("u8" ,"__TLDiscriminants::from_u8_slice","__DiscriminantRepr::U8"),
        ("u16","__TLDiscriminants::from_u16_slice","__DiscriminantRepr::U16"),
        ("u32","__TLDiscriminants::from_u32_slice","__DiscriminantRepr::U32"),
        ("u64","__TLDiscriminants::from_u64_slice","__DiscriminantRepr::U64"),
        ("i8" ,"__TLDiscriminants::from_i8_slice","__DiscriminantRepr::I8"),
        ("i16","__TLDiscriminants::from_i16_slice","__DiscriminantRepr::I16"),
        ("i32","__TLDiscriminants::from_i32_slice","__DiscriminantRepr::I32"),
        ("i64","__TLDiscriminants::from_i64_slice","__DiscriminantRepr::I64"),
        ("usize","__TLDiscriminants::from_usize_slice","__DiscriminantRepr::Usize"),
        ("isize","__TLDiscriminants::from_isize_slice","__DiscriminantRepr::Isize"),
    ];

    for (repr_attr,tl_discr,discr_repr) in list {
        let input=format!(r##"
            #[repr({repr_attr})]
            enum What{{
                A=10,
                B=20,
                C=30,
            }}

            "##,
            repr_attr=repr_attr,
        );

        let output=derive_sabi(&input).unwrap().to_string();

        println!("Output:\n{}\n",output);

        let output=output
            .chars()
            .filter(|c|!c.is_whitespace())
            .collect::<String>();

        assert_eq!(output.matches("__TLDiscriminants").count(),1);
        assert_eq!(output.matches("__DiscriminantRepr").count(),1);

        assert!(output.contains(tl_discr)  );
        assert!(output.contains(discr_repr));
        assert!(output.contains("10"));
        assert!(output.contains("20"));
        assert!(output.contains("30"));
    }
}


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
