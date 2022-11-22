//! Wrapper type(s) where their value is ignored in some trait impls .

use std::{
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

/// Wrapper type used to ignore its contents in comparisons.
///
/// Use this if you want to derive trait while ignoring the contents of fields in the
/// `PartialEq`/`Eq`/`PartialOrd`/`Ord`/`Hash` traits.
///
/// It also replaces the hash of T with the hash of `()`.
///
/// # Example
///
/// This example defines a struct with a `CmpIgnored` field.
///
/// ```
/// use abi_stable::sabi_types::CmpIgnored;
///
/// use std::collections::HashSet;
///
/// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// pub struct User {
///     name: String,
///     surname: String,
///     alt_name: CmpIgnored<String>,
/// }
///
/// let a = User {
///     name: "J__n".to_string(),
///     surname: "E____t".to_string(),
///     alt_name: "Z______l P______d".to_string().into(),
/// };
///
/// let b = User {
///     name: "J__n".to_string(),
///     surname: "E____t".to_string(),
///     alt_name: "H___ of B_____".to_string().into(),
/// };
///
/// assert_eq!(a, b);
///
/// let mut map = HashSet::new();
///
/// map.replace(a.clone());
/// assert_eq!(
///     map.replace(b.clone()).unwrap().alt_name.as_str(),
///     "Z______l P______d"
/// );
///
/// assert_eq!(map.len(), 1);
/// assert_eq!(map.get(&a).unwrap().alt_name.as_str(), "H___ of B_____");
///
/// ```
///
#[repr(transparent)]
#[derive(Default, Copy, Clone, StableAbi)]
pub struct CmpIgnored<T> {
    ///
    pub value: T,
}

impl<T> CmpIgnored<T> {
    /// Constructs a CmpIgnored.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::CmpIgnored;
    ///
    /// let val = CmpIgnored::new(100);
    ///
    /// ```
    pub const fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> From<T> for CmpIgnored<T> {
    fn from(value: T) -> Self {
        Self { value }
    }
}

impl<T> Deref for CmpIgnored<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for CmpIgnored<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> Display for CmpIgnored<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<T> Debug for CmpIgnored<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<T> Eq for CmpIgnored<T> {}

impl<T> PartialEq for CmpIgnored<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T> Ord for CmpIgnored<T> {
    fn cmp(&self, _other: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl<T> PartialOrd for CmpIgnored<T> {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}

impl<T> Hash for CmpIgnored<T> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        Unit.hash(state)
    }
}

#[derive(Hash)]
struct Unit;
