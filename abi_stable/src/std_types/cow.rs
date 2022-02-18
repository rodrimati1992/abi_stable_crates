//! Contains the ffi-safe equivalent of `std::borrow::Cow`, and related items.

use std::{
    borrow::{Borrow, Cow},
    fmt,
    ops::Deref,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[allow(unused_imports)]
use core_extensions::{matches, SelfOps};

use crate::{
    std_types::{RSlice, RStr, RString, RVec},
    traits::IntoReprC,
    StableAbi,
};

// #[cfg(test)]
#[cfg(all(test, not(feature = "only_new_tests")))]
mod tests;

////////////////////////////////////////////////////////////////////

// TODO: documentation

pub trait QueryOwnedType: Deref {
    type Owned: Borrow<Self::Target>;
}

pub trait IntoOwned: Copy + QueryOwnedType {
    fn into_owned(self) -> Self::Owned;
}

impl<T> QueryOwnedType for &T {
    type Owned = T;
}

impl<T: Clone> IntoOwned for &T {
    fn into_owned(self) -> T {
        self.clone()
    }
}

impl QueryOwnedType for RStr<'_> {
    type Owned = RString;
}

impl IntoOwned for RStr<'_> {
    fn into_owned(self) -> RString {
        self.into()
    }
}

impl<T> QueryOwnedType for RSlice<'_, T> {
    type Owned = RVec<T>;
}

impl<T: Clone> IntoOwned for RSlice<'_, T> {
    fn into_owned(self) -> RVec<T> {
        self.to_rvec()
    }
}

////////////////////////////////////////////////////////////////////

// TODO: update documentation

/// Ffi-safe equivalent of `std::borrow::Cow`.
///
/// The most common examples of this type are:
///
/// - `RCow<'_, str>`: contains an `RStr<'_>` or an `RString`.
///
/// - `RCow<'_, [T]>`: contains an `RSlice<'_, T>` or an `RVec<T>`.
///
/// - `RCow<'_, T>`: contains a `&T` or a `T`.
///
/// # Example
///
/// ### Using a `RCow<'a, str>`.
///
/// This implements a solution to the well known fizzbuzz problem.
///
/// ```
/// use abi_stable::std_types::RCow;
///
/// fn fizzbuzz(n: u32) -> RCow<'static, str> {
///     match (n % 3, n % 5) {
///         (0, 0) => RCow::from("FizzBuzz"),
///         (0, _) => RCow::from("Fizz"),
///         (_, 0) => RCow::from("Buzz"),
///         (_, _) => RCow::from(n.to_string()),
///     }
/// }
///
/// for n in 1..=100 {
///     println!("{}", fizzbuzz(n));
/// }
/// ```
///
/// Note: this example allocates when the number is neither a multiple of 5 or 3.
///
///
#[repr(C)]
#[derive(StableAbi)]
#[sabi(bound = "<B as QueryOwnedType>::Owned: StableAbi")]
pub enum RCow<B>
where
    B: IntoOwned,
{
    Borrowed(B),
    Owned(B::Owned),
}

// TODO: remove BCow?
// TODO: add RCowSliceMut?
pub type BCow<'a, T> = RCow<&'a T>;
pub type RCowStr<'a> = RCow<RStr<'a>>;
pub type RCowSlice<'a, T> = RCow<RSlice<'a, T>>;

use self::RCow::{Borrowed, Owned};

// ///////////////////////////////////////////////////////////////////////////

