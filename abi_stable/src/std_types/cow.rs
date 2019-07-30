use std::{borrow::{Borrow,Cow}, fmt, ops::Deref};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[allow(unused_imports)]
use core_extensions::{
    prelude::*,
    matches,
};

use crate::{
    StableAbi, 
    std_types::{RSlice, RStr, RString, RVec},
    traits::IntoReprC,
};

// #[cfg(test)]
#[cfg(all(test,not(feature="only_new_tests")))]
mod tests;


////////////////////////////////////////////////////////////////////


/// The main bound of `RCow<_>`.
///
/// All the methods in this trait convert the parameter to the return type.
pub trait BorrowOwned<'a>: 'a + ToOwned {
    type ROwned;
    type RBorrowed: 'a + Copy ;
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
    T: Clone,
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
    T: Clone,
{
    type ROwned = T;
    type RBorrowed = &'a T;

    #[inline]
    fn r_borrow(this: &'a Self::ROwned) -> Self::RBorrowed {
        this
    }
    #[inline]
    fn r_to_owned(this: Self::RBorrowed) -> Self::ROwned {
        this.clone()
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
        this
    }
    #[inline]
    fn from_cow_owned(this: <Self as ToOwned>::Owned) -> Self::ROwned {
        this
    }
    #[inline]
    fn into_cow_borrow(this: Self::RBorrowed) -> &'a Self {
        this
    }
    #[inline]
    fn into_cow_owned(this: Self::ROwned) -> <Self as ToOwned>::Owned {
        this
    }
}

////////////////////////////////////////////////////////////////////

