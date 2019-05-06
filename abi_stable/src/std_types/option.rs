use std::{ops::Deref,mem};

use core_extensions::matches;

use serde::{Deserialize, Deserializer, Serialize, Serializer};


/// Ffi-safe equivalent of the `Option<_>` type.
///
/// `Option<_>` is also ffi-safe for NonNull/NonZero types,and references.
///
/// Use ROption<_> when `Option<_>` would not be viable.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub enum ROption<T> {
    RSome(T),
    RNone,
}

pub use self::ROption::*;

impl<T> ROption<T> {
    /// Converts from `ROption<T>` to `ROption<&T>`.
    #[inline]
    pub fn as_ref(&self) -> ROption<&T> {
        match self {
            RSome(v) => RSome(v),
            RNone => RNone,
        }
    }

    /// Converts from `ROption<T>` to `ROption<&mut T>`.
    #[inline]
    pub fn as_mut(&mut self) -> ROption<&mut T> {
        match self {
            RSome(v) => RSome(v),
            RNone => RNone,
        }
    }
    
    /// Returns whether `self` is an `RSome`
    #[inline]
    pub fn is_rsome(&self)->bool{
        matches!( RSome{..}=self )
    }

    /// Returns whether `self` is an `RNone`
    #[inline]
    pub fn is_rnone(&self)->bool{
        matches!( RNone{..}=self )
    }


    /// Converts from `ROption<T>` to `Option<T>`.
    #[inline]
    pub fn into_option(self) -> Option<T> {
        self.into()
    }

    /// Unwraps the `ROption<T>`, returning its contents.
    /// 
    /// # Panics
    /// 
    /// Panics if `self` is `RNone` with the `msg` message.
    #[inline]
    pub fn expect(self, msg: &str) -> T {
        self.into_option().expect(msg)
    }
    /// Unwraps the ROption, returning its contents.
    /// 
    /// # Panics
    /// 
    /// Panics if `self` is `RNone`.
    #[inline]
    pub fn unwrap(self) -> T {
        self.into_option().unwrap()
    }

    /// Returns the value in the `ROption<T>`,or `def` if `self` is `RNone`.
    #[inline]
    pub fn unwrap_or(self, def: T) -> T {
        match self {
            RSome(x) => x,
            RNone => def,
        }
    }

    /// Returns the value in the `ROption<T>`,or `T::default()` if `self` is `RNone`.
    #[inline]
    pub fn unwrap_or_default(self) -> T 
    where 
        T:Default
    {
        match self {
            RSome(x) => x,
            RNone => Default::default(),
        }
    }

    /// Returns the value in the `ROption<T>`,
    /// or the return value of calling `f` if `self` is `RNone`.
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
    /// or returns `default`.
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
    /// or returns the value `default` returns when called.
    #[inline]
    pub fn map_or_else<U, D, F>(self, default: D, f: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(T) -> U,
    {
        match self {
            RSome(t) => f(t),
            RNone => default(),
        }
    }

    /// Returns `self` if `preficate(&self)` is true,otherwise returns `RNone`.
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

    /// Returns `self` if it is RNone,otherwise returns `optb`.
    #[inline]
    pub fn and(self, optb: ROption<T>) -> ROption<T> {
        match self {
            RSome(_) => self,
            RNone => optb,
        }
    }

    /// Returns `self` if it is RNone, 
    /// otherwise calls `optb` with the value of `RSome`.
    #[inline]
    pub fn and_then<F>(self, f: F) -> ROption<T>
    where
        F: FnOnce() -> ROption<T>,
    {
        match self {
            RSome(_) => self,
            RNone => f(),
        }
    }

    /// Returns `self` if it contains a value, otherwise returns `optb`.
    #[inline]
    pub fn or(self, optb: ROption<T>) -> ROption<T> {
        match self {
            RSome(_) => self,
            RNone => optb,
        }
    }

    /// Returns `self` if it contains a value, 
    /// otherwise calls `optb` and returns the value it evaluates to.
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

    /// Returns RNone if both values are either `RNone` or `RSome`,
    /// otherwise returns `RSome` from either one.
    #[inline]
    pub fn xor(self, optb: ROption<T>) -> ROption<T> {
        match (self, optb) {
            (RSome(a), RNone) => RSome(a),
            (RNone, RSome(b)) => RSome(b),
            _ => RNone,
        }
    }

    /// Sets this ROption to `RSome(value)` if it was RNone.
    /// Returns a mutable reference to the inserted/pre-existing `RSome`.
    #[inline]
    pub fn get_or_insert(&mut self, value: T) -> &mut T {
        match *self {
            RNone => *self = RSome(value),
            _ => (),
        }

        match *self {
            RSome(ref mut v) => v,
            RNone => unreachable!(),
        }
    }

    /// Sets this ROption to `RSome(func())` if it was RNone.
    /// Returns a mutable reference to the inserted/pre-existing `RSome`.
    #[inline]
    pub fn get_or_insert_with<F>(&mut self, func: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        match *self {
            RNone => *self = RSome(func()),
            _ => (),
        }

        match *self {
            RSome(ref mut v) => v,
            RNone => unreachable!(),
        }
    }

    /// Takes the value of `self`,replacing it with `RNone`
    #[inline]
    pub fn take(&mut self) -> ROption<T> {
        mem::replace(self, RNone)
    }

    /// Replaces the value of `self` with `RSome(value)`.
    #[inline]
    pub fn replace(&mut self, value: T) -> ROption<T> {
        mem::replace(self, RSome(value))
    }
}
    

impl<T> ROption<&T>{
    /// Converts an `Option<&T>` to a `Option<T>` by cloning its contents.
    #[inline]
    pub fn cloned(self)->ROption<T>
    where 
        T:Clone
    {
        match self {
            RSome(expr) => RSome(expr.clone()),
            RNone => RNone,
        }
    }

    /// Converts an `Option<&T>` to a `Option<T>` by Copy-ing its contents.
    #[inline]
    pub fn copied(self)->ROption<T>
    where 
        T:Copy
    {
        match self {
            RSome(expr) => RSome(*expr),
            RNone => RNone,
        }
    }
}

impl<T> ROption<&mut T>{
    /// Converts an `Option<&mut T>` to a `Option<T>` by cloning its contents.
    #[inline]
    pub fn cloned(self)->ROption<T>
    where 
        T:Clone
    {
        match self {
            RSome(expr) => RSome(expr.clone()),
            RNone => RNone,
        }
    }

    /// Converts an `Option<&mut T>` to a `Option<T>` by Copy-ing its contents.
    #[inline]
    pub fn copied(self)->ROption<T>
    where 
        T:Copy
    {
        match self {
            RSome(expr) => RSome(*expr),
            RNone => RNone,
        }
    }
}


/// The default value is RNone.
impl<T> Default for ROption<T> {
    fn default()->Self{
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
