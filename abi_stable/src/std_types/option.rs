//! Contains the ffi-safe equivalent of `std::option::Option`.

use std::{mem, ops::Deref};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::std_types::RResult;

/// Ffi-safe equivalent of the `std::option::Option` type.
///
/// `Option` is also ffi-safe for NonNull/NonZero types, and references.
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[repr(u8)]
#[derive(StableAbi)]
// #[sabi(debug_print)]
pub enum ROption<T> {
    ///
    RSome(T),
    ///
    RNone,
}

pub use self::ROption::*;

#[allow(clippy::missing_const_for_fn)]
impl<T> ROption<T> {
    /// Converts from `ROption<T>` to `ROption<&T>`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).as_ref(), RSome(&10));
    /// assert_eq!(RNone::<u32>.as_ref(), RNone);
    ///
    /// ```
    #[inline]
    pub const fn as_ref(&self) -> ROption<&T> {
        match self {
            RSome(v) => RSome(v),
            RNone => RNone,
        }
    }

    /// Converts from `ROption<T>` to `ROption<&mut T>`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).as_mut(), RSome(&mut 10));
    /// assert_eq!(RNone::<u32>.as_mut(), RNone);
    ///
    /// ```
    #[inline]
    pub fn as_mut(&mut self) -> ROption<&mut T> {
        match self {
            RSome(v) => RSome(v),
            RNone => RNone,
        }
    }