/**
Ffi-safe equivalent of ::std::borrow::Cow.

The most common examples of this type are:

- `RCow<'_,str>`: contains an RStr<'_> or an RString.

- `RCow<'_,[T]>`: contains an RSlice<'_,T> or an RVec<T>.

- `RCow<'_,T>`: contains a `&T` or a `T`.

# Example

### Using a `RCow<'a,str>`.

This implements a solution to the well known fizzbuzz problem.

```
use abi_stable::std_types::RCow;

fn fizzbuzz(n:u32)->RCow<'static,str>{
    match (n%3,n%5) {
        (0,0)=>RCow::from("FizzBuzz"),
        (0,_)=>RCow::from("Fizz"),
        (_,0)=>RCow::from("Buzz"),
        (_,_)=>RCow::from(n.to_string()),
    }
}

for n in 1..=100{
    println!("{}",fizzbuzz(n));
}

```

Note:this example allocates when the number is neither a multiple of 5 or 3.


*/
#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    not_stableabi(B),
    bound="<B as BorrowOwned<'a>>::RBorrowed: StableAbi",
    bound="<B as BorrowOwned<'a>>::ROwned   : StableAbi",
)]
pub enum RCow<'a, B>
where
    B: BorrowOwned<'a> + ?Sized,
{
    Borrowed(<B as BorrowOwned<'a>>::RBorrowed),
    Owned(<B as BorrowOwned<'a>>::ROwned),
}

use self::RCow::{Borrowed, Owned};


// ///////////////////////////////////////////////////////////////////////////

impl<'a, B> RCow<'a, B>
where
    B: BorrowOwned<'a>+?Sized,
{
    /// Get a mutable reference to the owner form of RCow,
    /// converting to the owned form if it is currently the borrowed form.
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::std_types::RCow;
    /// 
    /// let mut cow:RCow<'_,str>=RCow::from("Hello");
    /// 
    /// assert_eq!(&*cow,"Hello");
    /// assert!(cow.is_borrowed());
    /// 
    /// cow.to_mut().push_str(", world!");
    /// 
    /// assert!(cow.is_owned());
    /// assert_eq!(cow,RCow::from("Hello, world!"));
    /// 
    /// ```
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
    /// Unwraps into the owned owner form of RCow,
    /// converting to the owned form if it is currently the borrowed form.
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::std_types::RCow;
    /// 
    /// let mut cow:RCow<'_,str>=RCow::from("Hello");
    ///
    /// assert_eq!(&*cow,"Hello");
    /// 
    /// let mut buff=cow.into_owned();
    /// buff.push_str(", world!");
    /// 
    /// assert_eq!(&*buff,"Hello, world!");
    /// 
    /// ```
    pub fn into_owned(self) -> B::ROwned {
        match self {
            Borrowed(x) => B::r_to_owned(x),
            Owned(x) => x,
        }
    }

    /// Gets the contents of the RCow casted to the borrowed variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::std_types::{RCow,RSlice};
    /// {
    ///     let cow:RCow<'_,[u8]>=RCow::from(&[0,1,2,3][..]);
    ///     assert_eq!( cow.borrowed(), RSlice::from_slice(&[0,1,2,3]) );
    /// }
    /// {
    ///     let cow:RCow<'_,[u8]>=RCow::from(vec![0,1,2,3]);
    ///     assert_eq!( cow.borrowed(), RSlice::from_slice(&[0,1,2,3]) );
    /// }
    /// ```
    pub fn borrowed<'b:'a>(&'b self)-><B as BorrowOwned<'b>>::RBorrowed{
        match self {
            Borrowed(x) => *x,
            Owned(x) => B::r_borrow(x),
        }
    }

    /// Whether this is a borrowing RCow.
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::std_types::RCow;
    /// 
    /// {
    ///     let cow:RCow<'_,[u8]>=RCow::from(&[0,1,2,3][..]);
    ///     assert!( cow.is_borrowed() );
    /// }
    /// {
    ///     let cow:RCow<'_,[u8]>=RCow::from(vec![0,1,2,3]);
    ///     assert!( !cow.is_borrowed() );
    /// }
    /// 
    /// ```
    pub fn is_borrowed(&self)->bool{
        matches!( Borrowed{..}=self )
    }

    /// Whether this is an owning RCow.
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::std_types::RCow;
    /// 
    /// let cow:RCow<'_,[u8]>=RCow::from(&[0,1,2,3][..]);
    /// assert!( !cow.is_owned() );
    /// 
    /// let cow:RCow<'_,[u8]>=RCow::from(vec![0,1,2,3]);
    /// assert!( cow.is_owned() );
    /// 
    /// ```
    pub fn is_owned(&self)->bool{
        matches!( Owned{..}=self )
    }
}


#[allow(dead_code)]
#[cfg(test)]
impl<'a, B> RCow<'a, B>
where
    B: BorrowOwned<'a>+?Sized,
{
    /// Access this as a borrowing RCow.Returns None if it's not a borrowing one.
    fn as_borrowed(&self)->Option<B::RBorrowed>{
        match *self {
            Borrowed(x) => Some(x),
            Owned(_) => None,
        }
    }

    /// Access this as an owned RCow.Returns None if it's not an owned one.
    fn as_owned(&self)->Option<&B::ROwned>{
        match self {
            Borrowed(_) => None,
            Owned(x) => Some(x),
        }
    }
}


impl<'a, B> Copy for RCow<'a, B>
where
    B: BorrowOwned<'a>+?Sized,
    B::ROwned: Copy,
{
}

impl<'a, B> Clone for RCow<'a, B>
where
    B: BorrowOwned<'a>+?Sized,
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
    B: BorrowOwned<'a>+?Sized,
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

////////////////////


impl<'a,B> Borrow<B> for RCow<'a, B>
where
    B: BorrowOwned<'a>+?Sized,
{
    fn borrow(&self)->&B{
        self
    }
}


impl<'a,B> AsRef<B> for RCow<'a, B>
where
    B: BorrowOwned<'a>+?Sized,
{
    fn as_ref(&self)->&B{
        self
    }
}

////////////////////////////

shared_impls! {
    mod=slice_impls
    new_type=RCow['a][] 
    extra[B]
    constrained[B]
    where [ B:BorrowOwned<'a>+?Sized ],
    original_type=void,
}

impl_into_rust_repr! {
    impl['a,B] Into<Cow<'a,B>> for RCow<'a,B>
    where[
        B: BorrowOwned<'a>+?Sized,
    ]{
        fn(this){
            match this{
                RCow::Borrowed(x)=>x.piped(B::into_cow_borrow).piped(Cow::Borrowed),
                RCow::Owned(x)=>x.piped(B::into_cow_owned).piped(Cow::Owned),
            }
        }
    }
}


////////////////////////////////////////////////////////////


impl_from_rust_repr! {
    impl['a,B] From<Cow<'a,B>> for RCow<'a,B>
    where [ 
        B: BorrowOwned<'a>+?Sized , 
    ]{
        fn(this){
            match this{
                Cow::Borrowed(x)=>x.piped(B::from_cow_borrow).piped(RCow::Borrowed),
                Cow::Owned(x)=>x.piped(B::from_cow_owned).piped(RCow::Owned),
            }
        }
    }
}



impl<'a> From<&'a str> for RCow<'a,str>{
    #[inline]
    fn from(this:&'a str)->Self{
        RCow::Borrowed(this.into_c())
    }
}

impl<'a> From<RStr<'a>> for RCow<'a,str>{
    #[inline]
    fn from(this:RStr<'a>)->Self{
        RCow::Borrowed(this)
    }
}

impl<'a> From<String> for RCow<'a,str>{
    #[inline]
    fn from(this:String)->Self{
        RCow::Owned(this.into())
    }
}

impl<'a> From<&'a String> for RCow<'a,str>{
    #[inline]
    fn from(this:&'a String)->Self{
        RCow::Borrowed(this.as_str().into())
    }
}

impl<'a> From<RString> for RCow<'a,str>{
    #[inline]
    fn from(this:RString)->Self{
        RCow::Owned(this)
    }
}

impl<'a> From<&'a RString> for RCow<'a,str>{
    #[inline]
    fn from(this:&'a RString)->Self{
        RCow::Borrowed(this.as_rstr())
    }
}



impl<'a,T> From<&'a [T]> for RCow<'a,[T]>
where 
    T:Clone
{
    #[inline]
    fn from(this:&'a [T])->Self{
        RCow::Borrowed(RSlice::from(this))
    }
}

impl<'a,T> From<RSlice<'a,T>> for RCow<'a,[T]>
where 
    T:Clone
{
    #[inline]
    fn from(this:RSlice<'a,T>)->Self{
        RCow::Borrowed(this)
    }
}

impl<'a,T> From<Vec<T>> for RCow<'a,[T]>
where 
    T:Clone
{
    #[inline]
    fn from(this:Vec<T>)->Self{
        RCow::Owned(RVec::from(this))
    }
}

impl<'a,T> From<RVec<T>> for RCow<'a,[T]>
where 
    T:Clone
{
    #[inline]
    fn from(this:RVec<T>)->Self{
        RCow::Owned(this)
    }
}


////////////////////////////////////////////////////////////


impl<'a, B> fmt::Display for RCow<'a, B>
where
    B: BorrowOwned<'a> +?Sized,
    B: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}


////////////////////////////////////////////////////////////



/// Deserializes an `RCow<'a,[u8]>` that borrows the slice from the deserializer 
/// whenever possible.
pub fn deserialize_borrowed_bytes<'de,'a,D>(deserializer: D) -> Result<RCow<'a, [u8]>, D::Error>
where
    D: Deserializer<'de>,
    'de:'a
{
    #[derive(Deserialize)]
    struct BorrowingCowSlice<'a>{
        #[serde(borrow)]
        cow:Cow<'a,[u8]>
    }

    <BorrowingCowSlice<'de> as Deserialize<'de>>::deserialize(deserializer)
        .map(|x|{
            match x.cow {
                Cow::Borrowed(y)=>RCow::Borrowed(y.into()),
                Cow::Owned(y)   =>RCow::Owned(y.into()),
            }
        })
}

/// Deserializes an `RCow<'a,str>` that borrows the string from the deserializer 
/// whenever possible.
pub fn deserialize_borrowed_str<'de,'a,D>(deserializer: D) -> Result<RCow<'a, str>, D::Error>
where
    D: Deserializer<'de>,
    'de:'a
{
    #[derive(Deserialize)]
    struct BorrowingCowStr<'a>(
        #[serde(borrow)]
        Cow<'a,str>
    );

    <BorrowingCowStr<'de> as Deserialize<'de>>::deserialize(deserializer)
        .map(|x| RCow::from(x.0) )
}