impl<B: IntoOwned> RCow<B> {
    /// Get a mutable reference to the owned form of RCow,
    /// converting to the owned form if it is currently the borrowed form.
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::std_types::RCow;
    ///
    /// let mut cow: RCow<'_, str> = RCow::from("Hello");
    ///
    /// assert_eq!(&*cow, "Hello");
    /// assert!(cow.is_borrowed());
    ///
    /// cow.to_mut().push_str(", world!");
    ///
    /// assert!(cow.is_owned());
    /// assert_eq!(cow, RCow::from("Hello, world!"));
    ///
    /// ```
    // TODO: update this doc
    pub fn make_mut(&mut self) -> &mut B::Owned {
        match self {
            RCow::Borrowed(x) => {
                *self = RCow::Owned(x.into_owned());
                if let RCow::Owned(x) = self {
                    x
                } else {
                    unreachable!()
                }
            }
            RCow::Owned(x) => x,
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
    /// let mut cow: RCow<'_, str> = RCow::from("Hello");
    ///
    /// assert_eq!(&*cow, "Hello");
    ///
    /// let mut buff = cow.into_owned();
    /// buff.push_str(", world!");
    ///
    /// assert_eq!(&*buff, "Hello, world!");
    ///
    /// ```
    // TODO: remove this method? Same as the trait, you just don't need the
    // trait imported.
    pub fn into_owned(self) -> B::Owned {
        match self {
            Borrowed(x) => B::into_owned(x),
            Owned(x) => x,
        }
    }

    // /// Gets the contents of the RCow casted to the borrowed variant.
    // ///
    // /// # Examples
    // ///
    // /// ```
    // /// use abi_stable::std_types::{RCow, RSlice};
    // /// {
    // ///     let cow: RCow<'_, [u8]> = RCow::from(&[0, 1, 2, 3][..]);
    // ///     assert_eq!(cow.borrowed(), RSlice::from_slice(&[0, 1, 2, 3]));
    // /// }
    // /// {
    // ///     let cow: RCow<'_, [u8]> = RCow::from(vec![0, 1, 2, 3]);
    // ///     assert_eq!(cow.borrowed(), RSlice::from_slice(&[0, 1, 2, 3]));
    // /// }
    // /// ```
    // pub fn borrowed(&self) -> B {
    //     match self {
    //         Borrowed(x) => *x,
    //         // TODO: owned -> borrowed?
    //         Owned(x) => &*x,
    //     }
    // }

    /// Whether this is a borrowing RCow.
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::std_types::RCow;
    ///
    /// {
    ///     let cow: RCow<'_, [u8]> = RCow::from(&[0, 1, 2, 3][..]);
    ///     assert!(cow.is_borrowed());
    /// }
    /// {
    ///     let cow: RCow<'_, [u8]> = RCow::from(vec![0, 1, 2, 3]);
    ///     assert!(!cow.is_borrowed());
    /// }
    ///
    /// ```
    pub fn is_borrowed(&self) -> bool {
        matches!(self, Borrowed { .. })
    }

    /// Whether this is an owning RCow.
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::std_types::RCow;
    ///
    /// let cow: RCow<'_, [u8]> = RCow::from(&[0, 1, 2, 3][..]);
    /// assert!(!cow.is_owned());
    ///
    /// let cow: RCow<'_, [u8]> = RCow::from(vec![0, 1, 2, 3]);
    /// assert!(cow.is_owned());
    ///
    /// ```
    pub fn is_owned(&self) -> bool {
        matches!(self, Owned { .. })
    }
}

impl<'a, B: IntoOwned> RCow<&'a B> {
    pub fn borrowed(&'a self) -> &'a B {
        match self {
            Borrowed(x) => *x,
            Owned(x) => x,
        }
    }
}
impl<'a, B: IntoOwned> RCowSlice<'a, B> {
    pub fn borrowed(&'a self) -> RSlice<'a, B> {
        match self {
            Borrowed(x) => *x,
            Owned(x) => x.as_rslice(),
        }
    }
}
impl<'a> RCowStr<'a> {
    pub fn borrowed(&'a self) -> RStr<'a> {
        match self {
            Borrowed(x) => *x,
            Owned(x) => x.as_rstr(),
        }
    }
}

#[allow(dead_code)]
#[cfg(test)]
impl<B: IntoOwned> RCow<B> {
    /// Access this as a borrowing RCow.Returns None if it's not a borrowing one.
    fn as_borrowed(&self) -> Option<B> {
        match *self {
            Borrowed(x) => Some(x),
            Owned(_) => None,
        }
    }

    /// Access this as an owned RCow.Returns None if it's not an owned one.
    fn as_owned(&self) -> Option<&B::Owned> {
        match self {
            Borrowed(_) => None,
            Owned(x) => Some(x),
        }
    }
}

impl<B> Copy for RCow<B>
where
    B: IntoOwned,
    B::Owned: Copy,
{
}

impl<B> Clone for RCow<B>
where
    B: IntoOwned,
    B::Owned: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Borrowed(x) => Borrowed(*x),
            Owned(x) => Owned((*x).clone()),
        }
    }
}

impl<B> Deref for RCow<&'_ B>
where
    B: IntoOwned,
{
    type Target = B;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Borrowed(x) => x,
            Owned(x) => x,
        }
    }
}
impl Deref for RCowStr<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Borrowed(x) => x,
            Owned(x) => x,
        }
    }
}
impl<B> Deref for RCowSlice<'_, B>
where
    B: Clone,
{
    type Target = [B];

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Borrowed(x) => x,
            Owned(x) => x,
        }
    }
}