    /// Returns whether `self` is an `RSome`
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).is_rsome(), true);
    /// assert_eq!(RNone::<u32>.is_rsome(), false);
    ///
    /// ```
    #[inline]
    pub const fn is_rsome(&self) -> bool {
        matches!(self, RSome { .. })
    }

    /// Returns whether `self` is an `RNone`
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).is_rnone(), false);
    /// assert_eq!(RNone::<u32>.is_rnone(), true);
    ///
    /// ```
    #[inline]
    pub const fn is_rnone(&self) -> bool {
        matches!(self, RNone { .. })
    }

    /// Returns whether `self` is an `RSome`
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).is_some(), true);
    /// assert_eq!(RNone::<u32>.is_some(), false);
    ///
    /// ```
    #[inline]
    pub const fn is_some(&self) -> bool {
        matches!(self, RSome { .. })
    }

    /// Returns whether `self` is an `RNone`
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).is_none(), false);
    /// assert_eq!(RNone::<u32>.is_none(), true);
    ///
    /// ```
    #[inline]
    pub const fn is_none(&self) -> bool {
        matches!(self, RNone { .. })
    }

    /// Converts from `ROption<T>` to `Option<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).into_option(), Some(10));
    /// assert_eq!(RNone::<u32>.into_option(), None);
    ///
    /// ```
    #[inline]
    pub fn into_option(self) -> Option<T> {
        self.into()
    }

    /// Unwraps the `ROption<T>`, returning its contents.
    ///
    /// # Panics
    ///
    /// Panics if `self` is `RNone`, with the `msg` message.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(100).expect("must contain a value"), 100);
    ///
    /// ```
    ///
    /// This one panics:
    /// ```should_panic
    /// # use abi_stable::std_types::*;
    ///
    /// let _ = RNone::<()>.expect("Oh noooo!");
    /// ```
    #[inline]
    pub fn expect(self, msg: &str) -> T {
        self.into_option().expect(msg)
    }
    /// Unwraps the ROption, returning its contents.
    ///
    /// # Panics
    ///
    /// Panics if `self` is `RNone`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(500).unwrap(), 500);
    ///
    /// ```
    ///
    /// This one panics:
    /// ```should_panic
    /// # use abi_stable::std_types::*;
    ///
    /// let _ = RNone::<()>.unwrap();
    /// ```
    #[inline]
    pub fn unwrap(self) -> T {
        self.into_option().unwrap()
    }

    /// Returns the value in the `ROption<T>`, or `def` if `self` is `RNone`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).unwrap_or(99), 10);
    /// assert_eq!(RNone::<u32>.unwrap_or(99), 99);
    ///
    /// ```
    #[inline]
    pub fn unwrap_or(self, def: T) -> T {
        match self {
            RSome(x) => x,
            RNone => def,
        }
    }

    /// Returns the value in the `ROption<T>`, or `T::default()` if `self` is `RNone`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).unwrap_or_default(), 10);
    /// assert_eq!(RNone::<u32>.unwrap_or_default(), 0);
    ///
    /// ```
    #[inline]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            RSome(x) => x,
            RNone => Default::default(),
        }
    }

    /// Returns the value in the `ROption<T>`,
    /// or the return value of calling `f` if `self` is `RNone`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).unwrap_or_else(|| 77), 10);
    /// assert_eq!(RNone::<u32>.unwrap_or_else(|| 77), 77);
    ///
    /// ```
    #[inline]
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            RSome(x) => x,
            RNone => f(),
        }
    }

    /// Converts the `ROption<T>` to a `ROption<U>`,
    /// transforming the contained value with the `f` closure.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).map(|x| x * 2), RSome(20));
    /// assert_eq!(RNone::<u32>.map(|x| x * 2), RNone);
    ///
    /// ```
    #[inline]
    pub fn map<U, F>(self, f: F) -> ROption<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            RSome(x) => RSome(f(x)),
            RNone => RNone,
        }
    }

    /// Transforms (and returns) the contained value with the `f` closure,
    /// or returns `default` if `self` is `RNone`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).map_or(77, |x| x * 2), 20);
    /// assert_eq!(RNone::<u32>.map_or(77, |x| x * 2), 77);
    ///
    /// ```
    #[inline]
    pub fn map_or<U, F>(self, default: U, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        match self {
            RSome(t) => f(t),
            RNone => default,
        }
    }

    /// Transforms (and returns) the contained value with the `f` closure,
    /// or returns `otherwise()` if `self` is `RNone`..
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).map_or_else(|| 77, |x| x * 2), 20);
    /// assert_eq!(RNone::<u32>.map_or_else(|| 77, |x| x * 2), 77);
    ///
    /// ```
    #[inline]
    pub fn map_or_else<U, D, F>(self, otherwise: D, f: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(T) -> U,
    {
        match self {
            RSome(t) => f(t),
            RNone => otherwise(),
        }
    }

    /// Transforms the `ROption<T>` into a `RResult<T, E>`, mapping `RSome(v)`
    /// to `ROk(v)` and `RNone` to `RErr(err)`.
    ///
    /// Arguments passed to `ok_or` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`ok_or_else`], which is
    /// lazily evaluated.
    ///
    /// [`ok_or_else`]: ROption::ok_or_else
    ///
    /// # Examples
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// let x = RSome("foo");
    /// assert_eq!(x.ok_or(0), ROk("foo"));
    ///
    /// let x: ROption<&str> = RNone;
    /// assert_eq!(x.ok_or(0), RErr(0));
    /// ```
    #[inline]
    pub fn ok_or<E>(self, err: E) -> RResult<T, E> {
        match self {
            RSome(v) => RResult::ROk(v),
            RNone => RResult::RErr(err),
        }
    }

    /// Transforms the `ROption<T>` into a `RResult<T, E>`, mapping `RSome(v)` to
    /// `ROk(v)` and `RNone` to `RErr(err())`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// let x = RSome("foo");
    /// assert_eq!(x.ok_or_else(|| 0), ROk("foo"));
    ///
    /// let x: ROption<&str> = RNone;
    /// assert_eq!(x.ok_or_else(|| 0), RErr(0));
    /// ```
    #[inline]
    pub fn ok_or_else<E, F>(self, err: F) -> RResult<T, E>
    where
        F: FnOnce() -> E,
    {
        match self {
            RSome(v) => RResult::ROk(v),
            RNone => RResult::RErr(err()),
        }
    }

    /// Returns `self` if `predicate(&self)` is true, otherwise returns `RNone`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).filter(|x| (x % 2) == 0), RSome(10));
    /// assert_eq!(RSome(10).filter(|x| (x % 2) == 1), RNone);
    /// assert_eq!(RNone::<u32>.filter(|_| true), RNone);
    /// assert_eq!(RNone::<u32>.filter(|_| false), RNone);
    ///
    /// ```
    pub fn filter<P>(self, predicate: P) -> Self
    where
        P: FnOnce(&T) -> bool,
    {
        if let RSome(x) = self {
            if predicate(&x) {
                return RSome(x);
            }
        }
        RNone
    }

    /// Returns `self` if it is `RNone`, otherwise returns `optb`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).and(RSome(20)), RSome(20));
    /// assert_eq!(RSome(10).and(RNone), RNone);
    /// assert_eq!(RNone::<u32>.and(RSome(20)), RNone);
    /// assert_eq!(RNone::<u32>.and(RNone), RNone);
    ///
    /// ```
    #[inline]
    pub fn and(self, optb: ROption<T>) -> ROption<T> {
        match self {
            RSome(_) => optb,
            RNone => self,
        }
    }

    /// Returns `self` if it is `RNone`,
    /// otherwise returns the result of calling `f` with the value in `RSome`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).and_then(|x| RSome(x * 2)), RSome(20));
    /// assert_eq!(RSome(10).and_then(|_| RNone::<u32>), RNone);
    /// assert_eq!(RNone::<u32>.and_then(|x| RSome(x * 2)), RNone);
    /// assert_eq!(RNone::<u32>.and_then(|_| RNone::<u32>), RNone);
    ///
    /// ```
    #[inline]
    pub fn and_then<F, U>(self, f: F) -> ROption<U>
    where
        F: FnOnce(T) -> ROption<U>,
    {
        match self {
            RSome(x) => f(x),
            RNone => RNone,
        }
    }

    /// Returns `self` if it contains a value, otherwise returns `optb`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).or(RSome(20)), RSome(10));
    /// assert_eq!(RSome(10).or(RNone    ), RSome(10));
    /// assert_eq!(RNone::<u32>.or(RSome(20)), RSome(20));
    /// assert_eq!(RNone::<u32>.or(RNone    ), RNone);
    ///
    /// ```
    #[inline]
    pub fn or(self, optb: ROption<T>) -> ROption<T> {
        match self {
            RSome(_) => self,
            RNone => optb,
        }
    }

    /// Returns `self` if it contains a value,
    /// otherwise calls `optb` and returns the value it evaluates to.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).or_else(|| RSome(20)), RSome(10));
    /// assert_eq!(RSome(10).or_else(|| RNone), RSome(10));
    /// assert_eq!(RNone::<u32>.or_else(|| RSome(20)), RSome(20));
    /// assert_eq!(RNone::<u32>.or_else(|| RNone), RNone);
    ///
    /// ```
    #[inline]
    pub fn or_else<F>(self, f: F) -> ROption<T>
    where
        F: FnOnce() -> ROption<T>,
    {
        match self {
            RSome(_) => self,
            RNone => f(),
        }
    }

    /// Returns `RNone` if both values are `RNone` or `RSome`,
    /// otherwise returns the value that is an`RSome`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).xor(RSome(20)), RNone);
    /// assert_eq!(RSome(10).xor(RNone), RSome(10));
    /// assert_eq!(RNone::<u32>.xor(RSome(20)), RSome(20));
    /// assert_eq!(RNone::<u32>.xor(RNone), RNone);
    ///
    /// ```
    #[inline]
    pub fn xor(self, optb: ROption<T>) -> ROption<T> {
        match (self, optb) {
            (RSome(a), RNone) => RSome(a),
            (RNone, RSome(b)) => RSome(b),
            _ => RNone,
        }
    }

    /// Sets this ROption to `RSome(value)` if it was `RNone`.
    /// Returns a mutable reference to the inserted/pre-existing `RSome`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).get_or_insert(40), &mut 10);
    /// assert_eq!(RSome(20).get_or_insert(55), &mut 20);
    /// assert_eq!(RNone::<u32>.get_or_insert(77), &mut 77);
    ///
    /// ```
    #[inline]
    pub fn get_or_insert(&mut self, value: T) -> &mut T {
        if self.is_rnone() {
            *self = RSome(value);
        }

        match *self {
            RSome(ref mut v) => v,
            RNone => unreachable!(),
        }
    }

    /// Sets this `ROption` to `RSome(func())` if it was `RNone`.
    /// Returns a mutable reference to the inserted/pre-existing `RSome`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(10).get_or_insert_with(|| 40), &mut 10);
    /// assert_eq!(RSome(20).get_or_insert_with(|| 55), &mut 20);
    /// assert_eq!(RNone::<u32>.get_or_insert_with(|| 77), &mut 77);
    ///
    /// ```
    #[inline]
    pub fn get_or_insert_with<F>(&mut self, func: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        if self.is_rnone() {
            *self = RSome(func());
        }

        match *self {
            RSome(ref mut v) => v,
            RNone => unreachable!(),
        }
    }

    /// Takes the value of `self`, replacing it with `RNone`
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// let mut opt0 = RSome(10);
    /// assert_eq!(opt0.take(), RSome(10));
    /// assert_eq!(opt0, RNone);
    ///
    /// let mut opt1 = RSome(20);
    /// assert_eq!(opt1.take(), RSome(20));
    /// assert_eq!(opt1, RNone);
    ///
    /// let mut opt2 = RNone::<u32>;
    /// assert_eq!(opt2.take(), RNone);
    /// assert_eq!(opt2, RNone);
    ///
    /// ```
    #[inline]
    pub fn take(&mut self) -> ROption<T> {
        mem::replace(self, RNone)
    }

    /// Replaces the value of `self` with `RSome(value)`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// let mut opt0 = RSome(10);
    /// assert_eq!(opt0.replace(55), RSome(10));
    /// assert_eq!(opt0, RSome(55));
    ///
    /// let mut opt1 = RSome(20);
    /// assert_eq!(opt1.replace(88), RSome(20));
    /// assert_eq!(opt1, RSome(88));
    ///
    /// let mut opt2 = RNone::<u32>;
    /// assert_eq!(opt2.replace(33), RNone);
    /// assert_eq!(opt2, RSome(33));
    ///
    /// ```
    #[inline]
    pub fn replace(&mut self, value: T) -> ROption<T> {
        mem::replace(self, RSome(value))
    }
}

