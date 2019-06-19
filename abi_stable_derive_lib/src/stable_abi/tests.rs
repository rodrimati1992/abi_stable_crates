use crate::derive_stable_abi_from_str as derive_sabi;

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
        ("u8" ,"TLDiscriminant::from_u8_slice","DiscriminantRepr::U8"),
        ("u16","TLDiscriminant::from_i8_slice","DiscriminantRepr::U16"),
        ("u32","TLDiscriminant::from_u16_slice","DiscriminantRepr::U32"),
        ("u64","TLDiscriminant::from_i16_slice","DiscriminantRepr::U64"),
        ("i8" ,"TLDiscriminant::from_u32_slice","DiscriminantRepr::I8"),
        ("i16","TLDiscriminant::from_i32_slice","DiscriminantRepr::I16"),
        ("i32","TLDiscriminant::from_u64_slice","DiscriminantRepr::I32"),
        ("i64","TLDiscriminant::from_i64_slice","DiscriminantRepr::I64"),
        ("usize","TLDiscriminant::from_usize_slice","DiscriminantRepr::Usize"),
        ("isize","TLDiscriminant::from_isize_slice","DiscriminantRepr::Isize"),
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

        let output=derive_sabi(&input).to_string();

        println!("Output:\n{}\n",output);

        let output=output
            .chars()
            .filter(|c|!c.is_whitespace())
            .collect::<String>();

        assert_eq!(output.matches("TLDiscriminant").count(),1);
        assert_eq!(output.matches("DiscriminantRepr").count(),1);

        assert!(output.contains(tl_discr)  );
        assert!(output.contains(discr_repr));
        assert!(output.contains("10"));
        assert!(output.contains("20"));
        assert!(output.contains("30"));
    }
}




#[test]
fn check_struct_repr_attrs(){

    let rect_def=RECTANGLE_DEF_REPR;

    must_panic(file_span!(),|| derive_sabi(rect_def) ).unwrap();
    
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
            derive_sabi(&with_repr_rust)
        }).unwrap();
    }


    derive_sabi(&format!("#[repr(C)]\n{}",rect_def));
    derive_sabi(&format!("#[repr(transparent)]\n{}",rect_def));
}
