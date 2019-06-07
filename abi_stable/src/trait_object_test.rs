use std::fmt::Debug;
use crate::{
    *,
    std_types::{RBox,RArc},
    sabi_trait::prelude::*,
};

//////////////////////////////////////

#[sabi_trait]
#[sabi(debug_print_trait)]
pub trait RSomething<T>:Send+Sync+Clone+Debug{
    type Element:Debug;

    fn get(&self)->&Self::Element;
    
    fn get_mut(&mut self)->&mut Self::Element;

    #[sabi(last_prefix_field)]
    fn into_value(self)->Self::Element;
}

impl RSomething<()> for u32{
    type Element=u32;

    fn get(&self)->&Self::Element{
        self
    }
    fn get_mut(&mut self)->&mut Self::Element{
        self
    }
    fn into_value(self)->Self::Element{
        self
    }
}

//////////////////////////////////////

#[sabi_trait]
//#[sabi(debug_print_trait)]
pub trait RSomethingElse<T:Copy>:Send+Sync+Clone+Debug{
    fn get(&self)->&T;
    
    fn get_mut(&mut self)->&mut T;

    #[sabi(last_prefix_field)]
    fn into_value(self)->T;
}

impl RSomethingElse<u32> for u32{
    fn get(&self)->&u32{
        self
    }
    fn get_mut(&mut self)->&mut u32{
        self
    }
    fn into_value(self)->u32{
        self
    }
}

//////////////////////////////////////

#[sabi_trait]
#[sabi(debug_print_trait)]
pub trait RFoo<'a,T:Copy+'a>:Send+Sync+Clone+Debug{
    fn get(&'a self)->&'a T;
}

//////////////////////////////////////
fn _rsomething_assertions(
    trait_0:&mut dyn RSomething<(),Element=()>,
    trait_1:&mut dyn RSomethingElse<u32>,
)where
    RSomething_TO<'static,RBox<()>,(),()>:RSomething<()>+Send+Sync+Clone+Debug,
    RSomethingElse_TO<'static,RBox<()>,u32>:RSomethingElse<u32>+Send+Sync+Clone+Debug,
{
    trait_0.get();
    trait_0.get_mut();
    trait_1.get();
    trait_1.get_mut();

    let _object_0=RSomething_from_value::<_,YesImplAny,()>(100_u32);
    let _erased_0=RSomething_from_ptr::<_,NoImplAny,()>(RBox::new(100_u32));

    let _object_1:RSomethingElse_TO<RBox<()>,u32>=
        RSomethingElse_from_value::<_,YesImplAny,u32>(100_u32);

    let _erased_1:RSomethingElse_TO<RBox<()>,u32>=
        RSomethingElse_from_ptr::<_,NoImplAny,u32>(RBox::new(100_u32));

}
//////////////////////////////////////


#[cfg(test)]
mod tests{
    use super::*;

    fn assert_sync<T:Sync>(_:&T){}
    fn assert_debug<T:Debug>(_:&T){}

    #[test]
    fn construct_trait_object(){
        let mut object=RSomething_from_value::<_,YesImplAny,()>(100_u32);
        let mut erased=RSomething_from_ptr::<_,NoImplAny,()>(RBox::new(100_u32));
        
        assert_sync(&object);
        assert_sync(&erased);

        assert_debug(&object);
        assert_debug(&erased);

        fn assertions_unerased(mut object:RSomething_TO<'_,RBox<()>,(),u32>){
            assert_eq!(object.sabi_as_unerased::<u32>().ok(),Some(&100));
            assert_eq!(object.sabi_as_unerased::<i8>().ok(),None::<&i8>);
            assert_eq!(object.sabi_as_unerased_mut::<u32>().ok(),Some(&mut 100));
            assert_eq!(object.sabi_as_unerased_mut::<i8>().ok(),None::<&mut i8>);
            object=object.sabi_into_unerased::<i8>().unwrap_err().into_inner();
            assert_eq!(object.sabi_into_unerased::<u32>().ok(),Some(RBox::new(100)));
        }

        fn assertions_erased(mut object:RSomething_TO<'_,RBox<()>,(),u32>){
            assert_eq!(object.sabi_as_unerased::<u32>().ok(),None);
            assert_eq!(object.sabi_as_unerased::<i8>().ok(),None);
            assert_eq!(object.sabi_as_unerased_mut::<u32>().ok(),None);
            assert_eq!(object.sabi_as_unerased_mut::<i8>().ok(),None);
            object=object.sabi_into_unerased::<u32>().unwrap_err().into_inner();
            object=object.sabi_into_unerased::<i8>().unwrap_err().into_inner();
        }

        assertions_unerased(object.clone());
        assertions_unerased(object);
        
        assertions_erased(erased.clone());
        assertions_erased(erased);
        
        
    }

    #[test]
    fn trait_object_methods(){
        let mut object=RSomething_from_value::<_,NoImplAny,()>(100);
        let mut cloned=object.clone();
        
        assert_eq!(object.get_(),&100);
        assert_eq!(object.get_mut_(),&mut 100);
        assert_eq!(object.into_value_(),100);

        assert_eq!(cloned.get(),&100);
        assert_eq!(cloned.get_mut(),&mut 100);
        assert_eq!(cloned.into_value(),100);
    }
}