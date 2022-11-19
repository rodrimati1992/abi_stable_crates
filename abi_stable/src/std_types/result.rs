//! Contains the ffi-safe equivalent of `std::result::Result`.

use std::fmt::Debug;

use crate::std_types::{RNone, ROption, RSome};

/// Ffi-safe equivalent of `Result<T, E>`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[repr(u8)]
#[derive(StableAbi)]
pub enum RResult<T, E> {
    ///
    #[serde(rename = "Ok")]
    ROk(T),
    ///
    #[serde(rename = "Err")]
    RErr(E),
}

pub use self::RResult::*;

#[allow(clippy::missing_const_for_fn)]
impl<T, E> RResult<T, E> {
    /// Converts from `RResult<T, E>` to `RResult<&T, &E>`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(ROk::<u32, u32>(10).as_ref(), ROk(&10));
    /// assert_eq!(RErr::<u32, u32>(5).as_ref(), RErr(&5));
    ///
    /// ```
    #[inline]
    pub const fn as_ref(&self) -> RResult<&T, &E> {
        match self {
            ROk(v) => ROk(v),
            RErr(v) => RErr(v),
        }
    }

    /// Converts from `RResult<T, E>` to `RResult<&mut T, &mut E>`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(ROk::<u32, u32>(10).as_mut(), ROk(&mut 10));
    /// assert_eq!(RErr::<u32, u32>(5).as_mut(), RErr(&mut 5));
    ///
    /// ```
    #[inline]
    pub fn as_mut(&mut self) -> RResult<&mut T, &mut E> {
        match self {
            ROk(v) => ROk(v),
            RErr(v) => RErr(v),
        }
    }

    /// Returns whether `self` is an `ROk`
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(ROk::<u32, u32>(10).is_rok(), true);
    /// assert_eq!(RErr::<u32, u32>(5).is_rok(), false);
    ///
    /// ```
    #[inline]
    pub const fn is_rok(&self) -> bool {
        matches! {self, ROk{..}}
    }

    /// Returns whether `self` is an `ROk`
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(ROk::<u32, u32>(10).is_ok(), true);
    /// assert_eq!(RErr::<u32, u32>(5).is_ok(), false);
    ///
    /// ```
    #[inline]
    pub const fn is_ok(&self) -> bool {
        matches! {self, ROk{..}}
    }

    /// Returns whether `self` is an `RErr`
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(ROk::<u32, u32>(10).is_rerr(), false);
    /// assert_eq!(RErr::<u32, u32>(5).is_rerr(), true);
    ///
    /// ```
    #[inline]
    pub const fn is_rerr(&self) -> bool {
        matches! {self, RErr{..}}
    }

    /// Returns whether `self` is an `RErr`
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(ROk::<u32, u32>(10).is_err(), false);
    /// assert_eq!(RErr::<u32, u32>(5).is_err(), true);
    ///
    /// ```
    #[inline]
    pub const fn is_err(&self) -> bool {
        matches! {self, RErr{..}}
    }

    /// Converts from `RResult<T, E>` to `Result<T, E>`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(ROk::<u32, u32>(10).into_result(), Ok(10));
    /// assert_eq!(RErr::<u32, u32>(5).into_result(), Err(5));
    ///
    /// ```
    #[inline]
    pub fn into_result(self) -> Result<T, E> {
        self.into()
    }

