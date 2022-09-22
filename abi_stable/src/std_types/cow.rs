//! Contains the ffi-safe equivalent of `std::borrow::Cow`, and related items.

use std::{
    borrow::{Borrow, Cow},
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    ops::Deref,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[allow(unused_imports)]
use core_extensions::{matches, SelfOps};

use crate::{
    std_types::{RSlice, RStr, RString, RVec},
    traits::{IntoOwned, IntoReprC, IntoReprRust},
    StableAbi,
};

// #[cfg(test)]
#[cfg(all(test, not(feature = "only_new_tests")))]
mod tests;

////////////////////////////////////////////////////////////////////

/// For making a `Cow<'a, Self>` convertible into an `RCow`.
pub trait RCowCompatibleRef<'a>: ToOwned {
    /// The (preferably) ffi-safe equivalent of `&Self`.
    type RefC: IntoOwned<ROwned = Self::ROwned, Target = Self>;

    /// The owned version of `Self::RefC`.
    type ROwned: Borrow<Self> + Into<Self::Owned> + From<Self::Owned>;

    /// Converts a reference to an FFI-safe type
    fn as_c_ref(from: &'a Self) -> Self::RefC;

    /// Converts an FFI-safe type to a reference
    fn as_rust_ref(from: Self::RefC) -> &'a Self;
}

impl<'a> RCowCompatibleRef<'a> for str {
    type RefC = RStr<'a>;
    type ROwned = RString;

    fn as_c_ref(from: &'a Self) -> Self::RefC {
        RStr::from_str(from)
    }
    fn as_rust_ref(from: Self::RefC) -> &'a Self {
        from.as_str()
    }
}

impl<'a, T: Clone + 'a> RCowCompatibleRef<'a> for [T] {
    type RefC = RSlice<'a, T>;
    type ROwned = RVec<T>;

    fn as_c_ref(from: &'a Self) -> Self::RefC {
        RSlice::from_slice(from)
    }
    fn as_rust_ref(from: Self::RefC) -> &'a Self {
        from.into()
    }
}

impl<'a, T: Clone + 'a> RCowCompatibleRef<'a> for T {
    type RefC = &'a T;
    type ROwned = T;

    fn as_c_ref(from: &'a Self) -> Self::RefC {
        from
    }
    fn as_rust_ref(from: Self::RefC) -> &'a Self {
        from
    }
}

////////////////////////////////////////////////////////////////////

/// Ffi-safe equivalent of `std::borrow::Cow`.
///
/// This has type aliases for the three most common usecases:
///
/// - [`RCowStr`]: contains an `RStr<'_>` or an `RString`.
///
/// - [`RCowSlice`]: contains an `RSlice<'_, T>` or an `RVec<T>`.
///
/// - [`RCowVal`]: contains a `&T` or a `T`.
///
/// # Example
///
/// ### Using a `RCowStr<'a>`.
///
/// This implements a solution to the well known fizzbuzz problem.
///
/// ```
/// use abi_stable::std_types::{RCow, RCowStr};
///
/// fn fizzbuzz(n: u32) -> RCowStr<'static> {
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
#[repr(C)]
#[derive(StableAbi)]
pub enum RCow<B, O> {
    ///
    Borrowed(B),
    ///
    Owned(O),
}

use self::RCow::{Borrowed, Owned};

/// Ffi-safe equivalent of `Cow<'a, T>`, either a `&T` or `T`.
///
/// # Example
///
/// ```rust
/// use abi_stable::std_types::{RCow, RCowVal};
///
/// fn foo(x: u8) -> RCowVal<'static, u8> {
///     if x % 2 == 0 {
///        RCow::Borrowed(&1)
///     } else {
///        RCow::Owned(x * 2)
///     }
/// }
///
/// assert_eq!(*foo(3), 6);
/// assert_eq!(*foo(4), 1);
/// assert_eq!(*foo(5), 10);
/// assert_eq!(*foo(6), 1);
/// assert_eq!(*foo(7), 14);
///
/// ```
pub type RCowVal<'a, T> = RCow<&'a T, T>;

/// Ffi-safe equivalent of `Cow<'a, str>`, either an [`RStr`] or [`RString`].
///
/// # Example
///
/// ```rust
/// use abi_stable::std_types::{RCow, RCowStr};
///
/// fn foo(x: &str) -> RCowStr<'_> {
///     if let Some(x) = x.strip_prefix("tri") {
///        RCow::from(x.repeat(3))
///     } else {
///        RCow::from(x)
///     }
/// }
///
/// assert_eq!(foo("foo"), "foo");
/// assert_eq!(foo("bar"), "bar");
/// assert_eq!(foo("tribaz"), "bazbazbaz");
/// assert_eq!(foo("triqux"), "quxquxqux");
///
/// ```
///
pub type RCowStr<'a> = RCow<RStr<'a>, RString>;

