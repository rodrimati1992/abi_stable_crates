use abi_stable::{
    abi_stability::abi_checking::{check_layout_compatibility, AbiInstability},
    type_layout::TypeLayout,
    StableAbi,
};

macro_rules! declare_int_repr {
    (
        mod=$mod_ident:ident
        repr=$repr:ident
        discriminants( $($discr_a:expr)* , $($discr_b:expr)* , $($discr_c:expr)* )
        discr_ty=$discr_ty:ty,
        check=( $($variant:ident=$discr_value:expr),* $(,)* )
    ) => (
        mod $mod_ident{
            use abi_stable::StableAbi;

            #[repr($repr)]
            #[derive(StableAbi)]
            #[allow(dead_code)]
            pub enum What{
                A $(=$discr_a)* ,
                B $(=$discr_b)* ,
                C $(=$discr_c)* ,
            }

            #[test]
            fn check_discriminant_values(){
                $(
                    assert_eq!(What::$variant as $discr_ty,$discr_value );
                )*
            }
        }
    )
}

declare_int_repr! {
    mod=c_repr_a
    repr=C
    discriminants(,,)
    discr_ty=isize,
    check=( A=0,B=1,C=2, )
}

declare_int_repr! {
    mod=c_repr_b
    repr=C
    discriminants(10,,)
    discr_ty=isize,
    check=( A=10,B=11,C=12, )
}

declare_int_repr! {
    mod=u8_repr_a
    repr=u8
    discriminants(,,)
    discr_ty=u8,
    check=( A=0,B=1,C=2, )
}

declare_int_repr! {
    mod=u8_repr_b
    repr=u8
    discriminants(10,,)
    discr_ty=u8,
    check=( A=10,B=11,C=12, )
}

declare_int_repr! {
    mod=u16_repr_a
    repr=u16
    discriminants(,,)
    discr_ty=u16,
    check=( A=0,B=1,C=2, )
}

declare_int_repr! {
    mod=usize_repr_a
    repr=usize
    discriminants(,,)
    discr_ty=usize,
    check=( A=0,B=1,C=2, )
}

declare_int_repr! {
    mod=i8_repr_a
    repr=i8
    discriminants(,,)
    discr_ty=i8,
    check=( A=0,B=1,C=2, )
}

declare_int_repr! {
    mod=i8_repr_b
    repr=i8
    discriminants(10,,)
    discr_ty=i8,
    check=( A=10,B=11,C=12, )
}

declare_int_repr! {
    mod=i8_repr_c
    repr=i8
    discriminants(,10,)
    discr_ty=i8,
    check=( A=0,B=10,C=11, )
}

declare_int_repr! {
    mod=i8_repr_d
    repr=i8
    discriminants(,,10)
    discr_ty=i8,
    check=( A=0,B=1,C=10, )
}

declare_int_repr! {
    mod=i16_repr_a
    repr=i16
    discriminants(,,)
    discr_ty=i16,
    check=( A=0,B=1,C=2, )
}

declare_int_repr! {
    mod=isize_repr_a
    repr=isize
    discriminants(,,)
    discr_ty=isize,
    check=( A=0,B=1,C=2, )
}

#[cfg(not(miri))]
fn check_imcompatible_with_others<F>(list: &[&'static TypeLayout], mut f: F)
where
    F: FnMut(&[AbiInstability]),
{
    for (l_i, l_abi) in list.iter().enumerate() {
        for (r_i, r_abi) in list.iter().enumerate() {
            let res = check_layout_compatibility(l_abi, r_abi);

            if l_i == r_i {
                assert_eq!(res, Ok(()));
            } else {
                let errs = res.unwrap_err().flatten_errors();

                f(&*errs);
            }
        }
    }
}

fn assert_discr_error(errs: &[AbiInstability]) {
    let mut had_some_err = false;
    for err in errs {
        match err {
            AbiInstability::ReprAttr { .. } => {
                had_some_err = true;
            }
            AbiInstability::EnumDiscriminant { .. } => {
                had_some_err = true;
            }
            _ => {}
        }
    }
    assert!(had_some_err, "\nerrors:{:#?}\n", errs);
}

#[cfg(not(miri))]
#[test]
fn check_discriminant_repr_enums() {
    let list = &[
        <c_repr_a::What as StableAbi>::LAYOUT,
        <c_repr_b::What as StableAbi>::LAYOUT,
        <u8_repr_a::What as StableAbi>::LAYOUT,
        <u8_repr_b::What as StableAbi>::LAYOUT,
        <u16_repr_a::What as StableAbi>::LAYOUT,
        <usize_repr_a::What as StableAbi>::LAYOUT,
        <i8_repr_a::What as StableAbi>::LAYOUT,
        <i8_repr_b::What as StableAbi>::LAYOUT,
        <i16_repr_a::What as StableAbi>::LAYOUT,
        <isize_repr_a::What as StableAbi>::LAYOUT,
    ];

    check_imcompatible_with_others(list, assert_discr_error)
}

#[cfg(miri)]
#[test]
fn check_discriminant_repr_enums() {
    let l0 = <c_repr_a::What as StableAbi>::LAYOUT;
    let l1 = <c_repr_b::What as StableAbi>::LAYOUT;
    let l2 = <u8_repr_a::What as StableAbi>::LAYOUT;

    assert_eq!(check_layout_compatibility(l0, l0), Ok(()));

    assert_discr_error(
        &check_layout_compatibility(l0, l1)
            .unwrap_err()
            .flatten_errors(),
    );

    assert_discr_error(
        &check_layout_compatibility(l0, l2)
            .unwrap_err()
            .flatten_errors(),
    );
}