impl<'de, 'a,T> Deserialize<'de> for RCow<'a, [T]>
where 
    T:Clone+Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <RVec<T>>::deserialize(deserializer)
            .map(RCow::<'a,[T]>::Owned)
    }
}



impl<'de,'a> Deserialize<'de> for RCow<'a, str>{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Cow<'a,str> as Deserialize<'de>>::deserialize(deserializer)
            .map(RCow::from)
    }
}

impl<'de, 'a, T> Deserialize<'de> for RCow<'a, T>
where
    T: Clone+Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {

        <T as Deserialize<'de>>::deserialize(deserializer)
            .map(RCow::Owned)
    }
}

impl<'a, B> Serialize for RCow<'a, B>
where
    B: BorrowOwned<'a>+?Sized,
    B: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (&**self).serialize(serializer)
    }
}


/// A helper type,to deserialize a RCow<'a,[u8]> which borrows from the deserializer.
#[derive(Deserialize)]
#[serde(transparent)]
pub struct BorrowingRCowU8Slice<'a>{
    #[serde(borrow,deserialize_with="deserialize_borrowed_bytes")]
    pub cow:RCow<'a,[u8]>
}

/// A helper type,to deserialize a RCow<'a,str> which borrows from the deserializer.
#[derive(Deserialize)]
#[serde(transparent)]
pub struct BorrowingRCowStr<'a>{
    #[serde(borrow,deserialize_with="deserialize_borrowed_str")]
    pub cow:RCow<'a,str>
}




//////////////////////////////////////////////////////////////////////////////////////

