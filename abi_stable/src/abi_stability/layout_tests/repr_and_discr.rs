use crate::{
    StableAbi,
    abi_stability::{
        abi_checking::{AbiInstability,check_layout_compatibility},
        AbiInfoWrapper,
    },
    test_utils::must_panic,
};


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


