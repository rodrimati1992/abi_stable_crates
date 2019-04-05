use std::{borrow::Cow, fmt, ops::Deref};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use crate::{reexports::StableAbi, traits::FromElement, CAbi, RSlice, RStr, RString, RVec};

////////////////////////////////////////////////////////////////////

pub trait BorrowOwned<'a>: 'a + ToOwned {
    type ROwned: StableAbi;
    type RBorrowed: 'a + Copy + StableAbi;
    fn r_borrow(this: &'a Self::ROwned) -> Self::RBorrowed;
    fn r_to_owned(this: Self::RBorrowed) -> Self::ROwned;
    fn deref_borrowed(this: &Self::RBorrowed) -> &Self;
    fn deref_owned(this: &Self::ROwned) -> &Self;
    fn from_cow_borrow(this: &'a Self) -> Self::RBorrowed;
    fn from_cow_owned(this: <Self as ToOwned>::Owned) -> Self::ROwned;
    fn into_cow_borrow(this: Self::RBorrowed) -> &'a Self;
    fn into_cow_owned(this: Self::ROwned) -> <Self as ToOwned>::Owned;
}

impl<'a> BorrowOwned<'a> for str {
    type ROwned = RString;
    type RBorrowed = RStr<'a>;
    #[inline]
    fn r_borrow(this: &'a Self::ROwned) -> Self::RBorrowed {
        this.as_rstr()
    }
    #[inline]
    fn r_to_owned(this: Self::RBorrowed) -> Self::ROwned {
        this.into()
    }
    #[inline]
    fn deref_borrowed(this: &Self::RBorrowed) -> &Self {
        this
    }
    #[inline]
    fn deref_owned(this: &Self::ROwned) -> &Self {
        this
    }
    #[inline]
    fn from_cow_borrow(this: &'a Self) -> Self::RBorrowed {
        this.into()
    }
    #[inline]
    fn from_cow_owned(this: <Self as ToOwned>::Owned) -> Self::ROwned {
        this.into()
    }
    #[inline]
    fn into_cow_borrow(this: Self::RBorrowed) -> &'a Self {
        this.into()
    }
    #[inline]
    fn into_cow_owned(this: Self::ROwned) -> <Self as ToOwned>::Owned {
        this.into()
    }
}

impl<'a, T: 'a> BorrowOwned<'a> for [T]
where
    T: Clone + StableAbi,
{
    type ROwned = RVec<T>;
    type RBorrowed = RSlice<'a, T>;
    #[inline]
    fn r_borrow(this: &'a Self::ROwned) -> Self::RBorrowed {
        this.as_rslice()
    }
    #[inline]
    fn r_to_owned(this: Self::RBorrowed) -> Self::ROwned {
        this.to_rvec()
    }
    #[inline]
    fn deref_borrowed(this: &Self::RBorrowed) -> &Self {
        this
    }
    #[inline]
    fn deref_owned(this: &Self::ROwned) -> &Self {
        this
    }
    #[inline]
    fn from_cow_borrow(this: &'a Self) -> Self::RBorrowed {
        this.into()
    }
    #[inline]
    fn from_cow_owned(this: <Self as ToOwned>::Owned) -> Self::ROwned {
        this.into()
    }
    #[inline]
    fn into_cow_borrow(this: Self::RBorrowed) -> &'a Self {
        this.into()
    }
    #[inline]
    fn into_cow_owned(this: Self::ROwned) -> <Self as ToOwned>::Owned {
        this.into()
    }
}

impl<'a, T: 'a> BorrowOwned<'a> for T
where
    T: Clone + StableAbi,
{
    type ROwned = T;
    type RBorrowed = CAbi<&'a T>;

    #[inline]
    fn r_borrow(this: &'a Self::ROwned) -> Self::RBorrowed {
        this.into()
    }
    #[inline]
    fn r_to_owned(this: Self::RBorrowed) -> Self::ROwned {
        (*this).clone()
    }
    #[inline]
    fn deref_borrowed(this: &Self::RBorrowed) -> &Self {
        this
    }
    #[inline]
    fn deref_owned(this: &Self::ROwned) -> &Self {
        this
    }
    #[inline]
    fn from_cow_borrow(this: &'a Self) -> Self::RBorrowed {
        this.into()
    }
    #[inline]
    fn from_cow_owned(this: <Self as ToOwned>::Owned) -> Self::ROwned {
        this
    }
    #[inline]
    fn into_cow_borrow(this: Self::RBorrowed) -> &'a Self {
        this.into_inner()
    }
    #[inline]
    fn into_cow_owned(this: Self::ROwned) -> <Self as ToOwned>::Owned {
        this
    }
}

////////////////////////////////////////////////////////////////////