////////////////////

impl<B> Borrow<B> for RCow<&'_ B>
where
    B: IntoOwned,
{
    fn borrow(&self) -> &B {
        &*self
    }
}
impl Borrow<str> for RCowStr<'_> {
    fn borrow(&self) -> &str {
        &*self
    }
}
impl<B> Borrow<[B]> for RCowSlice<'_, B>
where
    B: Clone,
{
    fn borrow(&self) -> &[B] {
        &*self
    }
}

impl<B> AsRef<B> for RCow<&'_ B>
where
    B: IntoOwned,
{
    fn as_ref(&self) -> &B {
        &*self
    }
}
impl AsRef<str> for RCowStr<'_> {
    fn as_ref(&self) -> &str {
        &*self
    }
}
impl<B> AsRef<[B]> for RCowSlice<'_, B>
where
    B: Clone,
{
    fn as_ref(&self) -> &[B] {
        &*self
    }
}

////////////////////////////

slice_like_impl_cmp_traits! {
    impl[] RCowSlice<'_, T>,
    where[T: Clone];
    Vec<U>,
    [U],
    &[U],
}

#[cfg(feature = "const_params")]
slice_like_impl_cmp_traits! {
    impl[const N: usize] RCowSlice<'_, T>,
    where[T: Clone];
    [U; N],
}

slice_like_impl_cmp_traits! {
    impl[] RCowSlice<'_, T>,
    where[T: Clone, U: Clone];
    Cow<'_, [U]>,
}

deref_coerced_impl_cmp_traits! {
    RCowStr<'_>;
    coerce_to = str,
    [
        String,
        str,
        &str,
        Cow<'_, str>,
    ]
}

// TODO: fix; macro is broken now
// shared_impls! {
//     mod = slice_impls_ref
//     new_type = RCow['a][&'a B]
//     extra[B]
//     constrained[B]
//     where [ B: IntoOwned ],
//     original_type = void,
// }
shared_impls! {
    mod = slice_impls_str
    new_type = RCowStr['a][]
    where [],
    original_type = void,
}
shared_impls! {
    mod = slice_impls_slice
    new_type = RCowSlice['a][]
    extra[B]
    constrained[B]
    where [ B: Clone ],
    original_type = void,
}

impl_into_rust_repr! {
    impl['a, B] Into<Cow<'a, B>> for RCow<&'a B>
    where[
        B: IntoOwned
    ]{
        fn(this) {
            match this {
                RCow::Borrowed(x) => Cow::Borrowed(x),
                RCow::Owned(x) => Cow::Owned(x)
            }
        }
    }
}

impl_into_rust_repr! {
    impl['a] Into<Cow<'a, str>> for RCowStr<'a>
    where[] {
        fn(this) {
            match this {
                RCow::Borrowed(x) => Cow::Borrowed(x.into()),
                RCow::Owned(x) => Cow::Owned(x.into())
            }
        }
    }
}

impl_into_rust_repr! {
    impl['a, B] Into<Cow<'a, [B]>> for RCowSlice<'a, B>
    where[
        B: Clone
    ]{
        fn(this) {
            match this {
                RCow::Borrowed(x) => Cow::Borrowed(x.into()),
                RCow::Owned(x) => Cow::Owned(x.into())
            }
        }
    }
}

////////////////////////////////////////////////////////////

impl_from_rust_repr! {
    impl['a, B] From<Cow<'a, B>> for RCow<&'a B>
    where [
        B: IntoOwned,
    ]{
        fn(this) {
            match this {
                Cow::Borrowed(x) => RCow::Borrowed(x),
                Cow::Owned(x) => RCow::Owned(x)
            }
        }
    }
}

impl_from_rust_repr! {
    impl['a] From<Cow<'a, str>> for RCowStr<'a>
    where [] {
        fn(this) {
            match this {
                Cow::Borrowed(x) => RCow::Borrowed(x.into()),
                Cow::Owned(x) => RCow::Owned(x.into())
            }
        }
    }
}

impl_from_rust_repr! {
    impl['a, B] From<Cow<'a, [B]>> for RCowSlice<'a, B>
    where [
        B: Clone,
    ]{
        fn(this) {
            match this {
                Cow::Borrowed(x) => RCow::Borrowed(x.into()),
                Cow::Owned(x) => RCow::Owned(x.into())
            }
        }
    }
}

impl<'a> From<&'a str> for RCowStr<'a> {
    #[inline]
    fn from(this: &'a str) -> Self {
        RCow::Borrowed(this.into_c())
    }
}

impl<'a> From<RStr<'a>> for RCowStr<'a> {
    #[inline]
    fn from(this: RStr<'a>) -> Self {
        RCow::Borrowed(this)
    }
}

impl<'a> From<String> for RCowStr<'a> {
    #[inline]
    fn from(this: String) -> Self {
        RCow::Owned(this.into())
    }
}

impl<'a> From<&'a String> for RCowStr<'a> {
    #[inline]
    fn from(this: &'a String) -> Self {
        RCow::Borrowed(this.as_str().into())
    }
}

impl<'a> From<RString> for RCowStr<'a> {
    #[inline]
    fn from(this: RString) -> Self {
        RCow::Owned(this)
    }
}

impl<'a> From<&'a RString> for RCowStr<'a> {
    #[inline]
    fn from(this: &'a RString) -> Self {
        RCow::Borrowed(this.as_rstr())
    }
}

impl<'a, T> From<&'a [T]> for RCowSlice<'a, T>
where
    T: Clone,
{
    #[inline]
    fn from(this: &'a [T]) -> Self {
        RCow::Borrowed(RSlice::from(this))
    }
}

impl<'a, T> RCowSlice<'a, T>
where
    T: Clone,
{
    /// For converting a `&'a [T]` to an `RCow<'a, [T]>`,
    /// most useful when converting from `&'a [T;N]` because it coerces the array to a slice.
    #[inline]
    pub fn from_slice(this: &'a [T]) -> Self {
        RCow::Borrowed(RSlice::from(this))
    }
}

impl<'a, T> From<RSlice<'a, T>> for RCowSlice<'a, T>
where
    T: Clone,
{
    #[inline]
    fn from(this: RSlice<'a, T>) -> Self {
        RCow::Borrowed(this)
    }
}

impl<'a, T> From<Vec<T>> for RCowSlice<'a, T>
where
    T: Clone,
{
    #[inline]
    fn from(this: Vec<T>) -> Self {
        RCow::Owned(RVec::from(this))
    }
}

impl<'a, T> From<RVec<T>> for RCowSlice<'a, T>
where
    T: Clone,
{
    #[inline]
    fn from(this: RVec<T>) -> Self {
        RCow::Owned(this)
    }
}

////////////////////////////////////////////////////////////

impl<'a, B> fmt::Display for RCow<&'a B>
where
    B: IntoOwned + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: &B = &*self;
        fmt::Display::fmt(s, f)
    }
}
impl<'a> fmt::Display for RCowStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: &str = &*self;
        fmt::Display::fmt(s, f)
    }
}