/// Ffi-safe equivalent of `Cow<'a, [T]>`, either an [`RSlice`] or [`RVec`].
///
/// # Example
///
/// ```rust
/// use abi_stable::std_types::{RCow, RCowSlice, RVec};
///
/// use std::iter::once;
///
/// fn foo(x: &[u32]) -> RCowSlice<'_, u32> {
///     match x {
///         [prev @ .., x] if *x == 5 => RCow::from(RVec::from(prev)),
///         _ => RCow::from(x),
///     }
/// }
///
/// assert_eq!(foo(&[3, 4]), &[3, 4][..]);
/// assert_eq!(foo(&[3, 4, 5]), &[3, 4][..]);
/// assert_eq!(foo(&[3, 4, 5, 6]), &[3, 4, 5, 6][..]);
/// assert_eq!(foo(&[3, 4, 5, 6, 7]), &[3, 4, 5, 6, 7][..]);
///
/// ```
///
pub type RCowSlice<'a, T> = RCow<RSlice<'a, T>, RVec<T>>;

// ///////////////////////////////////////////////////////////////////////////

impl<B> RCow<B, B::ROwned>
where
    B: IntoOwned,
{
    /// Get a mutable reference to the owned form of RCow,
    /// converting to the owned form if it is currently the borrowed form.
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::std_types::{RCow, RCowStr};
    ///
    /// let mut cow: RCowStr<'_> = RCow::from("Hello");
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
    pub fn to_mut(&mut self) -> &mut B::ROwned {
        if let Borrowed(v) = *self {
            let owned = B::into_owned(v);
            *self = Owned(owned)
        }
        match self {
            Borrowed(_) => unreachable!(),
            Owned(v) => v,
        }
    }
    /// Unwraps into the owned owner form of RCow,
    /// converting to the owned form if it is currently the borrowed form.
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::std_types::{RCow, RCowStr};
    ///
    /// let mut cow: RCowStr<'_> = RCow::from("Hello");
    ///
    /// assert_eq!(&*cow, "Hello");
    ///
    /// let mut buff = cow.into_owned();
    /// buff.push_str(", world!");
    ///
    /// assert_eq!(&*buff, "Hello, world!");
    ///
    /// ```
    pub fn into_owned(self) -> B::ROwned {
        match self {
            Borrowed(x) => B::into_owned(x),
            Owned(x) => x,
        }
    }

    /// Gets the contents of the RCow casted to the borrowed variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::std_types::{RCow, RCowSlice, RSlice};
    /// {
    ///     let cow: RCowSlice<'_, u8> = RCow::from(&[0, 1, 2, 3][..]);
    ///     assert_eq!(cow.borrowed(), RSlice::from_slice(&[0, 1, 2, 3]));
    /// }
    /// {
    ///     let cow: RCowSlice<'_, u8> = RCow::from(vec![0, 1, 2, 3]);
    ///     assert_eq!(cow.borrowed(), RSlice::from_slice(&[0, 1, 2, 3]));
    /// }
    /// ```
    pub fn borrowed(&self) -> &<B as Deref>::Target {
        match self {
            Borrowed(x) => x,
            Owned(x) => x.borrow(),
        }
    }
}

impl<B, O> RCow<B, O> {
    /// Whether this is a borrowing RCow.
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::std_types::{RCow, RCowSlice};
    ///
    /// {
    ///     let cow: RCowSlice<'_, u8> = RCow::from(&[0, 1, 2, 3][..]);
    ///     assert!(cow.is_borrowed());
    /// }
    /// {
    ///     let cow: RCowSlice<'_, u8> = RCow::from(vec![0, 1, 2, 3]);
    ///     assert!(!cow.is_borrowed());
    /// }
    ///
    /// ```
    pub const fn is_borrowed(&self) -> bool {
        matches!(self, Borrowed { .. })
    }

    /// Whether this is an owning RCow.
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::std_types::{RCow, RCowSlice};
    ///
    /// let cow: RCowSlice<'_, u8> = RCow::from(&[0, 1, 2, 3][..]);
    /// assert!(!cow.is_owned());
    ///
    /// let cow: RCowSlice<'_, u8> = RCow::from(vec![0, 1, 2, 3]);
    /// assert!(cow.is_owned());
    ///
    /// ```
    pub const fn is_owned(&self) -> bool {
        matches!(self, Owned { .. })
    }
}

