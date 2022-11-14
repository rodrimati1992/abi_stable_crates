use std::marker::PhantomData;

use crate::{
    type_level::impl_enum::{Implemented, Unimplemented},
    GetStaticEquivalent, InterfaceType, StableAbi,
};

use core_extensions::type_asserts::AssertEq;

////////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(
    Send,
    Sync,
    Unpin,
    Clone,
    Default,
    Display,
    Debug,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Iterator,
    DoubleEndedIterator,
    FmtWrite,
    IoWrite,
    IoSeek,
    IoRead,
    IoBufRead,
    Error
))]
pub struct AllTraitsImpld;

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

////////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(StableAbi)]
// #[sabi(debug_print)]
#[sabi(impl_InterfaceType())]
pub struct NoTraitsImpld<T>(PhantomData<T>);

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

////////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(GetStaticEquivalent)]
#[sabi(impl_InterfaceType(Debug, Display))]
pub struct FmtInterface<T>(PhantomData<T>)
where
    T: std::fmt::Debug;

#[test]
fn assert_fmt_traits_impld() {
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::Send, Unimplemented<_>>;
    let _: AssertEq<<FmtInterface<()> as InterfaceType>::Sync, Unimplemented<_>>;
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

////////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(GetStaticEquivalent)]
#[sabi(impl_InterfaceType(Hash, Ord))]
pub struct HashOrdInterface<T>(PhantomData<T>)
where
    T: std::fmt::Debug;

#[test]
fn assert_hash_ord_impld() {
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::Send, Unimplemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::Sync, Unimplemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::Unpin, Unimplemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::Clone, Unimplemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::Default, Unimplemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::Display, Unimplemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::Debug, Unimplemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::Serialize, Unimplemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::Deserialize, Unimplemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::Eq, Implemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::PartialEq, Implemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::Ord, Implemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::PartialOrd, Implemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::Hash, Implemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::Iterator, Unimplemented<_>>;
    let _: AssertEq<
        <HashOrdInterface<()> as InterfaceType>::DoubleEndedIterator,
        Unimplemented<_>,
    >;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::FmtWrite, Unimplemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::IoWrite, Unimplemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::IoSeek, Unimplemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::IoRead, Unimplemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::IoBufRead, Unimplemented<_>>;
    let _: AssertEq<<HashOrdInterface<()> as InterfaceType>::Error, Unimplemented<_>>;
}

////////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(GetStaticEquivalent)]
#[sabi(impl_InterfaceType(Eq))]
pub struct OnlyEq;

#[test]
fn assert_only_eq() {
    let _: AssertEq<<OnlyEq as InterfaceType>::Send, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::Sync, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::Unpin, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::Clone, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::Default, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::Display, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::Debug, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::Serialize, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::Deserialize, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::Eq, Implemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::PartialEq, Implemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::Ord, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::PartialOrd, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::Hash, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::Iterator, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::DoubleEndedIterator, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::FmtWrite, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::IoWrite, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::IoSeek, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::IoRead, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::IoBufRead, Unimplemented<_>>;
    let _: AssertEq<<OnlyEq as InterfaceType>::Error, Unimplemented<_>>;
}

////////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(GetStaticEquivalent)]
#[sabi(impl_InterfaceType(PartialOrd))]
pub struct OnlyPartialOrd;

#[test]
fn assert_only_partial_ord() {
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::Send, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::Sync, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::Unpin, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::Clone, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::Default, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::Display, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::Debug, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::Serialize, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::Deserialize, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::Eq, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::PartialEq, Implemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::Ord, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::PartialOrd, Implemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::Hash, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::Iterator, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::DoubleEndedIterator, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::FmtWrite, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::IoWrite, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::IoSeek, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::IoRead, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::IoBufRead, Unimplemented<_>>;
    let _: AssertEq<<OnlyPartialOrd as InterfaceType>::Error, Unimplemented<_>>;
}

////////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(GetStaticEquivalent)]
#[sabi(impl_InterfaceType(Error))]
pub struct OnlyError;

#[test]
fn assert_only_error() {
    let _: AssertEq<<OnlyError as InterfaceType>::Send, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::Sync, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::Unpin, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::Clone, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::Default, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::Display, Implemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::Debug, Implemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::Serialize, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::Deserialize, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::Eq, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::PartialEq, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::Ord, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::PartialOrd, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::Hash, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::Iterator, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::DoubleEndedIterator, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::FmtWrite, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::IoWrite, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::IoSeek, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::IoRead, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::IoBufRead, Unimplemented<_>>;
    let _: AssertEq<<OnlyError as InterfaceType>::Error, Implemented<_>>;
}

////////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(GetStaticEquivalent)]
#[sabi(impl_InterfaceType(Iterator))]
pub struct OnlyIter;

#[test]
fn assert_only_iter() {
    let _: AssertEq<<OnlyIter as InterfaceType>::Send, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::Sync, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::Unpin, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::Clone, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::Default, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::Display, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::Debug, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::Serialize, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::Deserialize, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::Eq, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::PartialEq, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::Ord, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::PartialOrd, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::Hash, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::Iterator, Implemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::DoubleEndedIterator, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::FmtWrite, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::IoWrite, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::IoSeek, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::IoRead, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::IoBufRead, Unimplemented<_>>;
    let _: AssertEq<<OnlyIter as InterfaceType>::Error, Unimplemented<_>>;
}

////////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(GetStaticEquivalent)]
#[sabi(impl_InterfaceType(DoubleEndedIterator))]
pub struct OnlyDEIter;

#[test]
fn assert_only_de_iter() {
    let _: AssertEq<<OnlyDEIter as InterfaceType>::Send, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::Sync, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::Unpin, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::Clone, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::Default, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::Display, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::Debug, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::Serialize, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::Deserialize, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::Eq, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::PartialEq, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::Ord, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::PartialOrd, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::Hash, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::Iterator, Implemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::DoubleEndedIterator, Implemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::FmtWrite, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::IoWrite, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::IoSeek, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::IoRead, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::IoBufRead, Unimplemented<_>>;
    let _: AssertEq<<OnlyDEIter as InterfaceType>::Error, Unimplemented<_>>;
}
