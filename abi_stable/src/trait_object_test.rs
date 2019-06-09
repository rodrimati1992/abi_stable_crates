use std::{
    fmt::Debug,
    hash::Hash,
    sync::Arc,
};

use crate::{
    *,
    std_types::{RBox,RString,RArc,Tuple1,Tuple2,Tuple3},
    sabi_trait::prelude::*,
};

//////////////////////////////////////

/**

```
use abi_stable::{
    sabi_trait::prelude::*,
    trait_object_test::*,
    std_types::*,
};

let _=RSomething_from_value::<_,TU_Opaque,()>(RBox::new(10_u32));

```

While RSomething_TO can be constructed from an RArc,
no method on the trait can be called because RSomething has mutable and by value methods.

```compile_fail
use abi_stable::{
    sabi_trait::prelude::*,
    trait_object_test::*,
    std_types::*,
};


let what=RSomething_from_ptr::<_,TU_Opaque,_>(RArc::new(100u32));
RSomething::into_value(what);

```


Cannot create RSomething from a !Sync type.
```compile_fail
use abi_stable::{
    marker_type::*,
    sabi_trait::prelude::*,
    trait_object_test::*,
    std_types::*,
};

use std::marker::PhantomData;

let ptr=RBox::new(PhantomData::<UnsyncSend>);
let _=RSomething_from_value::<_,TU_Opaque,()>(ptr);

```

Cannot create RSomething from a !Send type.
```compile_fail
use abi_stable::{
    marker_type::*,
    sabi_trait::prelude::*,
    trait_object_test::*,
    std_types::*,
};

use std::marker::PhantomData;

let ptr=RBox::new(PhantomData::<SyncUnsend>);
let _=RSomething_from_value::<_,TU_Opaque,()>(ptr);

```


*/
#[sabi_trait]
//#[sabi(debug_print_trait)]
pub trait RSomething<T>:Send+Sync+Clone+Debug{
    type Element:Debug;

    fn get(&self)->&Self::Element;
    
    fn get_mut(&mut self)->&mut Self::Element;

    #[sabi(last_prefix_field)]
    fn into_value(self)->Self::Element;
}