impl<T> ROption<&T> {
    /// Converts an `ROption<&T>` to an `ROption<T>` by cloning its contents.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(&vec![()]).cloned(), RSome(vec![()]));
    /// assert_eq!(RNone::<&Vec<()>>.cloned(), RNone);
    ///
    /// ```
    #[inline]
    pub fn cloned(self) -> ROption<T>
    where
        T: Clone,
    {
        match self {
            RSome(expr) => RSome(expr.clone()),
            RNone => RNone,
        }
    }

    /// Converts an `ROption<&T>` to an `ROption<T>` by Copy-ing its contents.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(&7).copied(), RSome(7));
    /// assert_eq!(RNone::<&u32>.copied(), RNone);
    ///
    /// ```
    #[inline]
    pub const fn copied(self) -> ROption<T>
    where
        T: Copy,
    {
        match self {
            RSome(expr) => RSome(*expr),
            RNone => RNone,
        }
    }
}

impl<T> ROption<&mut T> {
    /// Converts an `ROption<&mut T>` to a `ROption<T>` by cloning its contents.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(&mut vec![()]).cloned(), RSome(vec![()]));
    /// assert_eq!(RNone::<&mut Vec<()>>.cloned(), RNone);
    ///
    /// ```
    #[inline]
    pub fn cloned(self) -> ROption<T>
    where
        T: Clone,
    {
        match self {
            RSome(expr) => RSome(expr.clone()),
            RNone => RNone,
        }
    }

