use std::{
    mem,
};

use crate::{
    *,
    std_types::RBox,
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
        let empty=empty::Trait_from_value((),TU_Opaque);
        let object=mem::transmute::<_,Trait_TO<'_,RBox<()>>>(empty);
        must_panic(file_span!(),|| object.apply(2,5) ).unwrap();
    }
    unsafe{
        use self::method_default::*;
        let empty=empty::Trait_from_value((),TU_Opaque);
        let object=mem::transmute::<_,Trait_TO<'_,RBox<()>>>(empty);
        assert_eq!(object.apply(2,5),21);
    }
    

    {
        let no_default=method_no_default::Trait_from_value((),TU_Opaque);
        {
            use self::method_no_default::*;
            assert_eq!(no_default.apply(2,5), 14);
        }
        unsafe{            
            use self::method_default::*;
            let object=mem::transmute::<_,Trait_TO<'_,RBox<()>>>(no_default);
            assert_eq!(object.apply(2,5), 14);
        }
    }
    {
        let with_default=method_default::Trait_from_value(True,TU_Opaque);
        {
            use self::method_default::*;
            assert_eq!(with_default.apply(2,5), 28);
        }
        unsafe{
            use self::method_no_default::*;
            let object=mem::transmute::<_,Trait_TO<'_,RBox<()>>>(with_default);
            assert_eq!(object.apply(2,5), 28);
        }
    }
}




#[sabi_trait]
trait DefaultMethodPair{
    fn foo(&self,x:u32)->u32{
        self.bar(x+10)
    }
    fn bar(&self,y:u32)->u32{
        self.baz(y+20)
    }
    fn baz(&self,z:u32)->u32{
        z+40
    }
}

struct A;
struct B;
struct C;


impl DefaultMethodPair for A{
    fn foo(&self,x:u32)->u32{
        x+100
    }
}

impl DefaultMethodPair for B{
    fn bar(&self,y:u32)->u32{
        y+200
    }
}

impl DefaultMethodPair for C{
    fn baz(&self,z:u32)->u32{
        z+300
    }
}


#[test]
fn default_methods(){
    let a=DefaultMethodPair_from_value(A,TU_Opaque);
    let b=DefaultMethodPair_from_value(B,TU_Opaque);
    let c=DefaultMethodPair_from_value(C,TU_Opaque);

    assert_eq!(a.foo(1), 101);
    assert_eq!(b.foo(1), 211);
    assert_eq!(c.foo(1), 331);
}