#[repr(C)]
pub enum RCow<'a, B>
where
    B: BorrowOwned<'a> + ?Sized,
{
    Borrowed(<B as BorrowOwned<'a>>::RBorrowed),
    Owned(<B as BorrowOwned<'a>>::ROwned),
}

use self::RCow::{Borrowed, Owned};

///////////////////////////////////////////////////////////////////////////

// Have to implement StableAbi manually for now,
// because fields whose types are associated types cause
// an (already reported by someone else) ICE.
// ICE Reports:
//  https://github.com/rust-lang/rust/issues/58944
//  https://github.com/rust-lang/rust/issues/59324
//
// Try deriving this trait in 1.34 stable,which will receive a fix for this problem.
mod _stable_abi_impls_for_rcow {
    use super::*;
    use crate as abi_stable;
    #[allow(unused_imports)]
    use crate::reexports::{self as _sabi_reexports, renamed::*};
    unsafe impl<'a, B, Owned, Borrowed> __StableAbi for RCow<'a, B>
    where
        B: BorrowOwned<'a, RBorrowed = Borrowed, ROwned = Owned> + ?Sized,
        Owned: StableAbi,
        Borrowed: StableAbi + Copy + 'a,
    {
        type IsNonZeroType = _sabi_reexports::False;
        const LAYOUT: &'static _sabi_reexports::TypeLayout = {
            &_sabi_reexports::TypeLayout::from_params::<Self>({
                __TypeLayoutParams {
                    name: "RCow",
                    package: env!("CARGO_PKG_NAME"),
                    package_version: abi_stable::package_version_string!(),
                    data: __TLData::enum_(&[
                        __TLEnumVariant::new(
                            stringify!(Borrowed),
                            &[__TLField::new(
                                "field_0",
                                &[__LIParam(0usize)],
                                <Borrowed as __MakeGetAbiInfo<__StableAbi_Bound>>::CONST
                            )],
                        ),
                        __TLEnumVariant::new(
                            stringify!(Owned),
                            &[__TLField::new(
                                "field_0",
                                &[__LIParam(0usize)],
                                <Owned as __MakeGetAbiInfo<__StableAbi_Bound>>::CONST,
                            )],
                        ),
                    ]),
                    generics: abi_stable :: tl_genparams ! ( 'a ; ; ),
                    phantom_fields: &[],
                }
            })
        };
    }
}

///////////////////////////////////////////////////////////////////////////

impl<'a, B> RCow<'a, B>
where
    B: BorrowOwned<'a> + ?Sized,
{
    pub fn to_mut(&mut self) -> &mut B::ROwned {
        if let Borrowed(v) = *self {
            let owned = B::r_to_owned(v);
            *self = Owned(owned)
        }
        match self {
            Borrowed(_) => loop {},
            Owned(v) => v,
        }
    }
    pub fn into_owned(self) -> B::ROwned {
        match self {
            Borrowed(x) => B::r_to_owned(x),
            Owned(x) => x,
        }
    }
}

impl<'a, B> Copy for RCow<'a, B>
where
    B: BorrowOwned<'a> + ?Sized,
    B::ROwned: Copy,
{
}

impl<'a, B> Clone for RCow<'a, B>
where
    B: BorrowOwned<'a> + ?Sized,
    B::ROwned: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Borrowed(x) => Borrowed(x.clone()),
            Owned(x) => Owned((*x).clone()),
        }
    }
}

impl<'a, B> Deref for RCow<'a, B>
where
    B: BorrowOwned<'a> + ?Sized,
{
    type Target = B;
    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Borrowed(x) => B::deref_borrowed(x),
            Owned(x) => B::deref_owned(x),
        }
    }
}

shared_impls! {
    mod=slice_impls
    new_type=RCow['a][B] where [ B:BorrowOwned<'a>+?Sized ],
    original_type=B,
}

/// Had to choose the element conversion method used.
/// Chose to convert only from the owned variant.
impl<'a, B> FromElement for RCow<'a, B>
where
    B: BorrowOwned<'a> + ?Sized,
{
    type Element = B::ROwned;

    #[inline]
    fn from_elem(val: Self::Element) -> Self {
        RCow::Owned(val)
    }
}

impl_into_rust_repr! {
    impl['a,B] Into<Cow<'a,B>> for RCow<'a,B>
    where[
        B: BorrowOwned<'a> + ?Sized,
    ]{
        fn(this){
            match this{
                RCow::Borrowed(x)=>x.piped(B::into_cow_borrow).piped(Cow::Borrowed),
                RCow::Owned(x)=>x.piped(B::into_cow_owned).piped(Cow::Owned),
            }
        }
    }
}

impl_from_rust_repr! {
    impl['a,B] From<Cow<'a,B>> for RCow<'a,B>
    where[
        B: BorrowOwned<'a> + ?Sized,
    ]{
        fn(this){
            match this{
                Cow::Borrowed(x)=>x.piped(B::from_cow_borrow).piped(RCow::Borrowed),
                Cow::Owned(x)=>x.piped(B::from_cow_owned).piped(RCow::Owned),
            }
        }
    }
}

impl<'a, B> fmt::Display for RCow<'a, B>
where
    B: BorrowOwned<'a> + ?Sized + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<'de, 'a, B> Deserialize<'de> for RCow<'a, B>
where
    B: BorrowOwned<'a> + ?Sized,
    Cow<'a, B>: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Cow<'a, B> as Deserialize<'de>>::deserialize(deserializer).map(From::from)
    }
}

impl<'a, B> Serialize for RCow<'a, B>
where
    B: BorrowOwned<'a> + ?Sized + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (&**self).serialize(serializer)
    }
}

//////////////////////////////////////////////////////////////////////////////////////