    /// Converts the `RResult<T, E>` to a `RResult<U, E>` by transforming the value in
    /// `ROk` using the `op` closure.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(ROk::<u32, u32>(10).map(|x| x * 3), ROk(30));
    /// assert_eq!(RErr::<u32, u32>(5).map(|x| x / 2), RErr(5));
    ///
    /// ```
    #[inline]
    pub fn map<U, F>(self, op: F) -> RResult<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            ROk(t) => ROk(op(t)),
            RErr(e) => RErr(e),
        }
    }

    /// Converts the `RResult<T, E>` to a `RResult<U, F>` by
    /// transforming the value in `RErr` using the `op` closure.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(ROk::<u32, u32>(10).map_err(|x| x * 3), ROk(10));
    /// assert_eq!(RErr::<u32, u32>(5).map_err(|x| x / 2), RErr(2));
    ///
    /// ```
    #[inline]
    pub fn map_err<F, O>(self, op: O) -> RResult<T, F>
    where
        O: FnOnce(E) -> F,
    {
        match self {
            ROk(t) => ROk(t),
            RErr(e) => RErr(op(e)),
        }
    }

    /// Converts the `RResult<T, E>` to a `U` by
    /// transforming the value in `ROk` using the `with_ok` closure,
    /// otherwise transforming the value in RErr using the `with_err` closure,
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(ROk::<u32, u32>(10).map_or_else(|_| 77, |x| x * 3), 30);
    /// assert_eq!(RErr::<u32, u32>(5).map_or_else(|e| e * 4, |x| x / 2), 20);
    ///
    /// ```
    #[inline]
    pub fn map_or_else<U, M, F>(self, with_err: F, with_ok: M) -> U
    where
        M: FnOnce(T) -> U,
        F: FnOnce(E) -> U,
    {
        self.map(with_ok).unwrap_or_else(with_err)
    }

    /// Returns the result of calling the `op` closure with the value in `ROk`,
    /// otherwise returning the `RErr` unmodified.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(
    ///     ROk::<u32, u32>(10).and_then(|x| ROk::<u32, u32>(x * 3)),
    ///     ROk(30),
    /// );
    /// assert_eq!(
    ///     ROk::<u32, u32>(10).and_then(|x| RErr::<u32, u32>(x * 3)),
    ///     RErr(30),
    /// );
    /// assert_eq!(
    ///     RErr::<u32, u32>(5).and_then(|x| ROk::<u32, u32>(x / 2)),
    ///     RErr(5),
    /// );
    /// assert_eq!(
    ///     RErr::<u32, u32>(5).and_then(|x| RErr::<u32, u32>(x / 2)),
    ///     RErr(5),
    /// );
    ///
    /// ```
    #[inline]
    pub fn and_then<U, F>(self, op: F) -> RResult<U, E>
    where
        F: FnOnce(T) -> RResult<U, E>,
    {
        match self {
            ROk(t) => op(t),
            RErr(e) => RErr(e),
        }
    }

    /// Returns the result of calling the `op` closure with the value in `RErr`,
    /// otherwise returning the `ROk` unmodified.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(
    ///     ROk::<u32, u32>(10).or_else(|e| ROk::<u32, u32>(e * 3)),
    ///     ROk(10)
    /// );
    /// assert_eq!(
    ///     ROk::<u32, u32>(10).or_else(|e| RErr::<u32, u32>(e * 3)),
    ///     ROk(10)
    /// );
    /// assert_eq!(
    ///     RErr::<u32, u32>(5).or_else(|e| ROk::<u32, u32>(e / 2)),
    ///     ROk(2)
    /// );
    /// assert_eq!(
    ///     RErr::<u32, u32>(5).or_else(|e| RErr::<u32, u32>(e / 2)),
    ///     RErr(2)
    /// );
    ///
    /// ```
    #[inline]
    pub fn or_else<F, O>(self, op: O) -> RResult<T, F>
    where
        O: FnOnce(E) -> RResult<T, F>,
    {
        match self {
            ROk(t) => ROk(t),
            RErr(e) => op(e),
        }
    }

    /// Unwraps `self`, returning the value in `ROk`.
    ///
    /// # Panic
    ///
    /// Panics with an error message if `self` is an `RErr`,
    /// using `E`s Debug implementation.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(ROk::<_, ()>(500).unwrap(), 500);
    ///
    /// ```
    ///
    /// This one panics:
    /// ```should_panic
    /// # use abi_stable::std_types::*;
    ///
    /// let _ = RErr::<(), _>("Oh noooo!").unwrap();
    /// ```
    pub fn unwrap(self) -> T
    where
        E: Debug,
    {
        self.into_result().unwrap()
    }

    /// Unwraps `self`, returning the value in `ROk`.
    ///
    /// # Panic
    ///
    /// Panics with an error message if `self` is an `RErr`,
    /// using `E`s Debug implementation,
    /// as well as `message`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(ROk::<_, ()>(500).expect("Are you OK?"), 500);
    ///
    /// ```
    ///
    /// This one panics:
    /// ```should_panic
    /// # use abi_stable::std_types::*;
    ///
    /// let _ = RErr::<(), _>(()).expect("This can't be!");
    /// ```
    pub fn expect(self, message: &str) -> T
    where
        E: Debug,
    {
        self.into_result().expect(message)
    }

    /// Unwraps `self`, returning the value in `RErr`.
    ///
    /// # Panic
    ///
    /// Panics with an error message if `self` is an `ROk`,
    /// using `T`s Debug implementation.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RErr::<(), u32>(0xB007).unwrap_err(), 0xB007);
    ///
    /// ```
    ///
    /// This one panics:
    /// ```should_panic
    /// # use abi_stable::std_types::*;
    ///
    /// let _ = ROk::<(), ()>(()).unwrap_err();
    /// ```
    pub fn unwrap_err(self) -> E
    where
        T: Debug,
    {
        self.into_result().unwrap_err()
    }

    /// Unwraps `self`, returning the value in `RErr`.
    ///
    /// # Panic
    ///
    /// Panics with an error message if `self` is an `ROk`,
    /// using `T`s Debug implementation,
    /// as well as `message`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(RErr::<(), u32>(0xB001).expect_err("Murphy's law"), 0xB001);
    ///
    /// ```
    ///
    /// This one panics:
    /// ```should_panic
    /// # use abi_stable::std_types::*;
    ///
    /// let _ = ROk::<(), ()>(()).expect_err("Everything is Ok");
    /// ```
    pub fn expect_err(self, message: &str) -> E
    where
        T: Debug,
    {
        self.into_result().expect_err(message)
    }

    /// Returns the value in `ROk`, or `def` if `self` is `RErr`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(ROk::<u32, u32>(10).unwrap_or(0xEEEE), 10);
    /// assert_eq!(RErr::<u32, u32>(5).unwrap_or(0b101010), 0b101010);
    ///
    /// ```
    #[inline]
    pub fn unwrap_or(self, optb: T) -> T {
        match self {
            ROk(t) => t,
            RErr(_) => optb,
        }
    }

    /// Returns the value in `ROk`,
    /// or calls `def` with the error in `RErr`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(ROk::<u32, u32>(10).unwrap_or_else(|e| e * 3), 10);
    /// assert_eq!(RErr::<u32, u32>(5).unwrap_or_else(|e| e / 2), 2);
    ///
    /// ```
    #[inline]
    pub fn unwrap_or_else<F>(self, op: F) -> T
    where
        F: FnOnce(E) -> T,
    {
        match self {
            ROk(t) => t,
            RErr(e) => op(e),
        }
    }

    /// Returns the value in `ROk`,
    /// or returns `T::default()` it `self` is an `RErr`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(ROk::<u32, u32>(10).unwrap_or_default(), 10);
    /// assert_eq!(RErr::<u32, u32>(5).unwrap_or_default(), 0);
    ///
    /// ```
    #[inline]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            ROk(t) => t,
            RErr(_) => Default::default(),
        }
    }

    /// Converts from `RResult<T, E>` to `ROption<T>`,
    /// `ROk` maps to `RSome`, `RErr` maps to `RNone`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(ROk::<u32, u32>(10).ok(), RSome(10));
    /// assert_eq!(RErr::<u32, u32>(5).ok(), RNone);
    ///
    /// ```
    #[inline]
    pub fn ok(self) -> ROption<T> {
        match self {
            ROk(t) => RSome(t),
            RErr(_) => RNone,
        }
    }

    /// Converts from `RResult<T, E>` to `ROption<T>`,
    /// `ROk` maps to `RNone`, `RErr` maps to `RSome`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::std_types::*;
    ///
    /// assert_eq!(ROk::<u32, u32>(10).err(), RNone);
    /// assert_eq!(RErr::<u32, u32>(5).err(), RSome(5));
    ///
    /// ```
    #[inline]
    pub fn err(self) -> ROption<E> {
        match self {
            ROk(_) => RNone,
            RErr(v) => RSome(v),
        }
    }
}

impl_from_rust_repr! {
    impl[T, E] From<Result<T, E>> for RResult<T, E> {
        fn(this){
            match this {
                Ok(v) => ROk(v),
                Err(v) => RErr(v),
            }
        }
    }
}

impl_into_rust_repr! {
    impl[T, E] Into<Result<T, E>> for RResult<T, E> {
        fn(this){
            match this {
                ROk(v) => Ok(v),
                RErr(v) => Err(v),
            }
        }
    }
}

/////////////////////////////////////////////////////////////////////

//#[cfg(test)]
#[cfg(all(test, not(feature = "only_new_tests")))]
mod test {
    use super::*;

    #[test]
    fn from_into() {
        assert_eq!(RResult::from(Ok::<u32, u32>(10)), ROk(10));
        assert_eq!(RResult::from(Err::<u32, u32>(4)), RErr(4));

        assert_eq!(ROk::<u32, u32>(10).into_result(), Ok(10));
        assert_eq!(RErr::<u32, u32>(4).into_result(), Err(4));
    }
}
