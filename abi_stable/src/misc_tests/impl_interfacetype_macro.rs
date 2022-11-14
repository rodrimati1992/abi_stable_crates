use std::{fmt::Debug, marker::PhantomData};

use crate::{
    impl_InterfaceType,
    type_level::{
        bools::{False, True},
        impl_enum::{Implemented, Unimplemented},
    },
    GetStaticEquivalent, InterfaceType, StableAbi,
};

use core_extensions::type_asserts::AssertEq;

#[repr(C)]
#[derive(StableAbi)]
pub struct AllTraitsImpld;

impl_InterfaceType! {
    impl InterfaceType for AllTraitsImpld{
        type Send=True;
        type Sync=True;
        type Unpin=True;
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
fn assert_all_traits_impld() {
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::Send, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::Sync, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::Unpin, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::Clone, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::Default, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::Display, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::Debug, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::Serialize, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::Deserialize, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::Eq, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::PartialEq, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::Ord, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::PartialOrd, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::Hash, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::Iterator, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::DoubleEndedIterator, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::FmtWrite, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::IoWrite, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::IoSeek, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::IoRead, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::IoBufRead, Implemented<_>>;
    let _: AssertEq<<AllTraitsImpld as InterfaceType>::Error, Implemented<_>>;
}

#[repr(C)]
#[derive(StableAbi)]
pub struct NoTraitsImpld<T>(PhantomData<T>);

impl_InterfaceType! {
    impl<T> InterfaceType for NoTraitsImpld<T>{
        type Send=False;
        type Sync=False;
    }
}

#[test]
fn assert_all_traits_unimpld() {
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::Send, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::Sync, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::Unpin, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::Clone, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::Default, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::Display, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::Debug, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::Serialize, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::Deserialize, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::Eq, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::PartialEq, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::Ord, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::PartialOrd, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::Hash, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::Iterator, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::DoubleEndedIterator, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::FmtWrite, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::IoWrite, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::IoSeek, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::IoRead, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::IoBufRead, Unimplemented<_>>;
    let _: AssertEq<<NoTraitsImpld<()> as InterfaceType>::Error, Unimplemented<_>>;
}

#[repr(C)]
#[derive(GetStaticEquivalent)]
pub struct FmtInterface<T>(PhantomData<T>)
where
    T: Debug;

impl_InterfaceType! {
    impl<T> InterfaceType for FmtInterface<T>
    where
        T:Debug
    {
        type Send=True;
        type Sync=True;
        type Debug=True;
        type Display=True;
    }
}

#[test]
fn assert_fmt_traits_impld() {
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::Send, Implemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::Sync, Implemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::Unpin, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::Clone, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::Default, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::Display, Implemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::Debug, Implemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::Serialize, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::Deserialize, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::Eq, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::PartialEq, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::Ord, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::PartialOrd, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::Hash, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::Iterator, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::DoubleEndedIterator, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::FmtWrite, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::IoWrite, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::IoSeek, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::IoRead, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::IoBufRead, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::Error, Unimplemented<_>>;
}

#[repr(C)]
#[derive(GetStaticEquivalent)]
pub struct HashEqInterface<T>(PhantomData<T>)
where
    T: Debug;

impl_InterfaceType! {
    impl<T> InterfaceType for HashEqInterface<T>
    where
        T:Debug
    {
        type Send=True;
        type Sync=True;
        type Hash=True;
        type PartialEq=True;
        type Eq=True;
    }
}

#[test]
fn assert_hash_eq_impld() {
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::Send, Implemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::Sync, Implemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::Unpin, Unimplemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::Clone, Unimplemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::Default, Unimplemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::Display, Unimplemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::Debug, Unimplemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::Serialize, Unimplemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::Deserialize, Unimplemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::Eq, Implemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::PartialEq, Implemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::Ord, Unimplemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::PartialOrd, Unimplemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::Hash, Implemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::Iterator, Unimplemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::DoubleEndedIterator, Unimplemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::FmtWrite, Unimplemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::IoWrite, Unimplemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::IoSeek, Unimplemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::IoRead, Unimplemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::IoBufRead, Unimplemented<_>>;
    let _: AssertEq<<HashEqInterface<()> as InterfaceType>::Error, Unimplemented<_>>;
}
