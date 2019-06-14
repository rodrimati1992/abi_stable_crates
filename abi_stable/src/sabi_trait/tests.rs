use std::{
    fmt::Debug,
    hash::Hash,
    sync::Arc,
    mem,
};

use crate::{
    *,
    std_types::{RBox,RString,RArc,Tuple1,Tuple2,Tuple3},
    sabi_trait::prelude::*,
    type_level::bools::*,
};

use abi_stable_shared::{file_span,test_utils::{must_panic}};


mod empty{
    use super::*;
    #[sabi_trait]
    pub trait Trait{}

    impl Trait for () {}

    impl Trait for True {}
}

mod method_no_default{
    use super::*;
    #[sabi_trait]
    pub trait Trait{
        fn apply(&self,l:u32,r:u32)->u32;
    }
    
    impl Trait for () {
        fn apply(&self,l:u32,r:u32)->u32{
            (l+r)*2
        }
    }
}

mod method_default{
    use super::*;
    #[sabi_trait]
    pub trait Trait{
        fn apply(&self,l:u32,r:u32)->u32{
            (l+r)*3
        }
    }

    impl Trait for () {}

    impl Trait for True {
        fn apply(&self,l:u32,r:u32)->u32{
            (l+r)*4
        }
    }
}


#[test]
fn downcasting_tests(){


    unsafe{
        use self::method_no_default::*;
        let empty=empty::Trait_from_value::<_,TU_Opaque>(());
        let object=mem::transmute::<_,Trait_TO<RBox<()>>>(empty);
        must_panic(file_span!(),|| object.apply(2,5) ).unwrap();
    }
    unsafe{
        use self::method_default::*;
        let empty=empty::Trait_from_value::<_,TU_Opaque>(());
        let object=mem::transmute::<_,Trait_TO<RBox<()>>>(empty);
        assert_eq!(object.apply(2,5),21);
    }
    

    {
        let no_default=method_no_default::Trait_from_value::<_,TU_Opaque>(());
        {
            use self::method_no_default::*;
            assert_eq!(no_default.apply(2,5), 14);
        }
        unsafe{            
            use self::method_default::*;
            let object=mem::transmute::<_,Trait_TO<RBox<()>>>(no_default);
            assert_eq!(object.apply(2,5), 14);
        }
    }
    {
        let with_default=method_default::Trait_from_value::<_,TU_Opaque>(True);
        {
            use self::method_default::*;
            assert_eq!(with_default.apply(2,5), 28);
        }
        unsafe{
            use self::method_no_default::*;
            let object=mem::transmute::<_,Trait_TO<RBox<()>>>(with_default);
            assert_eq!(object.apply(2,5), 28);
        }
    }
}




mod single_method{
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

mod single_methods_more_supertraits{
    use super::*;
    #[sabi_trait]
    pub trait Trait:Clone+Debug{
        #[sabi(last_prefix_field)]
        fn apply(&self,l:u32,r:u32)->u32;
    }
}

mod single_methods_sync{
    use super::*;
    #[sabi_trait]
    pub trait Trait:Clone+Debug{
        #[sabi(last_prefix_field)]
        fn apply(&self,l:u32,r:u32)->u32;
    }
}

mod single_methods_send{
    use super::*;
    #[sabi_trait]
    pub trait Trait:Clone+Debug{
        #[sabi(last_prefix_field)]
        fn apply(&self,l:u32,r:u32)->u32;
    }
}

fn check_compatibility(){
    
}