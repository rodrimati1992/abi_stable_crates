use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::mem;

/// #[repr(C)] equivalent of the `Option<_>` type,
/// use this any time you need a stable abi for optional values
/// outside of optional references/function pointers.
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
    #[inline]
    pub fn as_ref(&self) -> ROption<&T> {
        match self {
            RSome(v) => RSome(v),
            RNone => RNone,
        }
    }

    #[inline]
    pub fn as_mut(&mut self) -> ROption<&mut T> {
        match self {
            RSome(v) => RSome(v),
            RNone => RNone,
        }
    }

    #[inline]
    pub fn as_option(&self) -> Option<&T> {
        match self {
            RSome(v) => Some(v),
            RNone => None,
        }
    }

    #[inline]
    pub fn as_option_mut(&mut self) -> Option<&mut T> {
        match self {
            RSome(v) => Some(v),
            RNone => None,
        }
    }

    #[inline]
    pub fn into_option(self) -> Option<T> {
        self.into()
    }

    #[inline]
    pub fn expect(self, msg: &str) -> T {
        self.into_option().expect(msg)
    }

    #[inline]
    pub fn unwrap(self) -> T {
        self.into_option().unwrap()
    }

    #[inline]
    pub fn unwrap_or(self, def: T) -> T {
        match self {
            RSome(x) => x,
            RNone => def,
        }
    }

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

    #[inline]
    pub fn or(self, optb: ROption<T>) -> ROption<T> {
        match self {
            RSome(_) => self,
            RNone => optb,
        }
    }

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

    #[inline]
    pub fn xor(self, optb: ROption<T>) -> ROption<T> {
        match (self, optb) {
            (RSome(a), RNone) => RSome(a),
            (RNone, RSome(b)) => RSome(b),
            _ => RNone,
        }
    }

    #[inline]
    pub fn get_or_insert(&mut self, v: T) -> &mut T {
        match *self {
            RNone => *self = RSome(v),
            _ => (),
        }

        match *self {
            RSome(ref mut v) => v,
            RNone => unsafe { ::std::hint::unreachable_unchecked() },
        }
    }

    #[inline]
    pub fn get_or_insert_with<F>(&mut self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        match *self {
            RNone => *self = RSome(f()),
            _ => (),
        }

        match *self {
            RSome(ref mut v) => v,
            RNone => unsafe { ::std::hint::unreachable_unchecked() },
        }
    }

    #[inline]
    pub fn take(&mut self) -> ROption<T> {
        mem::replace(self, RNone)
    }

    #[inline]
    pub fn replace(&mut self, value: T) -> ROption<T> {
        mem::replace(self, RSome(value))
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
        self.as_option().serialize(serializer)
    }
}
