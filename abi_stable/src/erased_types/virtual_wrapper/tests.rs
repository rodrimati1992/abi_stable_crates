use super::*;

use std::{
    cmp::{Ord, Ordering, PartialEq, PartialOrd},
    
    collections::hash_map::DefaultHasher,
    fmt::{self, Display},
    hash::{Hash},
    
};

use serde::{Serialize};

use serde_json;

#[allow(unused_imports)]
use crate::{
    erased_types::{
        VirtualWrapper,ImplType, InterfaceType, SerializeImplType,DeserializeInterfaceType,
    },
    impl_get_type_info,
    type_level::bools::{False,True},
    traits::IntoReprC,
    ZeroSized, 
    StableAbi,
    std_types::{
        RArc, RBox, RBoxError, RCow, RStr, RString,  StaticStr,
    },
};

#[allow(unused_imports)]
use core_extensions::prelude::*;

/// It doesn't need to be `#[repr(C)]` because  VirtualWrapper puts it behind a pointer,
/// and is only interacted with through regular Rust functions.
#[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
struct Foo<T> {
    l: u32,
    r: u32,
    name: T,
}


#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
struct FooInterface;


impl<S> Display for Foo<S>
where
    S: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "l:{}  r:{}  name:'{}'", self.l, self.r, self.name,)
    }
}

impl<T> ImplType for Foo<T>
where
    T: 'static + Send + Sync,
{
    type Interface = FooInterface;
    const INFO:&'static crate::erased_types::TypeInfo=impl_get_type_info! { Foo[T] };
}

impl<T> SerializeImplType for Foo<T>
where
    T: 'static + Send + Sync + Serialize,
{
    fn serialize_impl(&self) -> Result<RCow<'_, RStr<'_>>, RBoxError> {
        match serde_json::to_string(self) {
            Ok(v)=>Ok(v.into_c().piped(RCow::Owned)),
            Err(e)=>Err(RBoxError::new(e)),
        }
    }
}

crate::impl_InterfaceType!{
    impl crate::InterfaceType for FooInterface {
        type Clone = True;

        type Default = True;

        type Display = True;

        type Debug = True;

        type Serialize = True;

        type Deserialize = True;

        type Eq = True;

        type PartialEq = True;

        type Ord = True;

        type PartialOrd = True;

        type Hash = True;
    }
}


impl DeserializeInterfaceType for FooInterface {
    type Deserialized = VirtualFoo;

    fn deserialize_impl(s: RStr<'_>) -> Result<Self::Deserialized, RBoxError> {
        match ::serde_json::from_str::<Foo<String>>(&*s) {
            Ok(x) => Ok(VirtualWrapper::from_value(x)),
            Err(e) => Err(RBoxError::new(e)),
        }
    }
}

type VirtualFoo = VirtualWrapper<RBox<ZeroSized<FooInterface>>>;

/////////////////////////////////


fn new_foo()->Foo<String>{
    Foo{
        l:1000,
        r:100,
        name:"hello_world".into(),
    }
}

fn new_wrapped()->VirtualFoo{
    VirtualWrapper::from_value(new_foo())
}


#[test]
fn clone_test(){
    let wrapped =new_wrapped();
    let cloned=wrapped.clone();

    assert_eq!(wrapped,cloned);

    assert_ne!(
        wrapped,
        Foo::<String>::default().piped(VirtualWrapper::from_value)
    );
}

#[test]
fn default_test(){
    let concrete=Foo::<String>::default();
    let wrapped =new_wrapped().default();
    let wrapped_2=Foo::<String>::default().piped(VirtualWrapper::from_value);


    assert_eq!(wrapped,wrapped_2);
    assert_eq!(
        wrapped.as_unerased::<Foo<String>>().unwrap(),
        &concrete
    );

    assert_ne!(
        wrapped,
        new_wrapped(),
    );
}


#[test]
fn display_test(){

    let concrete=new_foo();
    let wrapped =new_wrapped();

    assert_eq!(
        format!("{}",concrete), 
        format!("{}",wrapped),
    );

    assert_eq!(
        format!("{:#}",concrete), 
        format!("{:#}",wrapped),
    );
}

#[test]
fn debug_test(){

    let concrete=new_foo();
    let wrapped =new_wrapped();

    assert_eq!(
        format!("{:?}",concrete), 
        format!("{:?}",wrapped),
    );

    assert_eq!(
        format!("{:#?}",concrete), 
        format!("{:#?}",wrapped),
    );
}

#[test]
fn deserialize_test() {

    let json=r#"
        {   
            "l":1000,
            "r":10,
            "name":"what the hell"
        }
    "#;
    let json_ss=serde_json::to_string(json).unwrap();

    let concrete = serde_json::from_str::<Foo<String>>(json).unwrap();

    let wrapped = VirtualFoo::deserialize_from_str(json).unwrap();
    let wrapped2 = serde_json::from_str::<VirtualFoo>(&json_ss).unwrap();

    assert_eq!(
        serde_json::from_str::<VirtualFoo>(json).map_err(drop),
        Err(()),
    );
    
    assert_eq!(
        wrapped.as_unerased::<Foo<String>>().unwrap(), 
        &concrete,
    );

    assert_eq!(
        wrapped2.as_unerased::<Foo<String>>().unwrap(), 
        &concrete
    );
}


