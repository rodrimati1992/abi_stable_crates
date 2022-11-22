//! Wrapper type(s) where their value is ignored in comparisons .

use std::{
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

/// Wrapper type used to ignore its contents in comparisons.
#[repr(transparent)]
#[derive(Default, Copy, Clone)]
pub struct Ignored<T> {
    pub value: T,
}

impl<T> Ignored<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> From<T> for Ignored<T> {
    fn from(value: T) -> Self {
        Self { value }
    }
}

impl<T> Deref for Ignored<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Ignored<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> Display for Ignored<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<T> Debug for Ignored<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<T> Eq for Ignored<T> {}

impl<T> PartialEq for Ignored<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T> Ord for Ignored<T> {
    fn cmp(&self, _other: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl<T> PartialOrd for Ignored<T> {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}

impl<T> Hash for Ignored<T> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        UnitType.hash(state)
    }
}

#[derive(Hash)]
struct UnitType;
