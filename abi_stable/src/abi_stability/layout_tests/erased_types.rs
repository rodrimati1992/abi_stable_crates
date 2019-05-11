#![allow(dead_code)]



#[allow(unused_imports)]
use core_extensions::{matches, prelude::*};

use crate::{
    abi_stability::{
        abi_checking::{check_abi_stability},
    },
    StableAbi,
    DynTrait,
    erased_types::IteratorItem,
    std_types::*,
    type_level::bools::*,
};


macro_rules! mod_iter_ty {
    (
        mod $module:ident;
        type Item<$lt:lifetime>=$ty:ty;
    ) => (
        pub mod $module{
            use super::*;

            #[repr(C)]
            #[derive(StableAbi)]
            #[sabi(inside_abi_stable_crate)]
            pub struct Interface;


            crate::impl_InterfaceType!{
                impl crate::InterfaceType for Interface {
                    type Iterator=True;
                }
            }


            impl<$lt> IteratorItem<$lt> for Interface{
                type Item=$ty;
            }
        }
    )
}



mod no_iterator_interface{
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(inside_abi_stable_crate)]
    pub struct Interface;


    crate::impl_InterfaceType!{
        impl crate::InterfaceType for Interface {}
    }

}


mod_iter_ty!{
    mod rstr_interface;
    type Item<'a>=RStr<'a>;
}

mod_iter_ty!{
    mod rstring_interface;
    type Item<'a>=RString;
}

mod_iter_ty!{
    mod u8_interface;
    type Item<'a>=u8;
}

mod_iter_ty!{
    mod unit_interface;
    type Item<'a>=();
}



#[test]
fn check_subsets(){
    type BoxTrait<'a,I>=DynTrait<'a,RBox<()>,I>;

    let pref_zero=<DynTrait<'_,RBox<()>,no_iterator_interface::Interface>>::ABI_INFO;

    let pref_iter_0=<BoxTrait<'_,rstring_interface::Interface>>::ABI_INFO;
    let pref_iter_1=<BoxTrait<'_,rstr_interface::Interface>>::ABI_INFO;
    let pref_iter_2=<BoxTrait<'_,u8_interface::Interface>>::ABI_INFO;
    let pref_iter_3=<BoxTrait<'_,unit_interface::Interface>>::ABI_INFO;

    let prefs=vec![pref_iter_0,pref_iter_1,pref_iter_2,pref_iter_3];

    assert_eq!(check_abi_stability(pref_zero, pref_zero), Ok(()) );
    
    for impl_ in prefs.iter().cloned() {
            
        assert_eq!(check_abi_stability(pref_zero, impl_), Ok(()) );

        assert_ne!(check_abi_stability(impl_, pref_zero), Ok(()) );
    }

    for (interf_i,interf) in prefs.iter().cloned().enumerate() {
        for (impl_i,impl_) in prefs.iter().cloned().enumerate() {
            if interf_i==impl_i {
                assert_eq!(check_abi_stability(interf, impl_), Ok(()) );
            }else{
                assert_ne!(check_abi_stability(interf, impl_), Ok(()) );
            }
        }
    }
}