    /// Converts an `ROption<&mut T>` to a `ROption<T>` by Copy-ing its contents.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RSome(&mut 7).copied(), RSome(7));
    /// assert_eq!(RNone::<&mut u32>.copied(), RNone);
    ///
    /// ```
    #[inline]
    pub fn copied(self) -> ROption<T>
    where
        T: Copy,
    {
        match self {
            RSome(expr) => RSome(*expr),
            RNone => RNone,
        }
    }
}

impl<T: Deref> ROption<T> {
    /// Converts from `ROption<T>` (or `&ROption<T>`) to `ROption<&T::Target>`.
    ///
    /// Leaves the original ROption in-place, creating a new one with a
    /// reference to the original one, additionally coercing the contents via
    /// [`Deref`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// let x: ROption<RString> = RSome(RString::from("hey"));
    /// assert_eq!(x.as_deref(), RSome("hey"));
    ///
    /// let x: ROption<RString> = RNone;
    /// assert_eq!(x.as_deref(), RNone);
    /// ```
    pub fn as_deref(&self) -> ROption<&T::Target> {
        self.as_ref().map(|t| t.deref())
    }
}

/// The default value is `RNone`.
impl<T> Default for ROption<T> {
    fn default() -> Self {
        RNone
    }
}

impl_from_rust_repr! {
    impl[T] From<Option<T>> for ROption<T> {
        fn(this){
            match this {
                Some(v) => RSome(v),
                None => RNone,
            }
        }
    }
}

impl_into_rust_repr! {
    impl[T] Into<Option<T>> for ROption<T> {
        fn(this){
            match this {
                RSome(v) => Some(v),
                RNone => None,
            }
        }
    }
}

impl<'de, T> Deserialize<'de> for ROption<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::deserialize(deserializer).map(Self::from)
    }
}

impl<T> Serialize for ROption<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_ref().into_option().serialize(serializer)
    }
}

/////////////////////////////////////////////////////////////////////

#[cfg(all(test, not(feature = "only_new_tests")))]
// #[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_into() {
        assert_eq!(ROption::from(Some(10)), RSome(10));
        assert_eq!(ROption::from(None::<u32>), RNone);

        assert_eq!(RSome(10).into_option(), Some(10));
        assert_eq!(RNone::<u32>.into_option(), None);
    }
}