#[test]
fn serialize_test() {

    let concrete = new_foo();
    let wrapped = new_wrapped();

    assert_eq!(
        &*concrete.piped_ref(serde_json::to_string).unwrap(),
        &*wrapped.serialized().unwrap()
    );

    assert_eq!(
        concrete
            .piped_ref(serde_json::to_string).unwrap()
            .piped_ref(serde_json::to_string).unwrap(),
        wrapped.piped_ref(serde_json::to_string).unwrap()
    );

    assert_eq!(
        wrapped.serialized().unwrap()
            .piped_ref(serde_json::to_string).unwrap(),
        wrapped.piped_ref(serde_json::to_string).unwrap()
    );
}



#[test]
fn cmp_test(){

    let wrapped_0=new_foo().mutated(|x| x.l-=100 ).piped(VirtualWrapper::from_value);
    let wrapped_1=new_wrapped();
    let wrapped_2=new_foo().mutated(|x| x.l+=100 ).piped(VirtualWrapper::from_value);

    assert_eq!(wrapped_1 == wrapped_0, false);
    assert_eq!(wrapped_1 <= wrapped_0, false);
    assert_eq!(wrapped_1 >= wrapped_0, true );
    assert_eq!(wrapped_1 < wrapped_0, false);
    assert_eq!(wrapped_1 > wrapped_0, true);
    assert_eq!(wrapped_1 != wrapped_0, true);
    assert_eq!(wrapped_1.partial_cmp(&wrapped_0), Some(Ordering::Greater));
    assert_eq!(wrapped_1.cmp(&wrapped_0), Ordering::Greater);
    assert_eq!(wrapped_1.eq(&wrapped_0), false);
    assert_eq!(wrapped_1.ne(&wrapped_0), true);

    assert_eq!(wrapped_1 == wrapped_1, true);
    assert_eq!(wrapped_1 <= wrapped_1, true);
    assert_eq!(wrapped_1 >= wrapped_1, true);
    assert_eq!(wrapped_1 < wrapped_1, false);
    assert_eq!(wrapped_1 > wrapped_1, false);
    assert_eq!(wrapped_1 != wrapped_1, false);
    assert_eq!(wrapped_1.partial_cmp(&wrapped_1), Some(Ordering::Equal));
    assert_eq!(wrapped_1.cmp(&wrapped_1), Ordering::Equal);
    assert_eq!(wrapped_1.eq(&wrapped_1), true);
    assert_eq!(wrapped_1.ne(&wrapped_1), false);


    assert_eq!(wrapped_1 == wrapped_2, false);
    assert_eq!(wrapped_1 <= wrapped_2, true);
    assert_eq!(wrapped_1 >= wrapped_2, false );
    assert_eq!(wrapped_1 < wrapped_2, true);
    assert_eq!(wrapped_1 > wrapped_2, false);
    assert_eq!(wrapped_1 != wrapped_2, true);
    assert_eq!(wrapped_1.partial_cmp(&wrapped_2), Some(Ordering::Less));
    assert_eq!(wrapped_1.cmp(&wrapped_2), Ordering::Less);
    assert_eq!(wrapped_1.eq(&wrapped_2), false);
    assert_eq!(wrapped_1.ne(&wrapped_2), true);

}



#[test]
fn hash_test(){
    fn hash_value<H:Hash>(v:&H)->u64{
        let mut hasher=DefaultHasher::new();
        v.hash(&mut hasher);
        hasher.finish()
    }
    
    {
        let hash_concrete=hash_value(&new_foo());
        let hash_wrapped=hash_value(&new_wrapped());
        
        assert_eq!(hash_concrete,hash_wrapped);
    }
    
    {
        let concrete=Foo::<String>::default();
        let hash_concrete=hash_value(&concrete);
        let hash_wrapped=hash_value(&VirtualWrapper::from_value(concrete.clone()));
        
        assert_eq!(hash_concrete,hash_wrapped);
    }

}


#[test]
fn from_any_test(){

    assert_eq!(
        VirtualWrapper::from_value(new_foo()),
        VirtualWrapper::from_any_value(new_foo(),FooInterface),
    );

    assert_eq!(
        VirtualWrapper::from_ptr(RArc::new(new_foo())),
        VirtualWrapper::from_any_ptr(RArc::new(new_foo()),FooInterface),
    );

}



#[test]
fn to_any_test(){

    let mut wrapped=VirtualWrapper::from_any_value(new_foo(),FooInterface);


    macro_rules! to_unerased {
        ( $wrapped:expr ; $method:ident ; $expected:expr ) => (
            assert_eq!(
                $wrapped.$method ::<Foo<RString>>().map_err(drop),
                Err(())
            );

            assert_eq!(
                $wrapped.$method ::<Foo<String>>().unwrap(),
                $expected
            );
        )
    }

    to_unerased!( wrapped.clone() ; into_unerased     ; RBox::new(new_foo()) );
    to_unerased!( wrapped.clone() ; into_any_unerased ; RBox::new(new_foo()) );
    
    to_unerased!( wrapped ; as_unerased     ; &new_foo() );
    to_unerased!( wrapped ; as_any_unerased ; &new_foo() );
    
    to_unerased!( wrapped ; as_unerased_mut ; &mut new_foo() );
    to_unerased!( wrapped ; as_any_unerased_mut ; &mut new_foo() );
    

}