////////////////////////////////////////////////////////////

/// Deserializes an `RCow<'a, [u8]>` that borrows the slice from the deserializer
/// whenever possible.
///
/// # Example
///
/// Defining a type containing an `RCow<'a, [u8]>` which borrows from the deserializer.
///
/// ```
/// use abi_stable::std_types::cow::{deserialize_borrowed_bytes, RCow};
///
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Deserialize, Serialize, PartialEq)]
/// pub struct TheSlice<'a> {
///     #[serde(borrow, deserialize_with = "deserialize_borrowed_bytes")]
///     slice: RCow<'a, [u8]>,
/// }
///
/// let the_slice = TheSlice {
///     slice: RCow::from(vec![0, 1, 2, 3, 4, 5]),
/// };
///
/// let vec = bincode::serialize(&the_slice).unwrap();
///
/// let deserialized_slice = bincode::deserialize(&vec).unwrap();
///
/// assert_eq!(the_slice, deserialized_slice);
///
/// assert!(deserialized_slice.slice.is_borrowed());
///
/// ```
///
pub fn deserialize_borrowed_bytes<'de, 'a, D>(
    deserializer: D,
) -> Result<RCowSlice<'a, u8>, D::Error>
where
    D: Deserializer<'de>,
    'de: 'a,
{
    #[derive(Deserialize)]
    struct BorrowingCowSlice<'a> {
        #[serde(borrow)]
        cow: Cow<'a, [u8]>,
    }

    <BorrowingCowSlice<'de> as Deserialize<'de>>::deserialize(deserializer).map(|x| match x.cow {
        Cow::Borrowed(y) => RCow::Borrowed(y.into()),
        Cow::Owned(y) => RCow::Owned(y.into()),
    })
}