macro_rules! impls_for_something {
    ( 
        $traitname:ident ,
        extra_bounds[ $($extra_bounds:tt)* ]
    ) => (

        impl $traitname<()> for u32{
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

        impl<'a,T> $traitname<()> for &'a T
        where
            T:Send+Sync+Debug+$($extra_bounds)*
        {
            type Element=Self;

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

        impl<T> $traitname<()> for RBox<T>
        where
            T:Send+Sync+Debug+Clone+$($extra_bounds)*
        {
            type Element=T;

            fn get(&self)->&Self::Element{
                &**self
            }
            fn get_mut(&mut self)->&mut Self::Element{
                &mut **self
            }
            fn into_value(self)->Self::Element{
                RBox::into_inner(self)
            }
        }


        impl<T> $traitname<()> for RArc<T>
        where
            T:Send+Sync+Debug+Clone+$($extra_bounds)*
        {
            type Element=T;

            fn get(&self)->&Self::Element{
                &**self
            }
            fn get_mut(&mut self)->&mut Self::Element{
                RArc::make_mut(self)
            }
            fn into_value(self)->Self::Element{
                (*self).clone()
            }
        }

    )
}


impls_for_something!{ RSomething, extra_bounds[ Sized ] }
impls_for_something!{ DSomething, extra_bounds[ Hash ] }

//////////////////////////////////////


#[sabi_trait]
//#[sabi(debug_print_trait)]
#[sabi(use_dyn_trait)]
pub trait DSomething<T>:Send+Sync+Clone+Debug+Hash{
    type Element:Debug;

    fn get(&self)->&Self::Element;
    
    fn get_mut(&mut self)->&mut Self::Element;

    #[sabi(last_prefix_field)]
    fn into_value(self)->Self::Element;
}


//////////////////////////////////////


#[sabi_trait]
//#[sabi(debug_print_trait)]
pub trait EmptyTrait{}


impl EmptyTrait for () {}

impl EmptyTrait for u32 {}

impl<T> EmptyTrait for RArc<T> {}

impl<T> EmptyTrait for RBox<T> {}


//////////////////////////////////////

#[sabi_trait]
pub trait StaticTrait:'static{}

//////////////////////////////////////

/**

While RSomethingElse_TO can be constructed from an RArc,
no method on the trait can be called because RSomethingElse has mutable and by value methods.

```compile_fail
use abi_stable::{
    marker_type::*,
    sabi_trait::prelude::*,
    trait_object_test::*,
    std_types::*,
};

let what=RSomethingElse_from_ptr::<_,TU_Opaque,_>(RArc::new(100u32));
RSomethingElse::into_value(what);


```


```
use abi_stable::{
    marker_type::*,
    sabi_trait::prelude::*,
    trait_object_test::*,
    std_types::*,
};

use std::marker::PhantomData;

let ptr=RBox::new(PhantomData::<UnsyncSend>);
let _=RSomethingElse_from_value::<_,TU_Opaque,_>(ptr);

```

Cannot create RSomethingElse from a !Send type.
```compile_fail
use abi_stable::{
    marker_type::*,
    sabi_trait::prelude::*,
    trait_object_test::*,
    std_types::*,
};

use std::marker::PhantomData;

let ptr=RBox::new(PhantomData::<SyncUnsend>);
let _=RSomethingElse_from_value::<_,TU_Opaque,_>(ptr);

```



*/
#[sabi_trait]
//#[sabi(debug_print_trait)]
pub trait RSomethingElse<T:Copy>:Send+Debug{
    fn get(&self)->&T;
    
    #[sabi(last_prefix_field)]
    fn into_value(self)->T;

    fn passthrough_string(&self,value:RString)->RString{
        value
    }

    fn passthrough_arc(&self,value:RArc<u32>)->RArc<u32>{
        value
    }
}

impl RSomethingElse<u32> for u32{
    fn get(&self)->&u32{
        self
    }
    fn into_value(self)->u32{
        self
    }

    fn passthrough_string(&self,value:RString)->RString{
        RString::new()
    }

    fn passthrough_arc(&self,value:RArc<u32>)->RArc<u32>{
        RArc::new(77)
    }
}

impl<T> RSomethingElse<T> for RArc<T>
where
    T:Copy+Send+Sync+Debug
{
    fn get(&self)->&T{
        &**self
    }
    fn into_value(self)->T{
        *self
    }
}

impl<T> RSomethingElse<T> for RBox<T>
where
    T:Copy+Send+Debug
{
    fn get(&self)->&T{
        &**self
    }
    fn into_value(self)->T{
        *self
    }
}

//////////////////////////////////////

#[sabi_trait]
//#[sabi(debug_print_trait)]
pub trait RFoo<'a,T:Copy+'a>{
    fn get(&'a self)->&'a T;
}


impl<'a,A:Copy+'a> RFoo<'a,A> for Tuple1<A>{
    fn get(&'a self)->&'a A{
        &self.0
    }
}


impl<'a,A:'a,B:Copy+'a> RFoo<'a,B> for Tuple2<A,B>{
    fn get(&'a self)->&'a B{
        &self.1
    }
}

impl<'a,A:'a,B:'a,C:Copy+'a> RFoo<'a,C> for Tuple3<A,B,C>{
    fn get(&'a self)->&'a C{
        &self.2
    }
}


impl<'a,T> RFoo<'a,T> for RArc<T>
where
    T:'a+Copy
{
    fn get(&'a self)->&'a T{
        &**self
    }
}



//////////////////////////////////////


//////////////////////////////////////

#[sabi_trait]
trait Baz {
    fn baz(self);
}



//////////////////////////////////////


#[cfg(test)]
mod tests{
    use super::*;

    use crate::{
        traits::IntoReprC,
        utils::leak_value,
    };

    fn assert_sync<T:Sync>(_:&T){}
    fn assert_debug<T:Debug>(_:&T){}
    fn assert_sync_send_debug_clone<T:Sync+Send+Debug+Clone>(_:&T){}


    macro_rules! _something_test {
        (
            fn $fn_name:ident,
            fn $something_methods:ident,
            $typename:ident,
            $traitname:ident,
            $from_value:ident,
            $from_ptr:ident,
        ) => (
            #[test]
            fn $fn_name(){
                let number=100_u32;
                let mut object=$from_value::<_,TU_Unerasable,()>(number);
                let mut arcobj=$from_ptr::<_,TU_Unerasable,()>(RArc::new(number));
                let mut erased=$from_ptr::<_,TU_Opaque,()>(RBox::new(number));
                
                assert_sync_send_debug_clone(&object);
                assert_sync_send_debug_clone(&arcobj);
                assert_sync_send_debug_clone(&erased);

                fn assertions_unerased(mut object:$typename<'_,RBox<()>,(),u32>){
                    assert_eq!(object.sabi_as_any_unerased::<u32>().ok(),Some(&100));
                    assert_eq!(object.sabi_as_any_unerased::<i8>().ok(),None::<&i8>);
                    assert_eq!(object.sabi_as_any_unerased_mut::<u32>().ok(),Some(&mut 100));
                    assert_eq!(object.sabi_as_any_unerased_mut::<i8>().ok(),None::<&mut i8>);
                    object=object.sabi_into_any_unerased::<i8>().unwrap_err().into_inner();
                    assert_eq!(object.sabi_into_any_unerased::<u32>().ok(),Some(RBox::new(100)));
                }

                fn assertions_unerased_arc(mut object:$typename<'_,RArc<()>,(),u32>){
                    assert_eq!(object.sabi_as_any_unerased::<u32>().ok(),Some(&100));
                    assert_eq!(object.sabi_as_any_unerased::<i8>().ok(),None::<&i8>);
                    object=object.sabi_into_any_unerased::<i8>().unwrap_err().into_inner();
                    assert_eq!(object.sabi_into_any_unerased::<u32>().ok(),Some(RArc::new(100)));
                }

                fn assertions_erased(mut object:$typename<'_,RBox<()>,(),u32>){
                    assert_eq!(object.sabi_as_any_unerased::<u32>().ok(),None);
                    assert_eq!(object.sabi_as_any_unerased::<i8>().ok(),None);
                    assert_eq!(object.sabi_as_any_unerased_mut::<u32>().ok(),None);
                    assert_eq!(object.sabi_as_any_unerased_mut::<i8>().ok(),None);
                    object=object.sabi_into_any_unerased::<u32>().unwrap_err().into_inner();
                    object=object.sabi_into_any_unerased::<i8>().unwrap_err().into_inner();
                }

                fn create_from_ref<'a,T>(value:&'a T)->$typename<'a,&'a(),(),T::Element>
                where
                    T:$traitname<()>+'a
                {
                    $from_ptr::<_,TU_Opaque,()>(value)
                }

                fn create_from_val<'a,T>(value:T)->$typename<'a,RBox<()>,(),T::Element>
                where
                    T:$traitname<()>+'a,
                {
                    $from_value::<_,TU_Opaque,()>(value)
                }

                let what=RBox::new(100);
                let _=create_from_ref(&*what);
                let _=create_from_val(&*what);

                assert_eq!(format!("{:?}",number),format!("{:?}",erased));
                assert_eq!(format!("{:?}",number),format!("{:?}",arcobj));
                assert_eq!(format!("{:?}",number),format!("{:?}",object));

                assert_eq!(format!("{:#?}",number),format!("{:?}",erased));
                assert_eq!(format!("{:#?}",number),format!("{:?}",arcobj));
                assert_eq!(format!("{:#?}",number),format!("{:?}",object));

                assertions_unerased(object.clone());
                assertions_unerased(object);
                
                assertions_unerased_arc(arcobj.clone());
                assertions_unerased_arc(arcobj);
                
                assertions_erased(erased.clone());
                assertions_erased(erased);        
            }

            #[test]
            fn $something_methods(){
                let mut object=$from_value::<_,TU_Opaque,()>(100);
                let mut cloned=object.clone();
                
                assert_eq!(object.get_(),&100);
                assert_eq!(object.get_mut_(),&mut 100);
                assert_eq!(object.into_value_(),100);

                assert_eq!(cloned.get(),&100);
                assert_eq!(cloned.get_mut(),&mut 100);
                assert_eq!(cloned.into_value(),100);
            }


        )
    }

    _something_test!{
        fn construct_rsomething,
        fn rsomething_methods,
        RSomething_TO,
        RSomething,
        RSomething_from_value,
        RSomething_from_ptr,
    }

    _something_test!{
        fn construct_dsomething,
        fn dsomething_methods,
        DSomething_TO,
        DSomething,
        DSomething_from_value,
        DSomething_from_ptr,
    }

    #[test]
    fn construct_rempty(){
        let arc=Arc::new(107_u32);
        let rarc=arc.clone().into_c();

        assert_eq!(Arc::strong_count(&arc), 2);

        let mut object:EmptyTrait_TO<RBox<()>>=
            EmptyTrait_from_value::<_,TU_Unerasable>(rarc.clone());
        
        assert_eq!(Arc::strong_count(&arc), 3);

        let mut erased:EmptyTrait_TO<RArc<()>>=
            EmptyTrait_from_ptr::<_,TU_Opaque>(rarc.clone());
        
        assert_eq!(Arc::strong_count(&arc), 4);

        assert_eq!(
            **object.sabi_as_any_unerased::<RArc<u32>>().unwrap(),
            107
        );
        assert_eq!(
            **object.sabi_as_any_unerased_mut::<RArc<u32>>().unwrap(),
            107
        );
        
        assert_eq!(Arc::strong_count(&arc), 4);
        object=object.sabi_into_any_unerased::<u32>().unwrap_err().into_inner();
        assert_eq!(Arc::strong_count(&arc), 4);
        
        assert_eq!(
            object.sabi_into_any_unerased::<RArc<u32>>().unwrap(),
            RBox::new(RArc::new(107))
        );
        
        assert_eq!(Arc::strong_count(&arc), 3);

        erased.sabi_into_any_unerased::<u32>().unwrap_err();
        
        assert_eq!(Arc::strong_count(&arc), 2);
               
    }

    #[test]
    fn test_reborrowing(){
        let arc=Arc::new(107_u32);
        let rarc=arc.clone().into_c();

        assert_eq!(Arc::strong_count(&arc), 2);

        let mut object:RSomething_TO<RBox<()>,(),u32>=
            RSomething_from_value::<_,TU_Unerasable,()>(rarc.clone());
        
        assert_eq!(Arc::strong_count(&arc), 3);
        
        for _ in 0..10{
            assert_eq!(
                object.reborrow().sabi_into_any_unerased::<RArc<u32>>().unwrap(),
                &RArc::new(107)
            );
        }
        assert_eq!(Arc::strong_count(&arc), 3);

        
        for _ in 0..10{
            assert_eq!(
                object.reborrow_mut().sabi_into_any_unerased::<RArc<u32>>().unwrap(),
                &mut RArc::new(107)
            );
        }


        assert_eq!(Arc::strong_count(&arc), 3);

        {
            let cloned=object.reborrow().clone();

            assert_eq!(format!("{:?}",cloned),"107");
        }

        assert_eq!(Arc::strong_count(&arc), 3);

        drop(object);

        assert_eq!(Arc::strong_count(&arc), 2);
               
    }

    #[test]
    fn rsomething_else(){
        {
            let object=RSomethingElse_from_value::<_,TU_Opaque,_>(RArc::new(100_u32));
            let _:&dyn RSomethingElse<u32>=&object;
            
            assert_eq!(object.get(),&100);
            assert_eq!(object.passthrough_arc(RArc::new(90)), RArc::new(90));
            assert_eq!(object.passthrough_string(RString::from("what")), RString::from("what"));
            assert_eq!(object.into_value(),100);

        }
        {
            let object=RSomethingElse_from_value::<_,TU_Opaque,_>(RArc::new(100_u32));
            assert_eq!(object.get_(),&100);
            assert_eq!(object.passthrough_arc_(RArc::new(90)), RArc::new(90));
            assert_eq!(object.passthrough_string_(RString::from("what")), RString::from("what"));
            assert_eq!(object.into_value_(),100);

        }
        {
            let object=RSomethingElse_from_value::<_,TU_Unerasable,u32>(100u32);
            assert_eq!(object.passthrough_arc_(RArc::new(90)), RArc::new(77));
            assert_eq!(object.passthrough_string_(RString::from("what")), RString::from(""));
        }
    }

    #[test]
    fn rfoo(){
        let object       =leak_value(RFoo_from_ptr::<_,TU_Opaque,_>(RBox::new(RArc::new(76))));
        let tuple1_object=leak_value(RFoo_from_ptr::<_,TU_Opaque,_>(RArc::new(Tuple1(100))));
        let tuple2_object=leak_value(RFoo_from_value::<_,TU_Opaque,_>(Tuple2(101u32,202_u32)));
        let tuple3_object=leak_value(RFoo_from_value::<_,TU_Opaque,_>(Tuple3(11,22,300_u32)));

        assert_eq!(object.get(),&76);
        assert_eq!(tuple1_object.get(),&100);
        assert_eq!(tuple2_object.get(),&202);
        assert_eq!(tuple3_object.get(),&300);

        assert_eq!(object.get_(),&76);
        assert_eq!(tuple1_object.get_(),&100);
        assert_eq!(tuple2_object.get_(),&202);
        assert_eq!(tuple3_object.get_(),&300);
    }
}