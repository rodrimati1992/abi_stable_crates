use super::*;
use std::marker::PhantomData;

/// Implements `InterfaceType`, requiring `Send + Sync + Clone`
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send, Sync, Clone))]
pub struct CloneInterface;

//////////////////////////////////////////////

/// Implements `InterfaceType`, requiring `Send + Sync + Debug + Clone + Eq`
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send, Sync, Debug, Clone, Eq))]
pub struct CloneEqInterface;

//////////////////////////////////////////////

/// Implements `InterfaceType`, requiring `Send + Sync + Debug + Clone + DoubleEndedIterator`
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send, Sync, Debug, Clone, DoubleEndedIterator))]
pub struct DEIteratorCloneInterface<T>(PhantomData<T>);

impl<T> DEIteratorCloneInterface<T> {
    pub const NEW: Self = Self(PhantomData);
}

impl<'a, T: 'a> IteratorItem<'a> for DEIteratorCloneInterface<T> {
    type Item = T;
}

//////////////////////////////////////////////

/// Implements `InterfaceType`, requiring `Send + Sync + Default`
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send, Sync, Default))]
pub struct DefaultInterface;

//////////////////////////////////////////////

/// Implements `InterfaceType`, requiring `Unpin`
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Unpin))]
pub struct UnpinInterface;

//////////////////////////////////////////////

/// Implements `InterfaceType`, requiring `Send + Sync + Debug + Eq + Default`
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send, Sync, Debug, Eq, Default))]
pub struct DebugDefEqInterface;

//////////////////////////////////////////////

/// Implements `InterfaceType`, requiring `Send + Sync + Debug + PartialEq`
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send, Sync, Debug, PartialEq))]
pub struct PartialEqInterface;

//////////////////////////////////////////////

/// Implements `InterfaceType`, requiring `Send + Sync + Debug + std::fmt::Write`
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send, Sync, Debug, FmtWrite))]
pub struct FmtWriteInterface;

//////////////////////////////////////////////

/// Implements `InterfaceType`, requiring `std::io::Write`
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(IoWrite))]
pub struct IoWriteInterface;

//////////////////////////////////////////////

/// Implements `InterfaceType`, requiring `Send + Sync + Debug + Display`
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send, Sync, Debug, Display))]
pub struct DebugDisplayInterface;

//////////////////////////////////////////////

/// Implements `InterfaceType`, requiring `Send + Sync + Iterator<Item = T>`
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send, Sync, Iterator))]
pub struct IteratorInterface<T>(PhantomData<T>);

impl<T> IteratorInterface<T> {
    pub const NEW: Self = Self(PhantomData);
}

impl<'a, T: 'a> IteratorItem<'a> for IteratorInterface<T> {
    type Item = T;
}

//////////////////////////////////////////////

/// Implements `InterfaceType`, requiring `Send + Sync + DoubleEndedIterator<Item = T>`
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send, Sync, DoubleEndedIterator))]
pub struct DEIteratorInterface<T>(PhantomData<T>);

impl<T> DEIteratorInterface<T> {
    pub const NEW: Self = Self(PhantomData);
}

impl<'a, T: 'a> IteratorItem<'a> for DEIteratorInterface<T> {
    type Item = T;
}
