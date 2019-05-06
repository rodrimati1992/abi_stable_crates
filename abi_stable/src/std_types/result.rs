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
    #[inline]
    pub fn as_ref(&self) -> RResult<&T, &E> {
        match self {
            ROk(v) => ROk(v),
            RErr(v) => RErr(v),
        }
    }

    #[inline]
    pub fn as_mut(&mut self) -> RResult<&mut T, &mut E> {
        match self {
            ROk(v) => ROk(v),
            RErr(v) => RErr(v),
        }
    }

    #[inline]
    pub fn as_result(&self) -> Result<&T, &E> {
        match self {
            ROk(v) => Ok(v),
            RErr(v) => Err(v),
        }
    }

    #[inline]
    pub fn as_result_mut(&mut self) -> Result<&mut T, &mut E> {
        match self {
            ROk(v) => Ok(v),
            RErr(v) => Err(v),
        }
    }

    #[inline]
    pub fn into_result(self) -> Result<T, E> {
        self.into()
    }

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

    #[inline]
    pub fn map_or_else<U, M, F>(self, fallback: F, map: M) -> U
    where
        M: FnOnce(T) -> U,
        F: FnOnce(E) -> U,
    {
        self.map(map).unwrap_or_else(fallback)
    }

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

    #[inline]
    pub fn unwrap_or(self, optb: T) -> T {
        match self {
            ROk(t) => t,
            RErr(_) => optb,
        }
    }

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

    #[inline]
    pub fn ok(self)->ROption<T>{
        match self {
            ROk(t)  => RSome(t),
            RErr(_) => RNone,
        }
    }


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
