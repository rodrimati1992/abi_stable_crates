use std::{
    fmt::Debug,
    marker::PhantomData,
};

use crate::{
    GetStaticEquivalent,
    StableAbi,
    InterfaceType,
    impl_InterfaceType,
    type_level::{
        impl_enum::{Implemented,Unimplemented},
        bools::{False,True},
    },
};


#[repr(C)]
#[derive(StableAbi)]
pub struct AllTraitsImpld;


impl_InterfaceType!{
    impl InterfaceType for AllTraitsImpld{
        // type Send=True;  // These are True by default
        // type Sync=True;  // These are True by default
        type Clone=True;
        type Default=True;
        type Display=True;
        type Debug=True;
        type Serialize=True;
        type Deserialize=True;
        type Eq=True;
        type PartialEq=True;
        type Ord=True;
        type PartialOrd=True;
        type Hash=True;
        type Iterator=True;
        type DoubleEndedIterator=True;
        type FmtWrite=True;
        type IoWrite=True;
        type IoSeek=True;
        type IoRead=True;
        type IoBufRead=True;
        type Error=True;
    }
}


#[test]
fn assert_all_traits_impld(){
    let _:<AllTraitsImpld as InterfaceType>::Send               =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Sync               =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Clone              =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Default            =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Display            =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Debug              =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Serialize          =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Deserialize        =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Eq                 =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::PartialEq          =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Ord                =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::PartialOrd         =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Hash               =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Iterator           =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::DoubleEndedIterator=Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::FmtWrite           =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::IoWrite            =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::IoSeek             =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::IoRead             =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::IoBufRead          =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Error              =Implemented::NEW;
}




#[repr(C)]
#[derive(StableAbi)]
pub struct NoTraitsImpld<T>(PhantomData<T>);

impl_InterfaceType!{
    impl<T> InterfaceType for NoTraitsImpld<T>{
        type Send=False;
        type Sync=False;
    }
}

#[test]
fn assert_all_traits_unimpld(){
    let _:<NoTraitsImpld<()> as InterfaceType>::Send               =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Sync               =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Clone              =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Default            =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Display            =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Debug              =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Serialize          =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Deserialize        =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Eq                 =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::PartialEq          =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Ord                =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::PartialOrd         =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Hash               =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Iterator           =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::DoubleEndedIterator=Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::FmtWrite           =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::IoWrite            =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::IoSeek             =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::IoRead             =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::IoBufRead          =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Error              =Unimplemented::NEW;
}



#[repr(C)]
#[derive(GetStaticEquivalent)]
pub struct FmtInterface<T>(PhantomData<T>)
where T:Debug;


impl_InterfaceType!{
    impl<T> InterfaceType for FmtInterface<T>
    where
        T:Debug
    {
        type Debug=True;
        type Display=True;
    }
}


#[test]
fn assert_fmt_traits_impld(){
    let _:<FmtInterface<()> as InterfaceType>::Send               =Implemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Sync               =Implemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Clone              =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Default            =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Display            =Implemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Debug              =Implemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Serialize          =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Deserialize        =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Eq                 =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::PartialEq          =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Ord                =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::PartialOrd         =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Hash               =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Iterator           =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::DoubleEndedIterator=Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::FmtWrite           =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::IoWrite            =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::IoSeek             =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::IoRead             =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::IoBufRead          =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Error              =Unimplemented::NEW;
}


#[repr(C)]
#[derive(GetStaticEquivalent)]
pub struct HashEqInterface<T>(PhantomData<T>)
where T:Debug;


impl_InterfaceType!{
    impl<T> InterfaceType for HashEqInterface<T>
    where
        T:Debug
    {
        type Hash=True;
        type PartialEq=True;
        type Eq=True;
    }
}


#[test]
fn assert_hash_eq_impld(){
    let _:<HashEqInterface<()> as InterfaceType>::Send               =Implemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::Sync               =Implemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::Clone              =Unimplemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::Default            =Unimplemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::Display            =Unimplemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::Debug              =Unimplemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::Serialize          =Unimplemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::Deserialize        =Unimplemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::Eq                 =Implemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::PartialEq          =Implemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::Ord                =Unimplemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::PartialOrd         =Unimplemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::Hash               =Implemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::Iterator           =Unimplemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::DoubleEndedIterator=Unimplemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::FmtWrite           =Unimplemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::IoWrite            =Unimplemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::IoSeek             =Unimplemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::IoRead             =Unimplemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::IoBufRead          =Unimplemented::NEW;
    let _:<HashEqInterface<()> as InterfaceType>::Error              =Unimplemented::NEW;
}