#[allow(dead_code)]
#[cfg(test)]
impl<B> RCow<B, B::ROwned>
where
    B: IntoOwned,
{
    /// Access this as a borrowing RCow.Returns None if it's not a borrowing one.
    fn as_borrowed(&self) -> Option<B> {
        match *self {
            Borrowed(x) => Some(x),
            Owned(_) => None,
        }
    }

    /// Access this as an owned RCow.Returns None if it's not an owned one.
    fn as_owned(&self) -> Option<&B::ROwned> {
        match self {
            Borrowed(_) => None,
            Owned(x) => Some(x),
        }
    }
}

impl<B> Copy for RCow<B, B::ROwned>
where
    B: IntoOwned,
    B::ROwned: Copy,
{
}

impl<B> Clone for RCow<B, B::ROwned>
where
    B: IntoOwned,
    B::ROwned: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Borrowed(x) => Borrowed(*x),
            Owned(x) => Owned((*x).clone()),
        }
    }
}

impl<B> Deref for RCow<B, B::ROwned>
where
    B: IntoOwned,
{
    type Target = B::Target;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Borrowed(x) => x,
            Owned(x) => x.borrow(),
        }
    }
}

////////////////////

macro_rules! impl_borrow_asref {
    (impl[$($impl_p:tt)*] $ty:ty, $target:ty) => {
        impl<$($impl_p)*> Borrow<$target> for $ty {
            fn borrow(&self) -> &$target {
                self
            }
        }

        impl<$($impl_p)*> AsRef<$target> for $ty {
            fn as_ref(&self) -> &$target {
                self
            }
        }
    };
}

impl_borrow_asref! {impl[T: Clone] RCowVal<'_, T>, T}
impl_borrow_asref! {impl[] RCowStr<'_>, str}
impl_borrow_asref! {impl[T: Clone] RCowSlice<'_, T>, [T]}

////////////////////////////////////////////////////////////

impl<B> Debug for RCow<B, B::ROwned>
where
    B: IntoOwned,
    B::Target: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <B::Target as Debug>::fmt(&**self, f)
    }
}

impl<B> fmt::Display for RCow<B, B::ROwned>
where
    B: IntoOwned,
    B::Target: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <B::Target as fmt::Display>::fmt(&**self, f)
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

impl<B> Eq for RCow<B, B::ROwned>
where
    B: IntoOwned,
    B::Target: Eq,
{
}

impl<B, V> PartialEq<RCow<V, V::ROwned>> for RCow<B, B::ROwned>
where
    B: IntoOwned,
    V: IntoOwned,
    B::Target: PartialEq<V::Target>,
{
    fn eq(&self, other: &RCow<V, V::ROwned>) -> bool {
        **self == **other
    }
}

impl<B> Ord for RCow<B, B::ROwned>
where
    B: IntoOwned,
    B::Target: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        (**self).cmp(&**other)
    }
}

impl<B, V> PartialOrd<RCow<V, V::ROwned>> for RCow<B, B::ROwned>
where
    B: IntoOwned,
    V: IntoOwned,
    B::Target: PartialOrd<V::Target>,
{
    fn partial_cmp(&self, other: &RCow<V, V::ROwned>) -> Option<Ordering> {
        (**self).partial_cmp(&**other)
    }
}

impl<B> Hash for RCow<B, B::ROwned>
where
    B: IntoOwned,
    B::Target: Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        (**self).hash(state)
    }
}

////////////////////////////////////////////////////////////

impl<'a, T, B> From<RCow<B, T::ROwned>> for Cow<'a, T>
where
    T: ?Sized + RCowCompatibleRef<'a, RefC = B>,
    B: IntoOwned<ROwned = T::ROwned, Target = T>,
{
    fn from(this: RCow<B, T::ROwned>) -> Cow<'a, T> {
        match this {
            RCow::Borrowed(x) => Cow::Borrowed(T::as_rust_ref(x)),
            RCow::Owned(x) => Cow::Owned(x.into()),
        }
    }
}

