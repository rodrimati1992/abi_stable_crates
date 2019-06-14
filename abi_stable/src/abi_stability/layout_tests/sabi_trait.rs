use crate::{
    sabi_trait,
    StableAbi,
    abi_stability::{
        abi_checking::{AbiInstability,check_layout_compatibility},
        AbiInfoWrapper,
    },
    std_types::{RBox},
    test_utils::must_panic,
};


use core_extensions::{matches};


fn check_subsets<F>(list:&[&'static AbiInfoWrapper],mut f:F)
where
    F:FnMut(&[AbiInstability])
{
    for (l_i,l_abi) in list.iter().enumerate() {
        for (r_i,r_abi) in list.iter().enumerate() {

            let res=check_layout_compatibility(l_abi,r_abi);

            if l_i <= r_i {
                assert_eq!(res,Ok(()));
            }else{
                if let Ok(_)=res {
                    let _=dbg!(l_i);
                    let _=dbg!(r_i);
                }
                let errs=res.unwrap_err().flatten_errors();

                f(&*errs);
            }
        }
    }
}


fn check_equality<F>(list:&[&'static AbiInfoWrapper],mut f:F)
where
    F:FnMut(&[AbiInstability])
{
    for (l_i,l_abi) in list.iter().enumerate() {
        for (r_i,r_abi) in list.iter().enumerate() {

            let res=check_layout_compatibility(l_abi,r_abi);

            if l_i == r_i {
                assert_eq!(res,Ok(()));
            }else{
                if let Ok(_)=res {
                    let _=dbg!(l_i);
                    let _=dbg!(r_i);
                }
                let errs=res.unwrap_err().flatten_errors();

                f(&*errs);
            }
        }
    }
}



mod one_method{
    use super::*;
    #[sabi_trait]
    pub trait Trait{
        #[sabi(last_prefix_field)]
        fn apply(&self,l:u32,r:u32)->u32;
    }
}

mod two_methods{
    use super::*;
    #[sabi_trait]
    pub trait Trait{
        #[sabi(last_prefix_field)]
        fn apply(&self,l:u32,r:u32)->u32;
        fn apply2(&self,l:u32,r:u32)->u32;
    }
}

mod three_methods{
    use super::*;
    #[sabi_trait]
    pub trait Trait{
        #[sabi(last_prefix_field)]
        fn apply(&self,l:u32,r:u32)->u32;
        fn apply2(&self,l:u32,r:u32)->u32;
        fn apply3(&self,l:u32,r:u32)->u32;
    }
}

mod one_method_debug{
    use super::*;
    #[sabi_trait]
    pub trait Trait:Debug{
        #[sabi(last_prefix_field)]
        fn apply(&self,l:u32,r:u32)->u32;
    }
}

mod one_method_clone_debug{
    use super::*;
    #[sabi_trait]
    pub trait Trait:Clone+Debug{
        #[sabi(last_prefix_field)]
        fn apply(&self,l:u32,r:u32)->u32;
    }
}

mod one_method_sync{
    use super::*;
    #[sabi_trait]
    pub trait Trait:Sync{
        #[sabi(last_prefix_field)]
        fn apply(&self,l:u32,r:u32)->u32;
    }
}

mod one_method_send{
    use super::*;
    #[sabi_trait]
    pub trait Trait:Send{
        #[sabi(last_prefix_field)]
        fn apply(&self,l:u32,r:u32)->u32;
    }
}
mod one_method_sync_send{
    use super::*;
    #[sabi_trait]
    pub trait Trait:Sync+Send{
        #[sabi(last_prefix_field)]
        fn apply(&self,l:u32,r:u32)->u32;
    }
}

#[test]
fn adding_methods_at_the_end(){
    let list=vec![
        <one_method::Trait_TO<RBox<()>> as StableAbi>::ABI_INFO,
        <two_methods::Trait_TO<RBox<()>> as StableAbi>::ABI_INFO,
        <three_methods::Trait_TO<RBox<()>> as StableAbi>::ABI_INFO,
    ];

    check_subsets(&list[..],|errs|{
        assert!(
            errs
            .iter()
            .any(|err| matches!(AbiInstability::FieldCountMismatch{..}=err))
        );        
    });
}



#[test]
fn adding_supertraits(){
    let list=vec![
        <one_method::Trait_TO<RBox<()>> as StableAbi>::ABI_INFO,
        <one_method_debug::Trait_TO<RBox<()>> as StableAbi>::ABI_INFO,
        <one_method_clone_debug::Trait_TO<RBox<()>> as StableAbi>::ABI_INFO,
    ];
    check_subsets(&list[..],|errs|{
        assert!(
            errs
            .iter()
            .any(|err| matches!(AbiInstability::TagError{..}=err))
        );        
    });
}


#[test]
fn incompatible_supertraits(){
    let list=vec![
        <one_method::Trait_TO<RBox<()>> as StableAbi>::ABI_INFO,
        <one_method_sync::Trait_TO<RBox<()>> as StableAbi>::ABI_INFO,
        <one_method_send::Trait_TO<RBox<()>> as StableAbi>::ABI_INFO,
        <one_method_sync_send::Trait_TO<RBox<()>> as StableAbi>::ABI_INFO,
    ];
    check_equality(&list[..],|errs|{
        assert!(
            errs
            .iter()
            .any(|err| matches!(AbiInstability::TagError{..}=err))
        );        
    });
}

