use super::*;
use std::marker::PhantomData;

#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send,Sync,Clone))]
pub struct CloneInterface;

//////////////////////////////////////////////

#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send,Sync,Debug,Clone,Eq))]
pub struct CloneEqInterface;


//////////////////////////////////////////////


#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send,Sync,Default))]
pub struct DefaultInterface;


//////////////////////////////////////////////

#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send,Sync,Debug,PartialEq))]
pub struct PartialEqInterface;


//////////////////////////////////////////////

#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send,Sync,Debug,FmtWrite))]
pub struct FmtWriteInterface;


//////////////////////////////////////////////

#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(IoWrite))]
pub struct IoWriteInterface;


//////////////////////////////////////////////


#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send,Sync,Iterator))]
pub struct IteratorInterface<T>(PhantomData<T>);

impl<T> IteratorInterface<T>{
    pub const NEW:Self=Self(PhantomData);
}

impl<'a,T:'a> IteratorItem<'a> for IteratorInterface<T>{
    type Item=T;
}


//////////////////////////////////////////////



#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send,Sync,DoubleEndedIterator))]
pub struct DEIteratorInterface<T>(PhantomData<T>);

impl<T> DEIteratorInterface<T>{
    pub const NEW:Self=Self(PhantomData);
}

impl<'a,T:'a> IteratorItem<'a> for DEIteratorInterface<T>{
    type Item=T;
}

