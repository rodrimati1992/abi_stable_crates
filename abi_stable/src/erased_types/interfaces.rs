use super::*;
use std::marker::PhantomData;

#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct CloneInterface;

crate::impl_InterfaceType!{
    impl crate::erased_types::InterfaceType for CloneInterface {
        type Clone=True;
    }
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct DefaultInterface;

crate::impl_InterfaceType!{
    impl crate::erased_types::InterfaceType for DefaultInterface {
        type Default=True;
    }
}



#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct IteratorInterface<T>(PhantomData<T>);

impl<T> IteratorInterface<T>{
    pub const NEW:Self=Self(PhantomData);
}

crate::impl_InterfaceType!{
    impl<T> crate::erased_types::InterfaceType for IteratorInterface<T> {
        type Iterator=True;
    }
}

impl<'a,T:'a> IteratorItem<'a> for IteratorInterface<T>{
    type Item=T;
}



#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct DEIteratorInterface<T>(PhantomData<T>);

impl<T> DEIteratorInterface<T>{
    pub const NEW:Self=Self(PhantomData);
}

crate::impl_InterfaceType!{
    impl<T> crate::erased_types::InterfaceType for DEIteratorInterface<T> {
        type Iterator=True;
        type DoubleEndedIterator=True;
    }
}

impl<'a,T:'a> IteratorItem<'a> for DEIteratorInterface<T>{
    type Item=T;
}