use abi_stable::{
    StableAbi,
    abi_stability::abi_checking::{AbiInstability,check_layout_compatibility},
    type_layout::TypeLayout,
};

#[cfg(feature="const_params")]
mod with_const_generics;


mod one_phantom{
    use abi_stable::{
        const_utils::AssocStr,
        marker_type::UnsafeIgnoredType,
    };
    

    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    #[sabi(
        bound="T:AssocStr",
        phantom_const_param="T::STR",
    )]
    pub struct Struct<T>(UnsafeIgnoredType<T>);
}

mod one_phantom_u8{
    use abi_stable::{
        const_utils::AssocInt,
        marker_type::UnsafeIgnoredType,
    };
    

    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    #[sabi(
        bound="T:AssocInt",
        phantom_const_param="T::NUM",
    )]
    pub struct Struct<T>(UnsafeIgnoredType<T>);
}

mod two_phantom{
    use abi_stable::{
        const_utils::AssocStr,
        marker_type::UnsafeIgnoredType,
        std_types::*,
    };
    

    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    #[sabi(
        bound="T:AssocStr",
        bound="U:AssocStr",
        phantom_const_param="T::STR",
        phantom_const_param="U::STR",
    )]
    pub struct Struct<T,U>(UnsafeIgnoredType<Tuple2<T,U>>);
}



fn check_imcompatible_with_others<F>(list:&[&'static TypeLayout],mut f:F)
where
    F:FnMut(&[AbiInstability])
{
    for (l_i,l_abi) in list.iter().enumerate() {
        for (r_i,r_abi) in list.iter().enumerate() {

            let res=check_layout_compatibility(l_abi,r_abi);

            if l_i == r_i {
                assert_eq!(res,Ok(()));
            }else{
                // dbg!(l_abi.full_type(),r_abi.full_type());
                let errs=res.unwrap_err().flatten_errors();

                let mut had_some_err=false;
                for err in &*errs {
                    match err {
                        AbiInstability::GenericParamCount{..}=>{
                            had_some_err=true;
                        }
                        AbiInstability::MismatchedConstParam{..}=>{
                            had_some_err=true;
                        }
                        _=>{}
                    }
                }
                assert!(had_some_err,"\nerrors:{:#?}\n",errs);
                f(&*errs);
            }
        }
    }
}


/// Takes too long
#[test]
#[cfg(not(miri))]
fn test_compatibility(){
    #[allow(unused_mut)]
    let mut list=vec![
        <one_phantom::Struct<i8> as StableAbi>::LAYOUT,
        <one_phantom::Struct<i16> as StableAbi>::LAYOUT,
        <one_phantom::Struct<i32> as StableAbi>::LAYOUT,
        <one_phantom::Struct<i64> as StableAbi>::LAYOUT,
        <one_phantom_u8::Struct<i8> as StableAbi>::LAYOUT,
        <one_phantom_u8::Struct<i16> as StableAbi>::LAYOUT,
        <one_phantom_u8::Struct<i32> as StableAbi>::LAYOUT,
        <one_phantom_u8::Struct<i64> as StableAbi>::LAYOUT,
        <two_phantom::Struct<i8 ,i8> as StableAbi>::LAYOUT,
        <two_phantom::Struct<i16,i16> as StableAbi>::LAYOUT,
        <two_phantom::Struct<i32,i32> as StableAbi>::LAYOUT,
        <two_phantom::Struct<i64,i64> as StableAbi>::LAYOUT,
        <two_phantom::Struct<i64 ,i8> as StableAbi>::LAYOUT,
        <two_phantom::Struct<i32,i16> as StableAbi>::LAYOUT,
        <two_phantom::Struct<i16,i32> as StableAbi>::LAYOUT,
        <two_phantom::Struct<i8,i64> as StableAbi>::LAYOUT,
    ];

    #[cfg(feature="const_params")]
    {
        use self::with_const_generics::{
            single_integer,
            two_integer,
            single_integer_one_phantom,
        };
        list.extend(vec![
            <single_integer::Struct<100> as StableAbi>::LAYOUT,
            <single_integer::Struct<110> as StableAbi>::LAYOUT,
            <single_integer::Struct<120> as StableAbi>::LAYOUT,
            <single_integer::Struct<130> as StableAbi>::LAYOUT,
            <two_integer::Struct<100,100> as StableAbi>::LAYOUT,
            <two_integer::Struct<110,110> as StableAbi>::LAYOUT,
            <two_integer::Struct<120,120> as StableAbi>::LAYOUT,
            <two_integer::Struct<130,130> as StableAbi>::LAYOUT,
            <two_integer::Struct<100,130> as StableAbi>::LAYOUT,
            <two_integer::Struct<110,120> as StableAbi>::LAYOUT,
            <two_integer::Struct<120,110> as StableAbi>::LAYOUT,
            <two_integer::Struct<130,100> as StableAbi>::LAYOUT,
            <single_integer_one_phantom::Struct<i8 ,100> as StableAbi>::LAYOUT,
            <single_integer_one_phantom::Struct<i16,110> as StableAbi>::LAYOUT,
            <single_integer_one_phantom::Struct<i32,120> as StableAbi>::LAYOUT,
            <single_integer_one_phantom::Struct<i64,130> as StableAbi>::LAYOUT,
            <single_integer_one_phantom::Struct<i64,100> as StableAbi>::LAYOUT,
            <single_integer_one_phantom::Struct<i32,110> as StableAbi>::LAYOUT,
            <single_integer_one_phantom::Struct<i16,120> as StableAbi>::LAYOUT,
            <single_integer_one_phantom::Struct<i8 ,130> as StableAbi>::LAYOUT,
        ]);
    }
    
    check_imcompatible_with_others(&list,|_|());
}



#[test]
fn test_compatibility_for_miri(){
    let list = [
        <two_phantom::Struct<i8 ,i8> as StableAbi>::LAYOUT,
        <two_phantom::Struct<i16,i16> as StableAbi>::LAYOUT,
        <two_phantom::Struct<i32,i32> as StableAbi>::LAYOUT,
    ];
    
    check_imcompatible_with_others(&list,|_|());
}
