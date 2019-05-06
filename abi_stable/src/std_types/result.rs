use std::fmt::Debug;

use core_extensions::matches;

use crate::std_types::{ROption,RSome,RNone};



/// Ffi-safe equivalent of `Result<T,E>`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub enum RResult<T, E> {
    #[serde(rename = "Ok")]
    ROk(T),
    #[serde(rename = "Err")]
    RErr(E),
}

pub use self::RResult::*;

impl<T, E> RResult<T, E> {
    /// Converts from `RResult<T,E>` to `RResult<&T,&E>`.
    #[inline]
    pub fn as_ref(&self) -> RResult<&T, &E> {
        match self {
            ROk(v) => ROk(v),
            RErr(v) => RErr(v),
        }
    }

    /// Converts from `RResult<T,E>` to `RResult<&mut T,&mut E>`.
    #[inline]
    pub fn as_mut(&mut self) -> RResult<&mut T, &mut E> {
        match self {
            ROk(v) => ROk(v),
            RErr(v) => RErr(v),
        }
    }

    /// Returns whether `self` is an `ROk`
    #[inline]
    pub fn is_rok(&self)->bool{
        matches!{ ROk{..}=self }
    }

    /// Returns whether `self` is an `RErr`
    #[inline]
    pub fn is_rerr(&self)->bool{
        matches!{ RErr{..}=self }
    }


    /// Converts from `RResult<T,E>` to `Result<T,E>`.
    #[inline]
    pub fn into_result(self) -> Result<T, E> {
        self.into()
    }

    /// Converts the `RResult<T,E>` to a `RResult<U,E>` by transforming the value in ROk
    /// using the `op` closure.
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

    /// Converts the `RResult<T,E>` to a `RResult<U,F>` by transforming the value in RErr
    /// using the `op` closure.
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

    /// Converts the `RResult<T,E>` to a `U` by 
    /// transforming the value in ROk using the `with_ok` closure,
    /// otherwise transforming the value in RErr using the `with_err` closure,
    #[inline]
    pub fn map_or_else<U, M, F>(self, with_err: F, with_ok: M) -> U
    where
        M: FnOnce(T) -> U,
        F: FnOnce(E) -> U,
    {
        self.map(with_ok).unwrap_or_else(with_err)
    }

    /// Calls the `op` closure with the value of ROk,
    /// otherwise returning the RErr unmodified.
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

    /// Calls the `op` closure with the value of RErr,
    /// otherwise returning the ROk unmodified.
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

    /// Unwraps `self`, returning the value in Ok.
    ///
    /// # Panic
    ///
    /// Panics if `self` is an `Err(_)` with an error message 
    /// using `E`s Debug implementation.
    pub fn unwrap(self) -> T 
    where 
        E:Debug
    {
        self.into_result().unwrap()
    }

    /// Unwraps `self`, returning the value in Ok.
    ///
    /// # Panic
    ///
    /// Panics if `self` is an `Err(_)` with an error message 
    /// using `E`s Debug implementation,
    /// as well as `message`.
    pub fn expect(self, message: &str) -> T
    where 
        E:Debug
    {
        self.into_result().expect(message)
    }

    /// Unwraps `self`, returning the value in Err.
    ///
    /// # Panic
    ///
    /// Panics if `self` is an `Ok(_)` with an error message 
    /// using `T`s Debug implementation.
    pub fn unwrap_err(self) -> E
    where 
        T:Debug
    {
        self.into_result().unwrap_err()
    }

    /// Unwraps `self`, returning the value in Err.
    ///
    /// # Panic
    ///
    /// Panics if `self` is an `Ok(_)` with an error message 
    /// using `T`s Debug implementation,
    /// as well as `message`.
    pub fn expect_err(self, message: &str) -> E
    where 
        T:Debug
    {
        self.into_result().expect_err(message)
    }

    /// Returns the value in the `RResult<T,E>`,or `def` if `self` is `RErr`.
    #[inline]
    pub fn unwrap_or(self, optb: T) -> T {
        match self {
            ROk(t) => t,
            RErr(_) => optb,
        }
    }

    /// Returns the value in the `RResult<T,E>`,
    /// or calls `def` with the error in `RErr`.
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

    /// Returns the value in the `RResult<T,E>`,
    /// or returns `T::default()` it `self` is an `RErr`.
    #[inline]
    pub fn unwrap_or_default(self) -> T
    where
        T:Default
    {
        match self {
            ROk(t) => t,
            RErr(e) => Default::default(),
        }
    }

    /// Converts from RResult<T, E> to ROption<T>,ROk maps to RSome,RErr maps to RNone.
    #[inline]
    pub fn ok(self)->ROption<T>{
        match self {
            ROk(t)  => RSome(t),
            RErr(_) => RNone,
        }
    }


    /// Converts from RResult<T, E> to ROption<T>,ROk maps to RNone,RErr maps to RSome.
    #[inline]
    pub fn err(self)->ROption<E>{
        match self {
            ROk(_)  => RNone,
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