/// Deserializes an `RCow<'a, str>` that borrows the string from the deserializer
/// whenever possible.
///
///
/// # Example
///
/// Defining a type containing an `RCow<'a, str>` which borrows from the deserializer.
///
/// ```
/// use abi_stable::std_types::cow::{deserialize_borrowed_str, RCow};
///
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Deserialize, Serialize, PartialEq)]
/// pub struct TheSlice<'a> {
///     #[serde(borrow, deserialize_with = "deserialize_borrowed_str")]
///     slice: RCow<'a, str>,
/// }
///
/// let the_slice = TheSlice {
///     slice: RCow::from("That's a lot of fish."),
/// };
///
/// let string = serde_json::to_string(&the_slice).unwrap();
///
/// let deserialized_slice = serde_json::from_str::<TheSlice<'_>>(&string).unwrap();
///
/// assert_eq!(the_slice, deserialized_slice);
///
/// assert!(deserialized_slice.slice.is_borrowed());
///
/// ```
///
pub fn deserialize_borrowed_str<'de, 'a, D>(deserializer: D) -> Result<RCowStr<'a>, D::Error>
where
    D: Deserializer<'de>,
    'de: 'a,
{
    #[derive(Deserialize)]
    struct BorrowingCowStr<'a>(#[serde(borrow)] Cow<'a, str>);

    <BorrowingCowStr<'de> as Deserialize<'de>>::deserialize(deserializer)
        .map(|x| RCowStr::from(x.0))
}

impl<'de, 'a, T> Deserialize<'de> for RCowSlice<'a, T>
where
    T: Clone + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <RVec<T>>::deserialize(deserializer).map(RCowSlice::<'a, T>::Owned)
    }
}

impl<'de, 'a> Deserialize<'de> for RCowStr<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Cow<'a, str> as Deserialize<'de>>::deserialize(deserializer).map(RCowStr::from)
    }
}

impl<'de, 'a, T> Deserialize<'de> for RCow<&'a T>
where
    T: Clone + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <T as Deserialize<'de>>::deserialize(deserializer).map(RCow::Owned)
    }
}

impl<'a, B> Serialize for RCow<&'a B>
where
    B: IntoOwned + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (&**self).serialize(serializer)
    }
}
impl<'a> Serialize for RCowStr<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (&**self).serialize(serializer)
    }
}
impl<'a, B> Serialize for RCowSlice<'a, B>
where
    B: IntoOwned + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (&**self).serialize(serializer)
    }
}

/// A helper type, to deserialize an `RCow<'a, [u8]>` which borrows from the deserializer.
///
/// # Example
///
/// ```
/// use abi_stable::std_types::cow::{
///     deserialize_borrowed_bytes, BorrowingRCowU8Slice,
/// };
///
/// let the_slice: Vec<u8> = vec![0, 1, 2, 3, 4, 5];
///
/// let vec = bincode::serialize(&the_slice).unwrap();
///
/// let deserialized_slice =
///     bincode::deserialize::<BorrowingRCowU8Slice<'_>>(&vec).unwrap();
///
/// assert_eq!(&*deserialized_slice.cow, &*the_slice);
///
/// assert!(deserialized_slice.cow.is_borrowed());
///
/// ```
///
#[derive(Deserialize)]
#[serde(transparent)]
pub struct BorrowingRCowU8Slice<'a> {
    /// The deserialized `Cow`.
    #[serde(borrow, deserialize_with = "deserialize_borrowed_bytes")]
    pub cow: RCowSlice<'a, u8>,
}

/// A helper type, to deserialize a `RCow<'a, str>` which borrows from the deserializer.
///
/// # Example
///
/// Defining a type containing an `RCow<'a, str>` borrowing from the deserializer,
/// serializing it, and then deserializing it.
///
/// ```
/// use abi_stable::std_types::cow::{deserialize_borrowed_str, BorrowingRCowStr};
///
/// let json = r##""W____ of S____""##;
///
/// let deserialized_slice =
///     serde_json::from_str::<BorrowingRCowStr<'_>>(json).unwrap();
///
/// assert_eq!(&*deserialized_slice.cow, json.trim_matches('"'));
///
/// assert!(deserialized_slice.cow.is_borrowed());
///
/// ```
///
#[derive(Deserialize)]
#[serde(transparent)]
pub struct BorrowingRCowStr<'a> {
    /// The deserialized `Cow`.
    #[serde(borrow, deserialize_with = "deserialize_borrowed_str")]
    pub cow: RCowStr<'a>,
}

//////////////////////////////////////////////////////////////////////////////////////