macro_rules! impl_into_repr_rust {
    (impl[$($impl_params:tt)*] $rcow:ty, $cow_param:ty) => {
        impl<'a, $($impl_params)*> IntoReprRust for $rcow {
            type ReprRust = Cow<'a, $cow_param>;

            fn into_rust(self) -> Self::ReprRust {
                self.into()
            }
        }
    };
}
impl_into_repr_rust! {impl[T: Clone] RCowSlice<'a, T>, [T]}
impl_into_repr_rust! {impl[T: Clone] RCowVal<'a, T>, T}
impl_into_repr_rust! {impl[] RCowStr<'a>, str}

impl_from_rust_repr! {
    impl['a, T] From<Cow<'a, T>> for RCow<T::RefC, T::ROwned>
    where [
        T: ?Sized + RCowCompatibleRef<'a>
    ]{
        fn(this){
            match this {
                Cow::Borrowed(x) => RCow::Borrowed(T::as_c_ref(x)),
                Cow::Owned(x) => RCow::Owned(x.into()),
            }
        }
    }
}

////////////////////////////////////////////////////////////

impl<'a> RCowStr<'a> {
    /// For converting a `&'a [T]` to an `RCowSlice<'a, T>`,
    /// most useful when converting from `&'a [T;N]` because it coerces the array to a slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::std_types::{RCow, RCowStr};
    ///
    /// const C: RCowStr<'_> = RCow::from_str("hello");
    ///
    /// assert_eq!(C, "hello");
    ///
    /// ```
    #[inline]
    pub const fn from_str(this: &'a str) -> Self {
        RCow::Borrowed(RStr::from_str(this))
    }

    conditionally_const! {
        feature = "rust_1_64"
        /// Borrows this RCow as a str.
        ///
        ;
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::std_types::RCow;
        ///
        /// let cow = RCow::from_str("world");
        ///
        /// assert_eq!(cow.as_str(), "world")
        ///
        /// ```
        ///
        pub fn as_str(&self) -> &str {
            match self {
                RCow::Borrowed(x) => x.as_str(),
                RCow::Owned(x) => x.as_str(),
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

impl<'a, T> From<&'a Vec<T>> for RCowSlice<'a, T>
where
    T: Clone,
{
    #[inline]
    fn from(this: &'a Vec<T>) -> Self {
        RCow::Borrowed(RSlice::from_slice(this))
    }
}

impl<'a, T> From<&'a RVec<T>> for RCowSlice<'a, T>
where
    T: Clone,
{
    #[inline]
    fn from(this: &'a RVec<T>) -> Self {
        RCow::Borrowed(RSlice::from_slice(this))
    }
}

impl<'a, T> RCowSlice<'a, T> {
    /// For converting a `&'a [T]` to an `RCowSlice<'a, T>`,
    /// most useful when converting from `&'a [T;N]` because it coerces the array to a slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::std_types::{RCow, RCowSlice};
    ///
    /// const C: RCowSlice<'_, u8> = RCow::from_slice(&[3, 5, 8]);
    ///
    /// assert_eq!(C, [3, 5, 8]);
    ///
    /// ```
    #[inline]
    pub const fn from_slice(this: &'a [T]) -> Self {
        RCow::Borrowed(RSlice::from_slice(this))
    }

    conditionally_const! {
        feature = "rust_1_64"
        /// Borrows this RCow as a slice.
        ///
        ;
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::std_types::RCow;
        ///
        /// let cow = RCow::from_slice(&[3, 5, 8]);
        ///
        /// assert_eq!(cow.as_slice(), [3, 5, 8])
        ///
        /// ```
        ///
        pub fn as_slice(&self) -> &[T] {
            match self {
                RCow::Borrowed(x) => x.as_slice(),
                RCow::Owned(x) => x.as_slice(),
            }
        }
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

/// Deserializes an `RCow<'a, [u8]>` that borrows the slice from the deserializer
/// whenever possible.
///
/// # Example
///
/// Defining a type containing an `RCow<'a, [u8]>` which borrows from the deserializer.
///
/// ```
/// use abi_stable::std_types::cow::{deserialize_borrowed_bytes, RCow, RCowSlice};
///
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Deserialize, Serialize, PartialEq)]
/// pub struct TheSlice<'a> {
///     #[serde(borrow, deserialize_with = "deserialize_borrowed_bytes")]
///     slice: RCowSlice<'a, u8>,
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

/// Deserializes an `RCowStr<'a>` that borrows the string from the deserializer
/// whenever possible.
///
///
/// # Example
///
/// Defining a type containing an `RCowStr<'a>` which borrows from the deserializer.
///
/// ```
/// use abi_stable::std_types::cow::{deserialize_borrowed_str, RCow, RCowStr};
///
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Deserialize, Serialize, PartialEq)]
/// pub struct TheSlice<'a> {
///     #[serde(borrow, deserialize_with = "deserialize_borrowed_str")]
///     slice: RCowStr<'a>,
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
        <Cow<'a, str> as Deserialize<'de>>::deserialize(deserializer).map(RCow::from)
    }
}

impl<'de, 'a, T> Deserialize<'de> for RCowVal<'a, T>
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

impl<B> Serialize for RCow<B, B::ROwned>
where
    B: IntoOwned,
    B::Target: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (**self).serialize(serializer)
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

/// A helper type, to deserialize a `RCowStr<'a>` which borrows from the deserializer.
///
/// # Example
///
/// Defining a type containing an `RCowStr<'a>` borrowing from the deserializer,
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
