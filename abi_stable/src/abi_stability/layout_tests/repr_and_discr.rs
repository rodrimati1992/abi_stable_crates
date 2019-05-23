use abi_stable_derive_lib::derive_stable_abi_from_str as derive_sabi;

use crate::{
    StableAbi,
    abi_stability::{
        abi_checking::{AbiInstability,check_layout_compatibility},
        AbiInfoWrapper,
    },
    test_utils::must_panic,
};



#[test]
fn test_discriminant_gen_code(){
    let list=vec![
        ("u8" ,"TLDiscriminant::from_u8","DiscriminantRepr::U8"),
        ("u16","TLDiscriminant::from_u16","DiscriminantRepr::U16"),
        ("u32","TLDiscriminant::from_u32","DiscriminantRepr::U32"),
        ("u64","TLDiscriminant::from_u64","DiscriminantRepr::U64"),
        ("i8" ,"TLDiscriminant::from_i8","DiscriminantRepr::I8"),
        ("i16","TLDiscriminant::from_i16","DiscriminantRepr::I16"),
        ("i32","TLDiscriminant::from_i32","DiscriminantRepr::I32"),
        ("i64","TLDiscriminant::from_i64","DiscriminantRepr::I64"),
        ("usize","TLDiscriminant::Usize","DiscriminantRepr::Usize"),
        ("isize","TLDiscriminant::Isize","DiscriminantRepr::Isize"),
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

        assert_eq!(output.matches("TLDiscriminant").count(),3);
        assert_eq!(output.matches("DiscriminantRepr").count(),1);

        assert!(output.contains(tl_discr)  );
        assert!(output.contains(discr_repr));
        assert!(output.contains("(10)"));
        assert!(output.contains("(20)"));
        assert!(output.contains("(30)"));
    }
}


/// For testing that adding #[repr(C)] makes the derive macro not panic.
mod derive_validity_0 {
    and_stringify! {
        pub(super)const RECTANGLE_DEF_REPR;

        pub struct Rectangle {
            x:u32,
            y:u32,
            w:u16,
            h:u32,
        }
    }
}


#[test]
fn check_struct_repr_attrs(){

    let rect_def=derive_validity_0::RECTANGLE_DEF_REPR;

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



macro_rules! declare_int_repr {
    ( 
        mod=$mod_ident:ident 
        repr=$repr:ident 
        discriminants( $($discr_a:expr)* , $($discr_b:expr)* , $($discr_c:expr)* ) 
    ) => (
        mod $mod_ident{
            use crate::StableAbi;

            #[repr($repr)]
            #[derive(StableAbi)]
            pub enum What{
                A $(=$discr_a)* ,
                B $(=$discr_b)* ,
                C $(=$discr_c)* ,
            }
        }
    )
}

declare_int_repr!{
    mod=c_repr_a
    repr=C
    discriminants(,,)
}

declare_int_repr!{
    mod=c_repr_b
    repr=C
    discriminants(10,,)
}

declare_int_repr!{
    mod=u8_repr_a
    repr=u8
    discriminants(,,)
}

declare_int_repr!{
    mod=u8_repr_b
    repr=u8
    discriminants(10,,)
}

declare_int_repr!{
    mod=u16_repr_a
    repr=u16
    discriminants(,,)
}

declare_int_repr!{
    mod=usize_repr_a
    repr=usize
    discriminants(,,)
}

declare_int_repr!{
    mod=i8_repr_a
    repr=i8
    discriminants(,,)
}

declare_int_repr!{
    mod=i8_repr_b
    repr=i8
    discriminants(10,,)
}

declare_int_repr!{
    mod=i16_repr_a
    repr=i16
    discriminants(,,)
}

declare_int_repr!{
    mod=isize_repr_a
    repr=isize
    discriminants(,,)
}


fn check_imcompatible_with_others<F>(list:&[&'static AbiInfoWrapper],mut f:F)
where
    F:FnMut(&[AbiInstability])
{
    for (l_i,l_abi) in list.iter().enumerate() {
        for (r_i,r_abi) in list.iter().enumerate() {

            let res=check_layout_compatibility(l_abi,r_abi);

            if l_i == r_i {
                assert_eq!(res,Ok(()));
            }else{
                let errs=res.unwrap_err().flatten_errors();

                f(&*errs);
            }
        }
    }

}


#[test]
fn check_discriminant_repr_enums(){
    let list=&[
        <c_repr_a::What as StableAbi>::ABI_INFO,
        <c_repr_b::What as StableAbi>::ABI_INFO,
        <u8_repr_a::What as StableAbi>::ABI_INFO,
        <u8_repr_b::What as StableAbi>::ABI_INFO,
        <u16_repr_a::What as StableAbi>::ABI_INFO,
        <usize_repr_a::What as StableAbi>::ABI_INFO,
        <i8_repr_a::What as StableAbi>::ABI_INFO,
        <i8_repr_b::What as StableAbi>::ABI_INFO,
        <i16_repr_a::What as StableAbi>::ABI_INFO,
        <isize_repr_a::What as StableAbi>::ABI_INFO,
    ];

    check_imcompatible_with_others(list,|errs|{
        let mut repr_attr_errs=0;
        let mut enum_discr_errs=0;

        let mut had_some_err=false;
        for err in errs {
            match err {
                AbiInstability::ReprAttr{..}=>{
                    repr_attr_errs+=1;
                    had_some_err=true;
                }
                AbiInstability::EnumDiscriminant{..}=>{
                    enum_discr_errs+=1;
                    had_some_err=true;
                }
                _=>{}
            }
        }
        assert!(had_some_err,"\nerrors:{:#?}\n",errs);
    })



}


